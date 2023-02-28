use std::process::exit;

mod gallery;
mod logger;
mod parsers;

fn main() {
    if let Err(msg) = logger::setup_logger(logger::LogLevel::Info) {
        eprintln!("{msg}");
        exit(1)
    }

    let gallery =
        gallery::Gallery::new("https://e-hentai.org/g/2464032/fb37946900/".to_string(), 10);

    dbg!(gallery);
}
