use czkawka_core::common::{remove_folder_if_contains_only_empty_folders, remove_single_file};
use gtk4::gio::ListStore as GioListStore;
use gtk4::prelude::*;
use gtk4::{CheckButton, MultiSelection, TextView};
use itertools::Itertools;
use log::debug;
use rayon::prelude::*;

use crate::file_protection::PROTECTED_FILES;
use crate::flg;
use crate::gui_structs::common_tree_view::SubView;
use crate::gui_structs::duplicate_row::DuplicateRow;
use crate::gui_structs::gui_data::GuiData;
use crate::gui_structs::simple_row::SimpleRow;
use crate::help_functions::get_full_name_from_path_name;
use crate::helpers::async_dialog::confirm_window_with_checkbox;
use crate::helpers::list_store_operations::{check_how_much_elements_is_selected, clean_invalid_headers};
use crate::helpers::model_iter::iter_list;
use crate::notebook_enums::NotebookMainEnum;

// TODO add support for checking if really symlink doesn't point to correct directory/file

pub(crate) fn connect_button_delete(gui_data: &GuiData) {
    let buttons_delete = gui_data.bottom_buttons.buttons_delete.clone();

    let gui_data = gui_data.clone(); // TODO this maybe can be replaced, not sure if worth to clone everything

    buttons_delete.connect_clicked(move |_| {
        glib::MainContext::default().spawn_local(delete_things(gui_data.clone()));
    });
}

pub async fn delete_things(gui_data: GuiData) {
    let window_main = gui_data.window_main.clone();
    let check_button_settings_confirm_deletion = gui_data.settings.check_button_settings_confirm_deletion.clone();
    let check_button_settings_confirm_group_deletion = gui_data.settings.check_button_settings_confirm_group_deletion.clone();

    let check_button_settings_use_trash = gui_data.settings.check_button_settings_use_trash.clone();

    let text_view_errors = gui_data.text_view_errors.clone();

    let common_tree_views = gui_data.main_notebook.common_tree_views.clone();
    let sv = gui_data.main_notebook.common_tree_views.get_current_subview();

    let (number_of_selected_items, number_of_selected_groups) = check_how_much_elements_is_selected(sv);

    // Nothing is selected
    if number_of_selected_items == 0 {
        return;
    }

    if !check_if_can_delete_files(&check_button_settings_confirm_deletion, &window_main, number_of_selected_items, number_of_selected_groups).await {
        return;
    }

    if let Some(column_header) = sv.nb_object.column_header {
        if !check_button_settings_confirm_group_deletion.is_active() || !check_if_deleting_all_files_in_group(sv, &window_main, &check_button_settings_confirm_group_deletion).await
        {
            tree_remove(sv, column_header, &check_button_settings_use_trash, &text_view_errors);
        }
    } else if sv.nb_object.notebook_type == NotebookMainEnum::EmptyDirectories {
        empty_folder_remover(sv, &check_button_settings_use_trash, &text_view_errors);
    } else {
        basic_remove(sv, &check_button_settings_use_trash, &text_view_errors);
    }

    common_tree_views.hide_preview();
}

pub async fn check_if_can_delete_files(
    check_button_settings_confirm_deletion: &CheckButton,
    window_main: &gtk4::Window,
    number_of_selected_items: u64,
    number_of_selected_groups: u64,
) -> bool {
    if check_button_settings_confirm_deletion.is_active() {
        let items_msg = match number_of_selected_groups {
            0 => flg!("delete_items_label", items = number_of_selected_items),
            _ => flg!("delete_items_groups_label", items = number_of_selected_items, groups = number_of_selected_groups),
        };
        let question_msg = flg!("delete_question_label");
        let messages = [question_msg.as_str(), items_msg.as_str()];
        let (confirmed, ask_next) = confirm_window_with_checkbox(
            window_main,
            &flg!("delete_title_dialog"),
            &messages,
            &flg!("general_ok_button"),
            &flg!("general_close_button"),
            &flg!("dialogs_ask_next_time"),
        )
        .await;
        if confirmed {
            if !ask_next {
                check_button_settings_confirm_deletion.set_active(false);
            }
        } else {
            return false;
        }
    }
    true
}

