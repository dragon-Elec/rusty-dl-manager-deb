use std::sync::mpsc;

use tray_item::{IconSource, TrayItem};

#[derive(PartialEq, Eq)]
enum Message {
    Quit,
}
pub fn init_tray_icon() {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open("o76s1-jr5se-001.ico")
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon_red = IconSource::Data {
        data: icon_rgba,
        height: icon_height as i32,
        width: icon_width as i32,
    };
    let mut tray = TrayItem::new("File Download manager", icon_red).unwrap();
    let (tx, rx) = mpsc::sync_channel::<Message>(2);
    let quit_tx = tx.clone();
    tray.add_menu_item("Exit", move || {
        quit_tx.send(Message::Quit).unwrap();
    })
    .unwrap();
    std::thread::spawn(move || loop {
        if let Ok(msg) = rx.recv() {
            if msg == Message::Quit {
                std::process::exit(0)
            }
        }
    });
}
