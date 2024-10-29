use eframe::egui::Color32;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CYAN: Color32 = Color32::from_hex("#a4b9ef").expect("Bad Hex");
    pub static ref PURPLE: Color32 = Color32::from_hex("#1b1824").expect("Bad Hex");
    pub static ref DARKER_PURPLE: Color32 = Color32::from_hex("#111017").expect("Bad Hex");
    pub static ref DARK_INNER: Color32 = Color32::from_hex("#1e1e28").expect("Bad Hex");
    pub static ref GRAY: Color32 = Color32::from_hex("#808080").expect("Bad Hex");
    pub static ref GREEN: Color32 = Color32::from_hex("#b1e4ac").expect("Bad Hex");
    pub static ref RED: Color32 = Color32::from_hex("#dc8d8a").expect("Bad Hex");
}
