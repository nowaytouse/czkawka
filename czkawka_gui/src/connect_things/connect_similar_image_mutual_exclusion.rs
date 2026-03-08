use std::cell::Cell;
use std::rc::Rc;
use gtk4::prelude::*;
use crate::gui_structs::gui_data::GuiData;

pub(crate) fn connect_similar_image_mutual_exclusion(gui_data: &GuiData) {
    let check_button_image_only_same_size = gui_data.main_notebook.check_button_image_only_same_size.clone();
    let check_button_image_ignore_same_size = gui_data.main_notebook.check_button_image_ignore_same_size.clone();
    let check_button_image_size_ratio = gui_data.main_notebook.check_button_image_size_ratio.clone();
    let entry_image_size_ratio = gui_data.main_notebook.entry_image_size_ratio.clone();
    let scale_similarity_similar_images = gui_data.main_notebook.scale_similarity_similar_images.clone();

    let old_value = Rc::new(Cell::new(scale_similarity_similar_images.value()));

    // Initial state check
    if check_button_image_only_same_size.is_active() {
        check_button_image_ignore_same_size.set_active(false);
        check_button_image_ignore_same_size.set_sensitive(false);
        check_button_image_size_ratio.set_active(false);
        check_button_image_size_ratio.set_sensitive(false);
        entry_image_size_ratio.set_sensitive(false);
        scale_similarity_similar_images.set_sensitive(false);
        old_value.set(scale_similarity_similar_images.value());
        scale_similarity_similar_images.set_value(scale_similarity_similar_images.adjustment().upper());
    }

    let old_value_clone = old_value.clone();
    let scale_similarity_clone = scale_similarity_similar_images.clone();
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
            
            old_value_clone.set(scale_similarity_clone.value());
            scale_similarity_clone.set_value(scale_similarity_clone.adjustment().upper());
            scale_similarity_clone.set_sensitive(false);
        } else {
            ignore_btn_clone.set_sensitive(true);
            ratio_btn_clone.set_sensitive(true);
            // Entry sensitivity depends on its own checkbox
            ratio_entry_clone.set_sensitive(ratio_btn_clone.is_active());
            
            scale_similarity_clone.set_value(old_value_clone.get());
            scale_similarity_clone.set_sensitive(true);
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
