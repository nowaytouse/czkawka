use std::path::Path;

use fs_extra::dir::CopyOptions;
use gtk4::prelude::*;
use gtk4::TreePath;
use log::debug;
use crate::file_protection::PROTECTED_FILES;
use crate::flg;
use crate::gui_structs::common_tree_view::SubView;
use crate::gui_structs::duplicate_row::DuplicateRow;
use crate::gui_structs::simple_row::SimpleRow;
use crate::gui_structs::gui_data::GuiData;
use crate::help_functions::{add_text_to_text_view, get_full_name_from_path_name, reset_text_view};
use crate::helpers::list_store_operations::{check_how_much_elements_is_selected, clean_invalid_headers};
use crate::helpers::model_iter::iter_list;

pub(crate) fn connect_button_move(gui_data: &GuiData) {
    let buttons_move = gui_data.bottom_buttons.buttons_move.clone();
    let window_main = gui_data.window_main.clone();
    let entry_info = gui_data.entry_info.clone();
    let text_view_errors = gui_data.text_view_errors.clone();
    let file_dialog_move_to_folder = gui_data.file_dialog_move_to_folder.clone();
    let common_tree_views = gui_data.main_notebook.common_tree_views.clone();

    buttons_move.connect_clicked(move |_| {
        let sv = common_tree_views.get_current_subview();
        let (number_of_selected_items, _number_of_selected_groups) = check_how_much_elements_is_selected(sv);
        if number_of_selected_items == 0 {
            return;
        }
        reset_text_view(&text_view_errors);
        let entry_info = entry_info.clone();
        let text_view_errors = text_view_errors.clone();
        let common_tree_views = common_tree_views.clone();
        file_dialog_move_to_folder.select_folder(Some(&window_main), None::<&gtk4::gio::Cancellable>, move |result| {
            if let Ok(file) = result {
                if let Some(folder) = file.path() {
                    let sv = common_tree_views.get_current_subview();
                    if sv.get_duplicate_model().is_some() {
                        move_with_duplicate(sv, &folder, &entry_info, &text_view_errors);
                    } else if sv.get_simple_model().is_some() {
                        move_with_simple(sv, &folder, &entry_info, &text_view_errors);
                    } else if sv.nb_object.column_header.is_some() {
                        move_with_tree(sv, &folder, &entry_info, &text_view_errors);
                    } else {
                        move_with_list(sv, &folder, &entry_info, &text_view_errors);
                    }
                }
            } else if let Err(e) = result {
                add_text_to_text_view(&text_view_errors, &format!("{}", e));
            }
            common_tree_views.hide_preview();
        });
    });
}

fn move_with_tree(sv: &SubView, destination_folder: &Path, entry_info: &gtk4::Entry, text_view_errors: &gtk4::TextView) {
    let model = sv.get_model();
    let column_header = sv.nb_object.column_header.expect("Using move_with_tree without header column");

    let mut selected_rows = Vec::new();

    iter_list(&model, |m, i| {
        if m.get::<bool>(i, sv.nb_object.column_selection) {
            if !m.get::<bool>(i, column_header) {
                selected_rows.push(m.path(i));
            } else {
                panic!("Header row shouldn't be selected, please report bug.");
            }
        }
    });

    if selected_rows.is_empty() {
        return; // No selected rows
    }

    move_files_common(
        &selected_rows,
        &model,
        sv.nb_object.column_name,
        sv.nb_object.column_path,
        destination_folder,
        entry_info,
        text_view_errors,
    );

    clean_invalid_headers(&model, column_header, sv.nb_object.column_path);
}

fn move_with_duplicate(sv: &SubView, destination_folder: &Path, entry_info: &gtk4::Entry, text_view_errors: &gtk4::TextView) {
    let Some(store) = sv.get_duplicate_model() else { return };
    let n = store.n_items();
    let mut to_move: Vec<(u32, String)> = Vec::new();
    for pos in 0..n {
        let Some(item) = store.item(pos) else { continue };
        let Ok(row) = item.downcast::<DuplicateRow>() else { continue };
        if row.is_header() || !row.selection_button() {
            continue;
        }
        let full = get_full_name_from_path_name(&row.path(), &row.name());
        to_move.push((pos, full));
    }
    if to_move.is_empty() {
        return;
    }
    let pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
    let mut messages = String::new();
    let mut moved_files = 0u32;
    let mut positions_to_remove: Vec<u32> = Vec::new();
    for (pos, thing) in &to_move {
        if pf.is_protected(thing) {
            messages += &format!("File is protected: {thing}\n");
            continue;
        }
        let file_name = Path::new(thing).file_name().and_then(|p| p.to_str()).unwrap_or("");
        let destination_file = destination_folder.join(file_name);
        let ok = if Path::new(thing).is_dir() {
            fs_extra::dir::move_dir(thing, &destination_file, &CopyOptions::new()).is_ok()
        } else {
            fs_extra::file::move_file(thing, &destination_file, &fs_extra::file::CopyOptions::new()).is_ok()
        };
        if ok {
            moved_files += 1;
            positions_to_remove.push(*pos);
        }
    }
    drop(pf);
    positions_to_remove.sort_unstable_by(|a, b| b.cmp(a));
    for pos in positions_to_remove {
        store.remove(pos);
    }
    entry_info.set_text(flg!("move_stats", num_files = moved_files, all_files = to_move.len()).as_str());
    text_view_errors.buffer().set_text(messages.as_str());
}

