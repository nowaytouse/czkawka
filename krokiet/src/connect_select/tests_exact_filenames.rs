use slint::{Model, ModelRc};

use super::{Property, prevent_selecting_protected_rows, select_by_property};
use crate::common::{create_model_from_model_vec, split_u64_into_i32s};
use crate::test_common::get_main_list_model;
use crate::{ActiveTab, SingleMainListModel};

fn filename_groups(first: u64, second: u64) -> ModelRc<SingleMainListModel> {
    let mut header = get_main_list_model();
    header.header_row = true;
    let item = |value| {
        let (high, low) = split_u64_into_i32s(value);
        SingleMainListModel {
            val_int: create_model_from_model_vec(&[high, low, high, low]),
            ..get_main_list_model()
        }
    };
    create_model_from_model_vec(&[header.clone(), item(first), item(first), header, item(second), item(second)])
}

#[test]
fn exact_filename_extreme_selection_stays_in_zero_valued_groups() {
    for property in [Property::Size, Property::Date] {
        let model = filename_groups(100, 0);
        let (checked, _, selected) = select_by_property(&model, ActiveTab::ExactFilenames, property, true);
        assert_eq!(checked, 2);
        assert_eq!(selected.iter().enumerate().filter_map(|(idx, row)| row.checked.then_some(idx)).collect::<Vec<_>>(), [1, 4]);
    }
}

#[test]
fn exact_filename_minimum_selection_handles_maximum_values_and_ties() {
    let model = filename_groups(100, u64::MAX);
    let (_, _, selected) = select_by_property(&model, ActiveTab::ExactFilenames, Property::Size, false);
    assert_eq!(selected.iter().enumerate().filter_map(|(idx, row)| row.checked.then_some(idx)).collect::<Vec<_>>(), [1, 4]);
}

#[test]
fn exact_filename_extreme_selection_preserves_protection_and_selection_deltas() {
    let model = filename_groups(100, 0);
    let mut protected = model.row_data(1).unwrap();
    protected.protected = true;
    model.set_row_data(1, protected);
    let mut checked = model.row_data(5).unwrap();
    checked.checked = true;
    model.set_row_data(5, checked);

    let (_, _, selected) = select_by_property(&model, ActiveTab::ExactFilenames, Property::Size, true);
    let (checked, unchecked, selected) = prevent_selecting_protected_rows(&model, &selected);
    assert_eq!((checked, unchecked), (1, 1));
    assert!(!selected.row_data(1).unwrap().checked);
    assert!(selected.row_data(4).unwrap().checked);
    assert!(!selected.row_data(5).unwrap().checked);
}
