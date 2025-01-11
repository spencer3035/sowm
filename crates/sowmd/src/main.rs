use std::io::{BufReader, Read, Write};

use sowm_common::{get_pipe_path, packet::Packet, ClientMessage, ServerMessage};

use interprocess::local_socket::{prelude::*, GenericFilePath, ListenerOptions, Stream};

// Example of using feh to set the background for two monitors
// feh --no-fehbg --bg-fill girl_holding_power.jpg girl_face_in_pool.jpeg

// Define a function that checks for errors in incoming connections. We'll use this to filter
// through connections that fail on initialization for one reason or another.
fn handle_error(conn: std::io::Result<Stream>) -> Option<Stream> {
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
        println!("\nremoving socket: {}", tmp_path.display());
        if matches!(tmp_path.try_exists(), Ok(true)) {
            std::fs::remove_file(&tmp_path).unwrap();
        }
        std::process::exit(0);
    })
    .expect("Failed setting exception handler");

    let listener = match opts.create_sync() {
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
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

    for conn in listener.incoming().filter_map(handle_error) {
        let mut conn = BufReader::new(conn);
        println!("Got new connection");

        // Get message
        println!("Reading client message");
        let mut header: [u8; 8] = [0; 8];
        conn.read_exact(&mut header).unwrap();
        let len = Packet::len_from_header(&header).unwrap();
        let mut buf = vec![0; len];
        conn.read_exact(&mut buf).unwrap();
        let message: ClientMessage = ClientMessage::deserialize(&buf);
        println!("Client Sent: {message:#?}");

        // Send responce message
        let message: ServerMessage = ServerMessage::Ok;
        let data = message.serialize();
        println!("Sending {} + 8 bytes to the client", data.len());
        let packet = Packet::new(data);
        let bytes = packet.into_bytes();
        conn.get_mut().write_all(&bytes).unwrap();

        println!("Server: {message:#?}");
    }
}
