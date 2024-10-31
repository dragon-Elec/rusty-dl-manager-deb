use std::sync::{atomic::AtomicUsize, Arc};

use eframe::egui::{self, Align2, Button, Color32, Layout, Pos2, RichText, TextEdit, Vec2};
use egui_aesthetix::{themes::TokyoNight, Aesthetix};
use egui_plot::{Legend, Line};

use crate::{
    colors::{CYAN, DARKER_PURPLE, DARK_INNER, GRAY, RED},
    dl::{file2dl::File2Dl, metadata::init_metadata},
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
            ui.vertical_centered(|ui| {
                if !interface.popups.download.error.is_empty() {
                    if interface.popups.download.error == "Initiating..." {
                        ui.colored_label(*CYAN, &interface.popups.download.error);
                    } else {
                        ui.colored_label(*RED, &interface.popups.download.error);
                    }
                }
                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                });
                ui.colored_label(*CYAN, "URL:");
                ui.scope(|ui| {
                    ui.visuals_mut().extreme_bg_color = *CYAN;
                    let single_line = TextEdit::singleline(&mut interface.popups.download.link)
                        .text_color(*GRAY)
                        .hint_text("Link")
                        .desired_width(350.0);
                    ui.add(single_line);
                });
                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                });
                ui.colored_label(*CYAN, "speed in Mbs: (Will be ignored if empty)");
                ui.horizontal(|ui| {
                    ui.add_space(2.0);
                });
                ui.scope(|ui| {
                    ui.visuals_mut().extreme_bg_color = *CYAN;
                    let single_line = TextEdit::singleline(&mut interface.popups.download.speed)
                        .desired_width(50.0)
                        .text_color(*GRAY)
                        .hint_text("Mbs");
                    ui.add(single_line);
                });
                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                });
                ui.colored_label(*CYAN, "Action on save:");
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        let visuals = ui.visuals_mut();
                        visuals.widgets.inactive.weak_bg_fill = *CYAN;
                        visuals.widgets.open.weak_bg_fill = *CYAN;
                        visuals.widgets.hovered.weak_bg_fill = *CYAN;
                        visuals.widgets.active.weak_bg_fill = *CYAN;
                        visuals.widgets.inactive.fg_stroke.color = *DARK_INNER;
                        visuals.widgets.open.fg_stroke.color = *DARK_INNER;
                        visuals.widgets.hovered.fg_stroke.color = *DARK_INNER;
                        visuals.widgets.active.fg_stroke.color = *DARK_INNER;
                        visuals.override_text_color = Some(*DARK_INNER);
                        egui::ComboBox::from_label("")
                            .width(370.0)
                            .selected_text(format!("{:?}", &interface.temp_action))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut interface.temp_action,
                                    Actions::None,
                                    "None",
                                );
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
                ui.horizontal(|ui| {
                    ui.add_space(5.0);
                });
            });

            ui.add_space(5f32);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let butt = Button::new("Confirm").fill(*CYAN);
                    ui.scope(|ui| {
                        ui.visuals_mut().override_text_color = Some(*DARKER_PURPLE);
                        if ui.add(butt).clicked() && interface.popups.download.temp_file.is_none() {
                            let speed_string = &interface.popups.download.speed;
                            if !speed_string.is_empty() {
                                match speed_string.parse::<f64>() {
                                    Ok(_) => {}
                                    Err(_) => {
                                        interface.popups.download.error =
                                            String::from("Enter a valid number");
                                        return;
                                    }
                                };
                            }
                            let tx = interface.popups.download.error_channel.0.clone();
                            let file_tx = interface.popups.download.file_channel.0.clone();
                            let link = interface.popups.download.link.clone();
                            interface.popups.download.error = String::from("Initiating...");
                            interface.runtime.spawn(async move {
                                match File2Dl::new(&link, "Downloads").await {
                                    Ok(file) => file_tx.send(file).unwrap(),
                                    Err(e) => {
                                        tx.send(e.to_string()).unwrap();
                                    }
                                };
                            });
                        }
                    });

                    if let Ok(err) = interface.popups.download.error_channel.1.try_recv() {
                        interface.popups.download.error = err;
                        return;
                    }
                    if let Ok(file) = interface.popups.download.file_channel.1.try_recv() {
                        interface.popups.download.temp_file = Some(file);
                    };
                    if let Some(mut file) = interface.popups.download.temp_file.to_owned() {
                        let speed_string = &interface.popups.download.speed;
                        let speed = if speed_string.is_empty() {
                            0f64
                        } else {
                            speed_string.parse::<f64>().unwrap()
                        };
                        let speed = (speed * (1024.0 * 1024.0)) as usize;
                        file.speed = Arc::new(AtomicUsize::new(speed));
                        file.switch_status();
                        let file = FDl {
                            file,
                            new: true,
                            initiated: false,
                            selected: false,
                            action_on_save: interface.temp_action.clone(),
                        };
                        interface.popups.download.show = false;
                        interface.popups.download.error = String::default();
                        interface.popups.download.temp_file = None;
                        interface.temp_action = Actions::None;
                        interface.files.push(file);
                    }
                    ui.add_space(249.0);
                    let butt = Button::new("Cancel").fill(*CYAN);
                    ui.scope(|ui| {
                        ui.visuals_mut().override_text_color = Some(*DARKER_PURPLE);
                        if ui.add(butt).clicked() {
                            interface.popups.download.show = false;
                            interface.popups.download.error = String::default();
                        }
                    });
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
                    ui.visuals_mut().override_text_color = Some(*DARKER_PURPLE);
                    ui.add_space(20.0);
                    let butt = Button::new(egui_phosphor::regular::CHECK).fill(*CYAN);
                    if ui.add_sized(Vec2::new(40.0, 30.0), butt).clicked() {
                        action(interface);
                        interface.popups.confirm.show = false;
                    }
                    ui.add_space(125.0);
                    let butt = Button::new(egui_phosphor::regular::X).fill(*CYAN);
                    if ui.add_sized(Vec2::new(40.0, 30.0), butt).clicked() {
                        interface.popups.confirm.show = false;
                    }
                })
            });
            ui.add_space(10.0);
        });
}

