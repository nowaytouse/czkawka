//! 重复项行对象，用于 ColumnView + gio::ListStore 迁移（替代 TreeView/ListStore）。

use std::cell::RefCell;

use glib::prelude::*;
use glib::subclass::prelude::*;

mod imp {
    use super::{RefCell, ObjectSubclass, ObjectExt, ObjectImpl, DerivedObjectProperties};

    #[derive(glib::Properties, Default)]
    #[properties(wrapper_type = super::DuplicateRow)]
    pub struct DuplicateRow {
        #[property(get, set)]
        pub(super) activatable_select_button: RefCell<bool>,
        #[property(get, set)]
        pub(super) selection_button: RefCell<bool>,
        #[property(get, set)]
        pub(super) protected: RefCell<bool>,
        #[property(get, set)]
        pub(super) size: RefCell<String>,
        #[property(get, set)]
        pub(super) size_as_bytes: RefCell<u64>,
        #[property(get, set)]
        pub(super) name: RefCell<String>,
        #[property(get, set)]
        pub(super) path: RefCell<String>,
        #[property(get, set)]
        pub(super) modification: RefCell<String>,
        #[property(get, set)]
        pub(super) modification_as_secs: RefCell<u64>,
        #[property(get, set)]
        pub(super) color: RefCell<String>,
        #[property(get, set)]
        pub(super) is_header: RefCell<bool>,
        #[property(get, set)]
        pub(super) text_color: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DuplicateRow {
        const NAME: &'static str = "CzkawkaDuplicateRow";
        type Type = super::DuplicateRow;
    }

    #[glib::derived_properties]
    impl ObjectImpl for DuplicateRow {}
}

glib::wrapper! {
    pub struct DuplicateRow(ObjectSubclass<imp::DuplicateRow>);
}

impl DuplicateRow {
    pub fn new(
        activatable_select_button: bool,
        selection_button: bool,
        protected: bool,
        size: String,
        size_as_bytes: u64,
        name: String,
        path: String,
        modification: String,
        modification_as_secs: u64,
        color: String,
        is_header: bool,
        text_color: String,
    ) -> Self {
        glib::Object::builder()
            .property("activatable-select-button", activatable_select_button)
            .property("selection-button", selection_button)
            .property("protected", protected)
            .property("size", size)
            .property("size-as-bytes", size_as_bytes)
            .property("name", name)
            .property("path", path)
            .property("modification", modification)
            .property("modification-as-secs", modification_as_secs)
            .property("color", color)
            .property("is-header", is_header)
            .property("text-color", text_color)
            .build()
    }
}
