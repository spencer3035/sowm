use std::{
    path::{Path, PathBuf},
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use rand::{seq::SliceRandom, thread_rng};
use sowm_common::{ClientMessage, Init};
use walkdir::WalkDir;

enum State {
    Running,
    Stopped,
}

pub struct LoopingIter {
    arr: Vec<PathBuf>,
    ii: usize,
}

impl LoopingIter {
    fn new(arr: Vec<PathBuf>) -> Self {
        LoopingIter { arr, ii: 0 }
    }
}

impl Iterator for LoopingIter {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ii >= self.arr.len() {
            self.ii = 0;
        }
        self.arr.get(self.ii).cloned()
    }
}

struct Engine {
    images_iter: LoopingIter,
    state: State,
    wallpaper_change_dur: Duration,
    #[allow(dead_code)]
    init: Init,
    num_monitors: usize,
}

impl Engine {
    fn new(init: Init) -> Self {
        let wallpaper_change_dur = init.config.switch_interval();
        let num_monitors = init.config.num_monitors();
        let image_dir = init.config.image_dir();

        let mut images = get_images(&image_dir);
        images.shuffle(&mut thread_rng());
        println!("Found {} images.", images.len());
        let image_iter = LoopingIter::new(images);

        let state = State::Running;

        Engine {
            init,
            images_iter: image_iter,
            num_monitors,
            state,
            wallpaper_change_dur,
        }
    }

    /// Runs the next wallpaper cycle of the engine, if it is running
    fn cycle(&mut self) {
        if matches!(self.state, State::Running) {
            self.next();
        }
    }

    /// Loads the next set of images
    fn next(&mut self) {
        let mut selected_images = Vec::new();
        for _ in 0..self.num_monitors {
            selected_images.push(self.images_iter.next().unwrap());
        }
        set_background(&selected_images);
    }

    /// Handles a message from the client
    fn handle_message(&mut self, msg: ClientMessage) {
        match msg {
            ClientMessage::Stop => {
                self.state = State::Stopped;
            }
            ClientMessage::Start => {
                self.state = State::Running;
            }
            ClientMessage::Next => {
                self.next();
            }
        }
    }
}

pub fn run(rx: Receiver<ClientMessage>, init: Init) -> ! {
    let mut engine = Engine::new(init);
    let mut start_time;
    let message_poll_dur = Duration::from_millis(100);

    loop {
        start_time = Instant::now();

        engine.cycle();
        while start_time.elapsed() <= engine.wallpaper_change_dur {
            if let Ok(msg) = rx.try_recv() {
                engine.handle_message(msg);
            }
            std::thread::sleep(message_poll_dur);
        }
    }
}

/// Sets the background to the list of images. There should be as many images as there is monitors
fn set_background<P>(selected_images: &[P])
where
    P: AsRef<Path>,
{
    // Example: feh --no-fehbg --bg-fill image1.jpg image2.jpeg

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

/// Gets all images in the provided directory, recursively
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