pub async fn check_if_deleting_all_files_in_group(sv: &SubView, window_main: &gtk4::Window, check_button_settings_confirm_group_deletion: &CheckButton) -> bool {
    if sv.get_duplicate_model().is_some() {
        return false;
    }
    let column_header = sv.nb_object.column_header.expect("Column header must exist here");
    let model = sv.get_model();

    let mut selected_all_records: bool = true;

    if let Some(mut iter) = model.iter_first() {
        assert!(model.get::<bool>(&iter, column_header)); // First element should be header

        // It is safe to remove any number of files in reference mode
        if !model.get::<String>(&iter, sv.nb_object.column_path).is_empty() {
            return false;
        }

        loop {
            if !model.iter_next(&mut iter) {
                break;
            }

            if model.get::<bool>(&iter, column_header) {
                if selected_all_records {
                    break;
                }
                selected_all_records = true;
            } else if !model.get::<bool>(&iter, sv.nb_object.column_selection) {
                selected_all_records = false;
            }
        }
    } else {
        return false;
    }

    if !selected_all_records {
        return false;
    }

    let label1 = flg!("delete_all_files_in_group_label1");
    let label2 = flg!("delete_all_files_in_group_label2");
    let messages = [label1.as_str(), label2.as_str()];
    let (confirmed, ask_next) = confirm_window_with_checkbox(
        window_main,
        &flg!("delete_all_files_in_group_title"),
        &messages,
        &flg!("general_ok_button"),
        &flg!("general_close_button"),
        &flg!("dialogs_ask_next_time"),
    )
    .await;

    if confirmed {
        if !ask_next {
            check_button_settings_confirm_group_deletion.set_active(false);
        }
        false // don't skip deletion
    } else {
        true // skip deletion
    }
}

pub(crate) fn empty_folder_remover(sv: &SubView, check_button_settings_use_trash: &CheckButton, text_view_errors: &TextView) {
    common_file_remove(sv, check_button_settings_use_trash, text_view_errors, None, false);
}

pub(crate) fn basic_remove(sv: &SubView, check_button_settings_use_trash: &CheckButton, text_view_errors: &TextView) {
    common_file_remove(sv, check_button_settings_use_trash, text_view_errors, None, true);
}

pub(crate) fn tree_remove(sv: &SubView, column_header: i32, check_button_settings_use_trash: &CheckButton, text_view_errors: &TextView) {
    if let (Some(store), Some(selection)) = (sv.get_duplicate_model(), sv.get_duplicate_selection()) {
        common_file_remove_duplicate(sv, store, selection, check_button_settings_use_trash, text_view_errors, Some(column_header), true);
        return;
    }
    common_file_remove(sv, check_button_settings_use_trash, text_view_errors, Some(column_header), true);

    clean_invalid_headers(&sv.get_model(), column_header, sv.nb_object.column_path);
}

pub(crate) fn common_file_remove(sv: &SubView, check_button_settings_use_trash: &CheckButton, text_view_errors: &TextView, column_header: Option<i32>, file_remove: bool) {
    if let (Some(store), Some(selection)) = (sv.get_duplicate_model(), sv.get_duplicate_selection()) {
        common_file_remove_duplicate(sv, store, selection, check_button_settings_use_trash, text_view_errors, column_header, file_remove);
        return;
    }
    if let Some(store) = sv.get_simple_model() {
        common_file_remove_simple(store, check_button_settings_use_trash, text_view_errors, file_remove, sv.nb_object.name);
        return;
    }

    let use_trash = check_button_settings_use_trash.is_active();

    let model = sv.get_model();

    let mut messages: String = String::new();

    let mut selected_rows = Vec::new();

    iter_list(&model, |m, i| {
        if m.get::<bool>(i, sv.nb_object.column_selection) {
            if let Some(column_header) = column_header {
                if !m.get::<bool>(i, column_header) {
                    selected_rows.push(m.path(i));
                } else {
                    panic!("Header row shouldn't be selected, please report bug.");
                }
            } else {
                selected_rows.push(m.path(i));
            }
        }
    });

    if selected_rows.is_empty() {
        return; // No selected rows
    }

    debug!("Starting to delete {} files", selected_rows.len());
    let start_time = std::time::Instant::now();

    let to_remove = selected_rows
        .iter()
        .enumerate()
        .map(|(idx, tree_path)| {
            let iter = model.iter(tree_path).expect("Using invalid tree_path");

            let name = model.get::<String>(&iter, sv.nb_object.column_name);
            let path = model.get::<String>(&iter, sv.nb_object.column_path);

            (idx, get_full_name_from_path_name(&path, &name))
        })
        .collect::<Vec<_>>();

    let pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");

    let (mut removed, failed_to_remove): (Vec<usize>, Vec<String>) = to_remove
        .into_par_iter()
        .map(|(idx, path)| {
            if pf.is_protected(&path) {
                return Err(format!("File is protected: {path}"));
            }
            if file_remove {
                remove_single_file(&path, use_trash)?;
            } else {
                remove_folder_if_contains_only_empty_folders(&path, use_trash)?;
            }
            Ok(idx)
        })
        .partition_map(|res| match res {
            Ok(entry) => itertools::Either::Left(entry),
            Err(err) => itertools::Either::Right(err),
        });

    for failed in &failed_to_remove {
        messages += failed;
        messages += "\n";
    }

    removed.sort_unstable();
    removed.reverse(); // Must be deleted from end to start
    let deleted_files = removed.len();

    for idx in removed {
        let iter = model.iter(&selected_rows[idx]).expect("Using invalid tree_path");
        model.remove(&iter);
    }

    debug!(
        "Deleted {deleted_files}/{} items({} tab) in {:?}",
        selected_rows.len(),
        sv.nb_object.name,
        start_time.elapsed()
    );

    text_view_errors.buffer().set_text(messages.as_str());
}

