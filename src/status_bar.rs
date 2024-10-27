use crate::MyApp;
use eframe::egui::{menu, Color32};
use std::fs::{read_dir, remove_file};

pub fn init_menu_bar(interface: &mut MyApp, ui: &mut eframe::egui::Ui) {
    menu::bar(ui, |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    file_button_content(interface, ui);
                });
                ui.menu_button("Downloads", |ui| {
                    if ui.button("Add Download").clicked() {
                        interface.popups.download.show = true;
                    }
                    if ui.button("Resume all").clicked() {
                        for core in interface.files.iter_mut() {
                            core.file
                                .running
                                .store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    if ui.button("Pause all").clicked() {
                        for core in interface.files.iter_mut() {
                            core.file
                                .running
                                .store(false, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    if ui.button("Delete all completed").clicked() {
                        interface.files.retain(|core| {
                            !core
                                .file
                                .complete
                                .load(std::sync::atomic::Ordering::Relaxed)
                        });
                    }
                    if ui.button("Delete all incomplete").clicked() {
                        interface.files.retain(|core| {
                            core.file
                                .complete
                                .load(std::sync::atomic::Ordering::Relaxed)
                        });
                    }
                });
            });
        });
    });
}

fn file_button_content(interface: &mut MyApp, ui: &mut eframe::egui::Ui) {
    if ui.button("Remove selected from list").clicked() {
        interface.popups.confirm.color = Color32::GREEN;
        interface.popups.confirm.task = Box::new(|| {
            Box::new(move |app: &mut MyApp| {
                app.files.retain(|core| !core.selected);
            })
        });
        interface.popups.confirm.show = true;
        interface.popups.confirm.text = String::from("This will remove files selected from list")
    }

    if ui.button("Remove selected from disk").clicked() {
        interface.popups.confirm.color = Color32::RED;
        interface.popups.confirm.task = Box::new(|| {
            Box::new(move |app: &mut MyApp| {
                remove_selected_from_disk(app);
            })
        });
        interface.popups.confirm.show = true;
        interface.popups.confirm.text = String::from("This will remove selected files from disk")
    }
    if ui.button("Remove all from list").clicked() {
        interface.popups.confirm.color = Color32::GREEN;
        interface.popups.confirm.task = Box::new(|| {
            Box::new(move |app: &mut MyApp| {
                app.files.clear();
            })
        });
        interface.popups.confirm.show = true;
        interface.popups.confirm.text = "This will not delete files from disk".to_string();
    }
    if ui.button("Remove all from disk").clicked() {
        interface.popups.confirm.color = Color32::RED;
        interface.popups.confirm.task = Box::new(|| {
            Box::new(move |app: &mut MyApp| {
                delete_all_files_from_disk(app);
            })
        });
        interface.popups.confirm.show = true;
        interface.popups.confirm.text = String::from("This will remove all files from disk")
    }
}
fn delete_all_files_from_disk(interface: &mut MyApp) {
    let dir = match read_dir("Downloads") {
        Ok(dir) => dir,
        Err(e) => {
            interface.popups.error.value = e.to_string();
            interface.popups.error.show = true;
            return;
        }
    };
    for file in dir {
        let path = match file {
            Ok(file) => file.path(),
            Err(e) => {
                interface.popups.error.value = e.to_string();
                interface.popups.error.show = true;
                return;
            }
        };
        remove_file(path).unwrap();
    }
    interface.files.clear();
}
fn remove_selected_from_disk(app: &mut MyApp) {
    app.files.retain(|core| {
        if core.selected {
            let path = format!("Downloads/{}", core.file.name_on_disk);
            let tmp_path = format!("Downloads/.{}.metadata", core.file.name_on_disk);
            match remove_file(path) {
                Ok(_) => {}
                Err(e) => {
                    app.popups.error.value = e.to_string();
                    app.popups.error.show = true;
                }
            }
            match remove_file(tmp_path) {
                Ok(_) => {}
                Err(e) => {
                    app.popups.error.value = e.to_string();
                    app.popups.error.show = true;
                }
            }
            return false;
        }
        true
    });
}
