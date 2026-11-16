use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{fs, io};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EntryKind {
    File,
    Directory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EntryStamp {
    pub(super) kind: EntryKind,
    len: u64,
    modified: SystemTime,
    created: Option<SystemTime>,
    #[cfg(any(unix, windows))]
    pub(super) device: u64,
    #[cfg(any(unix, windows))]
    inode: u64,
}

impl EntryStamp {
    pub(super) fn read(path: &Path) -> io::Result<Self> {
        #[cfg(windows)]
        let (metadata, device, inode) = windows_metadata(path)?;
        #[cfg(not(windows))]
        let metadata = at(path, fs::symlink_metadata(path))?;
        let kind = if metadata.is_file() {
            EntryKind::File
        } else if metadata.is_dir() {
            EntryKind::Directory
        } else {
            return Err(invalid(path, "Symbolic links and special files are not supported"));
        };
        #[cfg(unix)]
        let (device, inode) = {
            use std::os::unix::fs::MetadataExt;
            (metadata.dev(), metadata.ino())
        };
        Ok(Self {
            kind,
            len: metadata.len(),
            modified: at(path, metadata.modified())?,
            created: metadata.created().ok(),
            #[cfg(any(unix, windows))]
            device,
            #[cfg(any(unix, windows))]
            inode,
        })
    }

    pub(super) fn same_identity(&self, other: &Self) -> bool {
        if self.kind != other.kind {
            return false;
        }
        #[cfg(any(unix, windows))]
        {
            self.device == other.device && self.inode == other.inode
        }
        #[cfg(not(any(unix, windows)))]
        {
            self.created.is_some() && self.created == other.created
        }
    }
}

#[cfg(windows)]
fn windows_metadata(path: &Path) -> io::Result<(fs::Metadata, u64, u64)> {
    use std::os::windows::fs::OpenOptionsExt;
    use std::os::windows::io::AsRawHandle;

    use windows_sys::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, GetFileInformationByHandle,
    };

    let file = at(
        path,
        fs::OpenOptions::new()
            .access_mode(0)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path),
    )?;
    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    // The handle stays open and info is writable for the duration of the API call.
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return at(path, Err(io::Error::last_os_error()));
    }
    if info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(invalid(path, "Reparse points are not supported"));
    }
    let inode = (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow);
    if inode == 0 {
        return Err(invalid(path, "The filesystem did not report a file identity"));
    }
    Ok((at(path, file.metadata())?, u64::from(info.dwVolumeSerialNumber), inode))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlannedMove {
    pub(super) source: PathBuf,
    pub(super) destination: PathBuf,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Plan {
    pub(super) a: PathBuf,
    pub(super) b: PathBuf,
    pub(super) c: PathBuf,
    pub(super) a_root: EntryStamp,
    pub(super) b_root: EntryStamp,
    pub(super) c_root: EntryStamp,
    pub(super) a_entries: BTreeMap<PathBuf, EntryStamp>,
    pub(super) b_entries: BTreeMap<PathBuf, EntryStamp>,
    pub(super) move_files: Vec<PlannedMove>,
    pub(super) archive_files: Vec<PathBuf>,
    pub(super) create_dirs: Vec<PathBuf>,
    pub(super) archive_dirs: Vec<PathBuf>,
    pub(super) unchanged: usize,
    pub(super) missing: Vec<PathBuf>,
    pub(super) ambiguous: Vec<PathBuf>,
    pub(super) blocked: Vec<PathBuf>,
}

pub(super) fn build_plan(a: &Path, b: &Path, c: &Path) -> io::Result<Plan> {
    let (a, a_root) = canonical_root(a)?;
    let (b, b_root) = canonical_root(b)?;
    let (c, c_root) = canonical_root(c)?;
    validate_disjoint(&a, &a_root, &b, &b_root)?;
    validate_disjoint(&a, &a_root, &c, &c_root)?;
    validate_disjoint(&b, &b_root, &c, &c_root)?;

    let mut visited = BTreeSet::new();
    let a_entries = inventory(&a, &a_root, [&b_root, &c_root], &mut visited)?;
    let b_entries = inventory(&b, &b_root, [&a_root, &c_root], &mut visited)?;
    for (path, entry) in &a_entries {
        if b_entries.get(path).is_some_and(|other| entry.kind != other.kind) {
            return Err(invalid(&a.join(path), "File and directory conflict with the template at this relative path"));
        }
    }

    let mut plan = Plan {
        a,
        b,
        c,
        a_root,
        b_root,
        c_root,
        a_entries,
        b_entries,
        move_files: Vec::new(),
        archive_files: Vec::new(),
        create_dirs: Vec::new(),
        archive_dirs: Vec::new(),
        unchanged: 0,
        missing: Vec::new(),
        ambiguous: Vec::new(),
        blocked: Vec::new(),
    };
    plan.classify_files();
    plan.classify_directories();
    plan.check_new_destinations()?;
    Ok(plan)
}

impl Plan {
    fn classify_files(&mut self) {
        let mut groups: BTreeMap<OsString, (Vec<PathBuf>, Vec<PathBuf>)> = BTreeMap::new();
        for (path, entry) in &self.a_entries {
            if entry.kind == EntryKind::File {
                if self.b_entries.contains_key(path) {
                    self.unchanged += 1;
                } else if let Some(name) = path.file_name() {
                    groups.entry(name.to_owned()).or_default().0.push(path.clone());
                }
            }
        }
        for (path, entry) in &self.b_entries {
            if entry.kind == EntryKind::File
                && !self.a_entries.contains_key(path)
                && let Some(name) = path.file_name()
            {
                groups.entry(name.to_owned()).or_default().1.push(path.clone());
            }
        }
        for (_, (mut sources, mut destinations)) in groups {
            match (sources.len(), destinations.len()) {
                (0, _) => self.missing.extend(destinations),
                (_, 0) => self.archive_files.extend(sources),
                (1, 1) => self.move_files.push(PlannedMove {
                    source: sources.remove(0),
                    destination: destinations.remove(0),
                }),
                _ => {
                    self.ambiguous.extend(sources);
                    self.blocked.extend(destinations);
                }
            }
        }
        self.move_files.sort_by(|left, right| left.source.cmp(&right.source));
        self.archive_files.sort();
        self.missing.sort();
        self.ambiguous.sort();
        self.blocked.sort();
    }

    fn classify_directories(&mut self) {
        let retained: BTreeSet<_> = self.ambiguous.iter().flat_map(|path| path.ancestors().skip(1)).collect();
        self.create_dirs = self
            .b_entries
            .iter()
            .filter(|(path, entry)| entry.kind == EntryKind::Directory && !self.a_entries.contains_key(*path))
            .map(|(path, _)| path.clone())
            .collect();
        self.create_dirs.sort_by_key(|path| path.components().count());
        self.archive_dirs = self
            .a_entries
            .iter()
            .filter(|(path, entry)| entry.kind == EntryKind::Directory && !self.b_entries.contains_key(*path) && !retained.contains(path.as_path()))
            .map(|(path, _)| path.clone())
            .collect();
        self.archive_dirs.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    }

    fn check_new_destinations(&self) -> io::Result<()> {
        #[cfg(any(target_os = "macos", windows))]
        {
            check_case_collisions(
                &self.a,
                self.create_dirs.iter().chain(self.move_files.iter().map(|item| &item.destination)).map(PathBuf::as_path),
            )?;
            check_case_collisions(&self.c, self.archive_dirs.iter().chain(&self.archive_files).map(PathBuf::as_path))?;
        }
        for relative in self.create_dirs.iter().chain(self.move_files.iter().map(|item| &item.destination)) {
            let path = self.a.join(relative);
            match fs::symlink_metadata(&path) {
                Ok(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::AlreadyExists,
                        format!("Destination aliases an existing path: {}", path.display()),
                    ));
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return at(&path, Err(error)),
            }
        }
        Ok(())
    }
}

