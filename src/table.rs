use std::{os::unix::process::CommandExt, process::Command};

use crate::{dl::file2dl::File2Dl, Actions, MyApp};
use eframe::egui::{
    self, style::Spacing, Align, Button, Checkbox, Color32, ComboBox, Context, CursorIcon, Label,
    Layout, Painter, Pos2, RichText, Sense, Separator, Stroke, Ui, Vec2,
};
use egui_extras::{Column, TableBuilder};
use irox_egui_extras::progressbar::ProgressBar;
pub fn lay_table(interface: &mut MyApp, ui: &mut Ui, ctx: &Context) {
    let HEADING_COLOR: Color32 = Color32::from_hex("#a4b9ef").expect("Bad Hex");
    let PB_COLOR = Color32::from_hex("#a4b9f0").expect("Bad Hex");
    ctx.request_repaint();
    let available_width = ui.available_width();
    TableBuilder::new(ui)
        .auto_shrink(false)
        .striped(true)
        .column(Column::exact(available_width * 0.02))
        .column(Column::initial(available_width * 0.1))
        .column(Column::initial(available_width * 0.455))
        .column(Column::exact(available_width * 0.2))
        .column(Column::initial(available_width * 0.225))
        .header(20.0, |mut header| {
            header.col(|ui| {
                ui.heading("");
            });
            header.col(|ui| {
                let text = RichText::new("Filename").color(HEADING_COLOR).strong();
                ui.heading(text);
                ui.add(Separator::grow(
                    Separator::default(),
                    ctx.screen_rect().width(),
                ));
            });
            header.col(|ui| {
                let text = RichText::new("Progress").color(HEADING_COLOR).strong();
                ui.vertical_centered(|ui| {
                    ui.heading(text);
                });
            });
            header.col(|ui| {
                let text = RichText::new("Action on save")
                    .color(HEADING_COLOR)
                    .strong();
                ui.vertical_centered(|ui| {
                    ui.heading(text);
                });
            });
            header.col(|ui| {
                let text = RichText::new("Toggle").color(HEADING_COLOR).strong();
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
                })
                .map(|f| f.to_owned())
                .collect::<Vec<_>>();
            for fdl in to_display.iter_mut() {
                let file = &fdl.file;
                let complete = file.complete.load(std::sync::atomic::Ordering::Relaxed);

                body.row(30.0, |mut row| {
                    row.col(|ui| {
                        ui.add(Checkbox::without_text(&mut fdl.selected));
                        let file = interface
                            .files
                            .iter_mut()
                            .find(|f| f.file.name_on_disk == fdl.file.name_on_disk);
                        if let Some(f) = file {
                            f.selected = fdl.selected;
                        }
                    });
                    row.col(|ui| {
                        file_name(file, ui);
                    });
                    row.col(|ui| progress_bar(file, PB_COLOR, ui, ctx, available_width * 0.35));
                    row.col(|ui| {
                        match fdl.action_on_save {
                            Actions::Reboot if complete => {
                                let _ = Command::new("reboot").exec();
                            }
                            Actions::Shutdown if complete => {
                                let _ = Command::new("reboot").exec();
                            }
                            _ => {}
                        }
                        ui.vertical_centered(|ui| {
                            let inner_color = Color32::from_hex("#1e1e28").expect("Bad Hex");
                            ui.visuals_mut().widgets.inactive.weak_bg_fill = PB_COLOR;
                            ui.visuals_mut().widgets.open.weak_bg_fill = PB_COLOR;
                            ui.visuals_mut().widgets.hovered.weak_bg_fill = PB_COLOR;
                            ui.visuals_mut().widgets.active.weak_bg_fill = PB_COLOR;
                            ui.visuals_mut().widgets.inactive.fg_stroke.color = inner_color;
                            ui.visuals_mut().widgets.open.fg_stroke.color = inner_color;
                            ui.visuals_mut().widgets.hovered.fg_stroke.color = inner_color;
                            ui.visuals_mut().widgets.active.fg_stroke.color = inner_color;
                            ui.visuals_mut().override_text_color = Some(inner_color);
                            if !complete {
                                ui.centered_and_justified(|ui| {
                                    egui::ComboBox::from_label("")
                                        .selected_text(format!("{:?}", fdl.action_on_save))
                                        .width(available_width * 0.20)
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(
                                                &mut fdl.action_on_save,
                                                Actions::None,
                                                "None",
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
                                    egui::ComboBox::from_label("")
                                        .width(available_width * 0.20)
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
                        action_button(PB_COLOR, file, ui, complete);
                    });
                });
            }
        });
}

fn action_button(color: Color32, file: &File2Dl, ui: &mut Ui, complete: bool) {
    let text = {
        let running = file.running.load(std::sync::atomic::Ordering::Relaxed);
        if !running {
            eframe::egui::RichText::new(egui_phosphor::fill::PLAY).size(20.0)
        } else {
            eframe::egui::RichText::new(egui_phosphor::fill::PAUSE).size(20.0)
        }
    };
    let but = {
        if !complete {
            let color = Color32::from_hex("#0065b1").expect("Bad Hex");
            Button::new(text.color(color)).frame(false)
        } else {
            Button::new(text).frame(false)
        }
    };
    ui.vertical_centered(|ui| {
        let res = ui.add(but);
        if res.hovered() && !complete {
            ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand)
        }
        if res.clicked() && !complete {
            file.switch_status();
        }
        if res.hovered() && !file.url.range_support {
            let label = RichText::new("File doesn't support resumption").color(color);
            res.show_tooltip_text(label);
        }
        if res.hovered() && complete {
            let label = RichText::new("File is complete").color(color);
            res.show_tooltip_text(label);
        }
    });
}
fn progress_bar(file: &File2Dl, color: Color32, ui: &mut Ui, ctx: &Context, fixed_size: f32) {
    let size = file.size_on_disk.load(std::sync::atomic::Ordering::Relaxed) as f32;
    let total_size = file.url.content_length as f32;
    let percentage = size / total_size;
    ui.centered_and_justified(|ui| {
        ui.scope(|ui| {
            ui.visuals_mut().extreme_bg_color = Color32::from_hex("#3c3c3c").expect("Bad Hex");
            ui.visuals_mut().selection.bg_fill = Color32::from_hex("#a4b9ef").expect("Bad Hex");
            ui.visuals_mut().override_text_color =
                Some(Color32::from_hex("#1e1e28").expect("Bad Hex"));
            let mut pb = ProgressBar::new(percentage)
                .desired_width(fixed_size)
                .desired_height(ui.available_height())
                .text_center(format!("{}%", (percentage * 100.0) as i32));
            pb.is_indeterminate = file.running.load(std::sync::atomic::Ordering::Relaxed);
            let res = ui.add(pb);
            if res.hovered() {
                let size_mbs = size / (1024.0 * 1024.0);
                let total_size_mbs = file.url.content_length as f32 / (1024.0 * 1024.0);
                let text = RichText::new(format!("{:.3}/{:.3} Mbs", size_mbs, total_size_mbs))
                    .color(color);
                res.show_tooltip_text(text);
            };
            ctx.request_repaint_of(res.ctx.viewport_id());
        });
    });
}

fn file_name(file: &File2Dl, ui: &mut Ui) {
    let text = RichText::new(&file.name_on_disk).strong().size(15.0);
    let label = Label::new(text.clone()).wrap_mode(egui::TextWrapMode::Truncate);
    ui.add(label);
}
