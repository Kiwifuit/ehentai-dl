use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    config: ConfigKeys,
}

#[derive(Deserialize)]
pub(super) struct ConfigKeys {
    features: Vec<String>,
    chunk_size: usize,
}
