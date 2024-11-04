use chrono::Local;
use egui_aesthetix::{themes::TokyoNight, Aesthetix};
use egui_plot::{Legend, Line};
use egui_sfml::egui::{
    frame, vec2, Align2, Button, Color32, ComboBox, Context, CursorIcon, Frame, Label, Layout,
    Pos2, RichText, ScrollArea, Separator, Stroke, TextEdit, Vec2, Window,
};
use native_dialog::FileDialog;
use serde_json::json;
use std::{
    fs::OpenOptions,
    io::Write,
    path::Path,
    sync::{atomic::AtomicUsize, Arc},
    time::Instant,
};

use crate::{
    colors::{CYAN, DARKER_PURPLE, DARK_INNER, GRAY, PURPLE, RED},
    dl::{file2dl::File2Dl, metadata::init_metadata},
    Actions, DownloadManager, FDl,
};

#[derive(Default)]
pub struct Bandwidth {
    pub total_bandwidth: usize,
    pub history: Vec<usize>,
}

pub fn show_input_window(ctx: &Context, interface: &mut DownloadManager) {
    let window_size = vec2(250.0, 200.0);
    let pos = Pos2::new(
        ctx.available_rect().width() / 2.0,
        ctx.available_rect().height() / 2.3,
    );
    interface.show_window = true;
    let dl_dir = interface.settings.dl_dir.clone();
    Window::new("Download window")
        .pivot(Align2::CENTER_CENTER)
        .fixed_pos(pos)
        .default_size(window_size)
        .frame(
            Frame::default()
                .fill(*DARKER_PURPLE)
                .inner_margin(TokyoNight.margin_style())
                .stroke(Stroke::new(
                    1.0,
                    Color32::from_rgba_premultiplied(31, 31, 51, 255),
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
                        .text_color(*PURPLE)
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
                        ComboBox::from_label("")
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
                                match File2Dl::new(&link, &dl_dir).await {
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
                            has_error: false,
                            toggled_at: Instant::now(),
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

pub fn show_error_window(ctx: &Context, interface: &mut DownloadManager, error: &str) {
    let window_size = vec2(250.0, 200.0);
    let pos = Pos2::new(
        ctx.available_rect().width() / 2.0,
        ctx.available_rect().height() / 2.3,
    );
    Window::new("Error Window")
        .pivot(Align2::CENTER_CENTER)
        .fixed_pos(pos)
        .default_size(window_size)
        .frame(
            Frame::none()
                .fill(*DARKER_PURPLE)
                .inner_margin(TokyoNight.margin_style())
                .stroke(Stroke::new(
                    1.0,
                    Color32::from_rgba_premultiplied(31, 31, 51, 255),
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
    ctx: &Context,
    interface: &mut DownloadManager,
    color: Color32,
    text: &str,
    action: Box<dyn FnOnce(&mut DownloadManager) + 'static>,
) {
    let window_size = vec2(250.0, 200.0);
    let pos = Pos2::new(
        ctx.available_rect().width() / 2.0,
        ctx.available_rect().height() / 2.3,
    );
    Window::new("Confirm Window")
        .fixed_size(window_size)
        .pivot(Align2::CENTER_CENTER)
        .fixed_pos(pos)
        .frame(
            Frame::none()
                .fill(*DARKER_PURPLE)
                .inner_margin(TokyoNight.margin_style())
                .stroke(Stroke::new(
                    1.0,
                    Color32::from_rgba_premultiplied(31, 31, 51, 255),
                )),
        )
        .resizable(false)
        .title_bar(false)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.colored_label(*CYAN, "Are u sure?");
                ui.add(Separator::grow(Separator::default(), ui.available_width()));
                ui.label(RichText::new(text).strong().color(color));
            });
            ui.horizontal_centered(|ui| {
                ui.visuals_mut().override_text_color = Some(*DARKER_PURPLE);
                ui.add_space(20.0);
                let butt = Button::new(egui_phosphor::regular::CHECK).fill(*CYAN);
                if ui.add_sized(Vec2::new(40.0, 30.0), butt).clicked() {
                    action(interface);
                    interface.popups.confirm.show = false;
                }
                ui.add_space(ui.available_width() - 60.0);
                let butt = Button::new(egui_phosphor::regular::X).fill(*CYAN);
                if ui.add_sized(Vec2::new(40.0, 30.0), butt).clicked() {
                    interface.popups.confirm.show = false;
                }
            });
            ui.add_space(10.0);
        });
}

pub fn show_plot_window(ctx: &Context, interface: &mut DownloadManager) {
    let window_size = vec2(
        ctx.available_rect().width() / 2.0,
        ctx.available_rect().height() / 2.0,
    );
    let pos = Pos2::new(window_size.x, window_size.y);
    Window::new("Plot Window")
        .pivot(Align2::CENTER_CENTER)
        .fixed_pos(pos)
        .default_size(window_size)
        .frame(
            Frame::none()
                .fill(*DARKER_PURPLE)
                .inner_margin(TokyoNight.margin_style())
                .stroke(Stroke::new(
                    1.0,
                    Color32::from_rgba_premultiplied(31, 31, 51, 255),
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

pub fn show_modify_speed_window(ctx: &Context, interface: &mut DownloadManager) {
    let window_size = vec2(250.0, 200.0);
    let pos = Pos2::new(
        ctx.available_rect().width() / 2.0,
        ctx.available_rect().height() / 2.3,
    );
    Window::new("Speed Window")
        .default_size(window_size)
        .pivot(Align2::CENTER_CENTER)
        .fixed_pos(pos)
        .frame(
            Frame::none()
                .fill(*DARKER_PURPLE)
                .inner_margin(TokyoNight.margin_style())
                .stroke(Stroke::new(
                    1.0,
                    Color32::from_rgba_premultiplied(31, 31, 51, 255),
                )),
        )
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
            ui.with_layout(Layout::left_to_right(egui_sfml::egui::Align::LEFT), |ui| {
                ui.scope(|ui| {
                    ui.visuals_mut().extreme_bg_color = *CYAN;
                    ui.visuals_mut().override_text_color = Some(*DARK_INNER);
                    let hint_text = RichText::new("Mbs").color(*GRAY);
                    let single_text = TextEdit::singleline(&mut interface.popups.speed.temp_val)
                        .hint_text(hint_text);
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
                        match init_metadata(&f.file, &interface.settings.dl_dir) {
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
            ui.with_layout(Layout::left_to_right(egui_sfml::egui::Align::LEFT), |ui| {
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
                        match init_metadata(&f.file, &interface.settings.dl_dir) {
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

pub fn show_log_window(ctx: &Context, interface: &mut DownloadManager) {
    let window_size = vec2(
        ctx.available_rect().width() / 2.0,
        ctx.available_rect().height() / 2.0,
    );
    let logs = interface.popups.log.logs.clone();
    let pos = Pos2::new(window_size.x, window_size.y);
    Window::new("Log Window")
        .pivot(Align2::CENTER_CENTER)
        .fixed_pos(pos)
        .default_size(window_size)
        .frame(
            Frame::none()
                .fill(*DARKER_PURPLE)
                .inner_margin(TokyoNight.margin_style())
                .stroke(Stroke::new(
                    1.0,
                    Color32::from_rgba_premultiplied(31, 31, 51, 255),
                )),
        )
        .resizable(false)
        .title_bar(false)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.with_layout(Layout::right_to_left(egui_sfml::egui::Align::LEFT), |ui| {
                    ui.scope(|ui| {
                        let text = RichText::new(egui_phosphor::regular::X)
                            .size(15.0)
                            .color(*RED);
                        let butt = Button::new(text).frame(false);
                        let res = ui.add(butt);
                        if res.hovered() {
                            ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                        }
                        if res.clicked() {
                            interface.popups.log.show = false;
                        }
                    });
                    ui.add_space(ui.available_width() / 2.0 - 30.0);
                    ui.colored_label(*CYAN, "Log");
                });
            });
            ui.vertical_centered(|ui| {
                frame::Frame::none().fill(*PURPLE).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.set_height(ui.available_height());
                    ScrollArea::both().min_scrolled_width(10.0).show(ui, |ui| {
                        ui.vertical(|ui| {
                            for log in logs {
                                let formatted = format!("{}: {}", log.0, log.1);
                                ui.add(Label::new(RichText::new(formatted).color(log.2)));
                            }
                        });
                    })
                })
            })
        });
}

pub fn show_settings_window(ctx: &Context, interface: &mut DownloadManager) {
    let window_size = vec2(400.0, 200.0);

    let pos = Pos2::new(
        ctx.available_rect().width() / 2.0,
        ctx.available_rect().height() / 2.3,
    );

    Window::new("Settings window")
        .pivot(Align2::CENTER_CENTER)
        .fixed_pos(pos)
        .fixed_size(window_size)
        .frame(
            Frame::none()
                .fill(*DARKER_PURPLE)
                .inner_margin(TokyoNight.margin_style())
                .stroke(Stroke::new(
                    1.0,
                    Color32::from_rgba_premultiplied(31, 31, 51, 255),
                )),
        )
        .title_bar(false)
        .show(ctx, |ui| {
            let now = Local::now();
            let formatted_time = now.format("%H:%M:%S").to_string();
            ui.vertical_centered(|ui| {
                ui.colored_label(*CYAN, "Change settings");
                ui.add_space(20.0);
                if !interface.popups.settings.error.is_empty() {
                    ui.colored_label(*RED, &interface.popups.settings.error);
                }
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() / 2.0 - 155.0);
                    ui.visuals_mut().extreme_bg_color = *CYAN;
                    ui.visuals_mut().override_text_color = Some(*DARKER_PURPLE);
                    let hint = RichText::new("Download directory").color(*GRAY);
                    let dl_dir =
                        TextEdit::singleline(&mut interface.popups.settings.dl_dir).hint_text(hint);
                    let btn_txt = RichText::new(egui_phosphor::regular::DOTS_THREE)
                        .color(*DARKER_PURPLE)
                        .size(20.0);
                    let btn = Button::new(btn_txt).fill(*CYAN);
                    ui.add_sized((275.0, 28.0), dl_dir);
                    let res = ui.add(btn);

                    if res.clicked() {
                        let path = FileDialog::new().show_open_single_dir().unwrap();
                        match path {
                            Some(path) => {
                                interface.popups.settings.dl_dir =
                                    path.to_string_lossy().to_string();
                            }
                            None => {
                                interface.popups.settings.error =
                                    String::from("Couldn't accept this dir");
                                interface.popups.log.logs.push((
                                    formatted_time.clone(),
                                    String::from("Couldn't set dir"),
                                    *RED,
                                ));
                            }
                        };
                    }
                });
            });
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.visuals_mut().extreme_bg_color = *CYAN;
                ui.visuals_mut().override_text_color = Some(*DARKER_PURPLE);
                let hint = RichText::new("Download retry interval in secs").color(*GRAY);
                let temp_str =
                    TextEdit::singleline(&mut interface.popups.settings.temp_str).hint_text(hint);
                ui.add_sized((310.0, 28.0), temp_str);
                ui.add_space(20.0);
            });
            ui.with_layout(Layout::left_to_right(egui_sfml::egui::Align::LEFT), |ui| {
                ui.visuals_mut().override_text_color = Some(*DARK_INNER);
                let text = RichText::new(egui_phosphor::regular::CHECK).size(20.0);
                let button = Button::new(text).fill(*CYAN);
                let res = ui.add(button);
                if res.clicked() {
                    if !interface.popups.settings.temp_str.is_empty() {
                        match interface.popups.settings.temp_str.parse::<u64>() {
                            Ok(val) => interface.settings.retry_interval = val,
                            Err(e) => {
                                let error = e.to_string();
                                interface.popups.log.logs.push((
                                    formatted_time.clone(),
                                    error.clone(),
                                    *RED,
                                ));
                                interface.popups.settings.error = error;
                                return;
                            }
                        };
                    } else {
                        interface.settings.retry_interval = 5;
                    }

                    if Path::new(&interface.popups.settings.dl_dir).is_dir() {
                        interface.settings.dl_dir = interface.popups.settings.dl_dir.clone();
                    } else {
                        let text = String::from("Not a valid dir");
                        interface.popups.log.logs.push((
                            formatted_time.clone(),
                            text.clone(),
                            *RED,
                        ));
                        interface.popups.settings.error = text;
                        return;
                    }

                    let settings = json!(interface.settings).to_string();
                    let file = OpenOptions::new()
                        .create(true)
                        .truncate(true)
                        .write(true)
                        .open("settings.json");
                    match file {
                        Ok(mut f) => {
                            if let Err(e) = f.write_all(settings.as_bytes()) {
                                let text = format!("Couldn't write to file: {:?}", e);
                                interface.popups.log.logs.push((
                                    formatted_time.clone(),
                                    text.clone(),
                                    *RED,
                                ));
                                interface.popups.settings.error = text;

                                return;
                            } else {
                                match DownloadManager::load_files(&interface.settings) {
                                    Ok(fs) => {
                                        interface.popups.settings.show = false;
                                        interface.files = fs;
                                        interface.popups.log.logs.push((
                                            formatted_time.clone(),
                                            String::from("Updated log"),
                                            *RED,
                                        ));
                                    }
                                    Err(e) => {
                                        let text = format!(
                                            "Couldn't load new files after dir change: {:?}",
                                            e
                                        );
                                        interface.popups.log.logs.push((
                                            formatted_time.clone(),
                                            text.clone(),
                                            *RED,
                                        ));
                                        interface.popups.settings.error = text;
                                        return;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            let text = format!("File open error: {:?}", e);
                            interface.popups.log.logs.push((
                                formatted_time.clone(),
                                text.clone(),
                                *RED,
                            ));
                            interface.popups.settings.error = text;
                            return;
                        }
                    }
                }
                ui.add_space(ui.available_width() - 30.0);
                let text = RichText::new(egui_phosphor::regular::X).size(20.0);
                let button = Button::new(text).fill(*CYAN);
                let res = ui.add(button);
                if res.clicked() {
                    interface.popups.settings.show = false;
                }
            })
        });
}
