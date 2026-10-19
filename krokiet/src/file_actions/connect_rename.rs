use std::path::{MAIN_SEPARATOR, Path};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use crossbeam_channel::Sender;
use czkawka_core::common::progress_data::ProgressData;
use log::{error, info, warn};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel, Weak};

use crate::common::StrDataBadNames;
use crate::file_protection::connect::is_file_protected;
use crate::model_operations::model_processor::{MessageType, ModelProcessor, ProcessFunction};
use crate::simpler_model::{SimplerSingleMainListModel, ToSimplerVec};
use crate::{ActiveTab, Callabler, GuiState, MainWindow, RenameFileRequest};

pub(crate) fn connect_rename(app: &MainWindow, progress_sender: Sender<ProgressData>, stop_flag: Arc<AtomicBool>) {
    let a = app.as_weak();
    app.global::<Callabler>().on_rename_files(move || {
        let weak_app = a.clone();
        let progress_sender = progress_sender.clone();
        let stop_flag = stop_flag.clone();
        stop_flag.store(false, Ordering::Relaxed);
        let app = a.upgrade().expect("Failed to upgrade app :(");
        let active_tab = app.global::<GuiState>().get_active_tab();

        let processor = ModelProcessor::new(active_tab);
        match active_tab {
            ActiveTab::BadExtensions => {
                processor.rename_bad_extensions(progress_sender, weak_app, stop_flag);
            }
            ActiveTab::BadNames => {
                processor.rename_bad_file_names(progress_sender, weak_app, stop_flag);
            }
            _ => panic!("{active_tab:?} is not supported for renaming bad extensions/bad file names"),
        }
    });

    connect_rename_single_file(app);
}

// Issue #162: arbitrary per-row rename to a user-supplied name.
// Unlike `rename_files` (a batch confirm over checked items in BadExtensions/BadNames),
// this renames a single row to any name and updates the row in place - the file stays in its group.
fn connect_rename_single_file(app: &MainWindow) {
    let a = app.as_weak();
    app.global::<Callabler>().on_rename_single_file(move |request| {
        let RenameFileRequest { row_idx: idx, new_name } = request;
        let app = a.upgrade().expect("Failed to upgrade app :(");
        let active_tab = app.global::<GuiState>().get_active_tab();
        let model = active_tab.get_tool_model(&app);
        let path_idx = active_tab.get_str_path_idx();
        let name_idx = active_tab.get_str_name_idx();

        let report_failure = |app: &MainWindow, msg: String| {
            app.invoke_show_error_popup(msg.into());
        };

        let new_name = new_name.trim();
        if !is_valid_rename_target_name(new_name) {
            warn!("Single rename rejected (row {idx}): invalid name {new_name:?}");
            report_failure(&app, crate::flk!("rust_rename_single_invalid_name"));
            return;
        }

        let row = model
            .row_data(idx as usize)
            .unwrap_or_else(|| panic!("Single rename: no row at index {idx} (model has {} rows)", model.row_count()));
        if row.header_row {
            warn!("Single rename aborted: row {idx} is a header row");
            report_failure(&app, crate::flk!("rust_rename_single_invalid_name"));
            return;
        }
        let folder = row.val_str.iter().nth(path_idx).map(|s| s.to_string()).unwrap_or_default();
        let old_name = row.val_str.iter().nth(name_idx).map(|s| s.to_string()).unwrap_or_default();
        if new_name == old_name.as_str() {
            warn!("Single rename skipped (row {idx}): new name equals current name {old_name:?}");
            report_failure(&app, crate::flk!("rust_rename_single_same_name"));
            return;
        }

        let old_full_path = build_full_path(&folder, &old_name);
        let new_full_path = build_full_path(&folder, new_name);

        if is_single_rename_protected(row.protected, &old_full_path) {
            warn!("Single rename rejected: source {old_full_path:?} is protected");
            report_failure(&app, crate::flk!("rust_rename_single_protected"));
            return;
        }
        if let Err(e) = checked_rename(Path::new(&old_full_path), Path::new(&new_full_path)) {
            error!("Failed to rename {old_full_path:?} to {new_full_path:?}: {e}");
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                report_failure(&app, crate::flk!("rust_rename_single_target_exists"));
                return;
            }
            report_failure(
                &app,
                crate::flk!("rust_failed_to_rename_file", old_path = old_full_path, new_path = new_full_path, error = e.to_string()),
            );
            return;
        }
        info!("Renamed {old_full_path:?} to {new_full_path:?}");

        // Update the Name cell in place - keeps the file in its duplicate/similar group.
        let new_val_str: Vec<SharedString> = row.val_str.iter().enumerate().map(|(i, s)| if i == name_idx { new_name.into() } else { s }).collect();
        let mut row = row;
        row.val_str = ModelRc::new(VecModel::from(new_val_str));
        model.set_row_data(idx as usize, row);

        app.global::<GuiState>()
            .set_info_text(crate::flk!("rust_renamed_file", old_name = old_name, new_name = new_name.to_string()).into());
    });

    // Live check used by the rename popup to disable OK and show a warning when the new name
    // already exists on disk. Returns false for empty/invalid/unchanged names (those are handled
    // by separate warnings), so it only reports a genuine pre-existing target.
    let a = app.as_weak();
    app.global::<Callabler>().on_rename_target_exists(move |request| {
        let RenameFileRequest { row_idx: idx, new_name } = request;
        let app = a.upgrade().expect("Failed to upgrade app :(");
        let active_tab = app.global::<GuiState>().get_active_tab();
        let model = active_tab.get_tool_model(&app);
        let path_idx = active_tab.get_str_path_idx();
        let name_idx = active_tab.get_str_name_idx();

        let new_name = new_name.trim();
        if !is_valid_rename_target_name(new_name) {
            return false;
        }

        let row = model
            .row_data(idx as usize)
            .unwrap_or_else(|| panic!("Rename target check: no row at index {idx} (model has {} rows)", model.row_count()));
        if row.header_row {
            return false;
        }
        let folder = row.val_str.iter().nth(path_idx).map(|s| s.to_string()).unwrap_or_default();
        let old_name = row.val_str.iter().nth(name_idx).map(|s| s.to_string()).unwrap_or_default();
        if new_name == old_name.as_str() {
            return false;
        }

        let new_full_path = build_full_path(&folder, new_name);
        match std::fs::symlink_metadata(&new_full_path) {
            Ok(_) => true,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => {
                warn!("Cannot inspect rename target {new_full_path:?}: {error}");
                true
            }
        }
    });
}

