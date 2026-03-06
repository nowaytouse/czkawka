use std::collections::HashSet;
use std::path::PathBuf;

#[cfg(target_family = "windows")]
use czkawka_core::common::normalize_windows_path;
use gdk4::{DragAction, FileList};
use gtk4::prelude::*;
use gtk4::{Align, DropTarget, Orientation, TreeView, Window};

use crate::connect_things::file_chooser_helpers::paths_from_list_model;
use crate::flg;
use crate::gui_structs::common_tree_view::TreeViewListStoreTrait;
use crate::gui_structs::common_upper_tree_view::UpperTreeViewEnum;
use crate::gui_structs::gui_data::GuiData;
use crate::helpers::enums::{ColumnsExcludedDirectory, ColumnsIncludedDirectory};
use crate::helpers::list_store_operations::{append_row_to_list_store, check_if_value_is_in_list_store};

pub(crate) fn connect_selection_of_directories(gui_data: &GuiData) {
    let tree_view_included_directories = gui_data.upper_notebook.common_upper_tree_views.get_tree_view(UpperTreeViewEnum::IncludedDirectories);
    let tree_view_excluded_directories = gui_data.upper_notebook.common_upper_tree_views.get_tree_view(UpperTreeViewEnum::ExcludedDirectories);

    // Add manually directory
    {
        let tree_view_included_directories = tree_view_included_directories.clone();
        let window_main = gui_data.window_main.clone();
        let buttons_manual_add_included_directory = gui_data.upper_notebook.buttons_manual_add_included_directory.clone();
        buttons_manual_add_included_directory.connect_clicked(move |_| {
            add_manually_directories(&window_main, &tree_view_included_directories, false);
        });
    }
    // Add manually excluded directory
    {
        let tree_view_excluded_directories = tree_view_excluded_directories.clone();
        let window_main = gui_data.window_main.clone();
        let buttons_manual_add_excluded_directory = gui_data.upper_notebook.buttons_manual_add_excluded_directory.clone();
        buttons_manual_add_excluded_directory.connect_clicked(move |_| {
            add_manually_directories(&window_main, &tree_view_excluded_directories, true);
        });
    }
    // Add included directory (FileDialog async API)
    {
        let window_main = gui_data.window_main.clone();
        let buttons_add_included_directory = gui_data.upper_notebook.buttons_add_included_directory.clone();
        let file_dialog = gui_data.file_dialog_include_exclude_folder_selection.clone();
        let tree_view = tree_view_included_directories.clone();
        buttons_add_included_directory.connect_clicked(move |_| {
            let tree = tree_view.clone();
            file_dialog.set_title(&flg!("include_folders_dialog_title"));
            file_dialog.select_multiple_folders(Some(&window_main), None::<&gtk4::gio::Cancellable>, move |result| {
                if let Ok(files) = result {
                    let folders = paths_from_list_model(&files);
                    add_directories(&tree, &folders, false);
                }
            });
        });
    }
    // Add excluded directory (FileDialog async API)
    {
        let window_main = gui_data.window_main.clone();
        let buttons_add_excluded_directory = gui_data.upper_notebook.buttons_add_excluded_directory.clone();
        let file_dialog = gui_data.file_dialog_include_exclude_folder_selection.clone();
        let tree_view = tree_view_excluded_directories.clone();
        buttons_add_excluded_directory.connect_clicked(move |_| {
            let tree = tree_view.clone();
            file_dialog.set_title(&flg!("exclude_folders_dialog_title"));
            file_dialog.select_multiple_folders(Some(&window_main), None::<&gtk4::gio::Cancellable>, move |result| {
                if let Ok(files) = result {
                    let folders = paths_from_list_model(&files);
                    add_directories(&tree, &folders, true);
                }
            });
        });
    }
    // Drag and drop
    {
        configure_directory_drop(tree_view_included_directories, false);
        configure_directory_drop(tree_view_excluded_directories, true);
    }
    // Remove Excluded Folder
    {
        let buttons_remove_excluded_directory = gui_data.upper_notebook.buttons_remove_excluded_directory.clone();
        let tree_view_excluded_directories = tree_view_excluded_directories.clone();
        buttons_remove_excluded_directory.connect_clicked(move |_| {
            remove_item_directory(&tree_view_excluded_directories);
        });
    }
    // Remove Included Folder
    {
        let buttons_remove_included_directory = gui_data.upper_notebook.buttons_remove_included_directory.clone();
        let tree_view_included_directories = tree_view_included_directories.clone();
        buttons_remove_included_directory.connect_clicked(move |_| {
            remove_item_directory(&tree_view_included_directories);
        });
    }
}

