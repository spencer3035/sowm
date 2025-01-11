use interprocess::local_socket::{prelude::*, GenericFilePath, ListenerOptions, Stream};
use std::io::{BufRead, BufReader, Write};

use sowm_common::get_pipe_path;

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
