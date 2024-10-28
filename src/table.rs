use std::{os::unix::process::CommandExt, process::Command};

use crate::{dl::file2dl::File2Dl, Actions, MyApp};
use eframe::egui::{
    self, Button, Checkbox, Color32, Context, CursorIcon, Label, RichText, Separator, Ui,
};
use egui_extras::{Column, TableBuilder};
use irox_egui_extras::progressbar::ProgressBar;
use tokio::runtime::Runtime;
pub fn lay_table(interface: &mut MyApp, ui: &mut Ui, ctx: &Context) {
    let HEADING_COLOR: Color32 = Color32::from_hex("#e28c8f").expect("Bad Hex");
    let PB_COLOR = Color32::from_hex("#a4b9f0").expect("Bad Hex");
    TableBuilder::new(ui)
        .auto_shrink(false)
        .striped(true)
        .column(Column::auto().at_most(20.0))
        .column(Column::auto().at_least(130.0))
        .column(Column::auto().at_least(270.0))
        .column(Column::remainder().at_most(150.0))
        .column(Column::remainder().at_least(120.0))
        .header(30.0, |mut header| {
            header.col(|ui| {
                ui.heading("");
                ui.add(Separator::grow(Separator::default(), ui.available_width()));
            });
            header.col(|ui| {
                let text = RichText::new("Filename")
                    .color(HEADING_COLOR)
                    .strong()
                    .italics();
                ui.heading(text);
                ui.add(Separator::grow(Separator::default(), ui.available_width()));
            });
            header.col(|ui| {
                let text = RichText::new("Progress")
                    .color(HEADING_COLOR)
                    .strong()
                    .italics();
                ui.heading(text);
                ui.add(Separator::grow(Separator::default(), ui.available_width()));
            });
            header.col(|ui| {
                let text = RichText::new("Action on save")
                    .color(HEADING_COLOR)
                    .strong()
                    .italics();
                ui.heading(text);
                ui.add(Separator::grow(Separator::default(), ui.available_width()));
            });
            header.col(|ui| {
                let text = RichText::new("Toggle")
                    .color(HEADING_COLOR)
                    .strong()
                    .italics();
                ui.heading(text);
                ui.add(Separator::grow(Separator::default(), ui.available_width()));
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
                    });
                    row.col(|ui| {
                        file_name(file, ui);
                    });
                    row.col(|ui| progress_bar(file, PB_COLOR, ui, ctx));
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
                        if !complete {
                            egui::ComboBox::from_label("")
                                .selected_text(format!("{:?}", fdl.action_on_save))
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
                        } else {
                            egui::ComboBox::from_label("")
                                .selected_text(format!("{:?}", fdl.action_on_save))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut fdl.action_on_save,
                                        Actions::None,
                                        "None",
                                    );
                                });
                        }
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
}

fn progress_bar(file: &File2Dl, color: Color32, ui: &mut Ui, ctx: &Context) {
    let size = file.size_on_disk.load(std::sync::atomic::Ordering::Relaxed) as f32;
    let total_size = file.url.content_length as f32;
    let percentage = size / total_size;
    ui.scope(|ui| {
        ui.visuals_mut().extreme_bg_color = Color32::from_hex("#3c3c3c").expect("Bad Hex");
        let mut pb = ProgressBar::new(percentage)
            .desired_width(250.0)
            .text_center(format!("{}%", (percentage * 100.0) as i32));
        pb.is_indeterminate = file.running.load(std::sync::atomic::Ordering::Relaxed);
        let res = ui.add(pb);
        if res.hovered() {
            let size_mbs = size / (1024.0 * 1024.0);
            let total_size_mbs = file.url.content_length as f32 / (1024.0 * 1024.0);
            let text =
                RichText::new(format!("{:.3}/{:.3} Mbs", size_mbs, total_size_mbs)).color(color);
            res.show_tooltip_text(text);
        };
        ctx.request_repaint_of(res.ctx.viewport_id());
    });
}

fn file_name(file: &File2Dl, ui: &mut Ui) {
    let text = RichText::new(&file.name_on_disk).strong().size(15.0);
    let label = Label::new(text.clone()).wrap_mode(egui::TextWrapMode::Truncate);
    ui.add(label);
}