fn remove_item_directory(tree_view: &TreeView) {
    let list_store = tree_view.get_model();
    let selection = tree_view.selection();

    let (vec_tree_path, _tree_model) = selection.selected_rows();

    for tree_path in vec_tree_path.iter().rev() {
        list_store.remove(&list_store.iter(tree_path).expect("Using invalid tree_path"));
    }
}

fn configure_directory_drop(tree_view: &TreeView, excluded_items: bool) {
    let tv = tree_view.clone();
    let drop_target = DropTarget::builder().name("file-drop-target").actions(DragAction::COPY).build();
    drop_target.set_types(&[FileList::static_type()]);
    drop_target.connect_drop(move |_, value, _, _| {
        if let Ok(file_list) = value.get::<FileList>() {
            let mut folders: HashSet<PathBuf> = HashSet::new();
            for f in file_list.files() {
                if let Some(path) = f.path() {
                    if path.is_dir() {
                        folders.insert(path);
                    } else if let Some(parent) = path.parent()
                        && parent.is_dir()
                    {
                        folders.insert(parent.to_path_buf());
                    }
                }
            }
            add_directories(&tv, &folders.into_iter().collect(), excluded_items);
        }
        true
    });

    tree_view.add_controller(drop_target);
}

fn add_directories(tree_view: &TreeView, folders: &Vec<PathBuf>, excluded_items: bool) {
    let list_store = tree_view.get_model();

    if excluded_items {
        for file_entry in folders {
            let values: [(u32, &dyn ToValue); 1] = [(ColumnsExcludedDirectory::Path as u32, &file_entry.to_string_lossy().to_string())];
            append_row_to_list_store(&list_store, &values);
        }
    } else {
        for file_entry in folders {
            let values: [(u32, &dyn ToValue); 2] = [
                (ColumnsIncludedDirectory::Path as u32, &file_entry.to_string_lossy().to_string()),
                (ColumnsIncludedDirectory::ReferenceButton as u32, &false),
            ];
            append_row_to_list_store(&list_store, &values);
        }
    }
}

fn add_manually_directories(window_main: &Window, tree_view: &TreeView, excluded_items: bool) {
    let window = gtk4::Window::builder()
        .title(flg!("include_manually_directories_dialog_title"))
        .transient_for(window_main)
        .modal(true)
        .destroy_with_parent(true)
        .resizable(false)
        .default_width(300)
        .build();

    let entry: gtk4::Entry = gtk4::Entry::new();
    let ok_btn = gtk4::Button::with_label(&flg!("general_ok_button"));
    let cancel_btn = gtk4::Button::with_label(&flg!("general_close_button"));
    ok_btn.add_css_class("suggested-action");

    let btn_box = gtk4::Box::builder().orientation(Orientation::Horizontal).halign(Align::Center).spacing(8).build();
    btn_box.append(&cancel_btn);
    btn_box.append(&ok_btn);

    let main_box = gtk4::Box::builder().orientation(Orientation::Vertical).spacing(8).margin_top(10).margin_bottom(10).margin_start(10).margin_end(10).build();
    main_box.append(&entry);
    main_box.append(&btn_box);

    window.set_child(Some(&main_box));
    window.present();

    let win_cancel = window.clone();
    cancel_btn.connect_clicked(move |_| {
        win_cancel.close();
    });

    let win_ok = window.clone();
    let tree_view = tree_view.clone();
    ok_btn.connect_clicked(move |_| {
        for text in entry.text().split(';') {
            let text = text.trim().to_string();
            #[cfg(target_family = "windows")]
            let text = normalize_windows_path(text).to_string_lossy().to_string();
            let mut text = text;

            remove_ending_slashes(&mut text);

            if !text.is_empty() {
                let list_store = tree_view.get_model();

                if excluded_items {
                    if !check_if_value_is_in_list_store(&list_store, ColumnsExcludedDirectory::Path as i32, &text) {
                        let values: [(u32, &dyn ToValue); 1] = [(ColumnsExcludedDirectory::Path as u32, &text)];
                        append_row_to_list_store(&list_store, &values);
                    }
                } else if !check_if_value_is_in_list_store(&list_store, ColumnsIncludedDirectory::Path as i32, &text) {
                    let values: [(u32, &dyn ToValue); 2] = [(ColumnsIncludedDirectory::Path as u32, &text), (ColumnsIncludedDirectory::ReferenceButton as u32, &false)];
                    append_row_to_list_store(&list_store, &values);
                }
            }
        }
        win_ok.close();
    });
}

