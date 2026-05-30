use std::path::{MAIN_SEPARATOR, PathBuf};

use log::info;
use slint::{ComponentHandle, Model};

use crate::connect_row_selection::checker::set_number_of_enabled_items;
use crate::file_protection::PROTECTED_FILES;
use crate::{ActiveTab, Callabler, GuiState, MainWindow, SingleMainListModel};

pub(crate) fn connect_file_protection(app: &MainWindow) {
    // Initialize the protected files count in GUI
    {
        let pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
        app.global::<GuiState>().set_protected_files_count(pf.count() as i32);
    }

    connect_protect(app);
    connect_unprotect(app);
    connect_toggle_single(app);
    connect_clear_all(app);
    connect_filter_after_scan(app);
}

/// Build the absolute path of a data row from its path/name columns.
/// Returns `None` for header rows or rows missing the expected columns.
fn row_full_path(item: &SingleMainListModel, path_idx: usize, name_idx: usize) -> Option<PathBuf> {
    if item.header_row {
        return None;
    }
    let val_str: Vec<String> = item.val_str.iter().map(|s| s.to_string()).collect();
    match (val_str.get(path_idx), val_str.get(name_idx)) {
        (Some(path), Some(name)) => Some(PathBuf::from(format!("{path}{MAIN_SEPARATOR}{name}"))),
        _ => None,
    }
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

        // Mark every checked data row as protected and clear its checkbox, so it
        // leaves the deletion queue but STAYS visible in the list. Mutating rows in
        // place keeps row indices (and therefore TOOLS_SELECTION) valid.
        for idx in 0..model.row_count() {
            if let Some(mut item) = model.row_data(idx)
                && item.checked
                && !item.header_row
                && let Some(full_path) = row_full_path(&item, path_idx, name_idx)
            {
                if pf.files.insert(full_path) {
                    protected_count += 1;
                }
                item.protected = true;
                item.checked = false;
                model.set_row_data(idx, item);
            }
        }

        if protected_count > 0 {
            pf.save();
            info!("Protected {} files, total: {}", protected_count, pf.count());
        }

        // Unchecked rows changed the enabled-items counter; recompute from scratch.
        let checked_count = model.iter().filter(|i| !i.header_row && i.checked).count() as u64;
        set_number_of_enabled_items(&app, active_tab, checked_count);

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

        // Protected rows can't be checked (their checkbox is disabled), so the bulk
        // Unprotect button acts on FOCUSED (highlighted) rows instead. The user clicks
        // the protected rows to highlight them, then presses Unprotect.
        for idx in 0..model.row_count() {
            if let Some(mut item) = model.row_data(idx)
                && item.focused_row
                && item.protected
                && let Some(full_path) = row_full_path(&item, path_idx, name_idx)
            {
                if pf.files.remove(&full_path) {
                    unprotected_count += 1;
                }
                item.protected = false;
                model.set_row_data(idx, item);
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

// CONNECT_TOGGLE_SINGLE_PLACEHOLDER

/// Per-row protect/unprotect from the right-click context menu. This is the surgical
/// recovery path: it flips exactly one file without disturbing the rest of the protected
/// set, and never removes the row from the list.
fn connect_toggle_single(app: &MainWindow) {
    let a = app.as_weak();
    app.global::<Callabler>().on_row_toggle_protected(move |idx| {
        let app = a.upgrade().expect("Failed to upgrade app :(");
        let active_tab = app.global::<GuiState>().get_active_tab();
        let model = active_tab.get_tool_model(&app);

        let path_idx = active_tab.get_str_path_idx();
        let name_idx = active_tab.get_str_name_idx();

        let Some(mut item) = model.row_data(idx as usize) else {
            return;
        };
        let Some(full_path) = row_full_path(&item, path_idx, name_idx) else {
            return;
        };

        let mut pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
        let now_protected = !item.protected;
        if now_protected {
            pf.files.insert(full_path);
            // A newly protected row must leave the deletion queue.
            item.checked = false;
        } else {
            pf.files.remove(&full_path);
        }
        item.protected = now_protected;
        pf.save();
        model.set_row_data(idx as usize, item);

        // The checkbox may have been cleared above; recompute the enabled-items counter.
        let checked_count = model.iter().filter(|i| !i.header_row && i.checked).count() as u64;
        set_number_of_enabled_items(&app, active_tab, checked_count);

        app.global::<GuiState>().set_protected_files_count(pf.count() as i32);
        let info = if now_protected {
            format!("Protected 1 file (total protected: {})", pf.count())
        } else {
            format!("Unprotected 1 file (total protected: {})", pf.count())
        };
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
        drop(pf);
        info!("Cleared all {count} protected files");

        // Clear the protected marker on every currently displayed row too, so the list
        // reflects the now-empty protected set without needing a rescan.
        if active_tab_is_tool(&app) {
            let active_tab = app.global::<GuiState>().get_active_tab();
            let model = active_tab.get_tool_model(&app);
            for idx in 0..model.row_count() {
                if let Some(mut item) = model.row_data(idx)
                    && item.protected
                {
                    item.protected = false;
                    model.set_row_data(idx, item);
                }
            }
        }

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
        let protected = {
            let pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
            pf.files.clone()
        };
        if protected.is_empty() {
            return;
        }
        // Defer marking until after scan_ended finishes toggling `scanning` and refreshing the
        // list. Synchronous set_row_data here was correlated with spurious row activation on macOS.
        let app_weak = app.as_weak();
        let _ = slint::invoke_from_event_loop(move || {
            let Some(app) = app_weak.upgrade() else {
                return;
            };
            mark_protected_in_model(&app, active_tab, &protected);
            info!("Marked protected files in scan results for {active_tab:?}");
        });
    });
}

fn active_tab_is_tool(app: &MainWindow) -> bool {
    !matches!(app.global::<GuiState>().get_active_tab(), ActiveTab::Settings | ActiveTab::About)
}

/// Check if a file path is protected. Used as a safety net in delete/move/link/rename operations.
pub fn is_file_protected(path: &str) -> bool {
    let pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
    pf.files.contains(&PathBuf::from(path))
}

/// Mark rows whose absolute path is in `protected` with `protected = true` (and uncheck them),
/// in place. Rows are never removed, so row indices stay valid and the selection state does not
/// need to be reset.
pub(crate) fn mark_protected_in_model(app: &MainWindow, active_tab: ActiveTab, protected: &std::collections::HashSet<PathBuf>) {
    if protected.is_empty() {
        return;
    }
    let model = active_tab.get_tool_model(app);
    let path_idx = active_tab.get_str_path_idx();
    let name_idx = active_tab.get_str_name_idx();

    for idx in 0..model.row_count() {
        if let Some(mut item) = model.row_data(idx)
            && let Some(full_path) = row_full_path(&item, path_idx, name_idx)
            && protected.contains(&full_path)
            && !item.protected
        {
            item.protected = true;
            item.checked = false;
            model.set_row_data(idx, item);
        }
    }

    let checked_count = model.iter().filter(|i| !i.header_row && i.checked).count() as u64;
    set_number_of_enabled_items(app, active_tab, checked_count);
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::path::PathBuf;

    use slint::{Model, ModelRc, SharedString, VecModel};

    use super::{MAIN_SEPARATOR, row_full_path};
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

    fn full(path: &str, name: &str) -> String {
        format!("{path}{MAIN_SEPARATOR}{name}")
    }

    // Pure re-implementation of the marking logic over a Vec, so the group-preserving
    // behavior can be asserted without a live MainWindow. Mirrors mark_protected_in_model.
    fn mark(items: &mut [SingleMainListModel], path_idx: usize, name_idx: usize, protected: &HashSet<PathBuf>) -> u64 {
        for item in items.iter_mut() {
            if let Some(fp) = row_full_path(item, path_idx, name_idx)
                && protected.contains(&fp)
                && !item.protected
            {
                item.protected = true;
                item.checked = false;
            }
        }
        items.iter().filter(|i| !i.header_row && i.checked).count() as u64
    }

    #[test]
    fn marks_protected_row_and_keeps_all_rows() {
        let mut items = vec![
            row("/a", "keep1.jpg", true, false),
            row("/a", "secret.jpg", true, false),
            row("/a", "keep2.jpg", true, false),
        ];
        let protected: HashSet<PathBuf> = [PathBuf::from(full("/a", "secret.jpg"))].into_iter().collect();

        let checked_count = mark(&mut items, 0, 1, &protected);

        // No row is removed — protection is a marker, not a deletion.
        assert_eq!(items.len(), 3);
        // The protected row is flagged and unchecked; the others stay checked.
        let secret = items.iter().find(|i| i.val_str.iter().nth(1).unwrap() == "secret.jpg").unwrap();
        assert!(secret.protected);
        assert!(!secret.checked);
        // Two still-checked survivors → counter reflects them.
        assert_eq!(checked_count, 2);
    }

    #[test]
    fn marking_preserves_group_structure_including_headers() {
        // A group that loses members to protection is NOT dropped — every row stays put.
        let mut items = vec![row("", "", false, true), row("/g1", "a.jpg", true, false), row("/g1", "b.jpg", true, false)];
        let protected: HashSet<PathBuf> = [PathBuf::from(full("/g1", "a.jpg"))].into_iter().collect();

        let checked_count = mark(&mut items, 0, 1, &protected);

        assert_eq!(items.len(), 3); // header + 2 data rows, nothing removed
        assert!(items[0].header_row);
        assert!(items[1].protected && !items[1].checked);
        assert!(!items[2].protected && items[2].checked);
        assert_eq!(checked_count, 1);
    }

    #[test]
    fn nothing_protected_leaves_everything_unchanged() {
        let mut items = vec![row("/a", "x.jpg", true, false), row("/a", "y.jpg", false, false)];
        let protected: HashSet<PathBuf> = [PathBuf::from(full("/a", "z.jpg"))].into_iter().collect();

        let checked_count = mark(&mut items, 0, 1, &protected);

        assert_eq!(items.len(), 2);
        assert!(items.iter().all(|i| !i.protected));
        assert_eq!(checked_count, 1);
    }

    #[test]
    fn header_rows_never_get_a_full_path() {
        let header = row("ignored", "ignored", false, true);
        assert!(row_full_path(&header, 0, 1).is_none());
    }
}
