use std::sync::mpsc::{channel, Receiver, Sender};

use dl::file2dl::File2Dl;
use eframe::egui::{self, Color32, Separator};
use extern_windows::{show_confirm_window, show_error_window, show_input_window};
use status_bar::init_menu_bar;
use table::lay_table;
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
                }
            })
            .collect::<Vec<_>>();
        Self {
            files,
            file_channel: channel(),
            popups: PopUps::default(),
        }
    }
}
#[derive(Debug, Default)]
struct FDl {
    file: File2Dl,
    initiated: bool,
    selected: bool,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        catppuccin_egui::set_theme(ctx, catppuccin_egui::MOCHA);
        egui::CentralPanel::default().show(ctx, |ui| {
            init_menu_bar(self, ui);
            ui.add(Separator::grow(Separator::default(), ui.available_width()));
            if self.popups.error.show {
                show_error_window(ctx, self, &self.popups.error.value.clone());
            };
            if self.popups.download.show {
                show_input_window(ctx, self);
            }
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
        });
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
            cc.egui_ctx.set_fonts(fonts);
            Ok(Box::new(MyApp::default()))
        }),
    )
}
