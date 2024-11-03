use crate::{
    colors::{CYAN, DARK_INNER, GRAY, RED},
    dl::file2dl::File2Dl,
    Actions, DownloadManager,
};
use egui_extras::{Column, TableBuilder};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::process::Command;

use egui_sfml::egui::*;
use irox_egui_extras::progressbar::ProgressBar;
use std::sync::atomic::Ordering::Relaxed;

pub fn lay_table(interface: &mut DownloadManager, ui: &mut Ui, ctx: &Context) {
    let available_width = ui.available_width();
    let mut select_size = 0f32;
    TableBuilder::new(ui)
        .auto_shrink(false)
        .striped(true)
        .column(Column::exact(available_width * 0.02))
        .column(Column::initial(available_width * 0.1857))
        .column(Column::initial(available_width * 0.255))
        .column(Column::initial(available_width * 0.15))
        .column(Column::exact(available_width * 0.2))
        .column(Column::initial(available_width * 0.1857))
        .header(20.0, |mut header| {
            header.col(|ui| {
                select_size = ui.available_width();
                ui.heading("");
            });
            header.col(|ui| {
                let text = RichText::new("Filename").color(*CYAN).strong();
                ui.horizontal(|ui| {
                    ui.add_space(ui.available_width() / 2.0 - (74.0 - select_size / 2.0));
                    ui.heading(text);
                });
                ui.add(Separator::grow(
                    Separator::default(),
                    ctx.screen_rect().width(),
                ));
            });
            header.col(|ui| {
                let text = RichText::new("Progress").color(*CYAN).strong();
                ui.vertical_centered(|ui| {
                    ui.heading(text);
                });
            });
            header.col(|ui| {
                let text = RichText::new("Speed").color(*CYAN).strong();
                ui.vertical_centered(|ui| {
                    ui.heading(text);
                });
            });
            header.col(|ui| {
                let text = RichText::new("Action on save").color(*CYAN).strong();
                ui.vertical_centered(|ui| {
                    ui.heading(text);
                });
            });
            header.col(|ui| {
                let text = RichText::new("Toggle").color(*CYAN).strong();
                ui.vertical_centered(|ui| {
                    ui.heading(text);
                });
            });
        })
        .body(|mut body| {
            let mut to_display = interface
                .files
                .iter()
                .filter(|f| {
                    f.file
                        .name_on_disk
                        .to_lowercase()
                        .contains(&interface.search)
                        || f.file.url.link.to_lowercase().contains(&interface.search)
                })
                .map(|f| f.to_owned())
                .collect::<Vec<_>>();
            to_display.sort_by(|a, b| {
                (a.file.complete.load(Relaxed), &a.file.name_on_disk)
                    .cmp(&(b.file.complete.load(Relaxed), &b.file.name_on_disk))
            });
            for fdl in to_display.iter_mut() {
                let file = &fdl.file;
                let complete = file.complete.load(std::sync::atomic::Ordering::Relaxed);
                let new = fdl.new;
                let file_has_error = fdl.has_error;
                body.row(30.0, |mut row| {
                    row.col(|ui| {
                        ui.vertical(|ui| {
                            ui.add_space(3.0);
                            ui.add_sized(
                                (ui.available_width(), ui.available_height() - 6.0),
                                Checkbox::without_text(&mut fdl.selected),
                            );
                            ui.add_space(3.0);
                        });
                        let file = interface
                            .files
                            .iter_mut()
                            .find(|f| f.file.name_on_disk == fdl.file.name_on_disk);
                        if let Some(f) = file {
                            f.selected = fdl.selected;
                        }
                    });
                    row.col(|ui| {
                        file_name(file_has_error, &file.name_on_disk, ui);
                    });
                    row.col(|ui| progress_bar(file, ui, ctx));
                    row.col(|ui| {
                        ui.vertical(|ui| {
                            ui.add_space(5.0);
                            let text = RichText::new(format!(
                                "{:.2} Mbs",
                                file.bytes_per_sec
                                    .load(std::sync::atomic::Ordering::Relaxed)
                                    as f64
                                    / (1024.0 * 1024.0)
                            ))
                            .size(15.0)
                            .strong();
                            let label = Label::new(text).wrap_mode(TextWrapMode::Truncate);
                            let res = ui.add_sized(
                                (ui.available_width(), ui.available_height() - 10.0),
                                label,
                            );
                            if res.hovered() {
                                let text = RichText::new(format!(
                                    "Limited to: {:.2}MBs",
                                    (file.speed.load(std::sync::atomic::Ordering::Relaxed) as f64
                                        / (1024 * 1024) as f64)
                                ));
                                res.show_tooltip_text(text);
                            }
                            ui.add_space(5.0);
                        });
                    });
                    row.col(|ui| {
                        match fdl.action_on_save {
                            Actions::Open if complete => {
                                let path = format!("Downloads/{}", fdl.file.name_on_disk);
                                match opener::open(path) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        interface.popups.error.value = e.to_string();
                                        interface.popups.error.show = true;
                                    }
                                }
                            }
                            Actions::Reboot if complete => {
                                reboot_system();
                            }
                            Actions::Shutdown if complete => {
                                shutdown_system();
                            }
                            _ => {}
                        }
                        ui.vertical_centered(|ui| {
                            ui.visuals_mut().widgets.inactive.weak_bg_fill = *CYAN;
                            ui.visuals_mut().widgets.open.weak_bg_fill = *CYAN;
                            ui.visuals_mut().widgets.hovered.weak_bg_fill = *CYAN;
                            ui.visuals_mut().widgets.active.weak_bg_fill = *CYAN;
                            ui.visuals_mut().widgets.inactive.fg_stroke.color = *DARK_INNER;
                            ui.visuals_mut().widgets.open.fg_stroke.color = *DARK_INNER;
                            ui.visuals_mut().widgets.hovered.fg_stroke.color = *DARK_INNER;
                            ui.visuals_mut().widgets.active.fg_stroke.color = *DARK_INNER;
                            ui.visuals_mut().override_text_color = Some(*DARK_INNER);
                            if !complete {
                                ui.centered_and_justified(|ui| {
                                    egui_sfml::egui::ComboBox::from_label("")
                                        .selected_text(format!("{:?}", fdl.action_on_save))
                                        .width(available_width * 0.2)
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut fdl.action_on_save,
                                                Actions::None,
                                                "None",
                                            );
                                            ui.selectable_value(
                                                &mut fdl.action_on_save,
                                                Actions::Open,
                                                "Open",
                                            );
                                            ui.selectable_value(
                                                &mut fdl.action_on_save,
                                                Actions::Shutdown,
                                                "Shutdown",
                                            );
                                            ui.selectable_value(
                                                &mut fdl.action_on_save,
                                                Actions::Reboot,
                                                "Reboot",
                                            );
                                        });
                                });
                            } else {
                                ui.centered_and_justified(|ui| {
                                    ComboBox::from_label("")
                                        .width(available_width * 0.2)
                                        .height(ui.available_height() - 10.0)
                                        .selected_text(format!("{:?}", fdl.action_on_save))
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut fdl.action_on_save,
                                                Actions::None,
                                                "None",
                                            );
                                        });
                                });
                            }
                        });
                    });
                    row.col(|ui| {
                        action_button(file, ui, complete, new);
                    });
                });
            }
        });
}

