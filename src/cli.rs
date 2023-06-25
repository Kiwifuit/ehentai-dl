use clap::Parser;
use crate::logger::{parse_log_level, LogLevel};
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short = 'l', long = "log-level", value_parser = parse_log_level::<LogLevel>)]
    pub log_level: Option<LogLevel>,

    #[arg(short = 'd', long = "delete-original")]
    pub delete_original: Option<bool>,

    #[arg(short = 'r', long = "rename")]
    pub rename: Option<bool>,

    #[arg(short = 'D', long = "description")]
    pub description: Option<String>,

    #[arg(value_name = "INPUT_FILE", value_hint = clap::ValueHint::FilePath)]
    pub links_file: PathBuf
}
