use crate::{colors::*, DownloadManager};
use egui_sfml::egui::{self, Button, CursorIcon, Label, Layout, RichText, Separator, Ui};
use std::{
    net::TcpStream,
    sync::mpsc::{channel, Receiver, Sender},
    thread::sleep,
    time::Duration,
};

#[derive(Debug)]
pub struct Connection {
    connected: bool,
    initiated: bool,
    channel: (Sender<bool>, Receiver<bool>),
}
impl Default for Connection {
    fn default() -> Self {
        Self {
            channel: channel(),
            connected: false,
            initiated: false,
        }
    }
}

pub fn init_status_bar(interface: &mut DownloadManager, ui: &mut Ui) {
    ui.with_layout(Layout::right_to_left(egui::Align::RIGHT), |ui| {
        ui.add_space(10.0);
        ui.horizontal_centered(|ui| {
            if interface.connection.connected {
                let text = RichText::new(egui_phosphor::fill::GLOBE)
                    .size(30.0)
                    .color(*GREEN);
                let label = Label::new(text).selectable(false);
                let res = ui.add(label);
                if res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::Default);
                    let text = RichText::new("Connected").color(*GREEN);
                    res.show_tooltip_text(text);
                }
            } else {
                let text = RichText::new(egui_phosphor::fill::GLOBE_X)
                    .size(30.0)
                    .color(*RED);
                let label = Label::new(text).selectable(false);
                let res = ui.add(label);
                if res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::Default);
                    let text = RichText::new("Disconnected").color(*RED);
                    res.show_tooltip_text(text);
                }
            }
        });
        ui.add(Separator::grow(Separator::default(), 35.0));
        ui.add_space(ui.available_width() - 120.0);
        ui.horizontal_centered(|ui| {
            {
                let text = egui_sfml::egui::RichText::new(egui_phosphor::fill::SCROLL)
                    .size(25.0)
                    .color(*DARK_INNER);
                let butt = if interface.popups.log.has_error {
                    Button::new(text).fill(*RED).rounding(25.0)
                } else {
                    Button::new(text).fill(*CYAN).rounding(25.0)
                };
                let res = ui.add(butt);
                if res.clicked() {
                    interface.popups.log.has_error = false;
                    interface.popups.log.show = true;
                }
                if res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                }
            }
            {
                let text = egui_sfml::egui::RichText::new(egui_phosphor::fill::NETWORK)
                    .size(25.0)
                    .color(*DARK_INNER);
                let butt = Button::new(text).fill(*CYAN).rounding(25.0);
                let res = ui.add(butt);
                if res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                    let text =
                        RichText::new("Modify download speed for selected files").color(*CYAN);
                    res.show_tooltip_text(text);
                }
                if res.clicked() {
                    interface.popups.speed.show = true;
                }
            }

            {
                let text = egui_sfml::egui::RichText::new(egui_phosphor::fill::CHART_LINE_UP)
                    .size(25.0)
                    .color(*DARK_INNER);
                let butt = Button::new(text).fill(*CYAN).rounding(25.0);
                let res = ui.add(butt);
                if res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                    let bw_mbs = interface.bandwidth.total_bandwidth as f64 / (1024.0 * 1024.0);
                    let bw_formatted = format!("{:.2} MBs", bw_mbs);
                    let text = RichText::new(bw_formatted.to_string()).color(*CYAN);
                    res.show_tooltip_text(text);
                }
                if res.clicked() {
                    interface.popups.plot.show = true;
                }
            }
        });
    });
}

pub fn check_connection(interface: &mut DownloadManager) {
    if !interface.connection.initiated {
        let tx = interface.connection.channel.0.clone();
        interface.runtime.spawn(async move {
            loop {
                let is_connected = tcp_ping();
                if let Err(e) = tx.send(is_connected) {
                    println!("Failed to send connection status: {}", e);
                }
                sleep(Duration::from_secs(5));
            }
        });
        interface.connection.initiated = true;
    }

    while let Ok(val) = interface.connection.channel.1.try_recv() {
        interface.connection.connected = val;
    }
}

fn tcp_ping() -> bool {
    let address = "8.8.8.8:53";
    let timeout = Duration::from_secs(3);
    TcpStream::connect_timeout(&address.parse().unwrap(), timeout).is_ok()
}
