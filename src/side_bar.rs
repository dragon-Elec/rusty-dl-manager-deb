use crate::colors::*;
use crate::DownloadManager;
use egui_phosphor::fill::*;
use egui_sfml::egui::Ui;
use egui_sfml::egui::*;

#[derive(Default)]
pub struct Explorer {
    pub current: Vec<String>,
    types: Types,
}

impl Explorer {
    pub fn toggle_off(&mut self) {
        self.types.all.clicked = false;
        self.types.binaries.clicked = false;
        self.types.archives.clicked = false;
        self.types.audio.clicked = false;
        self.types.books.clicked = false;
        self.types.fonts.clicked = false;
        self.types.images.clicked = false;
        self.types.sheets.clicked = false;
        self.types.slides.clicked = false;
        self.types.texts.clicked = false;
        self.types.videos.clicked = false;
    }
}

pub struct Types {
    all: Inner,
    binaries: Inner,
    archives: Inner,
    audio: Inner,
    books: Inner,
    fonts: Inner,
    images: Inner,
    sheets: Inner,
    slides: Inner,
    texts: Inner,
    videos: Inner,
}
pub struct Inner {
    clicked: bool,
    exts: Vec<String>,
}
impl Default for Inner {
    fn default() -> Self {
        Self {
            clicked: true,
            exts: vec![],
        }
    }
}

