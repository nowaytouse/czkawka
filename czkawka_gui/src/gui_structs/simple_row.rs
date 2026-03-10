//! GObject row type for all non-grouped (no header row) tabs:
//! EmptyFolders, BigFiles, EmptyFiles, Temporary,
//! InvalidSymlinks, BrokenFiles, BadExtensions

use glib::Object;
use gtk4::glib;

mod imp {
    use std::cell::{Cell, RefCell};

    use glib::Properties;
    use glib::object::ObjectExt;
    use gtk4::glib;
    use gtk4::subclass::prelude::*;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::SimpleRow)]
    pub struct SimpleRow {
        #[property(get, set)]
        pub selection_button: Cell<bool>,

        #[property(get, set)]
        pub protected: Cell<bool>,

        #[property(get, set)]
        pub name: RefCell<String>,

        #[property(get, set)]
        pub path: RefCell<String>,

        #[property(get, set)]
        pub modification: RefCell<String>,

        #[property(get, set)]
        pub modification_as_secs: Cell<u64>,

        /// Human-readable file size string (only used by BigFiles, empty for other tabs)
        #[property(get, set)]
        pub size: RefCell<String>,

        /// Raw byte size for sorting (only used by BigFiles, 0 for other tabs)
        #[property(get, set)]
        pub size_as_bytes: Cell<u64>,

        /// Extra column 1:
        /// - InvalidSymlinks → DestinationPath
        /// - BrokenFiles     → ErrorType
        /// - BadExtensions   → CurrentExtension
        /// - Other tabs: empty
        #[property(get, set)]
        pub extra1: RefCell<String>,

        /// Extra column 2:
        /// - InvalidSymlinks → TypeOfError
        /// - BadExtensions   → ValidExtensions
        /// - Other tabs: empty
        #[property(get, set)]
        pub extra2: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SimpleRow {
        const NAME: &'static str = "SimpleRow";
        type Type = super::SimpleRow;
    }

    impl ObjectImpl for SimpleRow {
        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }
        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec);
        }
        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }
}

glib::wrapper! {
    pub struct SimpleRow(ObjectSubclass<imp::SimpleRow>);
}

impl SimpleRow {
    pub fn new(
        selection_button: bool,
        protected: bool,
        name: String,
        path: String,
        modification: String,
        modification_as_secs: u64,
        size: String,
        size_as_bytes: u64,
        extra1: String,
        extra2: String,
    ) -> Self {
        Object::builder()
            .property("selection-button", selection_button)
            .property("protected", protected)
            .property("name", name)
            .property("path", path)
            .property("modification", modification)
            .property("modification-as-secs", modification_as_secs)
            .property("size", size)
            .property("size-as-bytes", size_as_bytes)
            .property("extra1", extra1)
            .property("extra2", extra2)
            .build()
    }
}
