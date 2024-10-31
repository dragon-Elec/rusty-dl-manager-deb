use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

use crate::MyApp;

pub fn check_urls(interface: &mut MyApp) {
    let tx = interface.urls.0.clone();
    interface.runtime.spawn(async move {
        if Path::new("urls.txt").exists() {
            let mut file = File::open("urls.txt").expect("Couldn't open file");
            let mut buffer = String::default();
            file.read_to_string(&mut buffer)
                .expect("Couldn't read file");

            if !buffer.is_empty() {
                tx.send(buffer).expect("Couldn't send");
            }
        }
    });
    if let Ok(val) = interface.urls.1.try_recv() {
        let mut lines = val.lines();
        if !interface.popups.download.show {
            if let Some(line) = lines.next() {
                interface.popups.download.link = line.to_string();
                interface.popups.download.show = true;
                let remaining_urls: Vec<&str> = lines.collect();
                let mut file = File::create("urls.txt").expect("Couldn't create");
                for url in remaining_urls {
                    file.write_all(url.as_bytes()).expect("Couldn't write");
                }
            }
        }
    }
}
pub fn set_total_bandwidth(interface: &mut MyApp) {
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
fn update_bandwidth_history(interface: &mut MyApp) {
    interface
        .bandwidth
        .history
        .push(interface.bandwidth.total_bandwidth);
    if interface.bandwidth.history.len() > 100 {
        interface.bandwidth.history.remove(0);
    }
}
pub fn run_downloads(interface: &mut MyApp) {
    for fdl in interface.files.iter_mut() {
        let file = &fdl.file;
        let complete = file.complete.load(std::sync::atomic::Ordering::Relaxed);
        let new = fdl.new;
        if !complete && !&fdl.initiated {
            let file = file.clone();
            let tx_error = interface.popups.error.channel.0.clone();
            interface.runtime.spawn(async move {
                if file.url.range_support {
                    loop {
                        match file.single_thread_dl().await {
                            Ok(_) => break,
                            Err(e) => {
                                let error = format!("{}: {:?}", file.name_on_disk, e);
                                tx_error.send(error).unwrap();
                            }
                        }
                        sleep(Duration::from_secs(5));
                    }
                } else if new {
                    match file.single_thread_dl().await {
                        Ok(_) => {}
                        Err(e) => {
                            let error = format!("{}: {:?}", file.name_on_disk, e);
                            tx_error.send(error).unwrap();
                        }
                    }
                }
            });

            fdl.initiated = true;
        }
    }
    if let Ok(err) = interface.popups.error.channel.1.try_recv() {
        interface.popups.error.value = err;
        interface.popups.error.show = true;
    }
}
