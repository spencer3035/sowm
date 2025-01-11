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

    let mut buf = String::with_capacity(128);
    let conn = Stream::connect(name).unwrap();
    let mut conn = BufReader::new(conn);

    println!("Writing to server");
    conn.get_mut().write_all(b"Ping\n").unwrap();
    println!("Reading server responce");
    conn.read_line(&mut buf).unwrap();

    println!("Server: {buf}");
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
    let p = PathBuf::from("../../mypipe.sock");
    p
}
