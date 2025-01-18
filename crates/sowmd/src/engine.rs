use std::{
    path::{Path, PathBuf},
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use rand::{seq::SliceRandom, thread_rng};
use sowm_common::{ClientMessage, Init, SowmError};

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
        let image = self.arr.get(self.ii).cloned();
        self.ii += 1;
        image
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
    fn new(init: Init) -> Result<Self, SowmError> {
        let wallpaper_change_dur = init.config.switch_interval();
        let num_monitors = init.config.num_monitors();

        let mut images = init.images.clone();
        images.shuffle(&mut thread_rng());
        let image_iter = LoopingIter::new(images);

        let state = State::Running;

        Ok(Engine {
            init,
            images_iter: image_iter,
            num_monitors,
            state,
            wallpaper_change_dur,
        })
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
            let image = self.images_iter.next().unwrap();
            selected_images.push(image);
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
    let mut engine = match Engine::new(init) {
        Err(e) => panic!("{e}"),
        Ok(v) => v,
    };
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
    // TODO: handle feh not found error
    cmd.spawn().unwrap();
}
