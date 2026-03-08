use gtk4::prelude::*;
use crate::gui_structs::gui_data::GuiData;

pub(crate) fn connect_similar_image_mutual_exclusion(gui_data: &GuiData) {
    let check_button_image_only_same_size = gui_data.main_notebook.check_button_image_only_same_size.clone();
    let check_button_image_ignore_same_size = gui_data.main_notebook.check_button_image_ignore_same_size.clone();
    let check_button_image_size_ratio = gui_data.main_notebook.check_button_image_size_ratio.clone();
    let entry_image_size_ratio = gui_data.main_notebook.entry_image_size_ratio.clone();

    // Initial state check
    let only_active = check_button_image_only_same_size.is_active();
    let ignore_active = check_button_image_ignore_same_size.is_active();
    let ratio_active = check_button_image_size_ratio.is_active();

    if only_active {
        check_button_image_ignore_same_size.set_sensitive(false);
        check_button_image_size_ratio.set_sensitive(false);
        entry_image_size_ratio.set_sensitive(false);
    } else {
        if ignore_active || ratio_active {
            check_button_image_only_same_size.set_sensitive(false);
        }
        entry_image_size_ratio.set_sensitive(ratio_active);
    }

    // Connect Only Same Size
    let ignore_btn_clone = check_button_image_ignore_same_size.clone();
    let ratio_btn_clone = check_button_image_size_ratio.clone();
    let ratio_entry_clone = entry_image_size_ratio.clone();
    check_button_image_only_same_size.connect_toggled(move |only_same_size_btn| {
        let is_active = only_same_size_btn.is_active();
        if is_active {
            ignore_btn_clone.set_active(false);
            ignore_btn_clone.set_sensitive(false);
            ratio_btn_clone.set_active(false);
            ratio_btn_clone.set_sensitive(false);
            ratio_entry_clone.set_sensitive(false);
        } else {
            ignore_btn_clone.set_sensitive(true);
            ratio_btn_clone.set_sensitive(true);
            ratio_entry_clone.set_sensitive(ratio_btn_clone.is_active());
        }
    });

    // Connect Ignore Same Size
    let only_btn_clone = check_button_image_only_same_size.clone();
    let ratio_btn_clone_for_ignore = check_button_image_size_ratio.clone();
    check_button_image_ignore_same_size.connect_toggled(move |ignore_btn| {
        let is_active = ignore_btn.is_active();
        if is_active {
            only_btn_clone.set_active(false);
            only_btn_clone.set_sensitive(false);
        } else if !ratio_btn_clone_for_ignore.is_active() {
            only_btn_clone.set_sensitive(true);
        }
    });

    // Connect Size Ratio Filter
    let only_btn_clone2 = check_button_image_only_same_size.clone();
    let ratio_entry_clone2 = entry_image_size_ratio.clone();
    let ignore_btn_clone2 = check_button_image_ignore_same_size.clone();
    check_button_image_size_ratio.connect_toggled(move |ratio_btn| {
        let is_active = ratio_btn.is_active();
        if is_active {
            only_btn_clone2.set_active(false);
            only_btn_clone2.set_sensitive(false);
            ratio_entry_clone2.set_sensitive(true);
        } else {
            ratio_entry_clone2.set_sensitive(false);
            if !ignore_btn_clone2.is_active() {
                only_btn_clone2.set_sensitive(true);
            }
        }
    });
}
