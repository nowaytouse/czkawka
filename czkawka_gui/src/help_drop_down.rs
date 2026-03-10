//! Helper functions: DropDown + StringList (replacing the deprecated ComboBoxText)

use gtk4::prelude::*;
use gtk4::{DropDown, StringList};

/// Populate a StringList with the given items and select the first entry in the DropDown.
pub fn set_drop_down_model_and_first<I, S>(drop_down: &DropDown, string_list: &StringList, items: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let v: Vec<String> = items.into_iter().map(|s| s.as_ref().to_string()).collect();
    let refs: Vec<&str> = v.iter().map(|s| s.as_str()).collect();
    let n = string_list.n_items();
    string_list.splice(0, n, &refs);
    drop_down.set_selected(0);
}

/// Get the text of the currently selected item (when the model is a StringList).
pub fn drop_down_selected_text(drop_down: &DropDown) -> Option<String> {
    drop_down
        .selected_item()
        .and_then(|o| o.downcast::<gtk4::StringObject>().ok())
        .map(|s| s.string().to_string())
}
