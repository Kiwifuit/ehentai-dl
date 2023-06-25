use clap::Parser;
use crate::logger::{parse_log_level, LogLevel};

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short = 'l', long = "log-level", value_parser = parse_log_level::<LogLevel>)]
    pub log_level: Option<LogLevel>,

    #[arg(short = 'd', long = "delete-original", default_value_t = false)]
    pub delete_original: Option<bool>,

    #[arg(short = 'r', long = "rename", default_value_t = false)]
    pub rename: Option<bool>,

    #[arg(short = 'D', long = "description")]
    pub description: Option<String>,
}
