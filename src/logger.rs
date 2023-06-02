#![allow(dead_code)]
// TODO: Remove ^ when we start writing the CLI
//       I would want something like a -v flag
//       but passing in more -v flags increases
//       the verbosity
use std::path::PathBuf;

use chrono::Local;
use fern_colored::{log_file, Dispatch};
use log::LevelFilter;
#[cfg(feature = "config")]
use serde::{Deserialize, Serialize};

cfg_if::cfg_if! {
    if #[cfg(feature = "config")]{
        #[derive(Serialize, Deserialize, Debug, Clone, Copy)]
        pub enum LogLevel {
            Off,
            Trace,
            Debug,
            Info,
            Warn,
            Error,
        }
    } else {
        pub enum LogLevel {
            Off,
            Trace,
            Debug,
            Info,
            Warn,
            Error,
        }
    }

}

impl Into<LevelFilter> for LogLevel {
    fn into(self) -> LevelFilter {
        match self {
            Self::Off => LevelFilter::Off,
            Self::Trace => LevelFilter::Trace,
            Self::Debug => LevelFilter::Debug,
            Self::Info => LevelFilter::Info,
            Self::Warn => LevelFilter::Warn,
            Self::Error => LevelFilter::Error,
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

pub fn setup_logger(level: LogLevel) -> Result<(), String> {
    Dispatch::new()
        .format(move |out, msg, record| {
            out.finish(format_args!(
                "[{}] [{}] [{}] {}",
                Local::now().format("%H:%M:%S"),
                record.module_path().or(Some("<module>")).unwrap(),
                record.level(),
                msg
            ))
        })
        .chain(
            log_file(get_log_file())
                .map_err(|e| format!("An error occurred while trying to open file: {}", e))?,
        )
        .level(level.into())
        // We deactivate the logs for certain crates (and the
        // 'blocking' module) because they alone puff up the
        // logfile size from 38K (with all of these off) to
        // 1.5G
        .level_for("html5ever", LevelFilter::Off)
        .level_for("selectors", LevelFilter::Off)
        .level_for("reqwest::blocking", LevelFilter::Off)
        .apply()
        .map_err(|e| format!("An error occurred while trying to set up the logger: {}", e))?;

    Ok(())
}

fn get_log_file() -> PathBuf {
    let filename = Local::now().format("%d%m%y-%H%M%S.log");

    PathBuf::from(filename.to_string())
}
