use std::cell::OnceCell;
use std::env::var;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{Error, Read};

#[cfg(feature = "config")]
mod entities;

#[cfg(feature = "config")]
pub const APP_CONFIG: OnceCell<entities::Config> = OnceCell::new();

#[cfg(feature = "config")]
pub enum ConfigError {
    OpenError(Error),
    ReadError(Error),
    TomlError(toml::de::Error),
    SetError,
}

#[cfg(feature = "config")]
impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "error while {}",
            match self {
                Self::OpenError(e) => format!("opening config file: {}", e),
                Self::ReadError(e) => format!("reading config file: {}", e),
                Self::TomlError(e) => format!("parsing config file: {}", e),
                Self::SetError =>
                    String::from("setting values. perhaps the config has already been loaded?"),
            }
        )
    }
}

#[cfg(feature = "config")]
pub fn read_config() -> Result<(), ConfigError> {
    let mut config = String::new();
    let mut config_file = OpenOptions::new()
        .read(true)
        .open(var("EH_CONFIG").unwrap_or(String::from("./config.toml")))
        .map_err(|e| ConfigError::OpenError(e))?;

    config_file
        .read_to_string(&mut config)
        .map_err(|e| ConfigError::ReadError(e))?;

    Ok(APP_CONFIG
        .set(toml::from_str::<entities::Config>(&config).map_err(|e| ConfigError::TomlError(e))?)
        .map_err(|_| ConfigError::SetError)?)
}
