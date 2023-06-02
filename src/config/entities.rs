#[cfg(feature = "config")]
use serde::Deserialize;

#[cfg(feature = "config")]
#[derive(Deserialize)]
pub struct Config {
    config: ConfigKeys,
}

#[cfg(feature = "config")]
#[derive(Deserialize)]
pub(super) struct ConfigKeys {
    features: Vec<String>,
    chunk_size: usize,
}
