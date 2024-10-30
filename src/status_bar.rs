use crate::{colors::*, MyApp};
use eframe::egui::{self, Button, CursorIcon, Label, Layout, RichText, Ui};
pub fn init_status_bar(interface: &mut MyApp, ui: &mut Ui) {
    ui.with_layout(Layout::right_to_left(egui::Align::RIGHT), |ui| {
        ui.add_space(10.0);
        ui.horizontal_centered(|ui| {
            let text = {
                if interface.connection.connected {
                    eframe::egui::RichText::new(format!("Connected {}", egui_phosphor::fill::GLOBE))
                        .size(17.0)
                        .color(*GREEN)
                } else {
                    eframe::egui::RichText::new(format!(
                        "Disconnected {}",
                        egui_phosphor::fill::GLOBE_X
                    ))
                    .size(17.0)
                    .color(*RED)
                }
            };
            let label = Label::new(text).wrap_mode(egui::TextWrapMode::Truncate);
            ui.add(label);
        });
        ui.separator();
        ui.add_space(ui.available_width() - 80.0);
        ui.horizontal_centered(|ui| {
            {
                let text = eframe::egui::RichText::new(egui_phosphor::fill::NETWORK)
                    .size(25.0)
                    .color(*DARK_INNER);
                let butt = Button::new(text).fill(*CYAN).rounding(25.0);
                let res = ui.add(butt);
                if res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                    let text =
                        RichText::new("Modify download speed for selected files").color(*CYAN);
                    res.show_tooltip_text(text);
                }
                if res.clicked() {
                    interface.popups.speed.show = true;
                }
            }
            let text = eframe::egui::RichText::new(egui_phosphor::fill::CHART_LINE_UP)
                .size(25.0)
                .color(*DARK_INNER);
            let butt = Button::new(text).fill(*CYAN).rounding(25.0);
            let res = ui.add(butt);
            if res.hovered() {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                let text = RichText::new("Plot of total downlaod speed used").color(*CYAN);
                res.show_tooltip_text(text);
            }
            if res.clicked() {
                interface.popups.plot.show = true;
            }
        });
    });
}
