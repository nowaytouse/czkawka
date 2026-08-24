pub(crate) mod connect;

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use czkawka_core::common::config_cache_path::get_config_cache_path;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtectedFiles {
    #[serde(default)]
    pub files: HashSet<PathBuf>,
    #[serde(skip)]
    storage_error: Option<String>,
}

impl ProtectedFiles {
    fn get_config_file() -> Result<PathBuf, String> {
        get_config_cache_path()
            .map(|paths| paths.config_folder.join("protected_files.json"))
            .ok_or_else(|| "Cannot get the protected files configuration path".to_string())
    }

    pub fn load() -> Self {
        match Self::get_config_file() {
            Ok(config_file) => Self::load_from(&config_file),
            Err(error) => Self::unavailable(error),
        }
    }

    fn load_from(config_file: &Path) -> Self {
        let backup_file = backup_path(config_file);
        let mut errors = Vec::new();
        for candidate in [config_file, backup_file.as_path()] {
            if !candidate.exists() {
                continue;
            }
            match fs::read_to_string(candidate)
                .map_err(|error| error.to_string())
                .and_then(|content| serde_json::from_str::<Self>(&content).map_err(|error| error.to_string()))
            {
                Ok(mut data) => {
                    data.storage_error = None;
                    debug!("Loaded {} protected files from {:?}", data.files.len(), candidate);
                    return data;
                }
                Err(error) => errors.push(format!("{}: {error}", candidate.display())),
            }
        }

        if errors.is_empty() {
            Self::default()
        } else {
            Self::unavailable(format!("Failed to load the protected files list: {}", errors.join("; ")))
        }
    }

    fn unavailable(error_message: String) -> Self {
        error!("{error_message}");
        Self {
            files: HashSet::new(),
            storage_error: Some(error_message),
        }
    }

    fn save_to(&self, config_file: &Path) -> Result<(), String> {
        let parent = config_file
            .parent()
            .ok_or_else(|| format!("Protected files path has no parent: {}", config_file.display()))?;
        fs::create_dir_all(parent).map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;

        let content = serde_json::to_vec_pretty(self).map_err(|error| format!("Failed to serialize protected files: {error}"))?;
        let temporary_file = temporary_path(config_file);
        let backup_file = backup_path(config_file);

        let mut output = File::create(&temporary_file).map_err(|error| format!("Failed to create {}: {error}", temporary_file.display()))?;
        output
            .write_all(&content)
            .and_then(|()| output.sync_all())
            .map_err(|error| format!("Failed to write {}: {error}", temporary_file.display()))?;

        if backup_file.exists() {
            fs::remove_file(&backup_file).map_err(|error| format!("Failed to remove stale backup {}: {error}", backup_file.display()))?;
        }
        if config_file.exists() {
            fs::rename(config_file, &backup_file).map_err(|error| format!("Failed to back up {}: {error}", config_file.display()))?;
        }

        if let Err(error) = fs::rename(&temporary_file, config_file) {
            if backup_file.exists()
                && let Err(restore_error) = fs::rename(&backup_file, config_file)
            {
                error!("Failed to restore protected files backup {}: {restore_error}", backup_file.display());
            }
            let _ = fs::remove_file(&temporary_file);
            return Err(format!("Failed to replace {}: {error}", config_file.display()));
        }

        if backup_file.exists()
            && let Err(error) = fs::remove_file(&backup_file)
        {
            warn!("Failed to remove protected files backup {}: {error}", backup_file.display());
        }
        debug!("Saved {} protected files to {:?}", self.files.len(), config_file);
        Ok(())
    }

    fn commit_files(&mut self, new_files: HashSet<PathBuf>) -> Result<(), String> {
        let config_file = Self::get_config_file()?;
        self.commit_files_to(new_files, &config_file)
    }

