use std::path::PathBuf;

use gtk4::prelude::*;
use log::info;

use crate::file_protection::PROTECTED_FILES;
use crate::flg;
use crate::gui_structs::common_tree_view::SubView;
use crate::gui_structs::gui_data::GuiData;
use crate::help_functions::get_full_name_from_path_name;
use crate::helpers::list_store_operations::clean_invalid_headers;
use crate::helpers::model_iter::iter_list;

pub(crate) fn connect_file_protection(gui_data: &GuiData) {
    connect_protect(gui_data);
    connect_unprotect(gui_data);
    connect_clear_all(gui_data);
    update_clear_button_label(gui_data);
}

fn connect_protect(gui_data: &GuiData) {
    let buttons_protect = gui_data.bottom_buttons.buttons_protect.clone();
    let gui_data = gui_data.clone();

    buttons_protect.connect_clicked(move |_| {
        let sv = gui_data.main_notebook.common_tree_views.get_current_subview();

        let mut pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
        let mut protected_count = 0;

        if let Some(store) = sv.get_duplicate_model() {
            let n = store.n_items();
            for pos in 0..n {
                let Some(item) = store.item(pos) else { continue };
                let Ok(row) = item.downcast::<crate::gui_structs::duplicate_row::DuplicateRow>() else { continue };
                if row.is_header() || !row.selection_button() {
                    continue;
                }
                let full_path = get_full_name_from_path_name(&row.path(), &row.name());
                if pf.files.insert(PathBuf::from(&full_path)) {
                    protected_count += 1;
                }
            }
        } else {
            let model = sv.get_model();
            iter_list(&model, |m, i| {
                if m.get::<bool>(i, sv.nb_object.column_selection) {
                    if let Some(column_header) = sv.nb_object.column_header {
                        if m.get::<bool>(i, column_header) {
                            return;
                        }
                    }
                    let name = m.get::<String>(i, sv.nb_object.column_name);
                    let path = m.get::<String>(i, sv.nb_object.column_path);
                    let full_path = get_full_name_from_path_name(&path, &name);
                    if pf.files.insert(PathBuf::from(&full_path)) {
                        protected_count += 1;
                    }
                }
            });
        }

        if protected_count > 0 {
            pf.save();
            info!("Protected {} files, total: {}", protected_count, pf.count());
        }

        let info_text = format!("Protected {} files (total protected: {})", protected_count, pf.count());
        gui_data.entry_info.set_text(&info_text);
        update_clear_button_label_inner(&gui_data, pf.count());
    });
}

fn connect_unprotect(gui_data: &GuiData) {
    let buttons_unprotect = gui_data.bottom_buttons.buttons_unprotect.clone();
    let gui_data = gui_data.clone();

    buttons_unprotect.connect_clicked(move |_| {
        let sv = gui_data.main_notebook.common_tree_views.get_current_subview();

        let mut pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
        let mut unprotected_count = 0;

        if let Some(store) = sv.get_duplicate_model() {
            let n = store.n_items();
            for pos in 0..n {
                let Some(item) = store.item(pos) else { continue };
                let Ok(row) = item.downcast::<crate::gui_structs::duplicate_row::DuplicateRow>() else { continue };
                if row.is_header() || !row.selection_button() {
                    continue;
                }
                let full_path = get_full_name_from_path_name(&row.path(), &row.name());
                if pf.files.remove(&PathBuf::from(&full_path)) {
                    unprotected_count += 1;
                }
            }
        } else {
            let model = sv.get_model();
            iter_list(&model, |m, i| {
                if m.get::<bool>(i, sv.nb_object.column_selection) {
                    if let Some(column_header) = sv.nb_object.column_header {
                        if m.get::<bool>(i, column_header) {
                            return;
                        }
                    }
                    let name = m.get::<String>(i, sv.nb_object.column_name);
                    let path = m.get::<String>(i, sv.nb_object.column_path);
                    let full_path = get_full_name_from_path_name(&path, &name);
                    if pf.files.remove(&PathBuf::from(&full_path)) {
                        unprotected_count += 1;
                    }
                }
            });
        }

        if unprotected_count > 0 {
            pf.save();
            info!("Unprotected {} files, total: {}", unprotected_count, pf.count());
        }

        let info_text = format!("Unprotected {} files (total protected: {})", unprotected_count, pf.count());
        gui_data.entry_info.set_text(&info_text);
        update_clear_button_label_inner(&gui_data, pf.count());
    });
}

fn connect_clear_all(gui_data: &GuiData) {
    let button_clear = gui_data.settings.button_settings_clear_protected_files.clone();
    let gui_data = gui_data.clone();

    button_clear.connect_clicked(move |_| {
        let mut pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
        let count = pf.count();
        pf.clear();
        info!("Cleared all {} protected files", count);

        let info_text = format!("Cleared all {} protected files", count);
        gui_data.entry_info.set_text(&info_text);
        update_clear_button_label_inner(&gui_data, 0);
    });
}

fn update_clear_button_label(gui_data: &GuiData) {
    let pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
    update_clear_button_label_inner(gui_data, pf.count());
}

fn update_clear_button_label_inner(gui_data: &GuiData, count: usize) {
    gui_data
        .settings
        .button_settings_clear_protected_files
        .set_label(&flg!("settings_clear_protected_files_button", count = count));
}

/// Filter protected files from model after scan results are populated.
pub(crate) fn filter_protected_from_model(sv: &SubView) {
    let pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
    if pf.files.is_empty() {
        return;
    }

    if let Some(store) = sv.get_duplicate_model() {
        let mut positions_to_remove = Vec::new();
        let n = store.n_items();
        for pos in 0..n {
            let Some(item) = store.item(pos) else { continue };
            let Ok(row) = item.downcast::<crate::gui_structs::duplicate_row::DuplicateRow>() else { continue };
            if row.is_header() {
                continue;
            }
            let full_path = get_full_name_from_path_name(&row.path(), &row.name());
            if pf.files.contains(&PathBuf::from(&full_path)) {
                positions_to_remove.push(pos);
            }
        }
        positions_to_remove.sort_unstable_by(|a, b| b.cmp(a));
        for pos in positions_to_remove {
            store.remove(pos);
        }
        return;
    }

    let model = sv.get_model();

    let mut rows_to_remove = Vec::new();
    iter_list(&model, |m, i| {
        if let Some(column_header) = sv.nb_object.column_header {
            if m.get::<bool>(i, column_header) {
                return;
            }
        }
        let name = m.get::<String>(i, sv.nb_object.column_name);
        let path = m.get::<String>(i, sv.nb_object.column_path);
        let full_path = get_full_name_from_path_name(&path, &name);
        if pf.files.contains(&PathBuf::from(&full_path)) {
            rows_to_remove.push(m.path(i));
        }
    });

    if rows_to_remove.is_empty() {
        return;
    }

    for tree_path in rows_to_remove.iter().rev() {
        if let Some(iter) = model.iter(tree_path) {
            model.remove(&iter);
        }
    }

    if let Some(column_header) = sv.nb_object.column_header {
        clean_invalid_headers(&model, column_header, sv.nb_object.column_path);
    }

    info!("Filtered {} protected files from scan results", rows_to_remove.len());
}