impl Default for Types {
    fn default() -> Self {
        let all = {
            let exts = vec![];
            Inner {
                clicked: true,
                exts,
            }
        };
        let binaries = {
            let exts = vec![
                "exe", "msi", "bin", "command", "sh", "bat", "crx", "bash", "csh", "fish", "ksh",
                "zsh",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>();
            Inner {
                clicked: false,
                exts,
            }
        };
        let archives = {
            let exts = vec![
                "7z", "a", "aar", "apk", "ar", "bz2", "br", "cab", "cpio", "deb", "dmg", "egg",
                "gz", "iso", "jar", "lha", "lz", "lz4", "lzma", "lzo", "mar", "pea", "rar", "rpm",
                "s7z", "shar", "tar", "tbz2", "tgz", "tlz", "txz", "war", "whl", "xpi", "zip",
                "zipx", "zst", "xz", "pak",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>();
            Inner {
                clicked: false,
                exts,
            }
        };
        let audio = {
            let exts = vec![
                "aac", "aiff", "ape", "au", "flac", "gsm", "it", "m3u", "m4a", "mid", "mod", "mp3",
                "mpa", "ogg", "pls", "ra", "s3m", "sid", "wav", "wma", "xm",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>();
            Inner {
                clicked: false,
                exts,
            }
        };
        let books = {
            let exts = vec![
                "mobi", "epub", "azw1", "azw3", "azw4", "azw6", "azw", "cbr", "cbz",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>();
            Inner {
                clicked: false,
                exts,
            }
        };
        let fonts = {
            let exts = vec!["eot", "otf", "ttf", "woff", "woff2"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>();
            Inner {
                clicked: false,
                exts,
            }
        };
        let images = {
            let exts = vec![
                "3dm", "3ds", "max", "avif", "bmp", "dds", "gif", "heic", "heif", "jpg", "jpeg",
                "jxl", "png", "psd", "xcf", "tga", "thm", "tif", "tiff", "yuv", "ai", "eps", "ps",
                "svg", "dwg", "dxf", "gpx", "kml", "kmz", "webp",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>();
            Inner {
                clicked: false,
                exts,
            }
        };
        let sheets = {
            let exts = vec!["ods", "xls", "xlsx", "csv", "tsv", "ics", "vcf"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>();
            Inner {
                clicked: false,
                exts,
            }
        };
        let slides = {
            let exts = vec!["ppt", "pptx", "odp"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>();
            Inner {
                clicked: false,
                exts,
            }
        };
        let texts = {
            let exts = vec![
                "doc", "docx", "ebook", "log", "md", "msg", "odt", "org", "pages", "pdf", "rtf",
                "rst", "tex", "txt", "wpd", "wps",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>();
            Inner {
                clicked: false,
                exts,
            }
        };
        let videos = {
            let exts = vec![
                "3g2", "3gp", "aaf", "asf", "avchd", "avi", "car", "dav", "drc", "flv", "m2v",
                "m2ts", "m4p", "m4v", "mkv", "mng", "mov", "mp2", "mp4", "mpe", "mpeg", "mpg",
                "mpv", "mts", "mxf", "nsv", "ogv", "ogm", "ogx", "qt", "rm", "rmvb", "roq", "srt",
                "svi", "vob", "webm", "wmv", "xba", "yuv",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<String>>();
            Inner {
                clicked: false,
                exts,
            }
        };
        Self {
            all,
            binaries,
            archives,
            audio,
            books,
            fonts,
            images,
            sheets,
            slides,
            texts,
            videos,
        }
    }
}
pub fn lay_side_bar_content(interface: &mut DownloadManager, ui: &mut Ui) {
    let text = if interface.explorer.types.all.clicked {
        let text = format!("{}{}All", CARET_RIGHT, FOLDER);
        RichText::new(text).color(*GREEN).size(15.0)
    } else {
        let text = format!("{}All", FOLDER);
        RichText::new(text).color(*CYAN).size(15.0)
    };
    ui.horizontal(|ui| {
        ui.add_space(5.0);
        let butt = Button::new(text).frame(false);
        let res = ui.add(butt);
        if res.hovered() {
            ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
        }
        if res.clicked() {
            interface.explorer.toggle_off();
            interface.explorer.types.all.clicked = true;
            interface.explorer.current = vec![];
        }
    });
    ui.vertical(|ui| {
        ui.add_space(5.0);
        {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                let text = if interface.explorer.types.binaries.clicked {
                    let text = format!("{}{} Binaries", CARET_RIGHT, BINARY);
                    RichText::new(text).color(*GREEN).size(15.0)
                } else {
                    let text = format!("{} Binaries", BINARY);
                    RichText::new(text).color(*CYAN).size(15.0)
                };

                let butt = Button::new(text).frame(false);
                let res = ui.add(butt);
                if res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                }
                if res.clicked() {
                    interface.explorer.toggle_off();
                    interface.explorer.types.binaries.clicked = true;
                    interface.explorer.current = interface.explorer.types.binaries.exts.clone()
                }
                ui.add_space(5.0);
            });
        }
        ui.add_space(10.0);
        {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                let text = if interface.explorer.types.archives.clicked {
                    let text = format!("{}{} Archives", CARET_RIGHT, ARCHIVE);
                    RichText::new(text).color(*GREEN).size(15.0)
                } else {
                    let text = format!("{} Archives", ARCHIVE);
                    RichText::new(text).color(*CYAN).size(15.0)
                };

                let butt = Button::new(text).frame(false);
                let res = ui.add(butt);
                if res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                }
                if res.clicked() {
                    interface.explorer.toggle_off();
                    interface.explorer.types.archives.clicked = true;
                    interface.explorer.current = interface.explorer.types.archives.exts.clone()
                }
                ui.add_space(5.0);
            });
        }
        ui.add_space(10.0);
        {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                let text = if interface.explorer.types.audio.clicked {
                    let text = format!("{}{} Audio", CARET_RIGHT, MUSIC_NOTE);
                    RichText::new(text).color(*GREEN).size(15.0)
                } else {
                    let text = format!("{} Audio", MUSIC_NOTE);
                    RichText::new(text).color(*CYAN).size(15.0)
                };

                let butt = Button::new(text).frame(false);
                let res = ui.add(butt);
                if res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                }
                if res.clicked() {
                    interface.explorer.toggle_off();
                    interface.explorer.types.audio.clicked = true;
                    interface.explorer.current = interface.explorer.types.audio.exts.clone()
                }
                ui.add_space(5.0);
            });
        }
        ui.add_space(10.0);
        {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                let text = if interface.explorer.types.books.clicked {
                    let text = format!("{}{} Books", CARET_RIGHT, BOOK_BOOKMARK);
                    RichText::new(text).color(*GREEN).size(15.0)
                } else {
                    let text = format!("{} Books", BOOK_BOOKMARK);
                    RichText::new(text).color(*CYAN).size(15.0)
                };

                let butt = Button::new(text).frame(false);
                let res = ui.add(butt);
                if res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                }
                if res.clicked() {
                    interface.explorer.toggle_off();
                    interface.explorer.types.books.clicked = true;
                    interface.explorer.current = interface.explorer.types.books.exts.clone()
                }
                ui.add_space(5.0);
            });
        }

