pub use log::Level;
use std::io;

use fern_colored::{
    colors::{Color, ColoredLevelConfig},
    Dispatch,
};

use chrono::Local;

pub fn setup_logger(level: Level) -> Result<(), String> {
    let colors = ColoredLevelConfig::new()
        .info(Color::TrueColor { r: 0, g: 0, b: 0 })
        .debug(Color::TrueColor { r: 0, g: 0, b: 0 })
        .trace(Color::TrueColor { r: 0, g: 0, b: 0 })
        .warn(Color::TrueColor { r: 0, g: 0, b: 0 })
        .error(Color::TrueColor { r: 0, g: 0, b: 0 });

    Dispatch::new()
        .format(move |out, msg, record| {
            out.finish(format_args!(
                "[{}] [{}] [{}] {}",
                Local::now().format("%H:%M:%S"),
                record.module_path().or(Some("<module>")).unwrap(),
                colors.color(record.level()),
                msg
            ))
        })
        .chain(io::stdout())
        .level(level.to_level_filter())
        .apply()
        .map_err(|e| format!("An error occurred while trying to set up the logger: {}", e))?;

    Ok(())
}
