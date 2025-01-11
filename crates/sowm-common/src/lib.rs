use std::path::PathBuf;

pub fn get_pipe_path() -> PathBuf {
    let uid = users::get_current_uid();
    let mut p = PathBuf::new();
    p.push("/run");
    p.push("user");
    p.push(uid.to_string());
    debug_assert!(p.exists(), "run path doesn't exist: {}", p.display());
    p.push("sowm.fifo");
    p
}

#[derive(Debug)]
pub enum Command {
    Reset,
    Stop,
    Start,
    Noop,
    // Add update config etc
}

pub fn parse_command(command: &str) -> Command {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipe_dir_exists() {
        let mut path = get_pipe_path();
        path.pop();
        assert!(matches!(path.try_exists(), Ok(true)));
    }
}
