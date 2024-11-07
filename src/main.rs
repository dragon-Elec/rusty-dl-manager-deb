#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use colors::{DARKER_PURPLE, PURPLE};
use dl::file2dl::File2Dl;
use download_mechanism::{check_urls, run_downloads, set_total_bandwidth, Actions};
use egui_aesthetix::{themes::TokyoNight, Aesthetix};
use egui_sfml::{
    egui::{Color32, Context, FontData, FontDefinitions, Id},
    sfml::{
        graphics::{FloatRect, RenderTarget, RenderWindow, View},
        window::{ContextSettings, Event, Style},
    },
    SfEgui,
};
use extern_windows::Bandwidth;
use menu_bar::init_menu_bar;
use popups::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use server::interception::init_server;
use side_bar::{lay_side_bar_content, Explorer};
use status_bar::{check_connection, init_status_bar, update_connected, Connection};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    sync::mpsc::channel,
    time::{Duration, Instant},
};
use table::lay_table;
use tokio::runtime::{self, Runtime};
use tray::{handle_tray_events, Message, Tray};

mod colors;
mod dl;
mod download_mechanism;
mod extern_windows;
mod menu_bar;
mod popups;
mod server;
mod side_bar;
mod status_bar;
mod table;
mod tray;

#[derive(Serialize, Deserialize, Debug)]
struct Settings {
    retry_interval: u64,
    dl_dir: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            retry_interval: 5,
            dl_dir: String::from("Downloads"),
        }
    }
}

impl Settings {
    fn parse() -> Result<Self, std::io::Error> {
        let path = Path::new("settings.json");
        let mut buffer = String::new();
        let mut file = File::open(path)?;
        file.read_to_string(&mut buffer)?;
        let settings: Settings = serde_json::from_str(&buffer)?;
        Ok(settings)
    }
}
#[derive(Default)]
struct Select {
    select_all: bool,
    initial_state: bool,
}

struct DownloadManager {
    runtime: Runtime,
    files: Vec<FDl>,
    popups: PopUps,
    explorer: Explorer,
    search: String,
    select: Select,
    connection: Connection,
    settings: Settings,
    bandwidth: Bandwidth,
    tray_menu: Tray,
    show_window: bool,
}

impl DownloadManager {
    fn update(&mut self, ctx: &egui_sfml::egui::Context) {
        if !self.show_window {
            std::thread::sleep(Duration::from_millis(300));
        }
        handle_popups(self, ctx);
        set_total_bandwidth(self);
        update_connected(self);
        egui_sfml::egui::TopBottomPanel::top(Id::new("Top"))
            .default_height(40.0)
            .resizable(false)
            .frame(egui_sfml::egui::Frame::none().fill(*DARKER_PURPLE))
            .show_separator_line(true)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(7.0);
                });
                init_menu_bar(self, ui);
            });

        egui_sfml::egui::TopBottomPanel::bottom(Id::new("Bottom"))
            .default_height(40.0)
            .show_separator_line(true)
            .resizable(false)
            .frame(egui_sfml::egui::Frame::none().fill(*DARKER_PURPLE))
            .show(ctx, |ui| {
                init_status_bar(self, ui);
            });
        egui_sfml::egui::SidePanel::left(Id::new("left"))
            .frame(egui_sfml::egui::Frame::none().fill(*DARKER_PURPLE))
            .default_width(150.0)
            .resizable(false)
            .show_separator_line(true)
            .show(ctx, |ui| {
                lay_side_bar_content(self, ui);
            });
        egui_sfml::egui::CentralPanel::default()
            .frame(
                egui_sfml::egui::Frame::none()
                    .fill(*PURPLE)
                    .inner_margin(TokyoNight.margin_style())
                    .stroke(egui_sfml::egui::Stroke::new(
                        1.0,
                        Color32::from_rgba_premultiplied(31, 31, 51, 255),
                    )),
            )
            .show(ctx, |ui| {
                lay_table(self, ui, ctx);
            });
    }

    fn default() -> Self {
        let runtime = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to build runtime");
        let settings_path = Path::new("settings.json");
        let settings = if settings_path.exists() {
            Settings::parse().expect("Couldn't parse settings")
        } else {
            let mut file = File::create(settings_path).expect("Couldn't create file");
            let settings = Settings::default();
            file.write_all(json!(&settings).to_string().as_bytes())
                .expect("Couldn't write to file");
            settings
        };
        let files = Self::load_files(&settings).unwrap_or_default();

        let popups = PopUps {
            error: Self::create_error_popup(&settings.dl_dir),
            download: DownloadPopUp::default(),
            settings: SettingsPopUp {
                show: false,
                temp_str: settings.retry_interval.to_string(),
                dl_dir: settings.dl_dir.clone(),
                error: String::default(),
            },
            confirm: ConfirmPopUp::default(),
            plot: PLotPopUp::default(),
            speed: EditSpeedPopUp::default(),
            log: LogPopUp::default(),
        };
        let explorer = Explorer::default();
        Self {
            runtime,
            files,
            explorer,
            select: Select::default(),
            settings,
            popups,
            search: String::default(),
            connection: Connection::default(),
            bandwidth: Bandwidth::default(),
            tray_menu: Tray::default(),
            show_window: true,
        }
    }
    fn create_error_popup(dl_dir: &str) -> ErrorPopUp {
        match File2Dl::from(dl_dir) {
            Ok(_) => ErrorPopUp::default(),
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => ErrorPopUp {
                value: e.to_string(),
                show: true,
                channel: channel(),
            },
            _ => ErrorPopUp::default(),
        }
    }

    fn load_files(settings: &Settings) -> Result<Vec<FDl>, std::io::Error> {
        let files = File2Dl::from(&settings.dl_dir)?;
        Ok(files
            .into_iter()
            .map(|file| {
                let is_complete = file.complete.load(std::sync::atomic::Ordering::Relaxed);
                FDl {
                    file,
                    new: false,
                    has_error: false,
                    initial_status: is_complete,
                    got_notif: false,
                    toggled_at: Instant::now(),
                    initiated: false,
                    selected: false,
                    action_on_save: Actions::default(),
                }
            })
            .collect())
    }
}

