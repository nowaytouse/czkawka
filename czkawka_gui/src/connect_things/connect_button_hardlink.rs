use czkawka_core::common::{make_file_symlink, make_hard_link};
use gtk4::gio::ListStore as GioListStore;
use gtk4::prelude::*;
use gtk4::{CheckButton, TextView, TreeIter, TreePath};
use rayon::prelude::*;

use crate::flg;
use crate::gui_structs::common_tree_view::SubView;
use crate::gui_structs::duplicate_row::DuplicateRow;
use crate::gui_structs::gui_data::GuiData;
use crate::help_functions::{add_text_to_text_view, get_full_name_from_path_name, reset_text_view};
use crate::helpers::async_dialog::{alert_confirm, confirm_window_with_checkbox};
use crate::helpers::list_store_operations::clean_invalid_headers;
use crate::helpers::model_iter::{iter_list, iter_list_break_with_init};

#[derive(PartialEq, Eq, Copy, Clone)]
enum TypeOfTool {
    Hardlinking,
    Symlinking,
}

#[derive(Debug)]
struct SymHardlinkData {
    original_data: String,
    files_to_symhardlink: Vec<String>,
}

pub(crate) fn connect_button_hardlink_symlink(gui_data: &GuiData) {
    // Hardlinking
    {
        let buttons_hardlink = gui_data.bottom_buttons.buttons_hardlink.clone();

        let gui_data = gui_data.clone();

        buttons_hardlink.connect_clicked(move |_| {
            glib::MainContext::default().spawn_local(sym_hard_link_things(gui_data.clone(), TypeOfTool::Hardlinking));
        });
    }

    // Symlinking
    {
        let buttons_symlink = gui_data.bottom_buttons.buttons_symlink.clone();

        let gui_data = gui_data.clone();

        buttons_symlink.connect_clicked(move |_| {
            glib::MainContext::default().spawn_local(sym_hard_link_things(gui_data.clone(), TypeOfTool::Symlinking));
        });
    }
}

async fn sym_hard_link_things(gui_data: GuiData, hardlinking: TypeOfTool) {
    let text_view_errors = gui_data.text_view_errors.clone();
    let window_main = gui_data.window_main.clone();

    let common_tree_views = &gui_data.main_notebook.common_tree_views.clone();
    let sv = common_tree_views.get_current_subview();

    let check_button_settings_confirm_link = gui_data.settings.check_button_settings_confirm_link.clone();

    if !check_if_anything_is_selected_async(sv) {
        return;
    }

    if !check_if_can_link_files(&check_button_settings_confirm_link, &window_main).await {
        return;
    }

    if !check_if_changing_one_item_in_group_and_continue(sv, &window_main).await {
        return;
    }

    hardlink_symlink(sv, hardlinking, &text_view_errors);

    common_tree_views.hide_preview();
}

