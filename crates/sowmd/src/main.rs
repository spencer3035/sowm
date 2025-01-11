use std::{
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
};

use interprocess::local_socket::{prelude::*, GenericFilePath, ListenerOptions, Stream};

// Define a function that checks for errors in incoming connections. We'll use this to filter
// through connections that fail on initialization for one reason or another.
fn handle_error(conn: io::Result<Stream>) -> Option<Stream> {
    match conn {
        Ok(c) => Some(c),
        Err(e) => {
            eprintln!("Incoming connection failed: {e}");
            None
        }
    }
}

fn main() {
    let path = get_pipe_path();
    println!("Socket path is {}", path.display());
    let name = path.as_path().to_fs_name::<GenericFilePath>().unwrap();
    let opts = ListenerOptions::new().name(name);

    let tmp_path = path.clone();
    ctrlc::set_handler(move || {
        println!("removing socket: {}", tmp_path.display());
        if matches!(tmp_path.try_exists(), Ok(true)) {
            std::fs::remove_file(&tmp_path).unwrap();
        }
        std::process::exit(0);
    })
    .expect("Failed setting exception handler");

    let listener = match opts.create_sync() {
        Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
            // When a program that uses a file-type socket name terminates its socket server
            // without deleting the file, a "corpse socket" remains, which can neither be
            // connected to nor reused by a new listener. Normally, Interprocess takes care of
            // this on affected platforms by deleting the socket file when the listener is
            // dropped. (This is vulnerable to all sorts of races and thus can be disabled.)
            // There are multiple ways this error can be handled, if it occurs, but when the
            // listener only comes from Interprocess, it can be assumed that its previous instance
            // either has crashed or simply hasn't exited yet. In this example, we leave cleanup
            // up to the user, but in a real application, you usually don't want to do that.
            panic!(
                "Error: could not start server because the socket file is occupied. Please check if {} is in use by another process and try again.",
                path.display()
            );
        }
        x => x.unwrap(),
    };

    println!("Socket running on {}", path.display());

    let mut buf = String::with_capacity(128);
    for conn in listener.incoming().filter_map(handle_error) {
        let mut conn = BufReader::new(conn);
        println!("Got new connection");

        println!("reading client message");
        conn.read_line(&mut buf).unwrap();
        println!("Sending responce");
        conn.get_mut().write_all(b"Pong").unwrap();

        println!("Client: {buf}");
        buf.clear();
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum Command {
    Reset,
    Stop,
    Start,
    Noop,
    // Add update config etc
}

#[allow(dead_code)]
fn parse_command(command: &str) -> Command {
    for (ii, word) in command.split_whitespace().enumerate() {
        if ii == 0 {
            match word {
                "reset" => return Command::Reset,
                "stop" => return Command::Stop,
                "start" => return Command::Start,
                _ => return Command::Noop,
            }
        }
    }
    Command::Noop
}

fn get_pipe_path() -> PathBuf {
    let uid = users::get_current_uid();
    let mut p = PathBuf::new();
    p.push("/run");
    p.push("user");
    p.push(uid.to_string());
    debug_assert!(p.exists(), "run path doesn't exist: {}", p.display());
    p.push("sowm.fifo");
    //p
    let p = PathBuf::from("./mypipe.sock");
    p
}
