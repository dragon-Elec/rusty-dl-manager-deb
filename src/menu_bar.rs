use crate::{
    colors::{CYAN, GREEN, RED},
    DownloadManager,
};
use chrono::Local;
use egui_sfml::egui::{menu, Color32, RichText};
use std::{fs::remove_file, path::Path};

pub fn init_menu_bar(interface: &mut DownloadManager, ui: &mut egui_sfml::egui::Ui) {
    menu::bar(ui, |ui| {
        let text = RichText::new("Files").color(*CYAN).strong().size(15.0);
        ui.add_space(5.0);

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.menu_button(text, |ui| {
                    file_button_content(interface, ui);
                });
                let text = RichText::new("Downloads").color(*CYAN).strong().size(15.0);
                ui.menu_button(text, |ui| {
                    let text = RichText::new("Resume all").color(*CYAN).strong();
                    if ui.button(text).clicked() {
                        for core in interface.files.iter_mut() {
                            core.file
                                .running
                                .store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    let text = RichText::new("Pause all").color(*CYAN).strong();
                    if ui.button(text).clicked() {
                        for core in interface.files.iter_mut() {
                            core.file
                                .running
                                .store(false, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                });
            });
            ui.add_space(7.0);
        });
    });
}

fn file_button_content(interface: &mut DownloadManager, ui: &mut egui_sfml::egui::Ui) {
    let text = RichText::new("Remove selected from list")
        .color(*CYAN)
        .strong();
    if ui.button(text).clicked() {
        interface.popups.confirm.color = Color32::GREEN;
        interface.popups.confirm.task = Box::new(|| {
            Box::new(move |app: &mut DownloadManager| {
                app.files.retain(|core| !core.selected);
            })
        });
        interface.popups.confirm.show = true;
        interface.popups.confirm.text = String::from("This will remove files selected from list")
    }
    let text = RichText::new("Remove selected from disk")
        .color(*CYAN)
        .strong();
    if ui.button(text).clicked() {
        interface.popups.confirm.color = *RED;
        interface.popups.confirm.task = Box::new(|| {
            Box::new(move |app: &mut DownloadManager| {
                remove_selected_from_disk(app);
            })
        });
        interface.popups.confirm.show = true;
        interface.popups.confirm.text = String::from("This will remove selected files from disk")
    }
    let text = RichText::new("Remove all from list").color(*CYAN).strong();
    if ui.button(text).clicked() {
        interface.popups.confirm.color = *GREEN;
        interface.popups.confirm.task = Box::new(|| {
            Box::new(move |app: &mut DownloadManager| {
                app.files.clear();
            })
        });
        interface.popups.confirm.show = true;
        interface.popups.confirm.text = "This will not delete files from disk".to_string();
    }
    let text = RichText::new("Remove all from disk").color(*CYAN).strong();
    if ui.button(text).clicked() {
        interface.popups.confirm.color = *RED;
        interface.popups.confirm.task = Box::new(|| {
            Box::new(move |app: &mut DownloadManager| {
                delete_all_files_from_disk(app);
            })
        });
        interface.popups.confirm.show = true;
        interface.popups.confirm.text = String::from("This will remove all files from disk")
    }
    let text = RichText::new("Remove complete from list")
        .color(*CYAN)
        .strong();
    if ui.button(text).clicked() {
        interface.popups.confirm.color = *GREEN;
        interface.popups.confirm.task = Box::new(|| {
            Box::new(move |app: &mut DownloadManager| {
                delete_complete_from_list(app);
            })
        });
        interface.popups.confirm.show = true;
        interface.popups.confirm.text =
            String::from("This will remove all complete files from list")
    }
    let text = RichText::new("Remove complete from disk")
        .color(*CYAN)
        .strong();
    if ui.button(text).clicked() {
        interface.popups.confirm.color = *RED;
        interface.popups.confirm.task = Box::new(|| {
            Box::new(move |app: &mut DownloadManager| {
                delete_complete_from_disk(app);
            })
        });
        interface.popups.confirm.show = true;
        interface.popups.confirm.text =
            String::from("This will remove all complete files from disk")
    }
}
fn delete_all_files_from_disk(interface: &mut DownloadManager) {
    let mut is_ok = true;
    for fdl in interface.files.iter_mut() {
        let name_on_disk = &fdl.file.name_on_disk;
        let dir = &fdl.file.dl_dir;
        let path = format!("{dir}/{}", name_on_disk);
        let tmp_path = format!("{dir}/.{}.metadl", name_on_disk);
        let now = Local::now();
        let formatted_time = now.format("%H:%M:%S").to_string();
        if Path::new(&path).exists() {
            match remove_file(&path) {
                Ok(_) => {
                    let text = format!("Deleted file: {}\n", path);
                    interface
                        .popups
                        .log
                        .logs
                        .push((formatted_time.clone(), text, *GREEN));
                }
                Err(e) => {
                    let err = format!("File Path: {}, Error: {}\n", path, e);
                    interface
                        .popups
                        .log
                        .logs
                        .push((formatted_time.clone(), err.clone(), *RED));
                    interface.popups.error.value = err;
                    interface.popups.error.show = true;
                    is_ok = false;
                }
            }
        }
        if Path::new(&tmp_path).exists() {
            match remove_file(&tmp_path) {
                Ok(_) => {
                    let text = format!("Deleted tmp file: {}\n", tmp_path);
                    interface
                        .popups
                        .log
                        .logs
                        .push((formatted_time.clone(), text, *GREEN));
                }
                Err(e) => {
                    let err = format!("File Path: {}, Error: {}\n", tmp_path, e);
                    interface
                        .popups
                        .log
                        .logs
                        .push((formatted_time, err.clone(), *RED));
                    interface.popups.error.value = err;
                    interface.popups.error.show = true;
                    is_ok = false;
                }
            }
        }
    }
    if is_ok {
        interface.files.clear();
    }
}
fn remove_selected_from_disk(app: &mut DownloadManager) {
    app.files.retain(|core| {
        if core.selected {
            let now = Local::now();
            let formatted_time = now.format("%H:%M:%S").to_string();
            let path = format!("{}/{}", app.settings.dl_dir, core.file.name_on_disk);
            let tmp_path = format!("{}/.{}.metadl", app.settings.dl_dir, core.file.name_on_disk);
            match remove_file(&path) {
                Ok(_) => {
                    let text = format!("File: {} was removed\n", &path);
                    app.popups
                        .log
                        .logs
                        .push((formatted_time.clone(), text, *GREEN));
                }
                Err(e) => {
                    let err = format!("File Path: {}, Error: {}\n", &path, e);
                    app.popups
                        .log
                        .logs
                        .push((formatted_time.clone(), err.clone(), *RED));
                    app.popups.error.value = err;
                    app.popups.error.show = true;
                }
            }
            match remove_file(&tmp_path) {
                Ok(_) => {
                    let text = format!("Tmp File: {} was removed\n", &tmp_path);
                    app.popups.log.logs.push((formatted_time, text, *GREEN));
                }
                Err(e) => {
                    let err = format!("File Path: {}, Error: {}\n", &tmp_path, e);
                    app.popups
                        .log
                        .logs
                        .push((formatted_time, err.clone(), *RED));
                    app.popups.error.value = err;
                    app.popups.error.show = true;
                }
            }
            return false;
        }
        true
    });
}

fn delete_complete_from_list(app: &mut DownloadManager) {
    let now = Local::now();
    let formatted_time = now.format("%H:%M:%S").to_string();
    app.files
        .retain(|f| !f.file.complete.load(std::sync::atomic::Ordering::Relaxed));
    app.popups.log.logs.push((
        formatted_time,
        String::from("Delete complete downloads from list"),
        *CYAN,
    ));
}

fn delete_complete_from_disk(app: &mut DownloadManager) {
    let now = Local::now();
    let formatted_time = now.format("%H:%M:%S").to_string();
    for fdl in app.files.iter_mut() {
        let file = &fdl.file;
        let complete = file.complete.load(std::sync::atomic::Ordering::Relaxed);
        if !complete {
            let location = format!("{}/{}", file.dl_dir, file.name_on_disk);
            let tmp_location = format!("{}/.{}.metadl", file.dl_dir, file.name_on_disk);
            if Path::new(&location).exists() {
                if let Err(e) = remove_file(&location) {
                    let formatted_error = format!("{}: {:?}", location, &e);
                    app.popups.error.value = formatted_error.clone();
                    app.popups.error.show = true;
                    app.popups
                        .log
                        .logs
                        .push((formatted_time.clone(), formatted_error, *RED));
                }
            }
            if Path::new(&tmp_location).exists() {
                if let Err(e) = remove_file(&tmp_location) {
                    let formatted_error = format!("{}: {:?}", tmp_location, &e);
                    app.popups.error.value = formatted_error.clone();
                    app.popups.error.show = true;
                    app.popups
                        .log
                        .logs
                        .push((formatted_time.clone(), formatted_error, *RED));
                }
            }
        }
    }
    app.files
        .retain(|f| !f.file.complete.load(std::sync::atomic::Ordering::Relaxed));
}
