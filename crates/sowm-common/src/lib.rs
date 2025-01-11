use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Start,
    Stop,
    Reset,
    // Add update config etc
}

impl ClientMessage {
    pub fn serialize(&self) -> Vec<u8> {
        bitcode::serialize(self).unwrap()
    }
    pub fn deserialize(v: &[u8]) -> Self {
        bitcode::deserialize(v).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    Ok,
    InvalidCommand,
    DirNotFound,
    NoImagesFound,
}

impl ServerMessage {
    pub fn serialize(&self) -> Vec<u8> {
        bitcode::serialize(self).unwrap()
    }
    pub fn deserialize(v: &[u8]) -> Self {
        bitcode::deserialize(v).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub switch_interval_sec: u64,
    pub shuffle: bool,
    pub image_dir: Option<String>,
}

impl Config {
    pub fn is_valid(&self) -> bool {
        if let Some(path) = self.image_dir.as_ref() {
            let path = PathBuf::from(path);
            println!("Path: '{}'", path.display());
            return matches!(path.try_exists(), Ok(true));
        }

        true
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            switch_interval_sec: 60 * 30,
            shuffle: true,
            image_dir: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_valid() {
        assert!(Config::default().is_valid(), "Default config wasn't valid")
    }

    #[test]
    fn pipe_dir_exists() {
        let mut path = get_pipe_path();
        path.pop();
        assert!(matches!(path.try_exists(), Ok(true)));
    }
}
