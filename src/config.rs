use std::env::var;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{prelude::*, Error};

#[cfg(feature = "config")]
mod entities;

cfg_if::cfg_if! {
    if #[cfg(feature = "config")] {
        pub use entities::Config;
    } else {
        #[derive(Debug)]
        pub struct Config;
    }
}

#[cfg(feature = "config")]
#[derive(Debug)]
pub enum ConfigError {
    OpenError(Error),
    ReadError(Error),
    TomlError(toml::de::Error),
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
            }
        )
    }
}

#[cfg(feature = "config")]
pub fn read_config() -> Result<Config, ConfigError> {
    #[allow(unused_must_use)]
    match read_file() {
        Ok(config) => Ok(config),
        Err(e) => {
            eprintln!("{e}\nre/generating config file");

            let config = Config::default();

            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(get_config_file())
                .map_err(|e| ConfigError::OpenError(e))?;

            write!(file, "{}", toml::to_string(&config).unwrap());
            eprintln!("loaded default config");

            Ok(config)
        }
    }
}

#[cfg(feature = "config")]
fn read_file() -> Result<Config, ConfigError> {
    let mut config = String::new();
    let mut config_file = OpenOptions::new()
        .read(true)
        .open(get_config_file())
        .map_err(|e| ConfigError::OpenError(e))?;

    config_file
        .read_to_string(&mut config)
        .map_err(|e| ConfigError::ReadError(e))?;

    let config = toml::from_str::<Config>(&config).map_err(|e| ConfigError::TomlError(e))?;

    Ok(config)
}

#[cfg(feature = "config")]
fn get_config_file() -> String {
    var("EH_CONFIG").unwrap_or(String::from("./config.toml"))
}
