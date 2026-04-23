use std::path::PathBuf;

use gtk4::gio;
use gtk4::prelude::*;

/// Collect paths from the ListModel returned by FileDialog::select_multiple_folders.
pub fn paths_from_list_model(list_model: &gio::ListModel) -> Vec<PathBuf> {
    let mut folders = Vec::new();
    let n = list_model.n_items();
    for i in 0..n {
        if let Some(obj) = list_model.item(i)
            && let Ok(file) = obj.downcast::<gio::File>()
            && let Some(p) = file.path()
        {
            folders.push(p);
        }
    }
    folders
}
