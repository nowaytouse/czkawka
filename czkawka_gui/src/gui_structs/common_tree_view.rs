use std::cell::RefCell;
use std::rc::Rc;

use czkawka_core::common::image::{check_if_can_display_image, get_dynamic_image_from_path};
use czkawka_core::common::traits::PrintResults;
use czkawka_core::tools::bad_extensions::BadExtensions;
use czkawka_core::tools::big_file::BigFile;
use czkawka_core::tools::broken_files::BrokenFiles;
use czkawka_core::tools::duplicate::DuplicateFinder;
use czkawka_core::tools::empty_files::EmptyFiles;
use czkawka_core::tools::empty_folder::EmptyFolder;
use czkawka_core::tools::invalid_symlinks::InvalidSymlinks;
use czkawka_core::tools::same_music::SameMusic;
use czkawka_core::tools::similar_images::SimilarImages;
use czkawka_core::tools::similar_videos::SimilarVideos;
use czkawka_core::tools::temporary::Temporary;
use gdk4::gdk_pixbuf::{InterpType, Pixbuf};
use gdk4::Texture;
use gtk4::gio::ListStore as GioListStore;
use gtk4::prelude::*;
use gtk4::{
    Builder, CellRendererText, CellRendererToggle, CheckButton, ColumnView, ColumnViewColumn, EventControllerKey, GestureClick, Label, ListStore, MultiSelection,
    Notebook, Picture, ScrolledWindow, SelectionMode, SignalListItemFactory, TextView, TreeModel, TreeSelection, TreeView, TreeViewColumn,
};

use crate::connect_things::connect_button_delete::delete_things;
use crate::flg;
use crate::gui_structs::gui_data::GuiData;
use crate::help_functions::{KEY_DELETE, SharedState, add_text_to_text_view, get_full_name_from_path_name};
use crate::helpers::enums::{
    ColumnsBadExtensions, ColumnsBigFiles, ColumnsBrokenFiles, ColumnsDuplicates, ColumnsEmptyFiles, ColumnsEmptyFolders, ColumnsInvalidSymlinks, ColumnsSameMusic,
    ColumnsSimilarImages, ColumnsSimilarVideos, ColumnsTemporaryFiles,
};
use crate::helpers::image_operations::{get_pixbuf_from_dynamic_image, resize_pixbuf_dimension};
use crate::notebook_enums::NotebookMainEnum;
use crate::gui_structs::duplicate_row::DuplicateRow;
use crate::gui_structs::simple_row::SimpleRow;
use crate::notebook_info::{NOTEBOOKS_INFO, NotebookObject};
use crate::opening_selecting_records::{opening_double_click_function, opening_enter_function_ported, opening_middle_mouse_function, select_function_header};

#[derive(Clone)]
pub struct CommonTreeViews {
    pub subviews: Vec<SubView>,
    pub notebook_main: Notebook,
    pub preview_path: Rc<RefCell<String>>,
}
impl CommonTreeViews {
    pub fn get_subview(&self, item: NotebookMainEnum) -> &SubView {
        self.subviews.iter().find(|s| s.enum_value == item).expect("Cannot find subview")
    }
    pub fn get_current_page(&self) -> NotebookMainEnum {
        let current_page = self.notebook_main.current_page().expect("Cannot get current page from notebook");
        NOTEBOOKS_INFO[current_page as usize].notebook_type
    }
    pub fn get_current_subview(&self) -> &SubView {
        let current_page = self.notebook_main.current_page().expect("Cannot get current page from notebook");
        let enum_value = NOTEBOOKS_INFO[current_page as usize].notebook_type;
        self.get_subview(enum_value)
    }
    pub fn hide_preview(&self) {
        let current_subview = self.get_current_subview();
        if let Some(preview_struct) = &current_subview.preview_struct {
            preview_struct.image_preview.set_visible(false);
        }
        *self.preview_path.borrow_mut() = String::new();
    }
    // pub fn get_tree_view_from_its_name(&self, name: &str) -> TreeView {
    //     for subview in &self.subviews {
    //         if subview.tree_view_name == name {
    //             return subview.tree_view.clone();
    //         }
    //     }
    //     panic!("Cannot find tree view with name {name}");
    // }
    pub fn setup(&self, gui_data: &GuiData) {
        for subview in &self.subviews {
            subview.setup(&self.preview_path, gui_data);
        }
    }
}

pub trait TreeViewListStoreTrait {
    fn get_model(&self) -> ListStore;
}
impl TreeViewListStoreTrait for TreeView {
    fn get_model(&self) -> ListStore {
        self.model()
            .expect("TreeView has no model")
            .downcast_ref::<ListStore>()
            .expect("TreeView model is not ListStore")
            .clone()
    }
}
pub trait GetTreeViewTrait {
    fn get_tree_view(&self) -> TreeView;
}
impl GetTreeViewTrait for &EventControllerKey {
    fn get_tree_view(&self) -> TreeView {
        self.widget()
            .expect("EventControllerKey has no widget")
            .downcast_ref::<TreeView>()
            .expect("EventControllerKey widget is not TreeView")
            .clone()
    }
}
impl GetTreeViewTrait for &GestureClick {
    fn get_tree_view(&self) -> TreeView {
        self.widget()
            .expect("GestureClick has no widget")
            .downcast_ref::<TreeView>()
            .expect("GestureClick widget is not TreeView")
            .clone()
    }
}

/// 不含分组行的简单 Tab（EmptyFolders/BigFiles/EmptyFiles/Temporary/Symlinks/BrokenFiles/BadExtensions）
pub fn is_simple_tab(v: NotebookMainEnum) -> bool {
    matches!(
        v,
        NotebookMainEnum::EmptyDirectories
            | NotebookMainEnum::BigFiles
            | NotebookMainEnum::EmptyFiles
            | NotebookMainEnum::Temporary
            | NotebookMainEnum::Symlinks
            | NotebookMainEnum::BrokenFiles
            | NotebookMainEnum::BadExtensions
    )
}

