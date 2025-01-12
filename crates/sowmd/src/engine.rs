use std::{
    path::{Path, PathBuf},
    sync::mpsc::Receiver,
    time::Duration,
};

use sowm_common::ClientMessage;
use walkdir::WalkDir;

// Example of using feh to set the background for two monitors
// feh --no-fehbg --bg-fill girl_holding_power.jpg girl_face_in_pool.jpeg

pub fn run(rx: Receiver<ClientMessage>) -> ! {
    let dur = Duration::from_secs(60 * 1);
    let dir = PathBuf::from("/home/spencer/pictures/wallpapers");
    let images = get_images(&dir);

    loop {
        std::thread::sleep(dur);
    }
}

fn get_images<P>(dir: P) -> Vec<PathBuf>
where
    P: AsRef<Path>,
{
    let valid_extensions = ["jpeg", "jpg", "png"];
    let mut images = Vec::new();

    for file in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if let Some(ext) = file.path().extension() {
            if valid_extensions.contains(&ext.to_ascii_lowercase().to_str().unwrap()) {
                images.push(file.path().to_owned());
            }
        }
    }
    images
}
