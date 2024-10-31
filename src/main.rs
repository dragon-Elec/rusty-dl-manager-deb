use colors::{DARKER_PURPLE, PURPLE};
use dl::file2dl::File2Dl;
use download_mechanism::{check_urls, run_downloads, set_total_bandwidth, Actions};
use eframe::{
    egui::{self, Id},
    EventLoopBuilder,
};
use egui_aesthetix::{themes::TokyoNight, Aesthetix};
use extern_windows::Bandwidth;
use menu_bar::init_menu_bar;
use popups::*;
use server::interception::init_server;
use status_bar::{check_connection, init_status_bar, Connection};
use std::{
    fs::remove_file,
    path::Path,
    sync::mpsc::{channel, Receiver, Sender},
};
use table::lay_table;
use tokio::runtime::{self, Runtime};
use tray::{handle_tray_events, Tray};

mod colors;
mod dl;
mod download_mechanism;
mod extern_windows;
mod menu_bar;
mod popups;
mod server;
mod status_bar;
mod table;
mod tray;

struct MyApp {
    runtime: Runtime,
    files: Vec<FDl>,
    urls: (Sender<String>, Receiver<String>),
    popups: PopUps,
    temp_action: Actions,
    search: String,
    connection: Connection,
    bandwidth: Bandwidth,
    tray_menu: Tray,
}
fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../JetBrainsMono-Regular.ttf")),
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "my_font".to_owned());

    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    ctx.set_fonts(fonts);
}

impl MyApp {
    fn default(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        let runtime = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to build runtime");

        let popups = PopUps {
            error: Self::create_error_popup(),
            download: DownloadPopUp::default(),
            confirm: ConfirmPopUp::default(),
            plot: PLotPopUp::default(),
            speed: EditSpeedPopUp::default(),
        };

        let files = Self::load_files().unwrap_or_default();

        Self {
            runtime,
            files,
            urls: channel(),
            popups,
            temp_action: Actions::default(),
            search: String::default(),
            connection: Connection::default(),
            bandwidth: Bandwidth::default(),
            tray_menu: Tray::default(),
        }
    }
    fn create_error_popup() -> ErrorPopUp {
        match File2Dl::from("Downloads") {
            Ok(_) => ErrorPopUp::default(),
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => ErrorPopUp {
                value: e.to_string(),
                show: true,
                channel: channel(),
            },
            _ => ErrorPopUp::default(),
        }
    }

    fn load_files() -> Result<Vec<FDl>, std::io::Error> {
        let files = File2Dl::from("Downloads")?;
        Ok(files
            .into_iter()
            .map(|file| FDl {
                file,
                new: false,
                initiated: false,
                selected: false,
                action_on_save: Actions::default(),
            })
            .collect())
    }
}

#[derive(Debug, Default, Clone)]
struct FDl {
    file: File2Dl,
    new: bool,
    initiated: bool,
    selected: bool,
    action_on_save: Actions,
}

fn main() -> eframe::Result {
    let path = Path::new("urls.txt");
    if path.exists() {
        remove_file(path).expect("Couldn't remove urls file");
    }
    std::thread::spawn(move || {
        init_server().unwrap_or_default();
    });
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([190.0, 190.0]),
        centered: true,
        vsync: true,
        ..Default::default()
    };
    eframe::run_native("Download Manager", options, {
        Box::new(|cc| Ok(Box::new(MyApp::default(cc))))
    })
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if ctx.input(|i| i.viewport().close_requested()) {
            self.tray_menu
                .channel
                .0
                .send(tray::Message::Quit)
                .expect("Couldn't exit tray")
        }
        set_total_bandwidth(self);
        check_connection(self);
        run_downloads(self);
        check_urls(self);
        handle_popups(self, ctx);
        handle_tray_events(self);
        egui::TopBottomPanel::top(Id::new("Top"))
            .exact_height(40.0)
            .frame(egui::Frame::none().fill(*DARKER_PURPLE))
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(7.0);
                });
                init_menu_bar(self, ui);
            });
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(*PURPLE)
                    .inner_margin(TokyoNight.margin_style())
                    .stroke(egui::Stroke::new(
                        1.0,
                        TokyoNight.bg_secondary_color_visuals(),
                    )),
            )
            .show(ctx, |ui| {
                lay_table(self, ui, ctx);
            });
        egui::TopBottomPanel::bottom(Id::new("Bottom"))
            .exact_height(40.0)
            .show_separator_line(false)
            .frame(egui::Frame::none().fill(*DARKER_PURPLE))
            .show(ctx, |ui| {
                init_status_bar(self, ui);
            });
    }
}
