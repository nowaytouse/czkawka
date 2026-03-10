//! GTK4 async dialog helpers.
//! Uses Window instead of the deprecated Dialog, passing results via futures-channel oneshot.

use futures_channel::oneshot;
use gtk4::prelude::*;
use gtk4::{Align, CheckButton, Orientation};

use crate::flg;

// ── Confirmation window with checkbox ─────────────────────────────────────────

/// Show a modal confirmation window with a "don't ask again" checkbox.
/// Returns `(confirmed, checkbox_active)`.
pub async fn confirm_window_with_checkbox(
    parent: &gtk4::Window,
    title: &str,
    messages: &[&str],
    ok_label: &str,
    cancel_label: &str,
    checkbox_label: &str,
) -> (bool, bool) {
    let (tx, rx) = oneshot::channel::<(bool, bool)>();
    let tx = std::cell::Cell::new(Some(tx));

    let window = gtk4::Window::builder()
        .title(title)
        .transient_for(parent)
        .modal(true)
        .destroy_with_parent(true)
        .resizable(false)
        .build();

    let main_box = gtk4::Box::builder().orientation(Orientation::Vertical).spacing(10).margin_top(15).margin_bottom(15).margin_start(15).margin_end(15).build();

    for msg in messages {
        main_box.append(&gtk4::Label::new(Some(msg)));
    }

    let checkbox = CheckButton::builder().label(checkbox_label).active(true).halign(Align::Center).margin_top(5).build();
    main_box.append(&checkbox);

    let btn_box = gtk4::Box::builder().orientation(Orientation::Horizontal).halign(Align::Center).spacing(10).margin_top(5).build();
    let ok_btn = gtk4::Button::with_label(ok_label);
    let cancel_btn = gtk4::Button::with_label(cancel_label);
    btn_box.append(&cancel_btn);
    btn_box.append(&ok_btn);
    ok_btn.add_css_class("suggested-action");
    main_box.append(&btn_box);

    window.set_child(Some(&main_box));

    let checkbox_ok = checkbox.clone();
    let win_ok = window.clone();
    ok_btn.connect_clicked(move |_| {
        if let Some(t) = tx.take() {
            let _ = t.send((true, checkbox_ok.is_active()));
        }
        win_ok.close();
    });

    let win_cancel = window.clone();
    cancel_btn.connect_clicked(move |_| {
        win_cancel.close();
    });

    window.connect_close_request(|_| glib::Propagation::Proceed);

    window.present();
    rx.await.unwrap_or((false, true))
}

// ── Simple AlertDialog-style confirmation (no custom widget) ─────────────────

/// Show a text-only modal confirmation using gtk4::AlertDialog (GTK 4.10+).
/// Returns whether the first button (typically OK) was clicked.
pub async fn alert_confirm(parent: &gtk4::Window, title: &str, detail: &str) -> bool {
    let dialog = gtk4::AlertDialog::builder()
        .modal(true)
        .message(title)
        .detail(detail)
        .buttons([flg!("general_close_button").as_str(), flg!("general_ok_button").as_str()])
        .cancel_button(0)
        .default_button(1)
        .build();
    matches!(dialog.choose_future(Some(parent)).await, Ok(1))
}