/// 返回每个简单 Tab 的 ColumnView 文本列配置：(初始标题, SimpleRow 属性名)
/// 不含第一列（勾选框）
fn simple_column_config(v: NotebookMainEnum) -> &'static [(&'static str, &'static str)] {
    match v {
        NotebookMainEnum::EmptyDirectories | NotebookMainEnum::EmptyFiles | NotebookMainEnum::Temporary => {
            &[("Name", "name"), ("Path", "path"), ("Modification", "modification")]
        }
        NotebookMainEnum::BigFiles => &[("Size", "size"), ("Name", "name"), ("Path", "path"), ("Modification", "modification")],
        NotebookMainEnum::Symlinks => &[
            ("Name", "name"),
            ("Path", "path"),
            ("Destination Path", "extra1"),
            ("Error Type", "extra2"),
            ("Modification", "modification"),
        ],
        NotebookMainEnum::BrokenFiles => &[("Name", "name"), ("Path", "path"), ("Error Type", "extra1"), ("Modification", "modification")],
        NotebookMainEnum::BadExtensions => &[("Name", "name"), ("Path", "path"), ("Extension", "extra1"), ("Valid Extensions", "extra2")],
        _ => unreachable!("Not a simple tab"),
    }
}

#[derive(Clone)]
pub struct SubView {
    pub scrolled_window: ScrolledWindow,
    pub tree_view: TreeView,
    /// 重复项 Tab 使用 ColumnView 时的视图与模型（替代 TreeView/ListStore）
    pub duplicate_column_view: Option<ColumnView>,
    pub duplicate_list_store: Option<GioListStore>,
    pub duplicate_selection: Option<MultiSelection>,
    /// 简单 Tab（无分组行）使用 ColumnView 时的视图与模型
    pub simple_column_view: Option<ColumnView>,
    pub simple_list_store: Option<GioListStore>,
    pub gesture_click: GestureClick,
    pub event_controller_key: EventControllerKey,
    pub nb_object: NotebookObject,
    pub enum_value: NotebookMainEnum,
    pub preview_struct: Option<PreviewStruct>,
    pub shared_model_enum: SharedModelEnum,
}

#[derive(Clone)]
pub struct PreviewStruct {
    pub image_preview: Picture,
    pub settings_show_preview: CheckButton,
}

/// 为重复项 Tab 创建 ColumnView + gio::ListStore\<DuplicateRow\> + MultiSelection，并设为 scrolled 子控件。
fn create_duplicate_column_view(scrolled_window: &ScrolledWindow) -> (ColumnView, GioListStore, MultiSelection) {
    let list_store = GioListStore::builder().item_type(DuplicateRow::static_type()).build();
    let selection = MultiSelection::new(Some(list_store.clone().upcast::<gtk4::gio::ListModel>()));
    let column_view = ColumnView::new(Some(selection.clone().upcast::<gtk4::SelectionModel>()));

    // 选择列（勾选框）
    let factory_select = SignalListItemFactory::new();
    factory_select.connect_setup(move |_factory, obj| {
        let list_item = obj.downcast_ref::<gtk4::ListItem>().expect("ListItem");
        let check = CheckButton::new();
        list_item.set_child(Some(&check));
    });
    factory_select.connect_bind(move |_factory, obj| {
        let list_item = obj.downcast_ref::<gtk4::ListItem>().expect("ListItem");
        let Some(item) = list_item.item() else { return };
        let Ok(row) = item.downcast::<DuplicateRow>() else { return };
        let child = list_item.child().and_downcast::<CheckButton>().expect("child is CheckButton");
        row.bind_property("selection-button", &child, "active")
            .sync_create()
            .bidirectional()
            .build();
        row.bind_property("activatable-select-button", &child, "sensitive")
            .sync_create()
            .build();
    });
    factory_select.connect_unbind(move |_factory, obj| {
        let list_item = obj.downcast_ref::<gtk4::ListItem>().expect("ListItem");
        if let Some(child) = list_item.child().and_downcast::<CheckButton>() {
            let id_opt = unsafe { list_item.data::<glib::SignalHandlerId>("toggled_id") };
            if let Some(id) = id_opt {
                let handler_id = unsafe { std::ptr::read(id.as_ptr()) };
                child.disconnect(handler_id);
            }
        }
    });
    let col_select = ColumnViewColumn::new(Some(""), Some(factory_select));
    col_select.set_fixed_width(30);
    col_select.set_resizable(false);
    column_view.insert_column(0, &col_select);

    // 文本列：Size, Name, Path, Modification
    let mut col_idx = 1u32;
    for (title, prop_name) in [
        ("Size", "size"),
        ("Name", "name"),
        ("Path", "path"),
        ("Modification", "modification"),
    ] {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_f, obj| {
            let list_item = obj.downcast_ref::<gtk4::ListItem>().expect("ListItem");
            let label = Label::new(None);
            label.set_xalign(0.0);
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            list_item.set_child(Some(&label));
        });
        factory.connect_bind(move |_f, obj| {
            let list_item = obj.downcast_ref::<gtk4::ListItem>().expect("ListItem");
            let Some(item) = list_item.item() else { return };
            let Ok(row) = item.downcast::<DuplicateRow>() else { return };
            let child = list_item.child().and_downcast::<Label>().expect("child is Label");
            let text = match prop_name {
                "size" => row.size(),
                "name" => row.name(),
                "path" => row.path(),
                "modification" => row.modification(),
                _ => String::new(),
            };
            child.set_text(&text);

            if row.protected() {
                child.add_css_class("protected-file");
            } else {
                child.remove_css_class("protected-file");
            }

            let child_clone = child;
            let id = row.connect_protected_notify(move |r| {
                if r.protected() {
                    child_clone.add_css_class("protected-file");
                } else {
                    child_clone.remove_css_class("protected-file");
                }
            });
            unsafe { list_item.set_data("protected_id", id) };
        });
        factory.connect_unbind(move |_f, obj| {
            let list_item = obj.downcast_ref::<gtk4::ListItem>().expect("ListItem");
            if let Some(item) = list_item.item() {
                let id_opt = unsafe { list_item.data::<glib::SignalHandlerId>("protected_id") };
                if let Some(id) = id_opt {
                    let handler_id = unsafe { std::ptr::read(id.as_ptr()) };
                    item.disconnect(handler_id);
                }
            }
        });
        let col = ColumnViewColumn::new(Some(title), Some(factory));
        col.set_resizable(true);
        match prop_name {
            "size" => col.set_fixed_width(100),
            "name" => col.set_fixed_width(200),
            "path" => {
                col.set_fixed_width(-1);
                col.set_expand(true);
            }
            "modification" => col.set_fixed_width(180),
            _ => col.set_fixed_width(120),
        }
        column_view.insert_column(col_idx, &col);
        col_idx += 1;
    }

    column_view.set_vexpand(true);
    scrolled_window.set_child(Some(&column_view));
    (column_view, list_store, selection)
}

