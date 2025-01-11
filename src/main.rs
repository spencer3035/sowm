use std::{io::Read, path::PathBuf, time::Duration};

use interprocess::os::unix::fifo_file::create_fifo;

fn main() {
    let fifo_path = get_pipe_path();
    println!("Fifo path is {}", fifo_path.display());

    let path = PathBuf::from("./mypipe.fifo");
    let mode = 0o777;
    create_fifo(&path, mode).unwrap();

    let tmp_path = path.clone();
    ctrlc::set_handler(move || {
        println!("removing fifo: {}", tmp_path.display());
        std::fs::remove_file(&tmp_path).unwrap();
        std::process::exit(0);
    })
    .expect("Failed setting exception handler");

    let mut fifo_file = std::fs::File::open(path).unwrap();
    let poll_durration = Duration::from_millis(100);
    loop {
        let mut contents = String::new();
        let bytes_read = fifo_file.read_to_string(&mut contents).unwrap();
        if bytes_read > 0 {
            println!("read: {contents}");
        }
        std::thread::sleep(poll_durration);
    }
}

#[derive(Debug)]
enum Command {
    Reset,
    Stop,
    Start,
    Noop,
    // Add update config etc
}

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
    p
}