fn action_button(file: &File2Dl, ui: &mut Ui, complete: bool, new: bool) {
    let text = {
        let running = file.running.load(std::sync::atomic::Ordering::Relaxed);
        if !running {
            RichText::new(egui_phosphor::fill::PLAY).size(20.0)
        } else {
            RichText::new(egui_phosphor::fill::PAUSE).size(20.0)
        }
    };
    let but = {
        let button_text = if file.url.range_support || new {
            if !complete {
                text.color(*CYAN)
            } else {
                text
            }
        } else {
            text
        };

        Button::new(button_text).frame(false)
    };
    ui.vertical(|ui| {
        ui.add_space(3.0);
        let res = ui.add_sized((ui.available_width(), ui.available_height() - 6.0), but);
        if res.hovered() && !complete {
            if file.url.range_support {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand)
            } else if new {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand)
            } else {
                let text = RichText::new("File does not support resumption").color(*CYAN);
                res.show_tooltip_text(text);
            }
        }
        if res.clicked() && !complete {
            if file.url.range_support {
                file.switch_status();
            }
            if new && !file.url.range_support {
                file.switch_status();
            }
        }
        ui.add_space(3.0);
    });
}
fn progress_bar(file: &File2Dl, ui: &mut Ui, ctx: &Context) {
    let is_running = file.running.load(Relaxed);
    if is_running {
        ctx.request_repaint();
    }
    let size = file.size_on_disk.load(std::sync::atomic::Ordering::Relaxed) as f32;
    let total_size = file.url.content_length as f32;
    let percentage = size / total_size;
    ui.vertical(|ui| {
        ui.add_space(1.0);
        ui.scope(|ui| {
            ui.visuals_mut().extreme_bg_color = *GRAY;
            ui.visuals_mut().selection.bg_fill = *CYAN;
            ui.visuals_mut().override_text_color = Some(*DARK_INNER);
            let mut pb = ProgressBar::new(percentage)
                .desired_width(ui.available_width())
                .desired_height(ui.available_height() - 2.0)
                .text_center(format!("{}%", (percentage * 100.0) as i32));
            pb.is_indeterminate = is_running;
            let res = ui.add(pb);
            if res.hovered() {
                ui.set_width(ui.available_width());
                let size_mbs = size / (1024.0 * 1024.0);
                let total_size_mbs = file.url.content_length as f32 / (1024.0 * 1024.0);
                let text = RichText::new(format!("{:.3}/{:.3} Mbs", size_mbs, total_size_mbs))
                    .color(*CYAN);
                res.show_tooltip_text(text);
            };
        });
        ui.add_space(1.0);
    });
}

fn file_name(has_error: bool, name: &str, ui: &mut Ui) {
    let text = if has_error {
        RichText::new(name).strong().size(15.0).color(*RED)
    } else {
        RichText::new(name).strong().size(15.0)
    };

    let label = Label::new(text).wrap_mode(TextWrapMode::Truncate);
    ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
        ui.horizontal_centered(|ui| {
            ui.add(label);
        })
    });
}

fn reboot_system() {
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("reboot").exec();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("shutdown")
            .args(&["/r", "/t", "0"])
            .spawn()
            .expect("Failed to reboot");
    }
}

fn shutdown_system() {
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("shutdown").arg("now").exec();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("shutdown")
            .args(&["/s", "/t", "0"])
            .spawn()
            .expect("Failed to shutdown");
    }
}
