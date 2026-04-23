use std::path::{MAIN_SEPARATOR, PathBuf};

use log::info;
use slint::{ComponentHandle, Model};

use crate::file_protection::PROTECTED_FILES;
use crate::{ActiveTab, Callabler, GuiState, MainWindow};

pub(crate) fn connect_file_protection(app: &MainWindow) {
    // Initialize the protected files count in GUI
    {
        let pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
        app.global::<GuiState>().set_protected_files_count(pf.count() as i32);
    }

    connect_protect(app);
    connect_unprotect(app);
    connect_clear_all(app);
    connect_filter_after_scan(app);
}

fn connect_protect(app: &MainWindow) {
    let a = app.as_weak();
    app.global::<Callabler>().on_protect_selected_items(move || {
        let app = a.upgrade().expect("Failed to upgrade app :(");
        let active_tab = app.global::<GuiState>().get_active_tab();
        let model = active_tab.get_tool_model(&app);

        let path_idx = active_tab.get_str_path_idx();
        let name_idx = active_tab.get_str_name_idx();

        let mut pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
        let mut protected_count = 0;

        // Collect paths of checked (selected) items and protect them
        for idx in 0..model.row_count() {
            if let Some(item) = model.row_data(idx)
                && item.checked
                && !item.header_row
            {
                let val_str: Vec<String> = item.val_str.iter().map(|s| s.to_string()).collect();
                if let (Some(path), Some(name)) = (val_str.get(path_idx), val_str.get(name_idx)) {
                    let full_path = PathBuf::from(format!("{path}{MAIN_SEPARATOR}{name}"));
                    if pf.files.insert(full_path) {
                        protected_count += 1;
                    }
                }
            }
        }

        if protected_count > 0 {
            pf.save();
            info!("Protected {} files, total: {}", protected_count, pf.count());
        }

        // Remove protected items from the model
        remove_protected_from_model(&app, active_tab, &pf.files);

        app.global::<GuiState>().set_protected_files_count(pf.count() as i32);
        let info = format!("Protected {} files (total protected: {})", protected_count, pf.count());
        app.global::<GuiState>().set_info_text(info.into());
    });
}

fn connect_unprotect(app: &MainWindow) {
    let a = app.as_weak();
    app.global::<Callabler>().on_unprotect_selected_items(move || {
        let app = a.upgrade().expect("Failed to upgrade app :(");
        let active_tab = app.global::<GuiState>().get_active_tab();
        let model = active_tab.get_tool_model(&app);

        let path_idx = active_tab.get_str_path_idx();
        let name_idx = active_tab.get_str_name_idx();

        let mut pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
        let mut unprotected_count = 0;

        for idx in 0..model.row_count() {
            if let Some(item) = model.row_data(idx)
                && item.checked
                && !item.header_row
            {
                let val_str: Vec<String> = item.val_str.iter().map(|s| s.to_string()).collect();
                if let (Some(path), Some(name)) = (val_str.get(path_idx), val_str.get(name_idx)) {
                    let full_path = PathBuf::from(format!("{path}{MAIN_SEPARATOR}{name}"));
                    if pf.files.remove(&full_path) {
                        unprotected_count += 1;
                    }
                }
            }
        }

        if unprotected_count > 0 {
            pf.save();
            info!("Unprotected {} files, total: {}", unprotected_count, pf.count());
        }

        app.global::<GuiState>().set_protected_files_count(pf.count() as i32);
        let info = format!("Unprotected {} files (total protected: {})", unprotected_count, pf.count());
        app.global::<GuiState>().set_info_text(info.into());
    });
}

fn connect_clear_all(app: &MainWindow) {
    let a = app.as_weak();
    app.global::<Callabler>().on_clear_all_protected_files(move || {
        let app = a.upgrade().expect("Failed to upgrade app :(");
        let mut pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
        let count = pf.count();
        pf.clear();
        info!("Cleared all {count} protected files");

        app.global::<GuiState>().set_protected_files_count(0);
        let info = format!("Cleared all {count} protected files");
        app.global::<GuiState>().set_info_text(info.into());
    });
}

fn connect_filter_after_scan(app: &MainWindow) {
    let a = app.as_weak();
    app.global::<Callabler>().on_filter_protected_files_after_scan(move || {
        let app = a.upgrade().expect("Failed to upgrade app :(");
        let active_tab = app.global::<GuiState>().get_active_tab();
        let pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
        if !pf.files.is_empty() {
            remove_protected_from_model(&app, active_tab, &pf.files);
            info!("Filtered protected files from scan results for {active_tab:?}");
        }
    });
}

/// Check if a file path is protected. Used as safety net in delete/move operations.
pub fn is_file_protected(path: &str) -> bool {
    let pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
    pf.files.contains(&PathBuf::from(path))
}

/// Remove protected files from the current model.
/// This also removes groups that become empty or single-item after filtering.
pub(crate) fn remove_protected_from_model(app: &MainWindow, active_tab: ActiveTab, protected: &std::collections::HashSet<PathBuf>) {
    if protected.is_empty() {
        return;
    }

    let model = active_tab.get_tool_model(app);
    let path_idx = active_tab.get_str_path_idx();
    let name_idx = active_tab.get_str_name_idx();
    let has_headers = active_tab.get_is_header_mode();

    let items: Vec<_> = model.iter().collect();

    if has_headers {
        // Group-based filtering: collect groups, filter protected items, remove empty/single groups
        let mut groups: Vec<Vec<crate::SingleMainListModel>> = Vec::new();
        let mut current_group: Vec<crate::SingleMainListModel> = Vec::new();

        for item in &items {
            if item.header_row && !current_group.is_empty() {
                groups.push(std::mem::take(&mut current_group));
            }
            current_group.push(item.clone());
        }
        if !current_group.is_empty() {
            groups.push(current_group);
        }

        let mut new_items = Vec::new();
        for group in groups {
            let (headers, data_items): (Vec<_>, Vec<_>) = group.into_iter().partition(|i| i.header_row);
            let filtered: Vec<_> = data_items
                .into_iter()
                .filter(|item| {
                    let val_str: Vec<String> = item.val_str.iter().map(|s| s.to_string()).collect();
                    if let (Some(path), Some(name)) = (val_str.get(path_idx), val_str.get(name_idx)) {
                        let full_path = PathBuf::from(format!("{path}{MAIN_SEPARATOR}{name}"));
                        !protected.contains(&full_path)
                    } else {
                        true
                    }
                })
                .collect();

            // Keep the group only if it has at least 2 data items
            if filtered.len() >= 2 {
                new_items.extend(headers);
                new_items.extend(filtered);
            }
        }

        let new_model = slint::ModelRc::new(slint::VecModel::from(new_items));
        active_tab.set_tool_model(app, new_model);
    } else {
        // Simple filtering without groups
        let filtered: Vec<_> = items
            .into_iter()
            .filter(|item| {
                if item.header_row {
                    return true;
                }
                let val_str: Vec<String> = item.val_str.iter().map(|s| s.to_string()).collect();
                if let (Some(path), Some(name)) = (val_str.get(path_idx), val_str.get(name_idx)) {
                    let full_path = PathBuf::from(format!("{path}{MAIN_SEPARATOR}{name}"));
                    !protected.contains(&full_path)
                } else {
                    true
                }
            })
            .collect();

        let new_model = slint::ModelRc::new(slint::VecModel::from(filtered));
        active_tab.set_tool_model(app, new_model);
    }
}
