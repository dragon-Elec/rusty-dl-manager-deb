use colors::{CYAN, DARKER_PURPLE, DARK_INNER, GREEN, PURPLE, RED};
use dl::{errors, file2dl::File2Dl};
use eframe::egui::{self, Button, Color32, Id, Label, Layout};
use egui_aesthetix::{themes::TokyoNight, Aesthetix};
use extern_windows::{
    show_confirm_window, show_error_window, show_input_window, show_modify_speed_window,
    show_plot_window,
};
use menu_bar::init_menu_bar;
use status_bar::init_status_bar;
use std::{
    net::TcpStream,
    sync::mpsc::{channel, Receiver, Sender},
    thread::sleep,
    time::Duration,
};
use table::lay_table;
use tokio::runtime::Runtime;

mod colors;
mod dl;
mod extern_windows;
mod menu_bar;
mod status_bar;
mod table;

#[derive(Debug, Default)]
struct PLotPopUp {
    show: bool,
}

struct ConfirmPopUp {
    text: String,
    color: Color32,
    show: bool,
    task: Box<dyn Fn() -> Box<dyn FnOnce(&mut MyApp)>>,
}
impl Default for ConfirmPopUp {
    fn default() -> Self {
        Self {
            text: String::new(),
            color: Color32::default(),
            show: false,
            task: Box::new(|| Box::new(|_app: &mut MyApp| {})),
        }
    }
}

#[derive(Default)]
struct EditSpeedPopUp {
    show: bool,
    error: String,
    temp_val: String,
}

