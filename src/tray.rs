use std::sync::mpsc::{self, Receiver, SyncSender};

use tray_item::{IconSource, TrayItem};

use crate::DownloadManager;

#[derive(PartialEq, Eq, Default, Debug)]
pub enum Message {
    #[default]
    None,
    Show,
    Quit,
    Hide,
    AddDl,
}
pub struct Tray {
    pub message: Message,
    pub tray: TrayItem,
    pub channel: (SyncSender<Message>, Receiver<Message>),
}

impl Default for Tray {
    fn default() -> Self {
        let channel = mpsc::sync_channel::<Message>(2);
        let mut tray = construct_tray();
        let add_dl_tx = channel.0.clone();
        tray.add_menu_item("Add Download", move || {
            add_dl_tx.send(Message::AddDl).unwrap();
        })
        .unwrap();

        let show_tx: SyncSender<Message> = channel.0.clone();
        tray.add_menu_item("Show", move || {
            show_tx.send(Message::Show).unwrap();
        })
        .unwrap();

        let hide_tx: SyncSender<Message> = channel.0.clone();
        tray.add_menu_item("Hide", move || {
            hide_tx.send(Message::Hide).unwrap();
        })
        .unwrap();

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

pub fn handle_tray_events(interface: &mut DownloadManager) {
    if let Ok(msg) = interface.tray_menu.channel.1.try_recv() {
        match msg {
            Message::AddDl => {
                interface.show_window = true;
                interface.popups.download.show = true;
            }
            Message::Show => {
                interface.tray_menu.message = Message::Show;
                interface.show_window = true
            }
            Message::Hide => {
                interface.show_window = false;
                interface.popups.download.show = false;
                interface.popups.confirm.show = false;
                interface.popups.error.show = false;
                interface.popups.plot.show = false;
                interface.popups.speed.show = false;
            }
            Message::Quit => std::process::exit(0),
            _ => {}
        }
    }
}

fn construct_tray() -> TrayItem {
    #[cfg(target_os = "linux")]
    {
        let (icon_rgba, icon_width, icon_height) = {
            let img_bytes = include_bytes!("../icon.png").to_vec();
            let image = image::load_from_memory(&img_bytes)
                .expect("Failed loading img from mem")
                .into_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width, height)
        };
        let icon = IconSource::Data {
            data: icon_rgba,
            height: icon_height as i32,
            width: icon_width as i32,
        };
        TrayItem::new("File Download Manager", icon).unwrap()
    }
    #[cfg(target_os = "windows")]
    {
        TrayItem::new("File Download Manager", IconSource::Resource("icon")).unwrap()
    }
}