/// 为简单 Tab 创建 ColumnView + GioListStore\<SimpleRow\> + MultiSelection，并设为 scrolled 子控件。
fn create_simple_column_view(scrolled_window: &ScrolledWindow, enum_value: NotebookMainEnum) -> (ColumnView, GioListStore, MultiSelection) {
    let list_store = GioListStore::builder().item_type(SimpleRow::static_type()).build();
    let selection = MultiSelection::new(Some(list_store.clone().upcast::<gtk4::gio::ListModel>()));
    let column_view = ColumnView::new(Some(selection.clone().upcast::<gtk4::SelectionModel>()));

    // 勾选框列
    let factory_select = SignalListItemFactory::new();
    factory_select.connect_setup(move |_factory, obj| {
        let list_item = obj.downcast_ref::<gtk4::ListItem>().expect("ListItem");
        let check = CheckButton::new();
        list_item.set_child(Some(&check));
    });
    factory_select.connect_bind(move |_factory, obj| {
        let list_item = obj.downcast_ref::<gtk4::ListItem>().expect("ListItem");
        let Some(item) = list_item.item() else { return };
        let Ok(row) = item.downcast::<SimpleRow>() else { return };
        let child = list_item.child().and_downcast::<CheckButton>().expect("child is CheckButton");
        row.bind_property("selection-button", &child, "active")
            .sync_create()
            .bidirectional()
            .build();
        row.bind_property("protected", &child, "sensitive")
            .transform_to(|_b, protected: bool| Some(!protected)) // Sensitive if NOT protected
            .sync_create()
            .build();
    });
    factory_select.connect_unbind(move |_factory, obj| {
        let list_item = obj.downcast_ref::<gtk4::ListItem>().expect("ListItem");
        if let Some(child) = list_item.child().and_downcast::<CheckButton>() {
            let id_opt = unsafe { list_item.data::<glib::SignalHandlerId>("toggled_id") };
            if let Some(id) = id_opt {
                let handler_id = unsafe { std::ptr::read(id.as_ptr()) };
                child.disconnect(handler_id);
            }
        }
    });
    let col_select = ColumnViewColumn::new(Some(""), Some(factory_select));
    col_select.set_fixed_width(30);
    col_select.set_resizable(false);
    column_view.insert_column(0, &col_select);

    // 文本列：根据 Tab 类型配置
    let mut col_idx = 1u32;
    for (title, prop_name) in simple_column_config(enum_value) {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_f, obj| {
            let list_item = obj.downcast_ref::<gtk4::ListItem>().expect("ListItem");
            let label = Label::new(None);
            label.set_xalign(0.0);
            label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
            list_item.set_child(Some(&label));
        });
        factory.connect_bind(move |_f, obj| {
            let list_item = obj.downcast_ref::<gtk4::ListItem>().expect("ListItem");
            let Some(item) = list_item.item() else { return };
            let Ok(row) = item.downcast::<SimpleRow>() else { return };
            let child = list_item.child().and_downcast::<Label>().expect("child is Label");
            let text = match *prop_name {
                "name" => row.name(),
                "path" => row.path(),
                "modification" => row.modification(),
                "size" => row.size(),
                "extra1" => row.extra1(),
                "extra2" => row.extra2(),
                _ => String::new(),
            };
            child.set_text(&text);

            if row.protected() {
                child.add_css_class("protected-file");
            } else {
                child.remove_css_class("protected-file");
            }

            let child_clone = child;
            let id = row.connect_protected_notify(move |r| {
                if r.protected() {
                    child_clone.add_css_class("protected-file");
                } else {
                    child_clone.remove_css_class("protected-file");
                }
            });
            unsafe { list_item.set_data("protected_id", id) };
        });
        factory.connect_unbind(move |_f, obj| {
            let list_item = obj.downcast_ref::<gtk4::ListItem>().expect("ListItem");
            if let Some(item) = list_item.item() {
                let id_opt = unsafe { list_item.data::<glib::SignalHandlerId>("protected_id") };
                if let Some(id) = id_opt {
                    let handler_id = unsafe { std::ptr::read(id.as_ptr()) };
                    item.disconnect(handler_id);
                }
            }
        });
        let col = ColumnViewColumn::new(Some(*title), Some(factory));
        col.set_resizable(true);
        match *prop_name {
            "size" => col.set_fixed_width(100),
            "name" => col.set_fixed_width(200),
            "path" => {
                col.set_fixed_width(-1);
                col.set_expand(true);
            }
            "modification" => col.set_fixed_width(180),
            _ => col.set_fixed_width(150),
        }
        column_view.insert_column(col_idx, &col);
        col_idx += 1;
    }

    column_view.set_vexpand(true);
    scrolled_window.set_child(Some(&column_view));
    (column_view, list_store, selection)
}