#[cfg(any(target_os = "macos", windows, test))]
fn check_case_collisions<'a>(root: &Path, paths: impl Iterator<Item = &'a Path>) -> io::Result<()> {
    let mut folded = BTreeMap::new();
    for path in paths {
        for parent in path.ancestors().take_while(|path| !path.as_os_str().is_empty()) {
            let previous = folded.insert(parent.to_string_lossy().to_uppercase(), parent);
            if let Some(other) = previous
                && other != parent
            {
                return Err(invalid(
                    &root.join(parent),
                    &format!("Destination may collide with {} on a case-insensitive filesystem", root.join(other).display()),
                ));
            }
        }
    }
    Ok(())
}

fn canonical_root(path: &Path) -> io::Result<(PathBuf, EntryStamp)> {
    if !path.is_absolute() || path.to_str().is_none() {
        return Err(invalid(path, "Choose an absolute UTF-8 folder path"));
    }
    if EntryStamp::read(path)?.kind != EntryKind::Directory {
        return Err(invalid(path, "Expected a directory"));
    }
    let canonical = at(path, fs::canonicalize(path))?;
    if canonical.to_str().is_none() {
        return Err(invalid(&canonical, "Folder resolves to a non-UTF-8 path"));
    }
    let stamp = EntryStamp::read(&canonical)?;
    if stamp.kind != EntryKind::Directory {
        return Err(invalid(&canonical, "Expected a directory"));
    }
    Ok((canonical, stamp))
}

