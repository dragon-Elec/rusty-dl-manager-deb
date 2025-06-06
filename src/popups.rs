use crate::{
    dl::file2dl::File2Dl,
    download_mechanism::Actions,
    extern_windows::{
        show_confirm_window, show_error_window, show_input_window, show_log_window,
        show_modify_speed_window, show_plot_window, show_settings_window,
    },
    DownloadManager,
};
use egui_sfml::egui::Color32;
use std::sync::mpsc::{channel, Receiver, Sender};

type TaskInner = Box<dyn FnOnce(&mut DownloadManager)>;
type Task = Box<dyn Fn() -> TaskInner>;

#[derive(Default)]
pub struct SettingsPopUp {
    pub show: bool,
    pub temp_str: String,
    pub dl_dir: String,
    pub error: String,
}

#[derive(Default)]
pub struct LogPopUp {
    pub has_error: bool,
    pub logs: Vec<(String, String, Color32)>,
    pub show: bool,
}
pub struct ConfirmPopUp {
    pub text: String,
    pub color: Color32,
    pub show: bool,
    pub task: Task,
}
impl Default for ConfirmPopUp {
    fn default() -> Self {
        Self {
            text: String::new(),
            color: Color32::default(),
            show: false,
            task: Box::new(|| Box::new(|_app: &mut DownloadManager| {})),
        }
    }
}

#[derive(Debug, Default)]
pub struct PLotPopUp {
    pub show: bool,
}

#[derive(Default)]
pub struct EditSpeedPopUp {
    pub show: bool,
    pub error: String,
    pub temp_val: String,
}

#[derive(Debug)]
pub struct ErrorPopUp {
    pub value: String,
    pub show: bool,
    pub channel: (Sender<String>, Receiver<String>),
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
pub struct DownloadPopUp {
    pub link: String,
    pub speed: String,
    pub temp_action: Actions,
    pub temp_file: Option<File2Dl>,
    pub file_channel: (Sender<File2Dl>, Receiver<File2Dl>),
    pub show: bool,
    pub error: String,
    pub error_channel: (Sender<String>, Receiver<String>),
}
impl Default for DownloadPopUp {
    fn default() -> Self {
        Self {
            link: String::default(),
            speed: String::default(),
            temp_action: Actions::None,
            temp_file: None,
            file_channel: channel(),
            show: bool::default(),
            error: String::default(),
            error_channel: channel(),
        }
    }
}
#[derive(Default)]
pub struct PopUps {
    pub download: DownloadPopUp,
    pub settings: SettingsPopUp,
    pub error: ErrorPopUp,
    pub confirm: ConfirmPopUp,
    pub plot: PLotPopUp,
    pub speed: EditSpeedPopUp,
    pub log: LogPopUp,
}
pub fn handle_popups(interface: &mut DownloadManager, ctx: &egui_sfml::egui::Context) {
    if interface.popups.log.show {
        show_log_window(ctx, interface);
    }
    if interface.popups.speed.show {
        show_modify_speed_window(ctx, interface);
    }
    if interface.popups.plot.show {
        show_plot_window(ctx, interface);
    }
    if interface.popups.confirm.show {
        let task = (interface.popups.confirm.task)();
        show_confirm_window(
            ctx,
            interface,
            interface.popups.confirm.color,
            &interface.popups.confirm.text.clone(),
            task,
        );
    }
    if interface.popups.error.show {
        show_error_window(ctx, interface, &interface.popups.error.value.clone());
    };
    if interface.popups.download.show {
        show_input_window(ctx, interface);
    }
    if interface.popups.settings.show {
        show_settings_window(ctx, interface);
    }
}
