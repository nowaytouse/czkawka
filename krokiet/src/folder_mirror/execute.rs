use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

use serde::Serialize;

use super::Report;
use super::plan::{EntryKind, EntryStamp, Plan, build_plan};

#[cfg(any(target_os = "linux", target_os = "macos", windows))]
pub(super) fn check_volume(plan: &Plan) -> io::Result<()> {
    let same_volume = plan.a_root.device == plan.c_root.device && plan.a_entries.values().all(|entry| entry.device == plan.a_root.device);
    #[cfg(windows)]
    let same_volume = same_volume && plan.a.components().next() == plan.c.components().next();
    if !same_volume {
        return Err(io::Error::new(
            io::ErrorKind::CrossesDevices,
            "A and the archive must use the same filesystem; mounted volumes inside A are unsupported",
        ));
    }
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
pub(super) fn check_volume(_plan: &Plan) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Folder Mirror requires a platform with non-overwriting rename support",
    ))
}

pub(super) fn apply(plan: &Plan, reference_dirs: &[PathBuf], protected: &HashSet<PathBuf>) -> io::Result<Report> {
    let fresh = build_plan(&plan.a, &plan.b, &plan.c)?;
    if &fresh != plan {
        return Err(io::Error::other("The folders changed after preview. Preview the new plan before applying it."));
    }
    if !plan.has_changes() {
        return Err(io::Error::other("The preview contains no operations to apply"));
    }
    check_volume(plan)?;
    check_references(plan, reference_dirs)?;
    check_protected(plan, protected)?;

    let archive = create_archive(&plan.c)?;
    let operations = operations(plan, &archive);
    let mut completed = 0;
    let result = (|| -> io::Result<()> {
        let receipt = OpenOptions::new().write(true).create_new(true).open(archive.join("plan.json"))?;
        serde_json::to_writer_pretty(
            &receipt,
            &serde_json::json!({
                "version": 1,
                "source": plan.a,
                "template": plan.b,
                "archive": archive,
                "operations": operations,
                "missing": plan.missing,
                "ambiguous": plan.ambiguous,
                "blocked": plan.blocked,
            }),
        )?;
        receipt.sync_all()?;
        let mut journal = OpenOptions::new().write(true).create_new(true).open(archive.join("completed.jsonl"))?;
        let files = archive.join("files");
        make_directory(&files)?;
        let archive_stamp = EntryStamp::read(&archive)?;
        let files_stamp = EntryStamp::read(&files)?;
        for operation in &operations {
            ensure_root(&plan.a, &plan.a_root)?;
            ensure_root(&plan.c, &plan.c_root)?;
            ensure_root(&archive, &archive_stamp)?;
            ensure_root(&files, &files_stamp)?;
            perform_operation(operation, plan, &files)?;
            completed += 1;
            serde_json::to_writer(&mut journal, operation)?;
            journal.write_all(b"\n")?;
            journal.sync_data()?;
        }
        Ok(())
    })();
    result.map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("Stopped after {completed} operations: {error}. Check {archive:?} for the plan and completed operations before retrying."),
        )
    })?;
    Ok(Report {
        archive,
        completed,
        missing: plan.missing.len(),
        unresolved: plan.blocked.len(),
    })
}

fn create_archive(parent: &Path) -> io::Result<PathBuf> {
    let mut builder = tempfile::Builder::new();
    builder.prefix("krokiet-mirror-");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        builder.permissions(fs::Permissions::from_mode(0o700));
    }
    Ok(builder.tempdir_in(parent)?.keep())
}

fn check_references(plan: &Plan, reference_dirs: &[PathBuf]) -> io::Result<()> {
    for reference in reference_dirs {
        let reference = match fs::canonicalize(reference) {
            Ok(path) => path,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(path_error(reference, &error)),
        };
        if overlaps(&plan.a, &reference) || overlaps(&plan.c, &reference) {
            return Err(io::Error::other(format!("A and the archive must be outside configured reference folders: {reference:?}")));
        }
    }
    Ok(())
}

