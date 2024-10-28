use eframe::egui::{self, Align2, Button, Color32, Vec2};

use crate::{dl::file2dl::File2Dl, Actions, FDl, MyApp};

pub fn show_input_window(ctx: &eframe::egui::Context, interface: &mut MyApp) {
    let window_size = egui::vec2(250.0, 200.0);
    egui::Window::new("Add Download")
        .default_size(window_size)
        .pivot(Align2::RIGHT_CENTER)
        .resizable(false)
        .title_bar(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("Add Download").strong());
            });
            ui.separator();
            ui.label("URL:");
            if !interface.popups.download.error.is_empty() {
                ui.colored_label(Color32::RED, &interface.popups.download.error);
            }
            ui.text_edit_singleline(&mut interface.popups.download.link);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Action on save:");
                    egui::ComboBox::from_label("")
                        .selected_text(format!("{:?}", &interface.temp_action))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut interface.temp_action, Actions::None, "None");
                            ui.selectable_value(
                                &mut interface.temp_action,
                                Actions::Shutdown,
                                "Shutdown",
                            );
                            ui.selectable_value(
                                &mut interface.temp_action,
                                Actions::Reboot,
                                "Reboot",
                            );
                        });
                })
            });

            ui.add_space(5f32);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if ui.button("Confirm").clicked() {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        let tx = interface.popups.download.error_channel.0.clone();
                        let file_tx = interface.file_channel.0.clone();
                        let link = interface.popups.download.link.clone();

                        rt.block_on(async move {
                            match File2Dl::new(&link, "Downloads").await {
                                Ok(file) => file_tx.send(file).unwrap(),
                                Err(e) => {
                                    tx.send(e.to_string()).unwrap();
                                }
                            };
                        });
                        if let Ok(val) = interface.popups.download.error_channel.1.try_recv() {
                            interface.popups.download.error = val;
                            return;
                        }
                        let file = match interface.file_channel.1.try_recv() {
                            Ok(file) => file,
                            Err(e) => {
                                interface.popups.download.error = e.to_string();
                                return;
                            }
                        };

                        file.switch_status();
                        let file = FDl {
                            file,
                            initiated: false,
                            selected: false,
                            action_on_save: interface.temp_action.clone(),
                        };
                        interface.temp_action = Actions::None;
                        interface.files.push(file);
                        interface.popups.download.show = false;
                        interface.popups.download.error = String::default();
                    }
                    ui.add_space(180.0);
                    if ui.button("Cancel").clicked() {
                        interface.popups.download.show = false;
                        interface.popups.download.error = String::default();
                    }
                });
            });
        });
}

pub fn show_error_window(ctx: &eframe::egui::Context, interface: &mut MyApp, error: &str) {
    let window_size = egui::vec2(250.0, 200.0);
    egui::Window::new("Error Window")
        .pivot(Align2::RIGHT_CENTER)
        .default_size(window_size)
        .resizable(false)
        .title_bar(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.colored_label(Color32::RED, "Error!");
                ui.label(error);
            });
            ui.add_space(15.0);
            ui.vertical_centered(|ui| {
                if ui
                    .add_sized(Vec2::new(60.0, 30.0), Button::new("Ok"))
                    .clicked()
                {
                    interface.popups.error.show = false;
                }
            });
        });
}

pub fn show_confirm_window(
    ctx: &eframe::egui::Context,
    interface: &mut MyApp,
    color: Color32,
    text: &str,
    action: Box<dyn FnOnce(&mut MyApp) + 'static>,
) {
    let window_size = egui::vec2(250.0, 200.0);
    egui::Window::new("Confirm")
        .default_size(window_size)
        .pivot(Align2::RIGHT_CENTER)
        .resizable(false)
        .title_bar(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label("Are u sure?");
                ui.label(egui::RichText::new(text).strong().color(color));
            });
            ui.separator();
            ui.add_space(10.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    if ui
                        .add_sized(Vec2::new(40.0, 30.0), Button::new("Yes"))
                        .clicked()
                    {
                        action(interface);
                        interface.popups.confirm.show = false;
                    }
                    ui.add_space(125.0);
                    if ui
                        .add_sized(Vec2::new(40.0, 30.0), Button::new("No"))
                        .clicked()
                    {
                        interface.popups.confirm.show = false;
                    }
                })
            });
            ui.add_space(10.0);
        });
}
