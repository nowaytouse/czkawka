use gtk4::prelude::*;
use gtk4::{Align, Orientation, Picture};

use crate::flg;
use crate::gui_structs::gui_data::CZK_ICON_KROKIET;
use crate::helpers::image_operations::svg_to_pixbuf;

pub fn show_krokiet_info_dialog(window_main: &gtk4::Window) {
    let window = gtk4::Window::builder()
        .title(flg!("krokiet_info_title"))
        .transient_for(window_main)
        .modal(true)
        .destroy_with_parent(true)
        .resizable(false)
        .default_width(500)
        .build();

    let main_box = gtk4::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .build();

    // Load Krokiet logo from SVG at 96x96 (from upstream #1919)
    if let Some(pixbuf) = svg_to_pixbuf(CZK_ICON_KROKIET, 96) {
        let picture = Picture::for_pixbuf(&pixbuf);
        picture.set_can_shrink(false);
        let wrapper = gtk4::Box::new(Orientation::Vertical, 0);
        wrapper.set_size_request(96, 96);
        wrapper.set_halign(Align::Center);
        wrapper.set_hexpand(false);
        wrapper.set_vexpand(false);
        wrapper.set_margin_top(15);
        wrapper.set_margin_bottom(5);
        wrapper.append(&picture);
        main_box.append(&wrapper);
    }

    let label = gtk4::Label::builder()
        .label(&flg!("krokiet_info_message"))
        .wrap(true)
        .justify(gtk4::Justification::Center)
        .halign(Align::Center)
        .build();

    let link_text = format!(
        "<a href=\"https://github.com/qarmin/czkawka/releases\">{}</a>  |  <a href=\"https://github.com/qarmin/czkawka/tree/master/krokiet\">{}</a>",
        flg!("krokiet_promo_link_download"),
        flg!("krokiet_promo_link_project")
    );
    let link = gtk4::Label::builder()
        .label(&link_text)
        .use_markup(true)
        .halign(Align::Center)
        .margin_top(5)
        .margin_bottom(5)
        .build();

    let ok_btn = gtk4::Button::builder().label(flg!("general_ok_button")).halign(Align::Center).margin_top(5).build();
    ok_btn.add_css_class("suggested-action");

    main_box.append(&label);
    main_box.append(&link);
    main_box.append(&ok_btn);

    window.set_child(Some(&main_box));

    let win = window.clone();
    ok_btn.connect_clicked(move |_| {
        win.close();
    });

    window.present();
}