impl SubView {
    pub fn get_model(&self) -> ListStore {
        self.tree_view.get_model()
    }
    pub fn get_tree_model(&self) -> TreeModel {
        self.tree_view.model().expect("TreeView has no model")
    }
    pub fn get_tree_selection(&self) -> TreeSelection {
        self.tree_view.selection()
    }
    pub fn get_duplicate_model(&self) -> Option<&GioListStore> {
        self.duplicate_list_store.as_ref()
    }
    pub fn get_duplicate_selection(&self) -> Option<&MultiSelection> {
        self.duplicate_selection.as_ref()
    }
    pub fn get_simple_model(&self) -> Option<&GioListStore> {
        self.simple_list_store.as_ref()
    }
    pub fn new(
        builder: &Builder,
        scrolled_name: &str,
        enum_value: NotebookMainEnum,
        preview_str: Option<&str>,
        settings_show_preview: Option<CheckButton>,
        shared_model_enum: SharedModelEnum,
    ) -> Self {
        let tree_view: TreeView = TreeView::new();
        let event_controller_key: EventControllerKey = EventControllerKey::new();
        let gesture_click: GestureClick = GestureClick::new();

        let image_preview = preview_str.map(|name| builder.object(name).unwrap_or_else(|| panic!("Cannot find preview image {name}")));

        let nb_object = NOTEBOOKS_INFO[enum_value as usize].clone();
        assert_eq!(nb_object.notebook_type, enum_value);

        let preview_struct = if let (Some(image_preview), Some(settings_show_preview)) = (image_preview, settings_show_preview) {
            Some(PreviewStruct {
                image_preview,
                settings_show_preview,
            })
        } else {
            None
        };

        let scrolled_window: ScrolledWindow = builder.object(scrolled_name).unwrap_or_else(|| panic!("Cannot find scrolled window {scrolled_name}"));

        let (duplicate_column_view, duplicate_list_store, duplicate_selection, simple_column_view, simple_list_store, _simple_selection) =
            if enum_value == NotebookMainEnum::Duplicate {
                let (cv, store, sel) = create_duplicate_column_view(&scrolled_window);
                cv.add_controller(event_controller_key.clone());
                cv.add_controller(gesture_click.clone());
                (Some(cv), Some(store), Some(sel), None, None, None)
            } else if is_simple_tab(enum_value) {
                let (cv, store, sel) = create_simple_column_view(&scrolled_window, enum_value);
                cv.add_controller(event_controller_key.clone());
                cv.add_controller(gesture_click.clone());
                (None, None, None, Some(cv), Some(store), Some(sel))
            } else {
                tree_view.add_controller(event_controller_key.clone());
                tree_view.add_controller(gesture_click.clone());
                (None, None, None, None, None, None)
            };

        Self {
            scrolled_window,
            tree_view,
            duplicate_column_view,
            duplicate_list_store,
            duplicate_selection,
            simple_column_view,
            simple_list_store,
            gesture_click,
            event_controller_key,
            nb_object,
            enum_value,
            preview_struct,
            shared_model_enum,
        }
    }

    fn _setup_tree_view(&self) {
        if self.duplicate_column_view.is_some() || self.simple_column_view.is_some() {
            return;
        }
        self.tree_view.set_model(Some(&ListStore::new(self.nb_object.columns_types)));
        self.tree_view.selection().set_mode(SelectionMode::Multiple);

        if let Some(column_header) = self.nb_object.column_header {
            self.tree_view.selection().set_select_function(select_function_header(column_header));
        }

        self.tree_view.set_vexpand(true);

        self._setup_tree_view_config();

        self.tree_view.set_widget_name(self.nb_object.tree_view_name);
        self.scrolled_window.set_child(Some(&self.tree_view));
        self.scrolled_window.set_visible(true);
    }
    fn _setup_gesture_click(&self) {
        self.gesture_click.set_button(0);
        self.gesture_click.connect_pressed(opening_double_click_function);
        self.gesture_click.connect_released(opening_middle_mouse_function); // TODO GTK 4 - https://github.com/gtk-rs/gtk4-rs/issues/1043
    }

    fn _setup_evk(&self, gui_data: &GuiData) {
        let gui_data_clone = gui_data.clone();
        self.event_controller_key.connect_key_pressed(opening_enter_function_ported);

        self.event_controller_key
            .connect_key_released(move |_event_controller_key, _key_value, key_code, _modifier_type| {
                if key_code == KEY_DELETE {
                    glib::MainContext::default().spawn_local(delete_things(gui_data_clone.clone()));
                }
            });
    }

    fn _connect_show_mouse_preview(&self, gui_data: &GuiData, preview_path: &Rc<RefCell<String>>) {
        // TODO GTK 4, currently not works, connect_pressed shows previous thing - https://gitlab.gnome.org/GNOME/gtk/-/issues/4939
        // Use connect_released when it will be fixed, currently using connect_row_activated workaround
        let use_rust_preview = gui_data.settings.check_button_settings_use_rust_preview.clone();
        let text_view_errors = gui_data.text_view_errors.clone();
        if let Some(preview_struct) = self.preview_struct.clone() {
            self.tree_view.set_property("activate-on-single-click", true);
            let preview_path = preview_path.clone();
            let nb_object = self.nb_object.clone();

            self.tree_view.clone().connect_row_activated(move |tree_view, _b, _c| {
                show_preview(
                    tree_view,
                    &text_view_errors,
                    &preview_struct.settings_show_preview,
                    &preview_struct.image_preview,
                    &preview_path,
                    nb_object.column_path,
                    nb_object.column_name,
                    use_rust_preview.is_active(),
                );
            });
        }
    }
    fn _connect_show_keyboard_preview(&self, gui_data: &GuiData, preview_path: &Rc<RefCell<String>>, preview_struct: &PreviewStruct) {
        let use_rust_preview = gui_data.settings.check_button_settings_use_rust_preview.clone();
        let text_view_errors = gui_data.text_view_errors.clone();
        let check_button_settings_show_preview = preview_struct.settings_show_preview.clone();
        let image_preview = preview_struct.image_preview.clone();
        let gui_data_clone = gui_data.clone();

        self.event_controller_key.connect_key_pressed(opening_enter_function_ported);
        let preview_path = preview_path.clone();
        let nb_object = self.nb_object.clone();

        self.event_controller_key
            .clone()
            .connect_key_released(move |event_controller_key, _key_value, key_code, _modifier_type| {
                if key_code == KEY_DELETE {
                    glib::MainContext::default().spawn_local(delete_things(gui_data_clone.clone()));
                }
                show_preview(
                    &event_controller_key.get_tree_view(),
                    &text_view_errors,
                    &check_button_settings_show_preview,
                    &image_preview,
                    &preview_path,
                    nb_object.column_path,
                    nb_object.column_name,
                    use_rust_preview.is_active(),
                );
            });
    }

