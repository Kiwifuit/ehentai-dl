use std::process::exit;

mod gallery;
mod logger;
mod parsers;

fn main() {
    if let Err(msg) = logger::setup_logger(logger::LogLevel::Info) {
        eprintln!("{msg}");
        exit(1)
    }

    let gallery = gallery::Gallery::new("https://e-hentai.org/g/2264011/de4596c2f0/".to_string());

    dbg!(gallery);
}
