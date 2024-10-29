use eframe::egui::{self, Align, Align2, Button, Color32, Pos2, Vec2};
use egui_aesthetix::{themes::TokyoNight, Aesthetix};
use egui_plot::{Line, PlotPoint, PlotPoints};

use crate::{
    colors::{CYAN, DARKER_PURPLE},
    dl::file2dl::File2Dl,
    Actions, FDl, MyApp,
};

pub fn show_input_window(ctx: &eframe::egui::Context, interface: &mut MyApp) {
    let window_size = egui::vec2(250.0, 200.0);
    let pos = Pos2::new(
        ctx.available_rect().width() / 2.0,
        ctx.available_rect().height() / 2.3,
    );
    egui::Window::new("Add Download")
        .pivot(Align2::CENTER_CENTER)
        .current_pos(pos)
        .default_size(window_size)
        .resizable(false)
        .frame(
            egui::Frame::default()
                .fill(*DARKER_PURPLE)
                .inner_margin(TokyoNight.margin_style())
                .stroke(egui::Stroke::new(
                    1.0,
                    TokyoNight.bg_secondary_color_visuals(),
                )),
        )
        .title_bar(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.colored_label(*CYAN, "Add download");
            });
            ui.separator();
            ui.colored_label(*CYAN, "URL:");
            if !interface.popups.download.error.is_empty() {
                ui.colored_label(Color32::RED, &interface.popups.download.error);
            }
            ui.text_edit_singleline(&mut interface.popups.download.link);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(*CYAN, "Action on save:");
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
    let pos = Pos2::new(
        ctx.available_rect().width() / 2.0,
        ctx.available_rect().height() / 2.3,
    );
    egui::Window::new("Error Window")
        .pivot(Align2::CENTER_CENTER)
        .current_pos(pos)
        .default_size(window_size)
        .frame(
            egui::Frame::none()
                .fill(*DARKER_PURPLE)
                .inner_margin(TokyoNight.margin_style())
                .stroke(egui::Stroke::new(
                    1.0,
                    TokyoNight.bg_secondary_color_visuals(),
                )),
        )
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
    let pos = Pos2::new(
        ctx.available_rect().width() / 2.0,
        ctx.available_rect().height() / 2.3,
    );
    egui::Window::new("Confirm")
        .default_size(window_size)
        .pivot(Align2::CENTER_CENTER)
        .current_pos(pos)
        .frame(
            egui::Frame::none()
                .fill(*DARKER_PURPLE)
                .inner_margin(TokyoNight.margin_style())
                .stroke(egui::Stroke::new(
                    1.0,
                    TokyoNight.bg_secondary_color_visuals(),
                )),
        )
        .resizable(false)
        .title_bar(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.colored_label(*CYAN, "Are u sure?");
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

pub fn show_plot_window(ctx: &eframe::egui::Context, interface: &mut MyApp) {
    ctx.request_repaint();
    let window_size = egui::vec2(
        ctx.available_rect().width() / 2.0,
        ctx.available_rect().height() / 2.0,
    );
    let pos = Pos2::new(window_size.x, window_size.y);
    egui::Window::new("Error Window")
        .pivot(Align2::CENTER_CENTER)
        .current_pos(pos)
        .default_size(window_size)
        .resizable(false)
        .frame(
            egui::Frame::none()
                .fill(*DARKER_PURPLE)
                .inner_margin(TokyoNight.margin_style())
                .stroke(egui::Stroke::new(
                    1.0,
                    TokyoNight.bg_secondary_color_visuals(),
                )),
        )
        .resizable(false)
        .title_bar(false)
        .show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                egui_plot::Plot::new("plot")
                    .allow_zoom(false)
                    .allow_drag(false)
                    .allow_scroll(false)
                    .height(ui.available_height() - 40.0)
                    .width(ui.available_width())
                    .show_y(false)
                    .show(ui, |plot_ui| {
                        let points = interface
                            .bandwidth
                            .history
                            .iter()
                            .enumerate()
                            .map(|(i, &y)| [i as f64, y as f64 / (1024.0 * 1024.0)])
                            .collect::<Vec<[f64; 2]>>();
                        let points = PlotPoints::new(points);
                        plot_ui.line(Line::new(points).name("Total Bandwidth").color(*CYAN));
                    });
                if ui.button("Close").clicked() {
                    interface.popups.plot.show = false;
                }
            })
        });
}
