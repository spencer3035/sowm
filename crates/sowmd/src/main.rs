use std::sync::mpsc::channel;

mod engine;
mod listener;

fn main() {
    let (tx, rx) = channel();

    let h1 = std::thread::spawn(move || listener::listener(tx));
    let h2 = std::thread::spawn(move || engine::run(rx));

    h1.join().unwrap();
    println!("Listener closed");
    h2.join().unwrap();
    println!("Engine thread closed");

    println!("All threads closed")
}
