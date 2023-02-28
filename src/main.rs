use std::process::exit;

mod gallery;
mod logger;
mod parsers;

fn main() {
    if let Err(msg) = logger::setup_logger(logger::LogLevel::Info) {
        eprintln!("{msg}");
        exit(1)
    }

    let mut gallery = gallery::Gallery::new(10);

    gallery.fetch_images("https://e-hentai.org/g/2473007/d2997e276f/");

    dbg!(gallery);
}
