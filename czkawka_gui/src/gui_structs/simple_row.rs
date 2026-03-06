//! GObject 行类型，用于所有无分组（无 header 行）的 Tab：
//! EmptyFolders, BigFiles, EmptyFiles, Temporary,
//! InvalidSymlinks, BrokenFiles, BadExtensions

use glib::Object;
use glib::prelude::*;
use gtk4::glib;

mod imp {
    use std::cell::{Cell, RefCell};

    use glib::Properties;
    use gtk4::glib;
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::*;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::SimpleRow)]
    pub struct SimpleRow {
        #[property(get, set)]
        pub selection_button: Cell<bool>,

        #[property(get, set)]
        pub name: RefCell<String>,

        #[property(get, set)]
        pub path: RefCell<String>,

        #[property(get, set)]
        pub modification: RefCell<String>,

        #[property(get, set)]
        pub modification_as_secs: Cell<u64>,

        /// 显示用的文件大小字符串（仅 BigFiles 使用，其他 Tab 留空）
        #[property(get, set)]
        pub size: RefCell<String>,

        /// 原始字节大小（仅 BigFiles 使用，排序用，其他 Tab 为 0）
        #[property(get, set)]
        pub size_as_bytes: Cell<u64>,

        /// 附加列1：
        /// - InvalidSymlinks → DestinationPath
        /// - BrokenFiles     → ErrorType
        /// - BadExtensions   → CurrentExtension
        /// 其他 Tab 留空
        #[property(get, set)]
        pub extra1: RefCell<String>,

        /// 附加列2：
        /// - InvalidSymlinks → TypeOfError
        /// - BadExtensions   → ValidExtensions
        /// 其他 Tab 留空
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
            self.derived_set_property(id, value, pspec)
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
