use std::{process::exit, thread::spawn};

mod gallery;
mod logger;
mod parsers;

fn main() {
    if let Err(msg) = logger::setup_logger(logger::Level::Info) {
        eprintln!("{msg}");
        exit(1)
    }
}