        ui.add_space(10.0);
        {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                let text = if interface.explorer.types.fonts.clicked {
                    let text = format!("{}{} Fonts", CARET_RIGHT, TEXT_AA);
                    RichText::new(text).color(*GREEN).size(15.0)
                } else {
                    let text = format!("{} Fonts", TEXT_AA);
                    RichText::new(text).color(*CYAN).size(15.0)
                };

                let butt = Button::new(text).frame(false);
                let res = ui.add(butt);
                if res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                }
                if res.clicked() {
                    interface.explorer.toggle_off();
                    interface.explorer.types.fonts.clicked = true;
                    interface.explorer.current = interface.explorer.types.fonts.exts.clone();
                }
                ui.add_space(5.0);
            });
        }
        ui.add_space(10.0);
        {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                let text = if interface.explorer.types.images.clicked {
                    let text = format!("{}{} Images", CARET_RIGHT, IMAGE);
                    RichText::new(text).color(*GREEN).size(15.0)
                } else {
                    let text = format!("{} Images", IMAGE);
                    RichText::new(text).color(*CYAN).size(15.0)
                };

                let butt = Button::new(text).frame(false);
                let res = ui.add(butt);
                if res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                }
                if res.clicked() {
                    interface.explorer.toggle_off();
                    interface.explorer.types.images.clicked = true;
                    interface.explorer.current = interface.explorer.types.images.exts.clone();
                }
                ui.add_space(5.0);
            });
        }
        ui.add_space(10.0);
        {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                let text = if interface.explorer.types.sheets.clicked {
                    let text = format!("{}{} Sheets", CARET_RIGHT, NOTE);
                    RichText::new(text).color(*GREEN).size(15.0)
                } else {
                    let text = format!("{} Sheets", NOTE);
                    RichText::new(text).color(*CYAN).size(15.0)
                };

                let butt = Button::new(text).frame(false);
                let res = ui.add(butt);
                if res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                }
                if res.clicked() {
                    interface.explorer.toggle_off();
                    interface.explorer.types.sheets.clicked = true;
                    interface.explorer.current = interface.explorer.types.sheets.exts.clone();
                }
                ui.add_space(5.0);
            });
        }
        ui.add_space(10.0);
        {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                let text = if interface.explorer.types.slides.clicked {
                    let text = format!("{}{} Slides", CARET_RIGHT, SLIDESHOW);
                    RichText::new(text).color(*GREEN).size(15.0)
                } else {
                    let text = format!("{} Slides", SLIDESHOW);
                    RichText::new(text).color(*CYAN).size(15.0)
                };

                let butt = Button::new(text).frame(false);
                let res = ui.add(butt);
                if res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                }
                if res.clicked() {
                    interface.explorer.toggle_off();
                    interface.explorer.types.slides.clicked = true;
                    interface.explorer.current = interface.explorer.types.slides.exts.clone()
                }
                ui.add_space(5.0);
            });
        }
        ui.add_space(10.0);
        {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                let text = if interface.explorer.types.texts.clicked {
                    let text = format!("{}{} Texts", CARET_RIGHT, TEXT_T);
                    RichText::new(text).color(*GREEN).size(15.0)
                } else {
                    let text = format!("{} Texts", TEXT_T);
                    RichText::new(text).color(*CYAN).size(15.0)
                };

                let butt = Button::new(text).frame(false);
                let res = ui.add(butt);
                if res.hovered() {
                    ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                }
                if res.clicked() {
                    interface.explorer.toggle_off();
                    interface.explorer.types.texts.clicked = true;
                    interface.explorer.current = interface.explorer.types.texts.exts.clone();
                }
                ui.add_space(5.0);
            });
        }
        ui.add_space(10.0);
        {
            ui.horizontal(|ui| {
                ui.add_space(20.0);
                let text = if interface.explorer.types.videos.clicked {
                    let text = format!("{}{} Videos", CARET_RIGHT, VIDEO);
                    RichText::new(text).color(*GREEN).size(15.0)
                } else {
                    let text = format!("{} Videos", VIDEO);
                    RichText::new(text).color(*CYAN).size(15.0)
                };

                let butt = Button::new(text).frame(false);
                let res = ui.add(butt);
                if res.hovered() {
                    ui.output_mut(|o: &mut egui_sfml::egui::PlatformOutput| {
                        o.cursor_icon = CursorIcon::PointingHand
                    });
                }
                if res.clicked() {
                    interface.explorer.toggle_off();
                    interface.explorer.types.videos.clicked = true;
                    interface.explorer.current = interface.explorer.types.videos.exts.clone();
                }
                ui.add_space(5.0);
            });
        }
    });
}
