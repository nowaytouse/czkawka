use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Private temp directory of one scenario, removed on drop
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    pub fn new() -> Result<Self, String> {
        for _ in 0..100 {
            let id = COUNTER.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!("czkawka_e2e_{}_{id}", std::process::id()));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            match builder.create(&root) {
                Ok(()) => return Ok(Self { root }),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(format!("cannot create {}: {e}", root.display())),
            }
        }
        Err("cannot create a unique workspace after 100 attempts".to_string())
    }

    /// Fake HOME, so the app never touches real settings
    pub fn home(&self) -> PathBuf {
        self.root.join("home")
    }

    pub fn config_file(&self) -> PathBuf {
        self.home().join(".config/krokiet/config_general.json")
    }

    pub fn data_dir(&self) -> PathBuf {
        self.root.join("data")
    }

    pub fn log_file(&self) -> PathBuf {
        self.root.join("krokiet.log")
    }

    pub fn read_log(&self) -> Result<String, String> {
        let path = self.log_file();
        fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))
    }

    pub fn write_file(&self, relative: &str, content: &[u8]) -> Result<PathBuf, String> {
        if relative.is_empty() || !std::path::Path::new(relative).components().all(|c| matches!(c, std::path::Component::Normal(_))) {
            return Err("fixture path must be relative and stay within the workspace".to_string());
        }
        let path = self.data_dir().join(relative);
        let parent = path.parent().ok_or("file has no parent")?;
        fs::create_dir_all(parent).map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
        fs::write(&path, content).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspaces_are_distinct_and_private() {
        let first = Workspace::new().unwrap();
        let second = Workspace::new().unwrap();
        assert_ne!(first.root, second.root);
        assert!(first.write_file("../outside", b"unsafe").is_err());
        assert!(first.write_file("/absolute", b"unsafe").is_err());
        assert!(first.write_file("nested/fixture", b"synthetic").is_ok());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(fs::metadata(&first.root).unwrap().permissions().mode() & 0o077, 0);
        }
        let root = first.root.clone();
        drop(first);
        assert!(!root.exists());
        assert!(second.root.exists());
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).ok();
    }
}