fn check_protected(plan: &Plan, protected: &HashSet<PathBuf>) -> io::Result<()> {
    for saved in protected {
        let canonical = match fs::canonicalize(saved) {
            Ok(path) => path,
            Err(error) if error.kind() == io::ErrorKind::NotFound => saved.clone(),
            Err(error) => return Err(path_error(saved, &error)),
        };
        let affected = plan
            .archive_files
            .iter()
            .chain(&plan.archive_dirs)
            .chain(&plan.create_dirs)
            .chain(plan.move_files.iter().flat_map(|item| [&item.source, &item.destination]));
        if affected.into_iter().any(|path| overlaps(&plan.a.join(path), &canonical)) || overlaps(&plan.c, &canonical) {
            return Err(io::Error::other(format!("The mirror plan would touch a protected path: {saved:?}")));
        }
    }
    Ok(())
}

fn overlaps(first: &Path, second: &Path) -> bool {
    if first.starts_with(second) || second.starts_with(first) {
        return true;
    }
    if cfg!(any(target_os = "macos", windows)) {
        let first = first.to_string_lossy().to_lowercase();
        let second = second.to_string_lossy().to_lowercase();
        return Path::new(&first).starts_with(&second) || Path::new(&second).starts_with(&first);
    }
    false
}

#[derive(Serialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
enum Operation {
    ArchiveFile { source: PathBuf, destination: PathBuf },
    CreateFolder { destination: PathBuf },
    MoveFile { source: PathBuf, destination: PathBuf },
    ArchiveFolder { source: PathBuf, destination: PathBuf },
}

fn operations(plan: &Plan, archive: &Path) -> Vec<Operation> {
    let files = archive.join("files");
    let mut result = Vec::new();
    result.extend(plan.archive_files.iter().map(|path| Operation::ArchiveFile {
        source: plan.a.join(path),
        destination: files.join(path),
    }));
    result.extend(plan.create_dirs.iter().map(|path| Operation::CreateFolder { destination: plan.a.join(path) }));
    result.extend(plan.move_files.iter().map(|item| Operation::MoveFile {
        source: plan.a.join(&item.source),
        destination: plan.a.join(&item.destination),
    }));
    result.extend(plan.archive_dirs.iter().map(|path| Operation::ArchiveFolder {
        source: plan.a.join(path),
        destination: files.join(path),
    }));
    result
}

fn perform_operation(operation: &Operation, plan: &Plan, archive: &Path) -> io::Result<()> {
    match operation {
        Operation::ArchiveFile { source, destination } | Operation::MoveFile { source, destination } => {
            let destination_root = if matches!(operation, Operation::ArchiveFile { .. }) { archive } else { &plan.a };
            ensure_parent(&plan.a, source, false)?;
            ensure_parent(destination_root, destination, destination_root == archive)?;
            check_source(plan, source, false)?;
            rename_exclusive(source, destination)
        }
        Operation::CreateFolder { destination } => {
            ensure_parent(&plan.a, destination, false)?;
            make_directory(destination).map_err(|error| path_error(destination, &error))
        }
        Operation::ArchiveFolder { source, destination } => {
            ensure_parent(&plan.a, source, false)?;
            ensure_parent(archive, destination, true)?;
            check_source(plan, source, true)?;
            // Never move a directory tree: a newly arrived file must stay where it is.
            ensure_directory(destination, true)?;
            remove_empty_directory(source).map_err(|error| path_error(source, &error))
        }
    }
}

fn check_source(plan: &Plan, path: &Path, directory: bool) -> io::Result<()> {
    let relative = path.strip_prefix(&plan.a).map_err(io::Error::other)?;
    let expected = plan.a_entries.get(relative).ok_or_else(|| io::Error::other("Source is absent from the preview"))?;
    let actual = EntryStamp::read(path).map_err(|error| path_error(path, &error))?;
    let unchanged = if directory {
        actual.kind == EntryKind::Directory && actual.same_identity(expected)
    } else {
        &actual == expected
    };
    if unchanged {
        Ok(())
    } else {
        Err(io::Error::other(format!("Source changed after preview: {path:?}")))
    }
}

fn ensure_root(path: &Path, original: &EntryStamp) -> io::Result<()> {
    let actual = EntryStamp::read(path).map_err(|error| path_error(path, &error))?;
    if actual.kind == EntryKind::Directory && actual.same_identity(original) {
        Ok(())
    } else {
        Err(io::Error::other(format!("Root folder changed after preview: {path:?}")))
    }
}

fn ensure_parent(root: &Path, path: &Path, create: bool) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| io::Error::other("Destination has no parent folder"))?;
    let relative = parent.strip_prefix(root).map_err(io::Error::other)?;
    ensure_directory(root, false)?;
    let mut current = root.to_path_buf();
    for item in relative.components() {
        let Component::Normal(component) = item else {
            return Err(io::Error::other("Mirror paths must stay inside their designated roots"));
        };
        current.push(component);
        ensure_directory(&current, create)?;
    }
    Ok(())
}

