use std::path::{MAIN_SEPARATOR, PathBuf};

use log::info;
use slint::{ComponentHandle, Model};

use crate::connect_row_selection::checker::set_number_of_enabled_items;
use crate::connect_row_selection::reset_selection;
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
    let (new_items, checked_count) = filter_protected_items(items, path_idx, name_idx, has_headers, protected);

    let new_model = slint::ModelRc::new(slint::VecModel::from(new_items));
    active_tab.set_tool_model(app, new_model);
    // The model shrank; TOOLS_SELECTION still holds stale row indices/counts.
    // Reset it before any selection callback runs, or the assertions in
    // connect_row_selection.rs will panic on the next click/space/select-all.
    reset_selection(app, active_tab, true);
    set_number_of_enabled_items(app, active_tab, checked_count);
}

/// Pure filtering logic, split out so it can be unit-tested without a live `MainWindow`.
/// Returns the surviving rows and the number of still-checked data rows among them,
/// which the caller must feed back into the enabled-items counter to keep it in sync.
fn filter_protected_items(
    items: Vec<crate::SingleMainListModel>,
    path_idx: usize,
    name_idx: usize,
    has_headers: bool,
    protected: &std::collections::HashSet<PathBuf>,
) -> (Vec<crate::SingleMainListModel>, u64) {
    let is_protected = |item: &crate::SingleMainListModel| -> bool {
        let val_str: Vec<String> = item.val_str.iter().map(|s| s.to_string()).collect();
        if let (Some(path), Some(name)) = (val_str.get(path_idx), val_str.get(name_idx)) {
            let full_path = PathBuf::from(format!("{path}{MAIN_SEPARATOR}{name}"));
            protected.contains(&full_path)
        } else {
            false
        }
    };

    let new_items = if has_headers {
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
            let filtered: Vec<_> = data_items.into_iter().filter(|item| !is_protected(item)).collect();

            // Keep the group only if it has at least 2 data items
            if filtered.len() >= 2 {
                new_items.extend(headers);
                new_items.extend(filtered);
            }
        }
        new_items
    } else {
        // Simple filtering without groups
        items.into_iter().filter(|item| item.header_row || !is_protected(item)).collect()
    };

    let checked_count = new_items.iter().filter(|i| !i.header_row && i.checked).count() as u64;
    (new_items, checked_count)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::PathBuf;

    use slint::{Model, ModelRc, SharedString, VecModel};

    use super::{MAIN_SEPARATOR, filter_protected_items};
    use crate::SingleMainListModel;

    // Builds a row whose val_str places `path` at index 0 and `name` at index 1,
    // matching the (path_idx=0, name_idx=1) layout used by the tests below.
    fn row(path: &str, name: &str, checked: bool, header: bool) -> SingleMainListModel {
        let strs = vec![SharedString::from(path), SharedString::from(name)];
        SingleMainListModel {
            checked,
            header_row: header,
            val_str: ModelRc::new(VecModel::from(strs)),
            ..crate::test_common::get_main_list_model()
        }
    }

    fn protected_set(paths: &[&str]) -> HashSet<PathBuf> {
        paths.iter().map(PathBuf::from).collect()
    }

    fn full(path: &str, name: &str) -> String {
        format!("{path}{MAIN_SEPARATOR}{name}")
    }

    #[test]
    fn flat_mode_drops_protected_rows_and_recomputes_checked_count() {
        // 3 rows, all checked; the middle one is protected and must be removed.
        let items = vec![
            row("/a", "keep1.jpg", true, false),
            row("/a", "secret.jpg", true, false),
            row("/a", "keep2.jpg", true, false),
        ];
        let protected = protected_set(&[&full("/a", "secret.jpg")]);

        let (new_items, checked_count) = filter_protected_items(items, 0, 1, false, &protected);

        assert_eq!(new_items.len(), 2);
        // The returned count must equal the survivors' checked rows — this is exactly the
        // value that used to desync the enabled-items counter and crash selection.
        assert_eq!(checked_count, 2);
        assert!(new_items.iter().all(|i| i.val_str.iter().nth(1).unwrap() != "secret.jpg"));
    }

    #[test]
    fn header_mode_drops_group_that_shrinks_below_two_items() {
        // Group 1: header + 2 data rows, one protected → 1 survivor → whole group dropped.
        // Group 2: header + 2 data rows, none protected → kept intact.
        let items = vec![
            row("", "", false, true),
            row("/g1", "a.jpg", true, false),
            row("/g1", "b.jpg", false, false),
            row("", "", false, true),
            row("/g2", "c.jpg", true, false),
            row("/g2", "d.jpg", true, false),
        ];
        let protected = protected_set(&[&full("/g1", "a.jpg")]);

        let (new_items, checked_count) = filter_protected_items(items, 0, 1, true, &protected);

        // Only group 2 survives: 1 header + 2 data rows.
        assert_eq!(new_items.len(), 3);
        assert_eq!(new_items.iter().filter(|i| i.header_row).count(), 1);
        // Both surviving data rows were checked.
        assert_eq!(checked_count, 2);
    }

    #[test]
    fn header_mode_keeps_group_with_two_or_more_survivors() {
        // Group has 3 data rows; one protected → 2 survivors → group kept.
        let items = vec![
            row("", "", false, true),
            row("/g", "a.jpg", true, false),
            row("/g", "b.jpg", false, false),
            row("/g", "c.jpg", true, false),
        ];
        let protected = protected_set(&[&full("/g", "b.jpg")]);

        let (new_items, checked_count) = filter_protected_items(items, 0, 1, true, &protected);

        assert_eq!(new_items.len(), 3); // header + a + c
        assert_eq!(checked_count, 2);
        assert!(new_items.iter().all(|i| i.header_row || i.val_str.iter().nth(1).unwrap() != "b.jpg"));
    }

    #[test]
    fn nothing_protected_leaves_everything_unchanged() {
        let items = vec![row("/a", "x.jpg", true, false), row("/a", "y.jpg", false, false)];
        let protected = protected_set(&[&full("/a", "z.jpg")]);

        let (new_items, checked_count) = filter_protected_items(items, 0, 1, false, &protected);

        assert_eq!(new_items.len(), 2);
        assert_eq!(checked_count, 1);
    }
}
