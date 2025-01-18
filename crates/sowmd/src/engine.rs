use std::{
    path::{Path, PathBuf},
    slice::Iter,
    sync::{mpsc::Receiver, Arc},
    time::{Duration, Instant},
};

use rand::{seq::SliceRandom, thread_rng};
use sowm_common::{ClientMessage, Config, Init};
use walkdir::WalkDir;

struct Engine<'a, T> {
    images_iter: LoopingIter<'a, T>,
    config: Config,
    state: State,
    wallpaper_change_dur: Duration,
    message_poll_dur: Duration,
}

enum State {
    Running,
    Stopped,
}

pub struct LoopingIter<'a, T> {
    arr: &'a [T],
    ii: usize,
}

impl<'a, T> LoopingIter<'a, T> {
    fn new(arr: &'a [T]) -> LoopingIter<'a, T> {
        LoopingIter { arr, ii: 0 }
    }
}

impl<'a, T> Iterator for LoopingIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ii >= self.arr.len() {
            self.ii = 0;
        }
        self.arr.get(self.ii)
    }
}

// Example of using feh to set the background for two monitors
// feh --no-fehbg --bg-fill girl_holding_power.jpg girl_face_in_pool.jpeg

pub fn run(rx: Receiver<ClientMessage>, _init: Arc<Init>) -> ! {
    let wallpaper_change_dur = Duration::from_secs(60 * 30);
    let message_poll_dur = Duration::from_millis(100);
    let num_monitors = 2;

    let image_dir = PathBuf::from("/home/spencer/pictures/wallpapers");

    let mut images = get_images(&image_dir);
    images.shuffle(&mut thread_rng());
    let mut image_iter = LoopingIter::new(&images);

    println!("Found {} images.", images.len());
    let mut start_time;
    let mut state = State::Running;

    loop {
        start_time = Instant::now();

        if matches!(state, State::Running) {
            let mut selected_images = Vec::new();
            for _ in 0..num_monitors {
                selected_images.push(image_iter.next().unwrap());
            }
            set_background(&selected_images);
        }

        while start_time.elapsed() <= wallpaper_change_dur {
            if let Ok(msg) = rx.try_recv() {
                match msg {
                    ClientMessage::Stop => {
                        state = State::Stopped;
                    }
                    ClientMessage::Start => {
                        state = State::Running;
                    }
                    ClientMessage::Next => {
                        let mut selected_images = Vec::new();
                        for _ in 0..num_monitors {
                            selected_images.push(image_iter.next().unwrap());
                        }
                        set_background(&selected_images);
                    }
                }
            }
            std::thread::sleep(message_poll_dur);
        }

        std::thread::sleep(wallpaper_change_dur);
    }
}

fn set_background<P>(selected_images: &[P])
where
    P: AsRef<Path>,
{
    // TODO: Check feh exists before entering this method
    let mut cmd = std::process::Command::new("feh");
    cmd.arg("--no-fehbg").arg("--bg-fill");
    for image in selected_images.iter() {
        cmd.arg(image.as_ref());
    }
    println!("setting images");
    // TODO: handle feh not found error
    cmd.spawn().unwrap();
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
