#[cfg(feature = "config")]
use serde::Deserialize;

#[cfg(feature = "config")]
#[derive(Deserialize, Debug)]
pub struct Config {
    features: Vec<String>,
    log_level: crate::logger::LogLevel,
}