fn ensure_directory(path: &Path, create: bool) -> io::Result<()> {
    match EntryStamp::read(path) {
        Ok(stamp) if stamp.kind == EntryKind::Directory => Ok(()),
        Ok(_) => Err(io::Error::other(format!("Expected a regular directory: {path:?}"))),
        Err(error) if create && error.kind() == io::ErrorKind::NotFound => make_directory(path).map_err(|error| path_error(path, &error)),
        Err(error) => Err(path_error(path, &error)),
    }
}

fn path_error(path: &Path, error: &io::Error) -> io::Error {
    io::Error::new(error.kind(), format!("{path:?}: {error}"))
}

fn rename_exclusive(source: &Path, destination: &Path) -> io::Result<()> {
    rename_no_replace(source, destination).map_err(|error| io::Error::new(error.kind(), format!("Could not move {source:?} to {destination:?}: {error}")))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn rename_no_replace(source: &Path, destination: &Path) -> io::Result<()> {
    use rustix::fs::{RenameFlags, renameat_with};
    let (source_dir, source_name) = open_parent(source)?;
    let (destination_dir, destination_name) = open_parent(destination)?;
    renameat_with(source_dir, source_name, destination_dir, destination_name, RenameFlags::NOREPLACE).map_err(io::Error::from)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn open_parent(path: &Path) -> io::Result<(rustix::fd::OwnedFd, &std::ffi::OsStr)> {
    use rustix::fs::{Mode, OFlags, open, openat};

    let parent = path.parent().ok_or_else(|| io::Error::other("Missing parent folder"))?;
    let name = path.file_name().ok_or_else(|| io::Error::other("Missing filename"))?;
    let mut components = parent.components();
    if components.next() != Some(Component::RootDir) {
        return Err(io::Error::other("Expected an absolute folder path"));
    }
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let mut directory = open("/", flags, Mode::empty())?;
    for component in components {
        let Component::Normal(name) = component else {
            return Err(io::Error::other("Expected a normalized folder path"));
        };
        directory = openat(directory, name, flags, Mode::empty())?;
    }
    Ok((directory, name))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn make_directory(path: &Path) -> io::Result<()> {
    let (parent, name) = open_parent(path)?;
    rustix::fs::mkdirat(parent, name, rustix::fs::Mode::from_bits_truncate(0o777)).map_err(io::Error::from)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn remove_empty_directory(path: &Path) -> io::Result<()> {
    let (parent, name) = open_parent(path)?;
    rustix::fs::unlinkat(parent, name, rustix::fs::AtFlags::REMOVEDIR).map_err(io::Error::from)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn make_directory(path: &Path) -> io::Result<()> {
    fs::create_dir(path)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn remove_empty_directory(path: &Path) -> io::Result<()> {
    fs::remove_dir(path)
}

#[cfg(windows)]
fn rename_no_replace(source: &Path, destination: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    use windows_sys::Win32::Storage::FileSystem::MoveFileExW;

    let wide = |path: &Path| path.as_os_str().encode_wide().chain([0]).collect::<Vec<_>>();
    let source = wide(source);
    let destination = wide(destination);
    // SAFETY: Both buffers are valid NUL-terminated paths for the duration of the call.
    if unsafe { MoveFileExW(source.as_ptr(), destination.as_ptr(), 0) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos", windows)))]
fn rename_no_replace(_source: &Path, _destination: &Path) -> io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "Non-overwriting rename is unavailable"))
}

#[cfg(all(test, any(target_os = "linux", target_os = "macos", windows)))]
mod tests {
    use super::*;

    struct Fixture {
        _temp: tempfile::TempDir,
        a: PathBuf,
        b: PathBuf,
        c: PathBuf,
    }

    fn fixture() -> io::Result<Fixture> {
        let temp = tempfile::tempdir()?;
        let base = fs::canonicalize(temp.path())?;
        let a = base.join("a");
        let b = base.join("b");
        let c = base.join("archive");
        for directory in [&a, &b, &c] {
            fs::create_dir(directory)?;
        }
        Ok(Fixture { _temp: temp, a, b, c })
    }

    #[test]
    fn mirrors_existing_names_and_archives_extras_with_recovery_record() {
        let f = fixture().unwrap();
        fs::create_dir_all(f.a.join("old/deep")).unwrap();
        fs::create_dir(f.a.join("empty-extra")).unwrap();
        fs::create_dir_all(f.b.join("new/empty")).unwrap();
        fs::write(f.a.join("same.jpg"), "A contents").unwrap();
        fs::write(f.b.join("same.jpg"), "different B contents").unwrap();
        fs::write(f.a.join("old/photo.jpg"), "source").unwrap();
        fs::write(f.b.join("new/photo.jpg"), "template").unwrap();
        fs::write(f.b.join("missing.jpg"), "do not copy").unwrap();
        fs::write(f.a.join("old/deep/extra.jpg"), "archive me").unwrap();

        let plan = build_plan(&f.a, &f.b, &f.c).unwrap();
        assert_eq!(fs::read_dir(&f.c).unwrap().count(), 0);
        let result = apply(&plan, &[], &HashSet::new()).unwrap();
        assert_eq!(fs::read_to_string(f.a.join("same.jpg")).unwrap(), "A contents");
        assert_eq!(fs::read_to_string(f.a.join("new/photo.jpg")).unwrap(), "source");
        assert_eq!(fs::read_to_string(f.b.join("new/photo.jpg")).unwrap(), "template");
        assert!(!f.a.join("missing.jpg").exists());
        assert!(!f.a.join("old").exists());
        assert!(f.a.join("new/empty").is_dir());
        assert!(result.archive.join("files/empty-extra").is_dir());
        assert_eq!(fs::read_to_string(result.archive.join("files/old/deep/extra.jpg")).unwrap(), "archive me");
        assert!(result.archive.join("plan.json").is_file());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&result.archive).unwrap().permissions().mode() & 0o077, 0);
        }
        assert_eq!(fs::read_to_string(result.archive.join("completed.jsonl")).unwrap().lines().count(), result.completed);
        assert_eq!(result.missing, 1);
    }

    #[test]
    fn stale_or_protected_plan_stops_before_any_writes() {
        let f = fixture().unwrap();
        fs::write(f.a.join("extra"), "do not move").unwrap();
        let plan = build_plan(&f.a, &f.b, &f.c).unwrap();
        assert!(apply(&plan, &[], &HashSet::from([f.a.join("extra")])).is_err());
        assert!(apply(&plan, std::slice::from_ref(&f.a), &HashSet::new()).is_err());
        fs::write(f.a.join("new-file"), "changed inventory").unwrap();
        assert!(apply(&plan, &[], &HashSet::new()).is_err());
        assert_eq!(fs::read_to_string(f.a.join("extra")).unwrap(), "do not move");
        assert_eq!(fs::read_dir(&f.c).unwrap().count(), 0);
    }

    #[test]
    fn exclusive_move_cannot_clobber_an_existing_file() {
        let f = fixture().unwrap();
        let source = f.a.join("source");
        let target = f.c.join("target");
        fs::write(&source, "source").unwrap();
        fs::write(&target, "existing").unwrap();
        assert!(rename_no_replace(&source, &target).is_err());
        assert_eq!(fs::read_to_string(&source).unwrap(), "source");
        assert_eq!(fs::read_to_string(&target).unwrap(), "existing");
    }

    #[test]
    fn directory_with_a_late_arrival_is_never_removed() {
        let f = fixture().unwrap();
        let source = f.a.join("extra");
        fs::create_dir(&source).unwrap();
        let plan = build_plan(&f.a, &f.b, &f.c).unwrap();
        fs::write(source.join("new-file"), "keep it in A").unwrap();
        let operation = Operation::ArchiveFolder {
            source: source.clone(),
            destination: f.c.join("extra"),
        };
        assert!(perform_operation(&operation, &plan, &f.c).is_err());
        assert_eq!(fs::read_to_string(source.join("new-file")).unwrap(), "keep it in A");
        assert!(!f.c.join("extra/new-file").exists());
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn native_rename_refuses_a_symlink_in_the_destination_ancestry() {
        let f = fixture().unwrap();
        let source = f.a.join("source");
        fs::write(&source, "source").unwrap();
        std::os::unix::fs::symlink(&f.b, f.c.join("alias")).unwrap();
        assert!(rename_no_replace(&source, &f.c.join("alias/unexpected")).is_err());
        assert!(source.is_file());
        assert!(!f.b.join("unexpected").exists());
    }
}