fn validate_disjoint(left: &Path, left_stamp: &EntryStamp, right: &Path, right_stamp: &EntryStamp) -> io::Result<()> {
    #[cfg(windows)]
    let overlaps = {
        let left = PathBuf::from(left.to_string_lossy().to_lowercase());
        let right = PathBuf::from(right.to_string_lossy().to_lowercase());
        left.starts_with(&right) || right.starts_with(&left)
    };
    #[cfg(not(windows))]
    let overlaps = left.starts_with(right) || right.starts_with(left);
    if overlaps {
        return Err(invalid(left, &format!("Folders overlap with {}", right.display())));
    }
    #[cfg(any(unix, windows))]
    for (path, stamp) in [(left, right_stamp), (right, left_stamp)] {
        for ancestor in path.ancestors().filter(|ancestor| ancestor.is_absolute()) {
            if EntryStamp::read(ancestor)?.same_identity(stamp) {
                return Err(invalid(path, "Folders overlap through a filesystem alias"));
            }
        }
    }
    #[cfg(not(any(unix, windows)))]
    let _ = (left_stamp, right_stamp);
    Ok(())
}

fn inventory(root: &Path, root_stamp: &EntryStamp, other_roots: [&EntryStamp; 2], visited: &mut BTreeSet<(u64, u64)>) -> io::Result<BTreeMap<PathBuf, EntryStamp>> {
    let mut entries = BTreeMap::new();
    let mut pending = vec![PathBuf::new()];
    #[cfg(any(unix, windows))]
    if !visited.insert((root_stamp.device, root_stamp.inode)) {
        return Err(invalid(root, "Directory aliases another scanned or selected folder"));
    }
    while let Some(relative) = pending.pop() {
        let parent = root.join(&relative);
        for entry in at(&parent, fs::read_dir(&parent))? {
            let entry = at(&parent, entry)?;
            let name = entry.file_name();
            let path = entry.path();
            if name.to_str().is_none() {
                return Err(invalid(&path, "Non-UTF-8 names are not supported"));
            }
            let stamp = EntryStamp::read(&path)?;
            #[cfg(any(unix, windows))]
            if stamp.device != root_stamp.device {
                return Err(invalid(&path, "Nested filesystems are not supported"));
            }
            let child = relative.join(name);
            if stamp.kind == EntryKind::Directory {
                #[cfg(any(unix, windows))]
                if !visited.insert((stamp.device, stamp.inode)) || other_roots.iter().any(|root| stamp.same_identity(root)) {
                    return Err(invalid(&path, "Directory aliases another scanned or selected folder"));
                }
                pending.push(child.clone());
            }
            entries.insert(child, stamp);
        }
    }
    #[cfg(not(any(unix, windows)))]
    let _ = (root_stamp, other_roots, visited);
    Ok(entries)
}

fn at<T>(path: &Path, result: io::Result<T>) -> io::Result<T> {
    result.map_err(|error| io::Error::new(error.kind(), format!("{}: {error}", path.display())))
}

