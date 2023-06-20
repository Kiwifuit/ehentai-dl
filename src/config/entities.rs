#[cfg(feature = "config")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "config")]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub app: AppConfig,

    // FIXME: This fucking program can't compile
    // without their respective features being
    // turned on.
    // This shit gon get marked as broken lmfao.
    // just like me :D
    #[cfg(feature = "aniyomi")]
    pub aniyomi: AniyomiConfig,

    #[cfg(feature = "zip")]
    pub zip: ZipConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub features: Vec<String>,

    #[serde(rename = "log-level")]
    pub log_level: crate::logger::LogLevel,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            features: crate::version::get_features()
                .iter()
                .filter(|i| i != &&"config".to_string())
                .map(|i| i.to_owned())
                .collect(),
            log_level: crate::logger::LogLevel::default(),
        }
    }
}

#[cfg(all(feature = "config", feature = "aniyomi"))]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct AniyomiConfig {
    pub description: Option<String>,
    pub rename: bool,
}

#[cfg(all(feature = "config", feature = "zip"))]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ZipConfig {
    #[serde(rename = "delete-original")]
    pub delete_original: bool,
}
