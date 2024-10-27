use crate::{dl::file2dl::File2Dl, MyApp};
use eframe::egui::{
    Button, Checkbox, Color32, Context, CursorIcon, Label, RichText, Separator, Ui,
};
use egui_extras::{Column, TableBuilder};
use irox_egui_extras::progressbar::ProgressBar;
use tokio::runtime::Runtime;
pub fn lay_table(interface: &mut MyApp, ui: &mut Ui, ctx: &Context) {
    let HEADING_COLOR: Color32 = Color32::from_hex("#e28c8f").expect("Bad Hex");
    let PB_COLOR = Color32::from_hex("#a4b9f0").expect("Bad Hex");
    TableBuilder::new(ui)
        .auto_shrink(false)
        .column(Column::auto().at_least(20.0))
        .column(Column::auto().at_least(130.0))
        .column(Column::auto().at_least(300.0))
        .column(Column::remainder())
        .header(30.0, |mut header| {
            header.col(|ui| {
                ui.heading("");
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
                let text = RichText::new("Toggle")
                    .color(HEADING_COLOR)
                    .strong()
                    .italics();
                ui.heading(text);
                ui.add(Separator::grow(Separator::default(), ui.available_width()));
            });
        })
        .body(|mut body| {
            for fdl in interface.files.iter_mut() {
                let file = &fdl.file;
                let complete = file.complete.load(std::sync::atomic::Ordering::Relaxed);
                if !complete && !fdl.initiated {
                    let rt = Runtime::new().unwrap();
                    let file = file.clone();
                    let tx_error = interface.popups.error.channel.0.clone();
                    std::thread::spawn(move || {
                        rt.block_on(async move {
                            loop {
                                match file.single_thread_dl().await {
                                    Ok(_) => break,
                                    Err(e) => {
                                        tx_error.send(e.to_string()).unwrap();
                                    }
                                }
                            }
                        })
                    });
                    let rx = &interface.popups.error.channel.1;
                    if let Ok(val) = rx.try_recv() {
                        interface.popups.error.value = val;
                        interface.popups.error.show = true;
                    }
                    fdl.initiated = true;
                }
                body.row(30.0, |mut row| {
                    row.col(|ui| {
                        ui.add(Checkbox::without_text(&mut fdl.selected));
                    });
                    row.col(|ui| {
                        file_name(file, ui);
                    });
                    row.col(|ui| progress_bar(file, PB_COLOR, ui, ctx));
                    row.col(|ui| {
                        action_button((HEADING_COLOR, PB_COLOR), file, ui, complete);
                    });
                });
            }
        });
}

fn action_button(colors: (Color32, Color32), file: &File2Dl, ui: &mut Ui, complete: bool) {
    let text = {
        let running = file.running.load(std::sync::atomic::Ordering::Relaxed);
        if !running {
            eframe::egui::RichText::new(egui_phosphor::regular::PLAY).size(20.0)
        } else {
            eframe::egui::RichText::new(egui_phosphor::regular::PAUSE).size(25.0)
        }
    };
    let but = {
        if !complete && file.url.range_support {
            Button::new(text.color(colors.0)).frame(false)
        } else {
            Button::new(text).frame(false)
        }
    };
    let res = ui.add(but);

    if res.hovered() && !complete && file.url.range_support {
        ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand)
    }
    if res.clicked() && !complete {
        file.switch_status();
    }
    if res.hovered() && !file.url.range_support {
        let label = RichText::new("File doesn't support resumption").color(colors.1);
        res.show_tooltip_text(label);
    }
    if res.hovered() && complete {
        let label = RichText::new("File is complete").color(colors.1);
        res.show_tooltip_text(label);
    }
}

fn progress_bar(file: &File2Dl, color: Color32, ui: &mut Ui, ctx: &Context) {
    let size = file.size_on_disk.load(std::sync::atomic::Ordering::Relaxed) as f32;
    let total_size = file.url.content_length as f32;
    let percentage = size / total_size;
    let mut pb = ProgressBar::new(percentage)
        .desired_width(250.0)
        .text_center(format!("{}%", (percentage * 100.0) as i32));
    pb.animate = true;
    let res = ui.add(pb);
    if res.hovered() {
        let text =
            RichText::new((format!("{}%", (percentage * 100.0) as i32)).to_string()).color(color);

        res.show_tooltip_text(text);
    };
    ctx.request_repaint_of(res.ctx.viewport_id());
}

fn file_name(file: &File2Dl, ui: &mut Ui) {
    let text = RichText::new(&file.name_on_disk).strong().size(15.0);
    let label = Label::new(text.clone()).wrap();
    let res = ui.add(label);
    if res.hovered() {
        res.show_tooltip_text(text);
    }
}