fn invalid(path: &Path, reason: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, format!("{}: {reason}", path.display()))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    struct Folders(PathBuf);

    impl Folders {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let dir = loop {
                let number = NEXT.fetch_add(1, Ordering::Relaxed);
                let path = std::env::temp_dir().join(format!("krokiet-mirror-plan-{}-{number}", std::process::id()));
                match fs::create_dir(&path) {
                    Ok(()) => break path,
                    Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                    Err(error) => panic!("Cannot create test folder: {error}"),
                }
            };
            for name in ["a", "b", "c"] {
                fs::create_dir(dir.join(name)).unwrap();
            }
            Self(dir)
        }

        fn file(&self, relative: &str, content: &[u8]) {
            let path = self.0.join(relative);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, content).unwrap();
        }

        fn plan(&self) -> io::Result<Plan> {
            build_plan(&self.0.join("a"), &self.0.join("b"), &self.0.join("c"))
        }
    }

    impl Drop for Folders {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn exact_paths_precede_unique_filename_matches_and_content_is_ignored() {
        let dirs = Folders::new();
        dirs.file("a/keep/photo.JPG", b"original A bytes");
        dirs.file("a/old/photo.JPG", b"move A bytes");
        dirs.file("a/old/extra", b"archive A bytes");
        dirs.file("a/upper/Case.txt", b"case-sensitive name");
        dirs.file("b/keep/photo.JPG", b"different B bytes");
        dirs.file("b/new/photo.JPG", b"template B bytes");
        dirs.file("b/lower/case.txt", b"missing exact filename");
        fs::create_dir_all(dirs.0.join("a/old/empty")).unwrap();
        fs::create_dir_all(dirs.0.join("b/empty/nested")).unwrap();

        let plan = dirs.plan().unwrap();
        assert_eq!(plan, dirs.plan().unwrap());
        assert_eq!(plan.unchanged, 1);
        assert_eq!(
            plan.move_files,
            [PlannedMove {
                source: "old/photo.JPG".into(),
                destination: "new/photo.JPG".into()
            }]
        );
        assert_eq!(plan.archive_files, [PathBuf::from("old/extra"), PathBuf::from("upper/Case.txt")]);
        assert_eq!(plan.missing, [PathBuf::from("lower/case.txt")]);
        assert_eq!(plan.archive_dirs, [PathBuf::from("old/empty"), PathBuf::from("old"), PathBuf::from("upper")]);
        assert_eq!(
            plan.create_dirs,
            [PathBuf::from("empty"), PathBuf::from("lower"), PathBuf::from("new"), PathBuf::from("empty/nested")]
        );
        assert!(plan.ambiguous.is_empty());
        assert!(plan.blocked.is_empty());
        assert_eq!(fs::read(dirs.0.join("a/keep/photo.JPG")).unwrap(), b"original A bytes");
        assert_eq!(fs::read_dir(dirs.0.join("c")).unwrap().count(), 0);
    }

    #[test]
    fn ambiguous_sources_and_their_original_directories_are_kept() {
        let dirs = Folders::new();
        for path in ["a/first/inner/same", "a/second/same", "a/current/other", "b/one/same", "b/one/other", "b/two/other"] {
            dirs.file(path, b"fixture");
        }
        let plan = dirs.plan().unwrap();
        assert_eq!(
            plan.ambiguous,
            [PathBuf::from("current/other"), PathBuf::from("first/inner/same"), PathBuf::from("second/same")]
        );
        assert_eq!(plan.blocked, [PathBuf::from("one/other"), PathBuf::from("one/same"), PathBuf::from("two/other")]);
        assert!(plan.move_files.is_empty());
        assert!(plan.archive_files.is_empty());
        assert!(plan.archive_dirs.is_empty());
        assert!(plan.missing.is_empty());
    }

    #[test]
    fn plan_detects_inventory_changes_and_refuses_overlaps_and_shape_conflicts() {
        let dirs = Folders::new();
        dirs.file("a/original.txt", b"original");
        let before = dirs.plan().unwrap();
        dirs.file("a/original.txt", b"modified content of a different size");
        assert!(before.a_root.same_identity(&EntryStamp::read(&before.a).unwrap()));
        assert_ne!(before, dirs.plan().unwrap());
        assert!(build_plan(&dirs.0.join("a"), &dirs.0.join("b"), &dirs.0).unwrap_err().to_string().contains("overlap"));
        fs::create_dir(dirs.0.join("b/original.txt")).unwrap();
        assert!(dirs.plan().unwrap_err().to_string().contains("File and directory conflict"));
    }

    #[test]
    fn case_collision_check_includes_virtual_parent_directories() {
        let root = std::env::temp_dir();
        check_case_collisions(&root, ["Folder/one", "Folder/two", "Folder"].map(Path::new).into_iter()).unwrap();
        assert!(check_case_collisions(&root, ["Folder/one", "folder/two"].map(Path::new).into_iter()).is_err());
        assert!(check_case_collisions(&root, ["Folder/Name.txt", "Folder/name.TXT"].map(Path::new).into_iter()).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn inventory_refuses_symlinks_and_non_utf8_names() {
        use std::os::unix::ffi::OsStringExt;
        use std::os::unix::fs::symlink;

        let dirs = Folders::new();
        let link = dirs.0.join("a/link");
        symlink(dirs.0.join("b"), &link).unwrap();
        assert!(dirs.plan().unwrap_err().to_string().contains("Symbolic links"));
        fs::remove_file(link).unwrap();

        // Use an encoded path on every Unix. Some filesystems reject it before a scan can see it.
        let name = OsString::from_vec(vec![0xff]);
        let path = dirs.0.join("a").join(name);
        match fs::write(&path, b"fixture") {
            Ok(()) => assert!(dirs.plan().unwrap_err().to_string().contains("Non-UTF-8")),
            #[cfg(target_os = "macos")]
            Err(error) if error.raw_os_error() == Some(92) => assert!(canonical_root(&path).unwrap_err().to_string().contains("UTF-8")),
            Err(error) => panic!("Cannot write non-UTF-8 fixture: {error}"),
        }
    }
}
