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
        build_plan(&self.0.join("a"), &self.0.join("b"), &self.0.join("c"), false)
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
    assert!(
        build_plan(&dirs.0.join("a"), &dirs.0.join("b"), &dirs.0, false)
            .unwrap_err()
            .to_string()
            .contains("overlap")
    );
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