fn move_with_list(sv: &SubView, destination_folder: &Path, entry_info: &gtk4::Entry, text_view_errors: &gtk4::TextView) {
    let model = sv.get_model();

    let mut selected_rows = Vec::new();

    iter_list(&model, |m, i| {
        if m.get::<bool>(i, sv.nb_object.column_selection) {
            selected_rows.push(m.path(i));
        }
    });

    if selected_rows.is_empty() {
        return; // No selected rows
    }

    move_files_common(
        &selected_rows,
        &model,
        sv.nb_object.column_name,
        sv.nb_object.column_path,
        destination_folder,
        entry_info,
        text_view_errors,
    );
}

fn move_with_simple(sv: &SubView, destination_folder: &Path, entry_info: &gtk4::Entry, text_view_errors: &gtk4::TextView) {
    let Some(store) = sv.get_simple_model() else { return };
    let n = store.n_items();
    let mut to_move: Vec<(u32, String)> = Vec::new();
    for pos in 0..n {
        let Some(item) = store.item(pos) else { continue };
        let Ok(row) = item.downcast::<SimpleRow>() else { continue };
        if !row.selection_button() {
            continue;
        }
        let full = get_full_name_from_path_name(&row.path(), &row.name());
        to_move.push((pos, full));
    }
    if to_move.is_empty() {
        return;
    }
    let pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
    let mut messages = String::new();
    let mut moved_files = 0u32;
    let mut positions_to_remove: Vec<u32> = Vec::new();
    for (pos, thing) in &to_move {
        if pf.is_protected(thing) {
            messages += &format!("File is protected: {thing}\n");
            continue;
        }
        let file_name = Path::new(thing).file_name().and_then(|p| p.to_str()).unwrap_or("");
        let destination_file = destination_folder.join(file_name);
        let ok = if Path::new(thing).is_dir() {
            fs_extra::dir::move_dir(thing, &destination_file, &CopyOptions::new()).is_ok()
        } else {
            fs_extra::file::move_file(thing, &destination_file, &fs_extra::file::CopyOptions::new()).is_ok()
        };
        if ok {
            moved_files += 1;
            positions_to_remove.push(*pos);
        }
    }
    drop(pf);
    positions_to_remove.sort_unstable_by(|a, b| b.cmp(a));
    for pos in positions_to_remove {
        store.remove(pos);
    }
    entry_info.set_text(flg!("move_stats", num_files = moved_files, all_files = to_move.len()).as_str());
    text_view_errors.buffer().set_text(messages.as_str());
}

fn move_files_common(
    selected_rows: &[TreePath],
    model: &gtk4::ListStore,
    column_file_name: i32,
    column_path: i32,
    destination_folder: &Path,
    entry_info: &gtk4::Entry,
    text_view_errors: &gtk4::TextView,
) {
    let mut messages: String = String::new();

    let mut moved_files: u32 = 0;

    debug!("Starting to move {} files", selected_rows.len());
    let start_time = std::time::Instant::now();

    // Save to variable paths of files, and remove it when not removing all occurrences.
    let pf = PROTECTED_FILES.lock().expect("Failed to lock protected files");
    'next_result: for tree_path in selected_rows.iter().rev() {
        let iter = model.iter(tree_path).expect("Using invalid tree_path");

        let file_name = model.get::<String>(&iter, column_file_name);
        let path = model.get::<String>(&iter, column_path);

        let thing = get_full_name_from_path_name(&path, &file_name);

        if pf.is_protected(&thing) {
            messages += &format!("File is protected: {thing}");
            messages += "\n";
            continue 'next_result;
        }

        let destination_file = destination_folder.join(&file_name);
        if Path::new(&thing).is_dir() {
            if let Err(e) = fs_extra::dir::move_dir(&thing, &destination_file, &CopyOptions::new()) {
                messages += flg!("move_folder_failed", name = thing, reason = e.to_string()).as_str();
                messages += "\n";
                continue 'next_result;
            }
        } else if let Err(e) = fs_extra::file::move_file(&thing, &destination_file, &fs_extra::file::CopyOptions::new()) {
            messages += flg!("move_file_failed", name = thing, reason = e.to_string()).as_str();
            messages += "\n";

            continue 'next_result;
        }
        model.remove(&iter);
        moved_files += 1;
    }

    debug!("Moved {moved_files} files in {:?}", start_time.elapsed());

    entry_info.set_text(flg!("move_stats", num_files = moved_files, all_files = selected_rows.len()).as_str());

    text_view_errors.buffer().set_text(messages.as_str());
}
