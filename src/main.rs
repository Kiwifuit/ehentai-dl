use std::process::exit;

mod gallery;
mod logger;
mod parsers;

fn main() {
    if let Err(msg) = logger::setup_logger(logger::Level::Info) {
        eprintln!("{msg}");
        exit(1)
    }

    let galleries = gallery::make_galleries("res/galleries.txt").unwrap();
}
