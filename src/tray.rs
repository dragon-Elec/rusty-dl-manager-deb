use std::{
    borrow::Borrow,
    sync::mpsc::{self, Receiver, SyncSender},
};

use tray_item::{IconSource, TrayItem};

use crate::MyApp;

#[derive(PartialEq, Eq, Default, Debug)]
pub enum Message {
    #[default]
    None,
    Quit,
}
pub struct Tray {
    pub message: Message,
    pub tray: TrayItem,
    pub channel: (SyncSender<Message>, Receiver<Message>),
}

impl Default for Tray {
    fn default() -> Self {
        let (icon_rgba, icon_width, icon_height) = {
            let img_bytes = include_bytes!("../icon.png").to_vec();
            let image = image::load_from_memory(&img_bytes)
                .expect("Failed loading img from mem")
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
        let channel = mpsc::sync_channel::<Message>(2);
        let quit_tx = channel.0.clone();
        tray.add_menu_item("Exit", move || {
            quit_tx.send(Message::Quit).unwrap();
        })
        .unwrap();
        Self {
            tray,
            message: Message::default(),
            channel,
        }
    }
}

pub fn handle_tray_events(interface: &mut MyApp) {
    if let Ok(msg) = interface.tray_menu.channel.1.try_recv() {
        match msg {
            Message::None => {}
            Message::Quit => std::process::exit(0),
        }
    }
}