fn common_file_remove_duplicate(
    _sv: &SubView,
    store: &GioListStore,
    _selection: &MultiSelection,
    check_button_settings_use_trash: &CheckButton,
    text_view_errors: &TextView,
    column_header: Option<i32>,
    file_remove: bool,
) {
    let use_trash = check_button_settings_use_trash.is_active();
    let n = store.n_items();
    let mut selected: Vec<(u32, String)> = Vec::new();
    for pos in 0..n {
        let Some(item) = store.item(pos) else { continue };
        let Ok(row) = item.downcast::<DuplicateRow>() else { continue };
        if column_header.is_some() && row.is_header() {
            continue;
        }
        if !row.selection_button() {
            continue;
        }
        let path = row.path();
        let name = row.name();
        let full = get_full_name_from_path_name(&path, &name);
        selected.push((pos, full));
    }
    if selected.is_empty() {
        return;
    }
    debug!("Starting to delete {} files (duplicate column view)", selected.len());
    let start_time = std::time::Instant::now();
    let pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
    let (removed, failed_to_remove): (Vec<usize>, Vec<String>) = selected
        .iter()
        .enumerate()
        .map(|(idx, (_pos, path))| {
            if pf.is_protected(path) {
                return Err(format!("File is protected: {path}"));
            }
            if file_remove {
                remove_single_file(path, use_trash)?;
            } else {
                remove_folder_if_contains_only_empty_folders(path, use_trash)?;
            }
            Ok(idx)
        })
        .partition_map(|res| match res {
            Ok(entry) => itertools::Either::Left(entry),
            Err(err) => itertools::Either::Right(err),
        });
    drop(pf);
    let mut messages = String::new();
    for failed in &failed_to_remove {
        messages += failed;
        messages += "\n";
    }
    let mut positions_to_remove: Vec<u32> = removed.iter().map(|&idx| selected[idx].0).collect();
    positions_to_remove.sort_unstable_by(|a, b| b.cmp(a));
    for pos in positions_to_remove {
        store.remove(pos);
    }
    debug!("Deleted {}/{} items (duplicate tab) in {:?}", removed.len(), selected.len(), start_time.elapsed());
    text_view_errors.buffer().set_text(messages.as_str());
}

fn common_file_remove_simple(store: &GioListStore, check_button_settings_use_trash: &CheckButton, text_view_errors: &TextView, file_remove: bool, tab_name: &str) {
    let use_trash = check_button_settings_use_trash.is_active();
    let n = store.n_items();
    let mut selected: Vec<(u32, String)> = Vec::new();
    for pos in 0..n {
        let Some(item) = store.item(pos) else { continue };
        let Ok(row) = item.downcast::<SimpleRow>() else { continue };
        if !row.selection_button() {
            continue;
        }
        let full = get_full_name_from_path_name(&row.path(), &row.name());
        selected.push((pos, full));
    }
    if selected.is_empty() {
        return;
    }
    debug!("Starting to delete {} files (simple tab: {})", selected.len(), tab_name);
    let start_time = std::time::Instant::now();
    let pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
    let (removed, failed_to_remove): (Vec<usize>, Vec<String>) = selected
        .iter()
        .enumerate()
        .map(|(idx, (_pos, path))| {
            if pf.is_protected(path) {
                return Err(format!("File is protected: {path}"));
            }
            if file_remove {
                remove_single_file(path, use_trash)?;
            } else {
                remove_folder_if_contains_only_empty_folders(path, use_trash)?;
            }
            Ok(idx)
        })
        .partition_map(|res| match res {
            Ok(entry) => itertools::Either::Left(entry),
            Err(err) => itertools::Either::Right(err),
        });
    drop(pf);
    let mut messages = String::new();
    for failed in &failed_to_remove {
        messages += failed;
        messages += "\n";
    }
    let mut positions_to_remove: Vec<u32> = removed.iter().map(|&idx| selected[idx].0).collect();
    positions_to_remove.sort_unstable_by(|a, b| b.cmp(a));
    for pos in positions_to_remove {
        store.remove(pos);
    }
    debug!(
        "Deleted {}/{} items (simple tab: {}) in {:?}",
        removed.len(),
        selected.len(),
        tab_name,
        start_time.elapsed()
    );
    text_view_errors.buffer().set_text(messages.as_str());
}
