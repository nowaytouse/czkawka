use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::{default_test_files, EntryKind, TestFileEntry};

static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        loop {
            let path = std::env::temp_dir().join(format!("czkawka-ci-tester-{}-{}", std::process::id(), NEXT_TEMP_DIR.fetch_add(1, Ordering::Relaxed)));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => panic!("create test directory {}: {e}", path.display()),
            }
        }
    }

    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove generated test directory");
    }
}

fn entry(path: &str, kind: EntryKind, data: &[u8]) -> TestFileEntry {
    TestFileEntry {
        path: path.to_owned(),
        kind,
        data: data.to_vec(),
        mtime_unix_secs: None,
    }
}

#[test]
fn rejects_unsafe_or_colliding_paths_before_writing() {
    let mut fixtures: Vec<Vec<TestFileEntry>> = [
        "",
        ".",
        "../escape",
        "/absolute",
        "folder/../escape",
        "folder/./file",
        "folder//file",
        "folder/",
        "C:/escape",
        "C:escape",
        "folder\\escape",
        "nul\0file",
    ]
    .into_iter()
    .map(|path| vec![entry(path, EntryKind::File, b"bad")])
    .collect();
    fixtures.push(vec![entry("same", EntryKind::File, b"one"), entry("same", EntryKind::Dir, b"")]);
    for kind in [EntryKind::File, EntryKind::Symlink] {
        for reverse in [false, true] {
            let mut entries = vec![entry("redirect", kind, b"../outside"), entry("redirect/payload", EntryKind::File, b"bad")];
            if reverse {
                entries.reverse();
            }
            fixtures.push(entries);
        }
    }

    for mut entries in fixtures {
        entries.insert(0, entry("valid-first", EntryKind::File, b"must not be written"));
        let root = TempDir::new();
        assert!(default_test_files(root.path().to_str().unwrap(), &entries).is_err());
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 0);
    }
}

#[test]
fn refuses_nonempty_or_symlink_roots_without_overwriting() {
    let root = TempDir::new();
    let file = root.path().join("sentinel");
    fs::write(&file, b"preserve").unwrap();
    let entries = [entry("sentinel", EntryKind::File, b"overwrite")];
    assert!(default_test_files(root.path().to_str().unwrap(), &entries).is_err());
    assert_eq!(fs::read(&file).unwrap(), b"preserve");
    #[cfg(unix)]
    {
        let holder = TempDir::new();
        let link = holder.path().join("root-link");
        std::os::unix::fs::symlink(root.path(), &link).unwrap();
        assert!(default_test_files(link.to_str().unwrap(), &entries).is_err());
        assert_eq!(fs::read(&file).unwrap(), b"preserve");
    }
}

#[test]
fn writes_nested_files_and_broken_symlinks_with_mtime() {
    let root = TempDir::new();
    let mut file = entry("nested/file.bin", EntryKind::File, b"fixture");
    file.mtime_unix_secs = Some(1_700_000_000);
    let entries = vec![file, entry("broken-link", EntryKind::Symlink, b"missing-target")];

    default_test_files(root.path().to_str().unwrap(), &entries).unwrap();

    let file_path = root.path().join("nested/file.bin");
    assert_eq!(fs::read(&file_path).unwrap(), b"fixture");
    assert_eq!(
        filetime::FileTime::from_last_modification_time(&fs::metadata(file_path).unwrap()).unix_seconds(),
        1_700_000_000
    );
    #[cfg(unix)]
    assert_eq!(fs::read_link(root.path().join("broken-link")).unwrap(), PathBuf::from("missing-target"));
    #[cfg(not(unix))]
    assert_eq!(fs::read(root.path().join("broken-link")).unwrap(), b"");
}

#[test]
fn case_aliases_never_overwrite_or_escape() {
    let root = TempDir::new();
    let entries = [entry("Case", EntryKind::File, b"one"), entry("case", EntryKind::File, b"two")];
    let result = default_test_files(root.path().to_str().unwrap(), &entries);
    assert_eq!(fs::read(root.path().join("Case")).unwrap(), b"one");
    if result.is_ok() {
        assert_eq!(fs::read(root.path().join("case")).unwrap(), b"two");
    }
    let outside = TempDir::new();
    let root = TempDir::new();
    let entries = [
        entry("Link", EntryKind::Symlink, outside.path().to_str().unwrap().as_bytes()),
        entry("link/payload", EntryKind::File, b"bad"),
    ];
    let result = default_test_files(root.path().to_str().unwrap(), &entries);
    assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 0);
    if fs::symlink_metadata(root.path().join("Link")).unwrap().is_dir() {
        assert!(result.is_err());
    }
}

#[cfg(unix)]
#[test]
fn aliased_symlink_parent_never_redirects_another_symlink() {
    let root = TempDir::new();
    let outside = TempDir::new();
    let entries = [
        entry("Link", EntryKind::Symlink, outside.path().to_str().unwrap().as_bytes()),
        entry("link/child", EntryKind::Symlink, b"missing-target"),
    ];
    let result = default_test_files(root.path().to_str().unwrap(), &entries);
    assert_eq!(fs::read_dir(outside.path()).unwrap().count(), 0);
    if root.path().join("link").is_symlink() {
        assert!(result.is_err());
    }
}
