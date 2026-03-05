pub(crate) mod connect;

use std::collections::HashSet;
use std::path::PathBuf;

use czkawka_core::common::config_cache_path::get_config_cache_path;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtectedFiles {
    #[serde(default)]
    pub files: HashSet<PathBuf>,
}

impl ProtectedFiles {
    fn get_config_file() -> Option<PathBuf> {
        let config_folder = get_config_cache_path()?.config_folder;
        Some(config_folder.join("protected_files.json"))
    }

    pub fn load() -> Self {
        let Some(config_file) = Self::get_config_file() else {
            error!("Cannot get config file path for protected files");
            return Self::default();
        };
        if !config_file.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(&config_file) {
            Ok(content) => match serde_json::from_str::<ProtectedFiles>(&content) {
                Ok(data) => {
                    debug!("Loaded {} protected files from {:?}", data.files.len(), config_file);
                    data
                }
                Err(e) => {
                    error!("Failed to parse protected files from {:?}: {}", config_file, e);
                    Self::default()
                }
            },
            Err(e) => {
                error!("Failed to read protected files from {:?}: {}", config_file, e);
                Self::default()
            }
        }
    }

    pub fn save(&self) {
        let Some(config_file) = Self::get_config_file() else {
            error!("Cannot get config file path for protected files");
            return;
        };
        if let Some(parent) = config_file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = std::fs::write(&config_file, content) {
                    error!("Failed to write protected files to {:?}: {}", config_file, e);
                } else {
                    debug!("Saved {} protected files to {:?}", self.files.len(), config_file);
                }
            }
            Err(e) => {
                error!("Failed to serialize protected files: {}", e);
            }
        }
    }

    pub fn clear(&mut self) {
        self.files.clear();
        self.save();
    }

    pub fn count(&self) -> usize {
        self.files.len()
    }
}

use std::sync::{LazyLock, Mutex};

pub static PROTECTED_FILES: LazyLock<Mutex<ProtectedFiles>> = LazyLock::new(|| {
    let pf = ProtectedFiles::load();
    info!("Loaded {} protected files", pf.count());
    Mutex::new(pf)
});
