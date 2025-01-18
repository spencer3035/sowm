use std::sync::{mpsc::channel, Arc};

use listener::{open_socket, setup_signal_handler};
use sowm_common::init;

/// Engine to run the logic to update the wallpaper
mod engine;
/// Listen for messages that get sent over the socket
mod listener;

fn main() {
    let init = match init() {
        Err(e) => panic!("Init Error: {e}"),
        Ok(v) => v,
    };

    setup_signal_handler(&init);

    let listener = match open_socket(&init) {
        Err(e) => panic!("Socket Error: {e}"),
        Ok(v) => v,
    };

    let init = Arc::new(init);
    let (tx, rx) = channel();

    let init1 = init.clone();
    let h1 = std::thread::spawn(move || listener::listener(tx, init1, listener));
    let init2 = init.clone();
    let h2 = std::thread::spawn(move || engine::run(rx, init2));

    h1.join().expect("Listener failed");
    println!("Listener closed");
    h2.join().expect("Engine failed");
    println!("Engine thread closed");

    println!("All threads closed, exiting")
}
