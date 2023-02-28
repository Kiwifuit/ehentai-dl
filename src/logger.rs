use std::io::stdout;
use std::path::PathBuf;

use chrono::Local;
use fern_colored::{log_file, Dispatch};
use log::LevelFilter;

pub enum LogLevel {
    Off,
    Trace,
    Debug,
    Info,
    Warn,
    Error,
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
        .chain(stdout())
        .chain(
            log_file(get_log_file())
                .map_err(|e| format!("An error occurred while trying to open file: {}", e))?,
        )
        .level(level.into())
        .apply()
        .map_err(|e| format!("An error occurred while trying to set up the logger: {}", e))?;

    Ok(())
}

fn get_log_file() -> PathBuf {
    let filename = Local::now().format("%d%m%y-%H%M%S.log");

    PathBuf::from(filename.to_string())
}