#[derive(Debug)]
struct ErrorPopUp {
    value: String,
    show: bool,
    channel: (Sender<String>, Receiver<String>),
}
impl Default for ErrorPopUp {
    fn default() -> Self {
        Self {
            value: String::default(),
            show: bool::default(),
            channel: channel(),
        }
    }
}
#[derive(Debug)]
struct DownloadPopUp {
    link: String,
    speed: String,
    temp_file: Option<File2Dl>,
    file_channel: (Sender<File2Dl>, Receiver<File2Dl>),
    show: bool,
    error: String,
    error_channel: (Sender<String>, Receiver<String>),
}
impl Default for DownloadPopUp {
    fn default() -> Self {
        Self {
            link: String::default(),
            speed: String::default(),
            temp_file: None,
            file_channel: channel(),
            show: bool::default(),
            error: String::default(),
            error_channel: channel(),
        }
    }
}
#[derive(Default)]
struct PopUps {
    download: DownloadPopUp,
    error: ErrorPopUp,
    confirm: ConfirmPopUp,
    plot: PLotPopUp,
    speed: EditSpeedPopUp,
}
#[derive(Default)]
pub struct Bandwidth {
    total_bandwidth: usize,
    history: Vec<usize>,
}
struct MyApp {
    files: Vec<FDl>,
    popups: PopUps,
    temp_action: Actions,
    search: String,
    connection: Connection,
    bandwidth: Bandwidth,
}
impl Default for MyApp {
    fn default() -> Self {
        let files = match File2Dl::from("Downloads") {
            Ok(f) => f,
            Err(e) => {
                let error = {
                    if e.kind() != std::io::ErrorKind::NotFound {
                        ErrorPopUp {
                            value: e.to_string(),
                            show: true,
                            channel: channel(),
                        }
                    } else {
                        ErrorPopUp::default()
                    }
                };
                let download = DownloadPopUp::default();
                let confirm = ConfirmPopUp::default();
                return Self {
                    files: Vec::default(),
                    popups: PopUps {
                        error,
                        download,
                        confirm,
                        plot: PLotPopUp::default(),
                        speed: EditSpeedPopUp::default(),
                    },
                    temp_action: Actions::default(),
                    search: String::default(),
                    connection: Connection::default(),
                    bandwidth: Bandwidth::default(),
                };
            }
        };
        let files = files
            .iter()
            .map(|f| {
                let file = f.to_owned();
                FDl {
                    file,
                    new: false,
                    initiated: false,
                    selected: false,
                    action_on_save: Actions::default(),
                }
            })
            .collect::<Vec<_>>();
        Self {
            files,
            popups: PopUps::default(),
            temp_action: Actions::default(),
            search: String::default(),
            connection: Connection::default(),
            bandwidth: Bandwidth::default(),
        }
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

#[derive(Debug)]
struct Connection {
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
#[derive(Debug, Default, PartialEq, Clone)]
pub enum Actions {
    #[default]
    None,
    Reboot,
    Shutdown,
    Open,
}
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        set_total_bandwidth(self);
        check_connection(self);
        run_downloads(self);
        if self.popups.speed.show {
            show_modify_speed_window(ctx, self);
        }
        if self.popups.plot.show {
            show_plot_window(ctx, self);
        }
        if self.popups.confirm.show {
            let task = (self.popups.confirm.task)();
            show_confirm_window(
                ctx,
                self,
                self.popups.confirm.color,
                &self.popups.confirm.text.clone(),
                task,
            );
        }
        if self.popups.error.show {
            show_error_window(ctx, self, &self.popups.error.value.clone());
        };
        if self.popups.download.show {
            show_input_window(ctx, self);
        }
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
            .frame(egui::Frame::none().fill(*DARKER_PURPLE))
            .show(ctx, |ui| {
                init_status_bar(self, ui);
            });
    }
}

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([190.0, 190.0]),
        centered: true,
        vsync: true,
        ..Default::default()
    };
    eframe::run_native(
        "Download Manager",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

fn run_downloads(interface: &mut MyApp) {
    for fdl in interface.files.iter_mut() {
        let file = &fdl.file;
        let complete = file.complete.load(std::sync::atomic::Ordering::Relaxed);
        let new = fdl.new;
        if !complete && !&fdl.initiated {
            let rt = Runtime::new().unwrap();
            let file = file.clone();
            let tx_error = interface.popups.error.channel.0.clone();
            std::thread::spawn(move || {
                rt.block_on(async move {
                    if file.url.range_support {
                        loop {
                            match file.single_thread_dl().await {
                                Ok(_) => break,
                                Err(e) => {
                                    let error = format!("{}: {:?}", file.name_on_disk, e);
                                    tx_error.send(error).unwrap();
                                }
                            }
                            sleep(Duration::from_secs(5));
                        }
                    } else if new {
                        match file.single_thread_dl().await {
                            Ok(_) => {}
                            Err(e) => {
                                let error = format!("{}: {:?}", file.name_on_disk, e);
                                tx_error.send(error).unwrap();
                            }
                        }
                    }
                })
            });

            fdl.initiated = true;
        }
        if let Ok(err) = interface.popups.error.channel.1.try_recv() {
            interface.popups.error.value = err;
            interface.popups.error.show = true;
        }
    }
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
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        MyApp::default()
    }
}

fn check_connection(interface: &mut MyApp) {
    if !interface.connection.initiated {
        let tx = interface.connection.channel.0.clone();
        std::thread::spawn(move || loop {
            let is_connected = tcp_ping();
            if let Err(e) = tx.send(is_connected) {
                println!("Failed to send connection status: {}", e);
            }
            sleep(Duration::from_secs(5));
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

fn set_total_bandwidth(interface: &mut MyApp) {
    let size: usize = interface
        .files
        .iter()
        .map(|f| {
            f.file
                .bytes_per_sec
                .load(std::sync::atomic::Ordering::Relaxed)
        })
        .sum();
    interface.bandwidth.total_bandwidth = size;
    update_bandwidth_history(interface);
}
fn update_bandwidth_history(interface: &mut MyApp) {
    interface
        .bandwidth
        .history
        .push(interface.bandwidth.total_bandwidth);
    if interface.bandwidth.history.len() > 100 {
        interface.bandwidth.history.remove(0);
    }
}
