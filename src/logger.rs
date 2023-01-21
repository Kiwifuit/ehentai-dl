pub use log::Level;
use std::path::PathBuf;

use chrono::Local;
use fern_colored::{log_file, Dispatch};

pub fn setup_logger(level: Level) -> Result<(), String> {
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
        .level(level.to_level_filter())
        .apply()
        .map_err(|e| format!("An error occurred while trying to set up the logger: {}", e))?;

    Ok(())
}

fn get_log_file() -> PathBuf {
    let filename = Local::now().format("%d%m%y-%H%M%S.log");

    PathBuf::from(filename.to_string())
}
