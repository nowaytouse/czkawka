use gtk4::prelude::*;
use gtk4::{Align, Orientation};

use crate::flg;

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

    let label = gtk4::Label::builder()
        .label(&flg!("krokiet_info_message"))
        .wrap(true)
        .justify(gtk4::Justification::Center)
        .halign(Align::Center)
        .build();

    let link = gtk4::Label::builder()
        .label("<a href=\"https://github.com/qarmin/czkawka/tree/master/krokiet\">https://github.com/qarmin/czkawka/tree/master/krokiet</a> / <a href=\"https://github.com/qarmin/czkawka/releases\">https://github.com/qarmin/czkawka/releases</a>")
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
