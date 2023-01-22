use std::process::exit;

mod gallery;
mod logger;
mod parsers;

fn main() {
    if let Err(msg) = logger::setup_logger(logger::Level::Info) {
        eprintln!("{msg}");
        exit(1)
    }

    let gallery = gallery::Gallery::new("https://e-hentai.org/g/1924289/a013c43b21/".to_string());

    dbg!(gallery);
}