fn is_valid_rename_target_name(new_name: &str) -> bool {
    !matches!(new_name, "" | "." | "..") && !new_name.contains(['/', '\\', '\0'])
}

fn checked_rename(source: &Path, target: &Path) -> std::io::Result<()> {
    match std::fs::symlink_metadata(target) {
        Ok(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("Rename target already exists: {}", target.display()),
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(std::io::Error::new(error.kind(), format!("Cannot inspect rename target {}: {error}", target.display()))),
    }
    // shortcut: external changes can race this check; use atomic no-replace rename once supported by the Rust minimum.
    std::fs::rename(source, target)
}

fn build_full_path(folder: &str, name: &str) -> String {
    if folder.is_empty() {
        name.to_string()
    } else {
        format!("{folder}{MAIN_SEPARATOR}{name}")
    }
}

fn is_single_rename_protected(row_protected: bool, full_path: &str) -> bool {
    row_protected || is_file_protected(full_path)
}

impl ModelProcessor {
    fn rename_bad_extensions(self, progress_sender: Sender<ProgressData>, weak_app: Weak<MainWindow>, stop_flag: Arc<AtomicBool>) {
        let model = self.active_tab.get_tool_model(&weak_app.upgrade().expect("Failed to upgrade app :("));
        let simpler_model = model.to_simpler_enumerated_vec();
        thread::spawn(move || {
            let path_idx = self.active_tab.get_str_path_idx();
            let name_idx = self.active_tab.get_str_name_idx();
            let ext_idx = self.active_tab.get_str_proper_extension();

            let rm_fnc = move |data: &SimplerSingleMainListModel| {
                let full_path = format!("{}{}{}", data.val_str[path_idx], std::path::MAIN_SEPARATOR, data.val_str[name_idx]);
                if is_file_protected(&full_path) {
                    return Err(format!("File is protected: {full_path}"));
                }
                rename_single_extension_item(data, path_idx, name_idx, ext_idx)
            };

            self.process_and_update_gui_state(
                &weak_app,
                stop_flag,
                &progress_sender,
                simpler_model,
                &ProcessFunction::Simple(Box::new(rm_fnc)),
                MessageType::Rename,
                true,
            );
        });
    }
    fn rename_bad_file_names(self, progress_sender: Sender<ProgressData>, weak_app: Weak<MainWindow>, stop_flag: Arc<AtomicBool>) {
        let model = self.active_tab.get_tool_model(&weak_app.upgrade().expect("Failed to upgrade app :("));
        let simpler_model = model.to_simpler_enumerated_vec();
        thread::spawn(move || {
            let path_idx = self.active_tab.get_str_path_idx();
            let name_idx = self.active_tab.get_str_name_idx();
            let new_name_idx = StrDataBadNames::NewName as usize;

            let rm_fnc = move |data: &SimplerSingleMainListModel| {
                let full_path = format!("{}{}{}", data.val_str[path_idx], std::path::MAIN_SEPARATOR, data.val_str[name_idx]);
                if is_file_protected(&full_path) {
                    return Err(format!("File is protected: {full_path}"));
                }
                rename_single_file_name_item(data, path_idx, name_idx, new_name_idx)
            };

            self.process_and_update_gui_state(
                &weak_app,
                stop_flag,
                &progress_sender,
                simpler_model,
                &ProcessFunction::Simple(Box::new(rm_fnc)),
                MessageType::Rename,
                true,
            );
        });
    }
}

fn rename_single_file_name_item(data: &SimplerSingleMainListModel, path_idx: usize, name_idx: usize, new_file_name_idx: usize) -> Result<(), String> {
    use std::path::MAIN_SEPARATOR;
    let folder = &data.val_str[path_idx];
    let file_name = &data.val_str[name_idx];
    let new_file_name = &data.val_str[new_file_name_idx];

    let new_full_path = format!("{folder}{MAIN_SEPARATOR}{new_file_name}");
    let old_full_path = format!("{folder}{MAIN_SEPARATOR}{file_name}");

    if let Err(e) = checked_rename(Path::new(&old_full_path), Path::new(&new_full_path)) {
        Err(crate::flk!(
            "rust_failed_to_rename_file",
            old_path = old_full_path,
            new_path = new_full_path,
            error = e.to_string()
        ))
    } else {
        Ok(())
    }
}

fn rename_single_extension_item(data: &SimplerSingleMainListModel, path_idx: usize, name_idx: usize, ext_idx: usize) -> Result<(), String> {
    use std::path::MAIN_SEPARATOR;
    let folder = &data.val_str[path_idx];
    let file_name = &data.val_str[name_idx];
    let new_extension = &data.val_str[ext_idx];

    let file_stem = std::path::Path::new(&file_name).file_stem().map(|e| e.to_string_lossy().to_string()).unwrap_or_default();
    let new_full_path = format!("{folder}{MAIN_SEPARATOR}{file_stem}.{new_extension}");
    let old_full_path = format!("{folder}{MAIN_SEPARATOR}{file_name}");

    if let Err(e) = checked_rename(Path::new(&old_full_path), Path::new(&new_full_path)) {
        Err(crate::flk!(
            "rust_failed_to_rename_file",
            old_path = old_full_path,
            new_path = new_full_path,
            error = e.to_string()
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::AtomicU64;

    use crossbeam_channel::{Receiver, unbounded};
    use slint::{Model, ModelRc, VecModel};

    use super::*;
    use crate::common::create_model_from_model_vec;
    use crate::simpler_model::ToSlintModel;
    use crate::test_common::get_model_vec;
    use crate::{ActiveTab, SingleMainListModel};

    impl ModelProcessor {
        pub(crate) fn process_rename_test(
            &self,
            progress_sender: Sender<ProgressData>,
            model: ModelRc<SingleMainListModel>,
        ) -> Option<(Vec<SingleMainListModel>, Vec<String>, usize, usize)> {
            let items_queued_to_delete = model.iter().filter(|e| e.checked).count();
            if items_queued_to_delete == 0 {
                return None; // No items to delete
            }
            let simplified_model = model.to_simpler_enumerated_vec();

            let rm_fnc = |data: &SimplerSingleMainListModel| {
                if data.val_str[0].contains("test_error") {
                    Err(format!("Test error for item: {}", data.val_str[0]))
                } else {
                    Ok(())
                }
            };

            let output = Self::process_items(
                simplified_model,
                items_queued_to_delete,
                progress_sender,
                &Arc::default(),
                &ProcessFunction::Simple(Box::new(rm_fnc)),
                MessageType::Rename,
                self.active_tab.get_int_size_opt_idx(),
                false,
            );

            let (new_simple_model, errors, items_deleted) = Self::remove_processed_items_from_model(output);

            Some((new_simple_model.to_vec_model(), errors, items_queued_to_delete, items_deleted))
        }
    }

    #[test]
    fn test_no_rename_items() {
        let (progress, _receiver): (Sender<ProgressData>, Receiver<ProgressData>) = unbounded();
        let model = get_model_vec(10);
        let model = create_model_from_model_vec(&model);
        let processor = ModelProcessor::new(ActiveTab::EmptyFolders);
        assert!(processor.process_rename_test(progress, model).is_none());
    }

    #[test]
    fn test_rename_bad_extensions() {
        let (progress, _receiver): (Sender<ProgressData>, Receiver<ProgressData>) = unbounded();
        let mut model = get_model_vec(10);
        model[0].checked = true;
        model[0].val_str = ModelRc::new(VecModel::from(vec!["normal1".to_string().into(); 10]));
        model[1].checked = true;
        model[1].val_str = ModelRc::new(VecModel::from(vec!["normal2".to_string().into(); 10]));
        model[3].checked = true;
        model[3].val_str = ModelRc::new(VecModel::from(vec!["test_error".to_string().into(); 10]));
        let model = create_model_from_model_vec(&model);
        let processor = ModelProcessor::new(ActiveTab::EmptyFolders);
        let (new_model, errors, items_queued_to_delete, items_deleted) = processor.process_rename_test(progress, model).unwrap();

        assert_eq!(new_model.len(), 8);
        assert_eq!(errors.len(), 1);
        assert_eq!(items_queued_to_delete, 3);
        assert_eq!(items_deleted, 2);

        assert!(new_model[1].checked);
        assert!(new_model[1].val_str.iter().all(|s| s == "test_error"));
        assert!(!new_model[0].checked);
        assert!(new_model.iter().skip(2).all(|model| !model.checked));
    }

    #[test]
    fn test_is_valid_rename_target_name() {
        assert!(is_valid_rename_target_name("new_name.txt"));
        assert!(!is_valid_rename_target_name(""));
        assert!(!is_valid_rename_target_name("has/slash"));
        assert!(!is_valid_rename_target_name("has\\backslash"));
        assert!(!is_valid_rename_target_name("."));
        assert!(!is_valid_rename_target_name(".."));
        assert!(!is_valid_rename_target_name("has\0nul"));
    }

    #[test]
    fn test_build_full_path_empty_folder() {
        assert_eq!(build_full_path("", "file.txt"), "file.txt");
    }

    #[test]
    fn test_build_full_path_with_folder() {
        assert_eq!(build_full_path("/home/user", "file.txt"), format!("/home/user{MAIN_SEPARATOR}file.txt"));
    }

    #[test]
    fn single_rename_rejects_protected_row() {
        assert!(is_single_rename_protected(true, "/does/not/need/to/exist"));
    }

    fn rename_fixture() -> (std::path::PathBuf, SimplerSingleMainListModel) {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "krokiet_rename_{}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("source.bad"), b"source bytes").unwrap();
        let mut data = SimplerSingleMainListModel::from(&get_model_vec(1)[0]);
        data.val_str = vec![dir.to_str().unwrap().to_owned(), "source.bad".into(), "source.jpg".into(), "jpg".into()];
        (dir, data)
    }

    #[test]
    fn batch_rename_preserves_existing_target() {
        let (dir, data) = rename_fixture();
        fs::write(dir.join("source.jpg"), b"existing bytes").unwrap();
        let result = rename_single_file_name_item(&data, 0, 1, 2);
        let source = fs::read(dir.join("source.bad"));
        let destination = fs::read(dir.join("source.jpg")).unwrap();
        fs::remove_dir_all(&dir).unwrap();
        assert!(result.is_err());
        assert_eq!(source.unwrap(), b"source bytes");
        assert_eq!(destination, b"existing bytes");
    }

    #[test]
    fn extension_rename_preserves_existing_target() {
        let (dir, data) = rename_fixture();
        fs::write(dir.join("source.jpg"), b"existing bytes").unwrap();
        let result = rename_single_extension_item(&data, 0, 1, 3);
        let source = fs::read(dir.join("source.bad"));
        let destination = fs::read(dir.join("source.jpg")).unwrap();
        fs::remove_dir_all(&dir).unwrap();
        assert!(result.is_err());
        assert_eq!(source.unwrap(), b"source bytes");
        assert_eq!(destination, b"existing bytes");
    }

    #[test]
    fn batch_rename_moves_source_to_free_target() {
        let (dir, data) = rename_fixture();
        rename_single_file_name_item(&data, 0, 1, 2).unwrap();
        assert!(!dir.join("source.bad").exists());
        assert_eq!(fs::read(dir.join("source.jpg")).unwrap(), b"source bytes");
        fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn batch_rename_preserves_dangling_symlink_target() {
        let (dir, data) = rename_fixture();
        std::os::unix::fs::symlink("missing-file", dir.join("source.jpg")).unwrap();
        let result = rename_single_extension_item(&data, 0, 1, 3);
        assert!(result.is_err());
        assert_eq!(fs::read_link(dir.join("source.jpg")).unwrap(), Path::new("missing-file"));
        assert_eq!(fs::read(dir.join("source.bad")).unwrap(), b"source bytes");
        fs::remove_dir_all(dir).unwrap();
    }
}