fn remove_ending_slashes(original_string: &mut String) {
    let mut windows_disk_path: bool = false;
    let mut chars = original_string.chars();
    if let Some(first_character) = chars.next()
        && first_character.is_alphabetic()
        && let Some(second_character) = chars.next()
        && second_character == ':'
    {
        windows_disk_path = true;
        original_string.push('/'); // In case of adding window path without ending slash e.g. C: instead C:/ or C:\
    }

    while (original_string != "/" && (original_string.ends_with('/') || original_string.ends_with('\\'))) && (!windows_disk_path || original_string.len() > 3) {
        original_string.pop();
    }
}

#[test]
pub(crate) fn test_remove_ending_slashes() {
    let mut original = "/home/rafal".to_string();
    remove_ending_slashes(&mut original);
    assert_eq!(&original, "/home/rafal");

    let mut original = "/home/rafal/".to_string();
    remove_ending_slashes(&mut original);
    assert_eq!(&original, "/home/rafal");

    let mut original = "/home/rafal\\".to_string();
    remove_ending_slashes(&mut original);
    assert_eq!(&original, "/home/rafal");

    let mut original = "/home/rafal/////////".to_string();
    remove_ending_slashes(&mut original);
    assert_eq!(&original, "/home/rafal");

    let mut original = "/home/rafal/\\//////\\\\".to_string();
    remove_ending_slashes(&mut original);
    assert_eq!(&original, "/home/rafal");

    let mut original = "/home/rafal\\\\\\\\\\\\\\\\".to_string();
    remove_ending_slashes(&mut original);
    assert_eq!(&original, "/home/rafal");

    let mut original = "\\\\\\\\\\\\\\\\\\\\\\\\".to_string();
    remove_ending_slashes(&mut original);
    assert_eq!(&original, "");

    let mut original = "//////////".to_string();
    remove_ending_slashes(&mut original);
    assert_eq!(&original, "/");

    let mut original = "C:/".to_string();
    remove_ending_slashes(&mut original);
    assert_eq!(&original, "C:/");

    let mut original = "C:\\".to_string();
    remove_ending_slashes(&mut original);
    assert_eq!(&original, "C:\\");

    let mut original = "C://////////".to_string();
    remove_ending_slashes(&mut original);
    assert_eq!(&original, "C:/");

    let mut original = "C:/roman/function/".to_string();
    remove_ending_slashes(&mut original);
    assert_eq!(&original, "C:/roman/function");

    let mut original = "C:/staszek/without".to_string();
    remove_ending_slashes(&mut original);
    assert_eq!(&original, "C:/staszek/without");

    let mut original = "C:\\\\\\\\\\".to_string();
    remove_ending_slashes(&mut original);
    assert_eq!(&original, "C:\\");
}