    /// Connect preview for Duplicate tab's ColumnView.
    /// Uses MultiSelection's selection-changed signal to show preview when a row is clicked.
    fn _connect_show_duplicate_column_view_preview(&self, gui_data: &GuiData, preview_path: &Rc<RefCell<String>>) {
        let Some(preview_struct) = self.preview_struct.clone() else { return };
        let Some(selection) = self.duplicate_selection.clone() else { return };
        let Some(store) = self.duplicate_list_store.clone() else { return };

        let text_view_errors = gui_data.text_view_errors.clone();
        let use_rust_preview = gui_data.settings.check_button_settings_use_rust_preview.clone();
        let preview_path = preview_path.clone();

        selection.connect_selection_changed(move |sel, _pos, _n_items| {
            // Find the first selected non-header item
            let bitset = sel.selection();
            let mut file_name = String::new();

            if let Some((_iter, first_pos)) = gtk4::BitsetIter::init_first(&bitset)
                && let Some(item) = store.item(first_pos)
                    && let Ok(row) = item.downcast::<DuplicateRow>()
                        && !row.is_header() {
                            let path = row.path();
                            let name = row.name();
                            file_name = get_full_name_from_path_name(&path, &name);
                        }

            if file_name.is_empty() {
                preview_struct.image_preview.set_visible(false);
                *preview_path.borrow_mut() = String::new();
                return;
            }

            show_preview_for_file(
                &file_name,
                &text_view_errors,
                &preview_struct.settings_show_preview,
                &preview_struct.image_preview,
                &preview_path,
                use_rust_preview.is_active(),
            );
        });
    }

