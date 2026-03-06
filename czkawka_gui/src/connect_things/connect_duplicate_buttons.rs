use czkawka_core::common::model::CheckingMethod;
use gtk4::prelude::*;

use crate::gui_structs::gui_data::GuiData;
use crate::help_combo_box::DUPLICATES_CHECK_METHOD_COMBO_BOX;

pub(crate) fn connect_duplicate_combo_box(gui_data: &GuiData) {
    let combo_box_duplicate_check_method = gui_data.main_notebook.combo_box_duplicate_check_method.clone();
    let combo_box_duplicate_hash_type = gui_data.main_notebook.combo_box_duplicate_hash_type.clone();
    let label_duplicate_hash_type = gui_data.main_notebook.label_duplicate_hash_type.clone();
    let check_button_duplicate_case_sensitive_name = gui_data.main_notebook.check_button_duplicate_case_sensitive_name.clone();
    combo_box_duplicate_check_method.connect_selected_notify(move |combo_box_duplicate_check_method| {
        let chosen_index = combo_box_duplicate_check_method.selected();
        if chosen_index != u32::MAX && (chosen_index as usize) < DUPLICATES_CHECK_METHOD_COMBO_BOX.len() {
            if DUPLICATES_CHECK_METHOD_COMBO_BOX[chosen_index as usize].check_method == CheckingMethod::Hash {
                combo_box_duplicate_hash_type.set_visible(true);
                label_duplicate_hash_type.set_visible(true);
            } else {
                combo_box_duplicate_hash_type.set_visible(false);
                label_duplicate_hash_type.set_visible(false);
            }

            if [CheckingMethod::Name, CheckingMethod::SizeName].contains(&DUPLICATES_CHECK_METHOD_COMBO_BOX[chosen_index as usize].check_method) {
                check_button_duplicate_case_sensitive_name.set_visible(true);
            } else {
                check_button_duplicate_case_sensitive_name.set_visible(false);
            }
        }
    });
}
