use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};

pub mod packet;

/// Contains all relevant information for communication between client and server as well as other
/// error prone things that need to be discovered
///
/// Should only be created via init() method
pub struct Init {
    /// Path to config toml file
    pub config_path: PathBuf,
    /// Path to socket file
    pub socket_file: PathBuf,
    /// If the socket file exists at the time of creation of the object. Clients should check that
    /// this is true and server should check that this is false.
    pub does_socket_file_exist: bool,
    /// Current configuration
    pub config: Config,
}

/// Generates a new Init instance
pub fn init() -> Result<Init, SowmError> {
    let config_directories = BaseDirs::new().ok_or(SowmError::NoHomeDirectory)?;
    let socket_file = get_socket_directory()?;
    let does_socket_file_exist = socket_file
        .try_exists()
        .map_err(|_| SowmError::NoUserSocketDirectory(socket_file.clone()))?;

    // TODO: Some of the below errors could be recoverable, we should put these in associated
    // methods with results so that it doesn't block the server from starting for exampele
    let config_path = config_path(&config_directories)?;
    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|_| SowmError::NoConfigDir(config_path.clone()))?;
    let config: Config =
        toml::from_str(&config_content).map_err(|e| SowmError::ConfigParseFail(e))?;

    Ok(Init {
        socket_file,
        does_socket_file_exist,
        config_path,
        config,
    })
}

/// Gets the path to the config.toml, it also creates on based on the default if it doesn't exist
fn config_path(dirs: &BaseDirs) -> Result<PathBuf, SowmError> {
    let mut dir = dirs.config_dir().to_path_buf();
    dir.push("sowm");

    if !matches!(dir.try_exists(), Ok(true)) {
        std::fs::create_dir(&dir).map_err(|_| SowmError::NoConfigDir(dir.clone()))?;
    }

    dir.push("config.toml");

    if !matches!(dir.try_exists(), Ok(true)) {
        let conf = Config::default();
        let str = toml::to_string_pretty(&conf).expect("Failed to seralize default config to toml");
        std::fs::write(&dir, &str).map_err(|_| SowmError::NoConfigDir(dir.clone()))?
    }

    Ok(dir)
}

#[derive(Debug)]
pub enum SowmError {
    NoHomeDirectory,
    NoUserSocketDirectory(PathBuf),
    NoConfigDir(PathBuf),
    SerializationFailed(bitcode::Error),
    DeserializationFailed(bitcode::Error),
    ConfigParseFail(toml::de::Error),
}

impl SowmError {
    pub fn client_critical(&self) -> bool {
        match self {
            SowmError::NoUserSocketDirectory(_) => true,
            SowmError::NoHomeDirectory => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for SowmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: String = match self {
            Self::NoHomeDirectory => "Could not find home directory".into(),
            Self::NoUserSocketDirectory(path) => format!(
                "User's socket directory didn't exist or wasn't writable: {}",
                path.display()
            ),
            Self::NoConfigDir(path) => format!(
                "User's config directory didn't exist or wasn't writable: {}",
                path.display()
            ),
            Self::SerializationFailed(e) => format!("Serialization error: {e}"),
            Self::DeserializationFailed(e) => format!("Deserialization error: {e}"),
            Self::ConfigParseFail(e) => format!("Failed parsing config.toml : {e}"),
        };

        write!(f, "{s}")
    }
}

fn get_socket_directory() -> Result<PathBuf, SowmError> {
    let uid = users::get_current_uid();
    let mut p = PathBuf::new();
    p.push("/run");
    p.push("user");
    p.push(uid.to_string());
    if !matches!(p.try_exists(), Ok(true)) {
        debug_assert!(p.exists(), "run path doesn't exist: {}", p.display());
        return Err(SowmError::NoUserSocketDirectory(p));
    }
    p.push("sowm.fifo");
    Ok(p)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    Start,
    Stop,
    Next,
    // Add update config etc
}

impl ClientMessage {
    pub fn serialize(&self) -> Result<Vec<u8>, SowmError> {
        bitcode::serialize(self).map_err(|e| SowmError::SerializationFailed(e))
    }
    pub fn deserialize(v: &[u8]) -> Result<Self, SowmError> {
        bitcode::deserialize(v).map_err(|e| SowmError::DeserializationFailed(e))
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
    pub fn serialize(&self) -> Result<Vec<u8>, SowmError> {
        bitcode::serialize(self).map_err(|e| SowmError::SerializationFailed(e))
    }
    pub fn deserialize(v: &[u8]) -> Result<Self, SowmError> {
        bitcode::deserialize(v).map_err(|e| SowmError::DeserializationFailed(e))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    switch_interval_sec: u64,
    shuffle: bool,
    image_dir: PathBuf,
    num_monitors: usize,
}

impl Config {
    /// If the config is valid or not
    pub fn is_valid(&self) -> bool {
        matches!(self.image_dir.try_exists(), Ok(true))
    }

    /// Gets the interval that wallpapers should be switched
    pub fn switch_interval(&self) -> Duration {
        Duration::from_secs(self.switch_interval_sec)
    }

    /// Gets the image directory
    pub fn image_dir(&self) -> &Path {
        self.image_dir.as_path()
    }

    pub fn num_monitors(&self) -> usize {
        self.num_monitors
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            switch_interval_sec: 60 * 30,
            shuffle: true,
            image_dir: ".".into(),
            num_monitors: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_valid() {
        let c = Config::default();
        assert!(c.is_valid(), "Default config wasn't valid");
        toml::to_string(&c).expect("Failed to seralize default config to toml");
    }

    #[test]
    fn pipe_dir_exists() {
        let mut path = get_socket_directory().expect("Couldn't get socket directory");
        path.pop();
        assert!(matches!(path.try_exists(), Ok(true)));
    }
}