fn hardlink_symlink(sv: &SubView, hardlinking: TypeOfTool, text_view_errors: &TextView) {
    reset_text_view(text_view_errors);

    if let Some(store) = sv.get_duplicate_model() {
        hardlink_symlink_duplicate(store, hardlinking, text_view_errors);
        return;
    }

    let column_header = sv.nb_object.column_header.expect("Linking can be only used for tree views with grouped results");
    let model = sv.get_model();

    let mut vec_tree_path_to_remove: Vec<TreePath> = Vec::new(); // List of hardlinked files without its root
    let mut vec_symhardlink_data: Vec<SymHardlinkData> = Vec::new();

    let mut current_iter: TreeIter = match model.iter_first() {
        Some(t) => t,
        None => return, // No records
    };

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

    let mut current_symhardlink_data: Option<SymHardlinkData> = None;
    let mut current_selected_index = 0;
    loop {
        if model.get::<bool>(&current_iter, column_header) {
            if let Some(current_symhardlink_data) = current_symhardlink_data
                && !current_symhardlink_data.files_to_symhardlink.is_empty()
            {
                vec_symhardlink_data.push(current_symhardlink_data);
            }

            current_symhardlink_data = None;
            assert!(model.iter_next(&mut current_iter), "HEADER, shouldn't be a last item.");
            continue;
        }

        if model.path(&current_iter) == selected_rows[current_selected_index] {
            let file_name = model.get::<String>(&current_iter, sv.nb_object.column_name);
            let path = model.get::<String>(&current_iter, sv.nb_object.column_path);
            let full_file_path = get_full_name_from_path_name(&path, &file_name);

            if let Some(mut current_data) = current_symhardlink_data {
                vec_tree_path_to_remove.push(model.path(&current_iter));
                current_data.files_to_symhardlink.push(full_file_path);
                current_symhardlink_data = Some(current_data);
            } else {
                current_symhardlink_data = Some(SymHardlinkData {
                    original_data: full_file_path,
                    files_to_symhardlink: Vec::new(),
                });
            }

            if current_selected_index != selected_rows.len() - 1 {
                current_selected_index += 1;
            } else {
                if let Some(current_symhardlink_data) = current_symhardlink_data
                    && !current_symhardlink_data.files_to_symhardlink.is_empty()
                {
                    vec_symhardlink_data.push(current_symhardlink_data);
                }
                break; // There is no more selected items, so we just end checking
            }
        }

        if !model.iter_next(&mut current_iter) {
            if let Some(current_symhardlink_data) = current_symhardlink_data
                && !current_symhardlink_data.files_to_symhardlink.is_empty()
            {
                vec_symhardlink_data.push(current_symhardlink_data);
            }

            break;
        }
    }

    let errors = vec_symhardlink_data
        .into_par_iter()
        .flat_map(|symhardlink_data| {
            let mut err = Vec::new();
            for file_to_be_replaced in symhardlink_data.files_to_symhardlink {
                if hardlinking == TypeOfTool::Symlinking {
                    if let Err(e) = make_file_symlink(&symhardlink_data.original_data, &file_to_be_replaced) {
                        err.push(flg!(
                            "symlink_failed",
                            name = symhardlink_data.original_data.clone(),
                            target = file_to_be_replaced,
                            reason = e.to_string()
                        ));
                    }
                } else {
                    if let Err(e) = make_hard_link(&symhardlink_data.original_data, &file_to_be_replaced) {
                        err.push(flg!(
                            "hardlink_failed",
                            name = symhardlink_data.original_data.clone(),
                            target = file_to_be_replaced,
                            reason = e.to_string()
                        ));
                    }
                }
            }
            err
        })
        .collect::<Vec<_>>();

    for error in errors {
        add_text_to_text_view(text_view_errors, &error);
    }

    for tree_path in vec_tree_path_to_remove.iter().rev() {
        model.remove(&model.iter(tree_path).expect("Using invalid tree_path"));
    }

    clean_invalid_headers(&model, column_header, sv.nb_object.column_path);
}

fn hardlink_symlink_duplicate(store: &GioListStore, hardlinking: TypeOfTool, text_view_errors: &TextView) {
    // Collect groups: for each group the first selected item is the original,
    // subsequent selected items are the targets (to be replaced with links).
    struct GroupData {
        original: String,
        targets: Vec<(u32, String)>, // (position_in_store, full_path)
    }

    let n = store.n_items();
    let mut groups: Vec<GroupData> = Vec::new();
    let mut current_original: Option<String> = None;
    let mut current_targets: Vec<(u32, String)> = Vec::new();
    let mut in_group = false;

    for i in 0..n {
        if let Some(row) = store.item(i).and_downcast::<DuplicateRow>() {
            if row.is_header() {
                if let Some(orig) = current_original.take() {
                    if !current_targets.is_empty() {
                        groups.push(GroupData { original: orig, targets: std::mem::take(&mut current_targets) });
                    }
                }
                current_targets.clear();
                in_group = true;
                continue;
            }
            if !in_group {
                continue;
            }
            if row.selection_button() {
                let full = get_full_name_from_path_name(&row.path(), &row.name());
                if current_original.is_none() {
                    current_original = Some(full);
                } else {
                    current_targets.push((i, full));
                }
            }
        }
    }
    if let Some(orig) = current_original.take() {
        if !current_targets.is_empty() {
            groups.push(GroupData { original: orig, targets: current_targets });
        }
    }

    if groups.is_empty() {
        return;
    }

    // Perform operations in parallel, collecting (positions_to_remove, errors).
    let results: Vec<(Vec<u32>, Vec<String>)> = groups
        .into_par_iter()
        .map(|g| {
            let mut positions = Vec::new();
            let mut errors = Vec::new();
            for (pos, target) in g.targets {
                let result = if hardlinking == TypeOfTool::Symlinking {
                    make_file_symlink(&g.original, &target)
                        .map_err(|e| flg!("symlink_failed", name = g.original.clone(), target = target.clone(), reason = e.to_string()))
                } else {
                    make_hard_link(&g.original, &target)
                        .map_err(|e| flg!("hardlink_failed", name = g.original.clone(), target = target.clone(), reason = e.to_string()))
                };
                match result {
                    Ok(()) => positions.push(pos),
                    Err(msg) => errors.push(msg),
                }
            }
            (positions, errors)
        })
        .collect();

    let mut positions_to_remove: Vec<u32> = Vec::new();
    for (mut pos, errors) in results {
        positions_to_remove.append(&mut pos);
        for e in errors {
            add_text_to_text_view(text_view_errors, &e);
        }
    }

    positions_to_remove.sort_unstable_by(|a, b| b.cmp(a));
    for pos in positions_to_remove {
        store.remove(pos);
    }
}


