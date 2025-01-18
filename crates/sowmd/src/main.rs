use std::{process::exit, sync::mpsc::channel};

use listener::{close_socket, open_socket, setup_signal_handler};
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

    let socket_file = init.socket_file.clone();

    setup_signal_handler(&init);
    let listener = match open_socket(&socket_file) {
        Err(e) => panic!("Socket Error: {e}"),
        Ok(v) => v,
    };

    let (tx, rx) = channel();
    let _h1 = std::thread::spawn(move || listener::listener(tx, listener));
    let h2 = std::thread::spawn(move || engine::run(rx, init));

    if let Err(_) = h2.join() {
        close_socket(&socket_file).unwrap();
        println!("Engine thread paniced");
        exit(1);
    }
    //h1.join().expect("Listener failed");
    //println!("Listener closed");
    //println!("All threads closed, exiting")
}