pub fn show_plot_window(ctx: &eframe::egui::Context, interface: &mut MyApp) {
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
                    .allow_drag(true)
                    .allow_scroll(false)
                    .height(ui.available_height() - 40.0)
                    .width(ui.available_width())
                    .show_x(false)
                    .show_y(false)
                    .legend(Legend::default())
                    .show(ui, |plot_ui| {
                        let points = interface
                            .bandwidth
                            .history
                            .iter()
                            .enumerate()
                            .map(|(i, &y)| [i as f64, y as f64 / (1024.0 * 1024.0)])
                            .collect::<Vec<[f64; 2]>>();
                        plot_ui.line(Line::new(points).name("Total Bandwidth").color(*CYAN));
                    });
                ui.scope(|ui| {
                    ui.visuals_mut().override_text_color = Some(*DARKER_PURPLE);
                    let button = Button::new(egui_phosphor::regular::CHECK).fill(*CYAN);
                    if ui.add(button).clicked() {
                        interface.popups.plot.show = false;
                    }
                });
            })
        });
}

pub fn show_modify_speed_window(ctx: &eframe::egui::Context, interface: &mut MyApp) {
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
            ui.scope(|ui| {
                ui.set_height(20.0);
                ui.vertical_centered(|ui| {
                    ui.colored_label(*CYAN, "Modify speed");
                });
            });

            if !interface.popups.speed.error.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.colored_label(*RED, &interface.popups.speed.error);
                });
            }
            ui.separator();
            ui.with_layout(Layout::left_to_right(egui::Align::LEFT), |ui| {
                ui.scope(|ui| {
                    ui.visuals_mut().extreme_bg_color = *CYAN;
                    ui.visuals_mut().override_text_color = Some(*GRAY);
                    let single_text =
                        TextEdit::singleline(&mut interface.popups.speed.temp_val).hint_text("Mbs");
                    ui.add_sized((310.0, 28.0), single_text);
                });
                let text = RichText::new(egui_phosphor::regular::CLOCK_CLOCKWISE).size(20.0);
                let button = Button::new(text).fill(*CYAN);
                ui.visuals_mut().override_text_color = Some(*DARK_INNER);
                let res = ui.add(button);
                if res.hovered() {
                    let text =
                        RichText::new("Will reset selected download speeds to 0").color(*CYAN);
                    res.show_tooltip_text(text);
                }
                if res.clicked() {
                    let files = interface
                        .files
                        .iter_mut()
                        .filter(|f| f.selected)
                        .collect::<Vec<_>>();
                    for f in files {
                        f.file.speed.store(0, std::sync::atomic::Ordering::Relaxed);
                        match init_metadata(&f.file, "Downloads") {
                            Ok(_) => {}
                            Err(e) => {
                                interface.popups.error.value = e.to_string();
                                interface.popups.error.show = true;
                            }
                        }
                    }
                    interface.popups.speed.show = false;
                }
            });
            ui.add_space(10.0);
            ui.with_layout(Layout::left_to_right(egui::Align::LEFT), |ui| {
                ui.visuals_mut().override_text_color = Some(*DARK_INNER);
                let text = RichText::new(egui_phosphor::regular::CHECK).size(20.0);
                let button = Button::new(text).fill(*CYAN);
                let res = ui.add(button);
                if res.clicked() {
                    let speed = match interface.popups.speed.temp_val.parse::<f64>() {
                        Ok(f) => f,
                        Err(e) => {
                            interface.popups.speed.error = e.to_string();
                            return;
                        }
                    };
                    let speed = (speed * (1024.0 * 1024.0)) as usize;
                    let files = interface
                        .files
                        .iter_mut()
                        .filter(|f| f.selected)
                        .collect::<Vec<_>>();
                    for f in files {
                        f.file
                            .speed
                            .store(speed, std::sync::atomic::Ordering::Relaxed);
                        match init_metadata(&f.file, "Downloads") {
                            Ok(_) => {}
                            Err(e) => {
                                interface.popups.error.value = e.to_string();
                                interface.popups.error.show = true;
                            }
                        }
                    }
                    interface.popups.speed.show = false;
                }
                ui.add_space(280.0);
                let text = RichText::new(egui_phosphor::regular::X).size(20.0);
                let button = Button::new(text).fill(*CYAN);
                let res = ui.add(button);
                if res.clicked() {
                    interface.popups.speed.show = false;
                }
            })
        });
}
