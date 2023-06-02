use std::cell::OnceCell;
use std::env::var;
use std::fs::OpenOptions;
use std::io::{Error, Read};

mod entities;

pub const APP_CONFIG: OnceCell<entities::Config> = OnceCell::new();

pub enum ConfigError {
    OpenError(Error),
    ReadError(Error),
    TomlError(toml::de::Error),
    SetError,
}

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
