use clap::{Parser, Subcommand};
use interprocess::local_socket::{prelude::*, GenericFilePath, Stream};
use std::io::{BufRead, BufReader, Write};

use sowm_common::{get_pipe_path, ClientMessage};

#[derive(Debug, Parser)]
struct Cli {
    /// Which command to send to the daemon
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Start cycling wallpapers
    Start,
    /// Stop cycling wallpapers
    Stop,
    /// Reset
    Reset,
}

impl From<Command> for ClientMessage {
    fn from(value: Command) -> Self {
        match value {
            Command::Start => ClientMessage::Start,
            Command::Stop => ClientMessage::Stop,
            Command::Reset => ClientMessage::Reset,
        }
    }
}

fn main() {
    let cli = Cli::parse();
    println!("{cli:#?}");

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
