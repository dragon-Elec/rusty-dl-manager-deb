use std::sync::mpsc::{channel, Receiver, Sender};

use dl::file2dl::File2Dl;
use eframe::egui::{self, Color32, Id};
use egui_aesthetix::{themes::TokyoNight, Aesthetix};
use extern_windows::{show_confirm_window, show_error_window, show_input_window};
use status_bar::init_menu_bar;
use table::lay_table;
use tokio::runtime::Runtime;
mod dl;
mod extern_windows;
mod status_bar;
mod table;

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
    show: bool,
    error: String,
    error_channel: (Sender<String>, Receiver<String>),
    file_channel: (Sender<String>, Receiver<String>),
}
impl Default for DownloadPopUp {
    fn default() -> Self {
        Self {
            link: String::default(),
            show: bool::default(),
            error: String::default(),
            error_channel: channel(),
            file_channel: channel(),
        }
    }
}
#[derive(Default)]
struct PopUps {
    download: DownloadPopUp,
    error: ErrorPopUp,
    confirm: ConfirmPopUp,
}

struct MyApp {
    files: Vec<FDl>,
    file_channel: (Sender<File2Dl>, Receiver<File2Dl>),
    popups: PopUps,
    temp_action: Actions,
    search: String,
}
impl Default for MyApp {
    fn default() -> Self {
        let files = match File2Dl::from("Downloads") {
            Ok(f) => f,
            Err(e) => {
                let error = ErrorPopUp {
                    value: e.to_string(),
                    show: true,
                    channel: channel(),
                };
                let download = DownloadPopUp::default();
                let confirm = ConfirmPopUp::default();
                return Self {
                    files: Vec::default(),
                    file_channel: channel(),
                    popups: PopUps {
                        error,
                        download,
                        confirm,
                    },
                    temp_action: Actions::default(),
                    search: String::default(),
                };
            }
        };
        let files = files
            .iter()
            .map(|f| {
                let file = f.to_owned();
                FDl {
                    file,
                    initiated: false,
                    selected: false,
                    action_on_save: Actions::default(),
                }
            })
            .collect::<Vec<_>>();
        Self {
            files,
            file_channel: channel(),
            popups: PopUps::default(),
            temp_action: Actions::default(),
            search: String::default(),
        }
    }
}
#[derive(Debug, Default, Clone)]
struct FDl {
    file: File2Dl,
    initiated: bool,
    selected: bool,
    action_on_save: Actions,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub enum Actions {
    #[default]
    None,
    Reboot,
    Shutdown,
}
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::new(egui::panel::TopBottomSide::Top, Id::new("My panel"))
            .exact_height(30.0)
            .frame(egui::Frame::none().fill(Color32::from_hex("#111017").expect("Bad Hex")))
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(2.0);
                });
                init_menu_bar(self, ui);
            });
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_hex("#1b1824").expect("Bad Hex"))
                    .inner_margin(TokyoNight.margin_style())
                    .stroke(egui::Stroke::new(
                        1.0,
                        TokyoNight.bg_secondary_color_visuals(),
                    )),
            )
            .show(ctx, |ui| {
                run_downloads(self);
                lay_table(self, ui, ctx);
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
            });
    }
}

fn main() -> eframe::Result {
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
        if !complete && !fdl.initiated {
            let rt = Runtime::new().unwrap();
            let file = file.clone();
            let tx_error = interface.popups.error.channel.0.clone();
            std::thread::spawn(move || {
                rt.block_on(async move {
                    loop {
                        match file.single_thread_dl().await {
                            Ok(_) => break,
                            Err(e) => {
                                tx_error.send(e.to_string()).unwrap();
                            }
                        }
                    }
                })
            });
            let rx = &interface.popups.error.channel.1;
            if let Ok(val) = rx.try_recv() {
                interface.popups.error.value = val;
                interface.popups.error.show = true;
            }
            fdl.initiated = true;
        }
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    // Start with the default font definitions, including `egui_phosphor`.
    let mut fonts = egui::FontDefinitions::default();

    // Add `JetBrainsMono` as the main font.
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../JetBrainsMono-Regular.ttf")),
    );

    // Insert `JetBrainsMono` with the highest priority for proportional text.
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Also use `JetBrainsMono` as the monospace font.
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "my_font".to_owned());

    // Keep `egui_phosphor` icons available by re-adding them to fonts.
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

    // Apply the modified font definitions.
    ctx.set_fonts(fonts);
}

impl MyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        MyApp::default()
    }
}