    fn setup(&self, preview_path: &Rc<RefCell<String>>, gui_data: &GuiData) {
        if let Some(preview_struct) = &self.preview_struct {
            preview_struct.image_preview.set_visible(false);
        }
        self._setup_tree_view();
        self._setup_gesture_click();

        // Duplicate ColumnView has its own preview connection
        if self.duplicate_column_view.is_some() {
            self._connect_show_duplicate_column_view_preview(gui_data, preview_path);
            self._setup_evk(gui_data);
            return;
        }

        self._connect_show_mouse_preview(gui_data, preview_path);

        // Items with image preview, are differently handled
        if let Some(preview_struct) = &self.preview_struct {
            self._connect_show_keyboard_preview(gui_data, preview_path, preview_struct);
        } else {
            self._setup_evk(gui_data);
        }
    }
    fn _setup_tree_view_config(&self) {
        let tree_view = &self.tree_view;
        let model = self.get_model();
        match self.enum_value {
            NotebookMainEnum::Duplicate => {
                let columns_colors = (ColumnsDuplicates::Color as i32, ColumnsDuplicates::TextColor as i32);
                let activatable_colors = (ColumnsDuplicates::ActivatableSelectButton as i32, ColumnsDuplicates::Color as i32);
                create_default_selection_button_column(tree_view, ColumnsDuplicates::SelectionButton as i32, model, Some(activatable_colors));
                create_default_columns(
                    tree_view,
                    &[
                        (ColumnsDuplicates::Size as i32, ColumnSort::None),
                        (ColumnsDuplicates::Name as i32, ColumnSort::None),
                        (ColumnsDuplicates::Path as i32, ColumnSort::None),
                        (ColumnsDuplicates::Modification as i32, ColumnSort::None),
                    ],
                    Some(columns_colors),
                );
                assert_eq!(tree_view.columns().len(), 5);
            }
            NotebookMainEnum::EmptyDirectories => {
                create_default_selection_button_column(tree_view, ColumnsEmptyFolders::SelectionButton as i32, model, None);
                create_default_columns(
                    tree_view,
                    &[
                        (ColumnsEmptyFolders::Name as i32, ColumnSort::Default),
                        (ColumnsEmptyFolders::Path as i32, ColumnSort::Default),
                        (ColumnsEmptyFolders::Modification as i32, ColumnSort::Custom(ColumnsEmptyFolders::ModificationAsSecs as i32)),
                    ],
                    None,
                );
                assert_eq!(tree_view.columns().len(), 4);
            }
            NotebookMainEnum::BigFiles => {
                create_default_selection_button_column(tree_view, ColumnsBigFiles::SelectionButton as i32, model, None);
                create_default_columns(
                    tree_view,
                    &[
                        (ColumnsBigFiles::Size as i32, ColumnSort::Custom(ColumnsBigFiles::SizeAsBytes as i32)),
                        (ColumnsBigFiles::Name as i32, ColumnSort::Default),
                        (ColumnsBigFiles::Path as i32, ColumnSort::Default),
                        (ColumnsBigFiles::Modification as i32, ColumnSort::Custom(ColumnsBigFiles::ModificationAsSecs as i32)),
                    ],
                    None,
                );
                assert_eq!(tree_view.columns().len(), 5);
            }
            NotebookMainEnum::EmptyFiles => {
                create_default_selection_button_column(tree_view, ColumnsEmptyFiles::SelectionButton as i32, model, None);
                create_default_columns(
                    tree_view,
                    &[
                        (ColumnsEmptyFiles::Name as i32, ColumnSort::Default),
                        (ColumnsEmptyFiles::Path as i32, ColumnSort::Default),
                        (ColumnsEmptyFiles::Modification as i32, ColumnSort::Custom(ColumnsEmptyFiles::ModificationAsSecs as i32)),
                    ],
                    None,
                );
                assert_eq!(tree_view.columns().len(), 4);
            }
            NotebookMainEnum::Temporary => {
                create_default_selection_button_column(tree_view, ColumnsTemporaryFiles::SelectionButton as i32, model, None);
                create_default_columns(
                    tree_view,
                    &[
                        (ColumnsTemporaryFiles::Name as i32, ColumnSort::Default),
                        (ColumnsTemporaryFiles::Path as i32, ColumnSort::Default),
                        (
                            ColumnsTemporaryFiles::Modification as i32,
                            ColumnSort::Custom(ColumnsTemporaryFiles::ModificationAsSecs as i32),
                        ),
                    ],
                    None,
                );
                assert_eq!(tree_view.columns().len(), 4);
            }
            NotebookMainEnum::SimilarImages => {
                let columns_colors = (ColumnsSimilarImages::Color as i32, ColumnsSimilarImages::TextColor as i32);
                let activatable_colors = (ColumnsSimilarImages::ActivatableSelectButton as i32, ColumnsSimilarImages::Color as i32);
                create_default_selection_button_column(tree_view, ColumnsSimilarImages::SelectionButton as i32, model, Some(activatable_colors));
                create_default_columns(
                    tree_view,
                    &[
                        (ColumnsSimilarImages::Similarity as i32, ColumnSort::None),
                        (ColumnsSimilarImages::Size as i32, ColumnSort::None),
                        (ColumnsSimilarImages::Dimensions as i32, ColumnSort::None),
                        (ColumnsSimilarImages::Name as i32, ColumnSort::None),
                        (ColumnsSimilarImages::Path as i32, ColumnSort::None),
                        (ColumnsSimilarImages::Modification as i32, ColumnSort::None),
                    ],
                    Some(columns_colors),
                );
                assert_eq!(tree_view.columns().len(), 7);
            }
            NotebookMainEnum::SimilarVideos => {
                let columns_colors = (ColumnsSimilarVideos::Color as i32, ColumnsSimilarVideos::TextColor as i32);
                let activatable_colors = (ColumnsSimilarVideos::ActivatableSelectButton as i32, ColumnsSimilarVideos::Color as i32);
                create_default_selection_button_column(tree_view, ColumnsSimilarVideos::SelectionButton as i32, model, Some(activatable_colors));
                create_default_columns(
                    tree_view,
                    &[
                        (ColumnsSimilarVideos::Size as i32, ColumnSort::None),
                        (ColumnsSimilarVideos::Name as i32, ColumnSort::None),
                        (ColumnsSimilarVideos::Path as i32, ColumnSort::None),
                        (ColumnsSimilarVideos::Modification as i32, ColumnSort::None),
                        (ColumnsSimilarVideos::Fps as i32, ColumnSort::None),
                        (ColumnsSimilarVideos::Codec as i32, ColumnSort::None),
                        (ColumnsSimilarVideos::Bitrate as i32, ColumnSort::None),
                        (ColumnsSimilarVideos::Dimensions as i32, ColumnSort::None),
                    ],
                    Some(columns_colors),
                );
                assert_eq!(tree_view.columns().len(), 9);
            }
            NotebookMainEnum::SameMusic => {
                let columns_colors = (ColumnsSameMusic::Color as i32, ColumnsSameMusic::TextColor as i32);
                let activatable_colors = (ColumnsSameMusic::ActivatableSelectButton as i32, ColumnsSameMusic::Color as i32);
                create_default_selection_button_column(tree_view, ColumnsSameMusic::SelectionButton as i32, model, Some(activatable_colors));
                create_default_columns(
                    tree_view,
                    &[
                        (ColumnsSameMusic::Size as i32, ColumnSort::None),
                        (ColumnsSameMusic::Name as i32, ColumnSort::None),
                        (ColumnsSameMusic::Title as i32, ColumnSort::None),
                        (ColumnsSameMusic::Artist as i32, ColumnSort::None),
                        (ColumnsSameMusic::Year as i32, ColumnSort::None),
                        (ColumnsSameMusic::Bitrate as i32, ColumnSort::None),
                        (ColumnsSameMusic::Length as i32, ColumnSort::None),
                        (ColumnsSameMusic::Genre as i32, ColumnSort::None),
                        (ColumnsSameMusic::Path as i32, ColumnSort::None),
                        (ColumnsSameMusic::Modification as i32, ColumnSort::None),
                    ],
                    Some(columns_colors),
                );
                assert_eq!(tree_view.columns().len(), 11);
            }
            NotebookMainEnum::Symlinks => {
                create_default_selection_button_column(tree_view, ColumnsInvalidSymlinks::SelectionButton as i32, model, None);
                create_default_columns(
                    tree_view,
                    &[
                        (ColumnsInvalidSymlinks::Name as i32, ColumnSort::Default),
                        (ColumnsInvalidSymlinks::Path as i32, ColumnSort::Default),
                        (ColumnsInvalidSymlinks::DestinationPath as i32, ColumnSort::Default),
                        (ColumnsInvalidSymlinks::TypeOfError as i32, ColumnSort::Default),
                        (
                            ColumnsInvalidSymlinks::Modification as i32,
                            ColumnSort::Custom(ColumnsInvalidSymlinks::ModificationAsSecs as i32),
                        ),
                    ],
                    None,
                );
                assert_eq!(tree_view.columns().len(), 6);
            }
            NotebookMainEnum::BrokenFiles => {
                create_default_selection_button_column(tree_view, ColumnsBrokenFiles::SelectionButton as i32, model, None);
                create_default_columns(
                    tree_view,
                    &[
                        (ColumnsBrokenFiles::Name as i32, ColumnSort::Default),
                        (ColumnsBrokenFiles::Path as i32, ColumnSort::Default),
                        (ColumnsBrokenFiles::ErrorType as i32, ColumnSort::Default),
                        (ColumnsBrokenFiles::Modification as i32, ColumnSort::Custom(ColumnsBrokenFiles::ModificationAsSecs as i32)),
                    ],
                    None,
                );
                assert_eq!(tree_view.columns().len(), 5);
            }
            NotebookMainEnum::BadExtensions => {
                create_default_selection_button_column(tree_view, ColumnsBadExtensions::SelectionButton as i32, model, None);
                create_default_columns(
                    tree_view,
                    &[
                        (ColumnsBadExtensions::Name as i32, ColumnSort::Default),
                        (ColumnsBadExtensions::Path as i32, ColumnSort::Default),
                        (ColumnsBadExtensions::CurrentExtension as i32, ColumnSort::Default),
                        (ColumnsBadExtensions::ValidExtensions as i32, ColumnSort::Default),
                    ],
                    None,
                );
                assert_eq!(tree_view.columns().len(), 5);
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum ColumnSort {
    None,
    Default,
    Custom(i32),
}

pub(crate) fn create_default_selection_button_column(
    tree_view: &TreeView,
    column_id: i32,
    model: ListStore,
    activatable_color_columns: Option<(i32, i32)>,
) -> (CellRendererToggle, TreeViewColumn) {
    let renderer = CellRendererToggle::new();
    renderer.connect_toggled(move |_r, path| {
        let iter = model.iter(&path).expect("Failed to get iter from tree_path");
        let mut fixed = model.get::<bool>(&iter, column_id);
        fixed = !fixed;
        model.set_value(&iter, column_id as u32, &fixed.to_value());
    });
    let column = TreeViewColumn::new();
    column.pack_start(&renderer, true);
    column.set_resizable(false);
    column.set_fixed_width(30);
    column.add_attribute(&renderer, "active", column_id);
    if let Some(activatable_color_columns) = activatable_color_columns {
        column.add_attribute(&renderer, "activatable", activatable_color_columns.0);
        column.add_attribute(&renderer, "cell-background", activatable_color_columns.1);
    }
    tree_view.append_column(&column);
    (renderer, column)
}

pub(crate) fn create_default_columns(tree_view: &TreeView, columns: &[(i32, ColumnSort)], colors_columns_id: Option<(i32, i32)>) {
    for (col_id, sort_method) in columns {
        let renderer = CellRendererText::new();
        let column: TreeViewColumn = TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_resizable(true);
        column.set_min_width(50);
        column.add_attribute(&renderer, "text", *col_id);
        match sort_method {
            ColumnSort::None => {}
            ColumnSort::Default => column.set_sort_column_id(*col_id),
            ColumnSort::Custom(val) => column.set_sort_column_id(*val),
        }
        if let Some(colors_columns_id) = colors_columns_id {
            renderer.set_property("background-set", true);
            renderer.set_property("foreground-set", true);
            column.add_attribute(&renderer, "background", colors_columns_id.0);
            column.add_attribute(&renderer, "foreground", colors_columns_id.1);
        }
        tree_view.append_column(&column);
    }
}

pub(crate) fn show_preview(
    tree_view: &TreeView,
    text_view_errors: &TextView,
    check_button_settings_show_preview: &CheckButton,
    image_preview: &Picture,
    preview_path: &Rc<RefCell<String>>,
    column_path: i32,
    column_name: i32,
    use_rust_preview: bool,
) {
    let (selected_rows, tree_model) = tree_view.selection().selected_rows();

    let mut created_image = false;

    // Only show preview when selected is only one item, because there is no method to recognize current clicked item in multiselection
    if selected_rows.len() == 1 && check_button_settings_show_preview.is_active() {
        let tree_path = selected_rows[0].clone();
        // TODO labels on {} are in testing stage, so we just ignore for now this warning until found better idea how to fix this
        #[expect(clippy::never_loop)]
        'dir: loop {
            let path = tree_model.get::<String>(&tree_model.iter(&tree_path).expect("Invalid tree_path"), column_path);
            let name = tree_model.get::<String>(&tree_model.iter(&tree_path).expect("Invalid tree_path"), column_name);

            let file_name = get_full_name_from_path_name(&path, &name);

            if !check_if_can_display_image(&file_name) {
                break 'dir;
            }

            if file_name == preview_path.borrow().as_str() {
                return; // Preview is already created, no need to recreate it
            }

            let mut pixbuf = if use_rust_preview {
                let image = match get_dynamic_image_from_path(&file_name) {
                    Ok(t) => t,
                    Err(e) => {
                        add_text_to_text_view(text_view_errors, &flg!("preview_image_opening_failure", name = file_name, reason = e));
                        break 'dir;
                    }
                };

                match get_pixbuf_from_dynamic_image(image) {
                    Ok(t) => t,
                    Err(e) => {
                        add_text_to_text_view(text_view_errors, &flg!("preview_image_opening_failure", name = file_name, reason = e));
                        break 'dir;
                    }
                }
            } else {
                match Pixbuf::from_file(&file_name) {
                    Ok(pixbuf) => pixbuf,
                    Err(e) => {
                        add_text_to_text_view(text_view_errors, &flg!("preview_image_opening_failure", name = file_name, reason = e.to_string()));
                        break 'dir;
                    }
                }
            };
            pixbuf = match resize_pixbuf_dimension(&pixbuf, (800, 800), InterpType::Bilinear) {
                None => {
                    add_text_to_text_view(text_view_errors, &flg!("preview_image_resize_failure", name = file_name));
                    break 'dir;
                }
                Some(pixbuf) => pixbuf,
            };

            image_preview.set_paintable(Some(&Texture::for_pixbuf(&pixbuf)));
            {
                let mut preview_path = preview_path.borrow_mut();
                *preview_path = file_name;
            }

            created_image = true;

            break 'dir;
        }
    }
    if created_image {
        image_preview.set_visible(true);
    } else {
        image_preview.set_visible(false);
        {
            let mut preview_path = preview_path.borrow_mut();
            *preview_path = String::new();
        }
    }
}

/// Show preview for a file given its full path directly (not TreeView-dependent).
pub(crate) fn show_preview_for_file(
    file_name: &str,
    text_view_errors: &TextView,
    check_button_settings_show_preview: &CheckButton,
    image_preview: &Picture,
    preview_path: &Rc<RefCell<String>>,
    use_rust_preview: bool,
) {
    if !check_button_settings_show_preview.is_active() {
        image_preview.set_visible(false);
        *preview_path.borrow_mut() = String::new();
        return;
    }

    if file_name.is_empty() || !check_if_can_display_image(file_name) {
        image_preview.set_visible(false);
        *preview_path.borrow_mut() = String::new();
        return;
    }

    if file_name == preview_path.borrow().as_str() {
        return; // Preview is already created
    }

    let pixbuf = if use_rust_preview {
        match get_dynamic_image_from_path(file_name).and_then(get_pixbuf_from_dynamic_image) {
            Ok(p) => p,
            Err(e) => {
                add_text_to_text_view(text_view_errors, &flg!("preview_image_opening_failure", name = file_name.to_string(), reason = e));
                image_preview.set_visible(false);
                *preview_path.borrow_mut() = String::new();
                return;
            }
        }
    } else {
        match Pixbuf::from_file(file_name) {
            Ok(p) => p,
            Err(e) => {
                add_text_to_text_view(text_view_errors, &flg!("preview_image_opening_failure", name = file_name.to_string(), reason = e.to_string()));
                image_preview.set_visible(false);
                *preview_path.borrow_mut() = String::new();
                return;
            }
        }
    };

    match resize_pixbuf_dimension(&pixbuf, (800, 800), InterpType::Bilinear) {
        Some(resized) => {
            image_preview.set_paintable(Some(&Texture::for_pixbuf(&resized)));
            *preview_path.borrow_mut() = file_name.to_string();
            image_preview.set_visible(true);
        }
        None => {
            add_text_to_text_view(text_view_errors, &flg!("preview_image_resize_failure", name = file_name.to_string()));
            image_preview.set_visible(false);
            *preview_path.borrow_mut() = String::new();
        }
    }
}
#[derive(Clone)]
pub enum SharedModelEnum {
    Duplicates(SharedState<DuplicateFinder>),
    EmptyFolder(SharedState<EmptyFolder>),
    EmptyFiles(SharedState<EmptyFiles>),
    Temporary(SharedState<Temporary>),
    BigFile(SharedState<BigFile>),
    SimilarImages(SharedState<SimilarImages>),
    SimilarVideos(SharedState<SimilarVideos>),
    SameMusic(SharedState<SameMusic>),
    Symlinks(SharedState<InvalidSymlinks>),
    BrokenFiles(SharedState<BrokenFiles>),
    BadExtensions(SharedState<BadExtensions>),
}

impl SharedModelEnum {
    pub(crate) fn save_all_in_one(&self, path: &str) -> Result<(), String> {
        match self {
            Self::Duplicates(state) => state.borrow().as_ref().map(|x| x.save_all_in_one(path, "results_duplicates")),
            Self::EmptyFolder(state) => state.borrow().as_ref().map(|x| x.save_all_in_one(path, "results_empty_directories")),
            Self::EmptyFiles(state) => state.borrow().as_ref().map(|x| x.save_all_in_one(path, "results_empty_files")),
            Self::Temporary(state) => state.borrow().as_ref().map(|x| x.save_all_in_one(path, "results_temporary_files")),
            Self::BigFile(state) => state.borrow().as_ref().map(|x| x.save_all_in_one(path, "results_big_files")),
            Self::SimilarImages(state) => state.borrow().as_ref().map(|x| x.save_all_in_one(path, "results_similar_images")),
            Self::SimilarVideos(state) => state.borrow().as_ref().map(|x| x.save_all_in_one(path, "results_similar_videos")),
            Self::SameMusic(state) => state.borrow().as_ref().map(|x| x.save_all_in_one(path, "results_same_music")),
            Self::Symlinks(state) => state.borrow().as_ref().map(|x| x.save_all_in_one(path, "results_invalid_symlinks")),
            Self::BrokenFiles(state) => state.borrow().as_ref().map(|x| x.save_all_in_one(path, "results_broken_files")),
            Self::BadExtensions(state) => state.borrow().as_ref().map(|x| x.save_all_in_one(path, "results_bad_extensions")),
        }
        .transpose()
        .map_err(|e| e.to_string())?;

        Ok(())
    }
    pub(crate) fn replace(&self, new_item: Self) {
        match (self, new_item) {
            (Self::Duplicates(old), Self::Duplicates(new)) => {
                old.borrow_mut().replace(new.take().expect("TEST"));
            }
            (Self::EmptyFolder(old), Self::EmptyFolder(new)) => {
                old.borrow_mut().replace(new.take().expect("TEST"));
            }
            (Self::EmptyFiles(old), Self::EmptyFiles(new)) => {
                old.borrow_mut().replace(new.take().expect("TEST"));
            }
            (Self::Temporary(old), Self::Temporary(new)) => {
                old.borrow_mut().replace(new.take().expect("TEST"));
            }
            (Self::BigFile(old), Self::BigFile(new)) => {
                old.borrow_mut().replace(new.take().expect("TEST"));
            }
            (Self::SimilarImages(old), Self::SimilarImages(new)) => {
                old.borrow_mut().replace(new.take().expect("TEST"));
            }
            (Self::SimilarVideos(old), Self::SimilarVideos(new)) => {
                old.borrow_mut().replace(new.take().expect("TEST"));
            }
            (Self::SameMusic(old), Self::SameMusic(new)) => {
                old.borrow_mut().replace(new.take().expect("TEST"));
            }
            (Self::Symlinks(old), Self::Symlinks(new)) => {
                old.borrow_mut().replace(new.take().expect("TEST"));
            }
            (Self::BrokenFiles(old), Self::BrokenFiles(new)) => {
                old.borrow_mut().replace(new.take().expect("TEST"));
            }
            (Self::BadExtensions(old), Self::BadExtensions(new)) => {
                old.borrow_mut().replace(new.take().expect("TEST"));
            }
            _ => panic!("Mismatched SharedModelEnum variants"),
        }
    }
}
impl From<DuplicateFinder> for SharedModelEnum {
    fn from(value: DuplicateFinder) -> Self {
        Self::Duplicates(Rc::new(RefCell::new(Some(value))))
    }
}
impl From<EmptyFolder> for SharedModelEnum {
    fn from(value: EmptyFolder) -> Self {
        Self::EmptyFolder(Rc::new(RefCell::new(Some(value))))
    }
}
impl From<EmptyFiles> for SharedModelEnum {
    fn from(value: EmptyFiles) -> Self {
        Self::EmptyFiles(Rc::new(RefCell::new(Some(value))))
    }
}
impl From<Temporary> for SharedModelEnum {
    fn from(value: Temporary) -> Self {
        Self::Temporary(Rc::new(RefCell::new(Some(value))))
    }
}
impl From<BigFile> for SharedModelEnum {
    fn from(value: BigFile) -> Self {
        Self::BigFile(Rc::new(RefCell::new(Some(value))))
    }
}
impl From<SimilarImages> for SharedModelEnum {
    fn from(value: SimilarImages) -> Self {
        Self::SimilarImages(Rc::new(RefCell::new(Some(value))))
    }
}
impl From<SimilarVideos> for SharedModelEnum {
    fn from(value: SimilarVideos) -> Self {
        Self::SimilarVideos(Rc::new(RefCell::new(Some(value))))
    }
}
impl From<SameMusic> for SharedModelEnum {
    fn from(value: SameMusic) -> Self {
        Self::SameMusic(Rc::new(RefCell::new(Some(value))))
    }
}
impl From<InvalidSymlinks> for SharedModelEnum {
    fn from(value: InvalidSymlinks) -> Self {
        Self::Symlinks(Rc::new(RefCell::new(Some(value))))
    }
}
impl From<BrokenFiles> for SharedModelEnum {
    fn from(value: BrokenFiles) -> Self {
        Self::BrokenFiles(Rc::new(RefCell::new(Some(value))))
    }
}
impl From<BadExtensions> for SharedModelEnum {
    fn from(value: BadExtensions) -> Self {
        Self::BadExtensions(Rc::new(RefCell::new(Some(value))))
    }
}