    fn commit_files_to(&mut self, new_files: HashSet<PathBuf>, config_file: &Path) -> Result<(), String> {
        self.ensure_available()?;
        let old_files = std::mem::replace(&mut self.files, new_files);
        if let Err(error) = self.save_to(config_file) {
            self.files = old_files;
            self.storage_error = Some(error.clone());
            return Err(error);
        }
        Ok(())
    }

    pub fn protect(&mut self, paths: impl IntoIterator<Item = PathBuf>) -> Result<usize, String> {
        self.ensure_available()?;
        let mut new_files = self.files.clone();
        let old_count = new_files.len();
        new_files.extend(paths);
        let added = new_files.len() - old_count;
        if added > 0 {
            self.commit_files(new_files)?;
        }
        Ok(added)
    }

    pub fn unprotect(&mut self, paths: impl IntoIterator<Item = PathBuf>) -> Result<usize, String> {
        self.ensure_available()?;
        let mut new_files = self.files.clone();
        let old_count = new_files.len();
        for path in paths {
            new_files.remove(&path);
        }
        let removed = old_count - new_files.len();
        if removed > 0 {
            self.commit_files(new_files)?;
        }
        Ok(removed)
    }

    pub fn toggle(&mut self, path: PathBuf) -> Result<bool, String> {
        self.ensure_available()?;
        let mut new_files = self.files.clone();
        let now_protected = if new_files.remove(&path) {
            false
        } else {
            new_files.insert(path);
            true
        };
        self.commit_files(new_files)?;
        Ok(now_protected)
    }

    pub fn clear(&mut self) -> Result<usize, String> {
        self.ensure_available()?;
        let count = self.files.len();
        if count > 0 {
            self.commit_files(HashSet::new())?;
        }
        Ok(count)
    }

    pub fn count(&self) -> usize {
        self.files.len()
    }

    pub fn storage_error(&self) -> Option<&str> {
        self.storage_error.as_deref()
    }

    fn ensure_available(&self) -> Result<(), String> {
        self.storage_error.clone().map_or(Ok(()), Err)
    }
}

fn temporary_path(config_file: &Path) -> PathBuf {
    config_file.with_extension("json.tmp")
}

fn backup_path(config_file: &Path) -> PathBuf {
    config_file.with_extension("json.bak")
}

pub static PROTECTED_FILES: LazyLock<Mutex<ProtectedFiles>> = LazyLock::new(|| {
    let protected_files = ProtectedFiles::load();
    info!("Loaded {} protected files", protected_files.count());
    Mutex::new(protected_files)
});

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::PathBuf;
    use std::{env, fs};

    use super::{ProtectedFiles, backup_path};

    #[test]
    fn protected_files_persistence_is_recoverable_and_fail_closed() {
        let temp_dir = env::temp_dir().join(format!("krokiet-protected-files-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        let config_file = temp_dir.join("protected_files.json");
        let original = ProtectedFiles {
            files: HashSet::from([PathBuf::from("/protected")]),
            storage_error: None,
        };
        original.save_to(&config_file).unwrap();
        assert_eq!(ProtectedFiles::load_from(&config_file).files, original.files);

        fs::copy(&config_file, backup_path(&config_file)).unwrap();
        fs::write(&config_file, "not json").unwrap();
        assert_eq!(ProtectedFiles::load_from(&config_file).files, original.files);

        fs::remove_file(backup_path(&config_file)).unwrap();
        let unavailable = ProtectedFiles::load_from(&config_file);
        assert!(unavailable.storage_error().is_some());

        let blocker = temp_dir.join("not-a-directory");
        fs::write(&blocker, "file").unwrap();
        let mut protected_files = original.clone();
        let changed = HashSet::from([PathBuf::from("/replacement")]);
        assert!(protected_files.commit_files_to(changed, &blocker.join("protected_files.json")).is_err());
        assert_eq!(protected_files.files, original.files);
        fs::remove_dir_all(temp_dir).unwrap();
    }
}
