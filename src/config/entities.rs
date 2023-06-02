#[cfg(feature = "config")]
use serde::Deserialize;

#[cfg(feature = "config")]
#[derive(Deserialize)]
pub struct Config {
    features: Vec<String>,
}
