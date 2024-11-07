use chrono::Local;
use notify_rust::Notification;

use crate::{
    colors::{GREEN, RED},
    server::interception::SERVER_STATE,
    DownloadManager,
};
use std::{
    thread::sleep,
    time::{Duration, Instant},
};

trait ConsumingIterator<T> {
    fn next(&mut self) -> Option<T>;
}

impl<T> ConsumingIterator<T> for Vec<T> {
    fn next(&mut self) -> Option<T> {
        if !self.is_empty() {
            Some(self.remove(0))
        } else {
            None
        }
    }
}

pub fn check_urls(interface: &mut DownloadManager) {
    if let Ok(mut locked) = SERVER_STATE.try_lock() {
        let mut links = locked.clone();

        if !interface.popups.download.show {
            if let Some(link) = links.next() {
                interface.popups.download.link = link.clone();
                let now = Local::now();
                let formatted_time = now.format("%H:%M:%S").to_string();
                let text = format!("Received link from server:{}", &link);
                interface
                    .popups
                    .log
                    .logs
                    .push((formatted_time, text, *GREEN));
                interface.popups.download.show = true;
                interface.show_window = true;

                *locked = links;
            }
        }
    }
}

pub fn set_total_bandwidth(interface: &mut DownloadManager) {
    let size: usize = interface
        .files
        .iter()
        .map(|f| {
            f.file
                .bytes_per_sec
                .load(std::sync::atomic::Ordering::Relaxed)
        })
        .sum();
    interface.bandwidth.total_bandwidth = size;
    update_bandwidth_history(interface);
}
fn update_bandwidth_history(interface: &mut DownloadManager) {
    interface
        .bandwidth
        .history
        .push(interface.bandwidth.total_bandwidth);
    if interface.bandwidth.history.len() > 100 {
        interface.bandwidth.history.remove(0);
    }
}
pub fn run_downloads(interface: &mut DownloadManager) {
    let retry_interval = interface.settings.retry_interval;
    let now = Local::now();
    let formatted_time = now.format("%H:%M:%S").to_string();
    for fdl in interface.files.iter_mut() {
        let file = &fdl.file;
        let complete = file.complete.load(std::sync::atomic::Ordering::Relaxed);
        let new = fdl.new;

        if complete && !fdl.initial_status && !fdl.got_notif {
            if interface.show_window {
                fdl.got_notif = true;
            } else {
                let text = format!("{} finished downloading", &file.name_on_disk);
                fdl.got_notif = true;
                let noti = Notification::new()
                    .summary("Download complete")
                    .body(&text)
                    .icon("/home/numerouscuts/Coding/final-dl-manager/icon.png")
                    .show();
                if let Err(e) = noti {
                    let text = format!("Notification error: {:?}", e);
                    interface
                        .popups
                        .log
                        .logs
                        .push((formatted_time.clone(), text, *RED));
                }
            }
        }

        if complete || fdl.initiated {
            continue;
        }

        let is_running = file.running.load(std::sync::atomic::Ordering::Relaxed);
        let speed = fdl
            .file
            .bytes_per_sec
            .load(std::sync::atomic::Ordering::Relaxed);

        fdl.has_error = if complete {
            false
        } else if is_running {
            speed == 0 && fdl.toggled_at.elapsed() >= Duration::from_secs(5)
        } else {
            fdl.toggled_at = Instant::now();
            false
        };

        let file = file.clone();
        let tx_error = interface.popups.error.channel.0.clone();
        let log_msg = format!("Initiating : {}", &file.url.link);
        interface
            .popups
            .log
            .logs
            .push((formatted_time.clone(), log_msg, *GREEN));
        interface.runtime.spawn(async move {
            if file.url.range_support {
                loop {
                    match file.single_thread_dl().await {
                        Ok(_) => break,
                        Err(e) => {
                            let error = format!("{}: {:?}\n", file.name_on_disk, e);
                            tx_error.send(error).unwrap();
                        }
                    }
                    sleep(Duration::from_secs(retry_interval));
                }
            } else if new {
                match file.single_thread_dl().await {
                    Ok(_) => {}
                    Err(e) => {
                        let error = format!("{}: {:?}\n", file.name_on_disk, e);
                        tx_error.send(error).unwrap();
                    }
                }
            }
        });

        fdl.initiated = true;
    }
    if let Ok(err) = interface.popups.error.channel.1.try_recv() {
        interface
            .popups
            .log
            .logs
            .push((formatted_time.clone(), err, *RED));
        interface.popups.log.has_error = true;
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub enum Actions {
    #[default]
    None,
    Reboot,
    Shutdown,
    Open,
}