pub async fn check_if_changing_one_item_in_group_and_continue(sv: &SubView, window_main: &gtk4::Window) -> bool {
    let only_one_in_group = if let Some(store) = sv.get_duplicate_model() {
        // Check groups in GioListStore
        let n = store.n_items();
        let mut found_only_one = false;
        let mut selected_in_group = 0u32;
        let mut in_group = false;
        for i in 0..n {
            if let Some(row) = store.item(i).and_downcast::<DuplicateRow>() {
                if row.is_header() {
                    if in_group && selected_in_group == 1 {
                        found_only_one = true;
                        break;
                    }
                    selected_in_group = 0;
                    in_group = true;
                } else if in_group && row.selection_button() {
                    selected_in_group += 1;
                }
            }
        }
        if in_group && selected_in_group == 1 {
            found_only_one = true;
        }
        if store.n_items() == 0 {
            return false;
        }
        found_only_one
    } else {
        let model = sv.get_model();
        let column_header = sv.nb_object.column_header.expect("Column header must exists for linking");

        let mut selected_values_in_group = 0;
        let mut found = false;

        if let Some(mut iter) = model.iter_first() {
            assert!(model.get::<bool>(&iter, column_header));

            loop {
                if !model.iter_next(&mut iter) {
                    break;
                }
                if model.get::<bool>(&iter, column_header) {
                    if selected_values_in_group == 1 {
                        found = true;
                        break;
                    }
                    selected_values_in_group = 0;
                } else if model.get::<bool>(&iter, sv.nb_object.column_selection) {
                    selected_values_in_group += 1;
                }
            }
        } else {
            return false;
        }
        found
    };

    if only_one_in_group {
        let detail = format!("{}\n{}\n{}", flg!("hard_sym_invalid_selection_label_1"), flg!("hard_sym_invalid_selection_label_2"), flg!("hard_sym_invalid_selection_label_3"));
        if !alert_confirm(window_main, &flg!("hard_sym_invalid_selection_title_dialog"), &detail).await {
            return false;
        }
    }

    true
}

pub(crate) fn check_if_anything_is_selected_async(sv: &SubView) -> bool {
    if let Some(store) = sv.get_duplicate_model() {
        for i in 0..store.n_items() {
            if let Some(row) = store.item(i).and_downcast::<DuplicateRow>() {
                if !row.is_header() && row.selection_button() {
                    return true;
                }
            }
        }
        return false;
    }

    let model = sv.get_model();

    let column_header = sv.nb_object.column_header.expect("Column header must exists for linking");

    let mut non_header_selected = false;

    iter_list_break_with_init(
        &model,
        |m, i| {
            assert!(m.get::<bool>(i, column_header)); // First element should be header
        },
        |m, i| {
            if !m.get::<bool>(i, column_header) && m.get::<bool>(i, sv.nb_object.column_selection) {
                non_header_selected = true;
                return false;
            }
            true
        },
    );

    non_header_selected
}

pub async fn check_if_can_link_files(check_button_settings_confirm_link: &CheckButton, window_main: &gtk4::Window) -> bool {
    if check_button_settings_confirm_link.is_active() {
        let (confirmed, ask_next) = confirm_window_with_checkbox(
            window_main,
            &flg!("hard_sym_link_title_dialog"),
            &[flg!("hard_sym_link_label").as_str()],
            &flg!("general_ok_button"),
            &flg!("general_close_button"),
            &flg!("dialogs_ask_next_time"),
        )
        .await;
        if confirmed {
            if !ask_next {
                check_button_settings_confirm_link.set_active(false);
            }
        } else {
            return false;
        }
    }
    true
}
