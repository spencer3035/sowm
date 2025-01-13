use clap::{Parser, Subcommand};
use interprocess::local_socket::{prelude::*, GenericFilePath, Stream};
use std::{
    io::{BufReader, Read, Write},
    sync::Arc,
};

use sowm_common::{init, packet::Packet, ClientMessage, ServerMessage};

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

    let init = match init() {
        Err(e) => panic!("Init Error: {e}"),
        Ok(v) => Arc::new(v),
    };

    let path = &init.socket_file;
    println!("Socket path is {}", path.display());
    let name = path.as_path().to_fs_name::<GenericFilePath>().unwrap();

    let conn = Stream::connect(name).unwrap();
    let mut conn = BufReader::new(conn);

    // Send message
    let message: ClientMessage = cli.command.into();
    let data = message.serialize().unwrap();
    println!("Sending {} + 8 bytes to the server", data.len());
    let packet = Packet::new(data);
    let bytes = packet.into_bytes();
    conn.get_mut().write_all(&bytes).unwrap();

    // Get responce message
    println!("Reading server responce");
    let mut header: [u8; 8] = [0; 8];
    conn.read_exact(&mut header).unwrap();
    let len = Packet::len_from_header(&header).unwrap();
    let mut buf = vec![0; len];
    conn.read_exact(&mut buf).unwrap();
    let message: ServerMessage = ServerMessage::deserialize(&buf).unwrap();

    println!("Server: {message:#?}");
}
