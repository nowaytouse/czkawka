use gtk4::prelude::*;
use crate::gui_structs::gui_data::GuiData;

pub(crate) fn connect_similar_image_mutual_exclusion(gui_data: &GuiData) {
    let check_button_image_only_same_size = gui_data.main_notebook.check_button_image_only_same_size.clone();
    let check_button_image_ignore_same_size = gui_data.main_notebook.check_button_image_ignore_same_size.clone();
    let check_button_image_size_ratio = gui_data.main_notebook.check_button_image_size_ratio.clone();
    let entry_image_size_ratio = gui_data.main_notebook.entry_image_size_ratio.clone();

    // Initial state check
    if check_button_image_only_same_size.is_active() {
        check_button_image_ignore_same_size.set_active(false);
        check_button_image_ignore_same_size.set_sensitive(false);
        check_button_image_size_ratio.set_active(false);
        check_button_image_size_ratio.set_sensitive(false);
        entry_image_size_ratio.set_sensitive(false);
    }

    check_button_image_only_same_size.connect_toggled(move |only_same_size_btn| {
        let is_active = only_same_size_btn.is_active();
        if is_active {
            check_button_image_ignore_same_size.set_active(false);
            check_button_image_ignore_same_size.set_sensitive(false);
            check_button_image_size_ratio.set_active(false);
            check_button_image_size_ratio.set_sensitive(false);
            entry_image_size_ratio.set_sensitive(false);
        } else {
            check_button_image_ignore_same_size.set_sensitive(true);
            check_button_image_size_ratio.set_sensitive(true);
            // Entry sensitivity depends on its own checkbox
            entry_image_size_ratio.set_sensitive(check_button_image_size_ratio.is_active());
        }
    });

    // Also need to handle the ratio entry sensitivity normally
    let entry_image_size_ratio_clone = gui_data.main_notebook.entry_image_size_ratio.clone();
    let check_button_image_only_same_size_clone = gui_data.main_notebook.check_button_image_only_same_size.clone();
    gui_data.main_notebook.check_button_image_size_ratio.connect_toggled(move |ratio_btn| {
        if !check_button_image_only_same_size_clone.is_active() {
            entry_image_size_ratio_clone.set_sensitive(ratio_btn.is_active());
        }
    });
}
