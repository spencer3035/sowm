use std::{
    path::{Path, PathBuf},
    sync::{mpsc::Receiver, Arc},
    time::Duration,
};

use rand::{seq::SliceRandom, thread_rng};
use sowm_common::{ClientMessage, Init};
use walkdir::WalkDir;

// Example of using feh to set the background for two monitors
// feh --no-fehbg --bg-fill girl_holding_power.jpg girl_face_in_pool.jpeg

pub fn run(_rx: Receiver<ClientMessage>, _init: Arc<Init>) -> ! {
    let dur = Duration::from_secs(60 * 30);
    let dir = PathBuf::from("/home/spencer/pictures/wallpapers");
    let mut images = get_images(&dir);
    images.shuffle(&mut thread_rng());

    let mut image_iter = images.iter();

    println!("Found {} images.", images.len());

    let num_monitors = 2;

    loop {
        let mut n = 0;
        let mut selected_images = Vec::new();
        while n < num_monitors {
            if let Some(im) = image_iter.next() {
                selected_images.push(im);
                n += 1;
            } else {
                // If we run out of images, start from the beginning
                image_iter = images.iter();
            }
        }

        // TODO: Check feh exists before entering this method
        let mut cmd = std::process::Command::new("feh");
        cmd.arg("--no-fehbg").arg("--bg-fill");
        for image in selected_images.iter() {
            cmd.arg(image);
        }
        println!("setting images");
        cmd.spawn().unwrap();

        std::thread::sleep(dur);
    }
}

fn get_images<P>(dir: P) -> Vec<PathBuf>
where
    P: AsRef<Path>,
{
    let image_extensions = ["jpeg", "jpg", "png"];
    let mut images = Vec::new();

    for file in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        if let Some(ext) = file.path().extension() {
            if image_extensions.contains(&ext.to_ascii_lowercase().to_str().unwrap()) {
                images.push(file.path().to_owned());
            }
        }
    }
    images
}
