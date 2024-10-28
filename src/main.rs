use std::sync::mpsc::{channel, Receiver, Sender};

use dl::file2dl::File2Dl;
use eframe::egui::{self, Color32, Separator};
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
    select_all: bool,
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
                    select_all: false,
                };
            }
        };
        let files = files
            .iter()
            .enumerate()
            .map(|(idx, f)| {
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
            select_all: false,
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
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(TokyoNight.bg_secondary_color_visuals())
                    .inner_margin(TokyoNight.margin_style())
                    .stroke(egui::Stroke::new(
                        1.0,
                        TokyoNight.bg_secondary_color_visuals(),
                    )),
            )
            .show(ctx, |ui| {
                init_menu_bar(self, ui);
                ui.add(Separator::grow(Separator::default(), ui.available_width()));
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
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(MyApp::default()))
        }),
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