#[derive(Debug, Clone)]
struct FDl {
    file: File2Dl,
    has_error: bool,
    got_notif: bool,
    initial_status: bool,
    new: bool,
    toggled_at: Instant,
    initiated: bool,
    selected: bool,
    action_on_save: Actions,
}

impl Default for FDl {
    fn default() -> Self {
        Self {
            file: File2Dl::default(),
            has_error: false,
            got_notif: false,
            initial_status: false,
            new: true,
            toggled_at: Instant::now(),
            initiated: false,
            selected: false,
            action_on_save: Actions::None,
        }
    }
}

fn main() {
    let init_size = (860, 480);
    let title = "Rusty Dl Manager";
    let win_settings = &ContextSettings {
        depth_bits: 0,
        stencil_bits: 0,
        antialiasing_level: 0,
        ..Default::default()
    };

    let mut rw = RenderWindow::new(init_size, title, Style::DEFAULT, win_settings).unwrap();
    rw.set_framerate_limit(60);

    let mut sf_egui = SfEgui::new(&rw);
    setup_custom_fonts(sf_egui.context());

    let mut state = DownloadManager::default();
    check_connection(&mut state);

    state.runtime.spawn_blocking(move || {
        init_server().unwrap_or_default();
    });
    while rw.is_open() {
        run_downloads(&mut state);
        handle_tray_events(&mut state);
        check_urls(&mut state);
        while let Some(ev) = rw.poll_event() {
            sf_egui.add_event(&ev);
            if matches!(ev, Event::Closed) {
                state.popups.download.show = false;
                state.popups.confirm.show = false;
                state.popups.error.show = false;
                state.popups.plot.show = false;
                state.popups.speed.show = false;
                state.tray_menu.message = Message::None;
                state.show_window = false;
            }
            if let Event::Resized { width, height } = ev {
                rw.set_view(
                    &View::from_rect(FloatRect::new(0f32, 0f32, width as f32, height as f32))
                        .unwrap(),
                );
            }
        }

        if state.show_window {
            rw.set_visible(true)
        } else {
            rw.set_visible(false);
        }

        let di = sf_egui
            .run(&mut rw, |_rw, ctx| {
                state.update(ctx);
            })
            .unwrap();

        sf_egui.draw(di, &mut rw, None);
        rw.display();
    }
}

fn setup_custom_fonts(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    fonts.font_data.insert(
        "my_font".to_owned(),
        FontData::from_static(include_bytes!("../JetBrainsMono-Regular.ttf")),
    );
    fonts
        .families
        .entry(egui_sfml::egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    fonts
        .families
        .entry(egui_sfml::egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "my_font".to_owned());
    ctx.set_fonts(fonts);
}
