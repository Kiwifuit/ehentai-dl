use log::info;

#[macro_use]
mod macros;
mod extractor;
mod gallery;
mod logger;
mod parser;

const CHUNK_SIZE: usize = 1024;

fn main() {
    logger::setup_logger(logger::LogLevel::Debug)
        .expect("unexpected error whie starting the logger");

    let raw = parser::read_file::<CHUNK_SIZE, str>("res/galleries.txt").unwrap();
    let galleries = parser::get_all_galleries(&raw).unwrap();

    info!("{} galleries to download", galleries.len());
    for gallery in galleries {
        info!("fetching data for {:?}", gallery);
        let html = extractor::get_html(gallery);

        dbg!(html);
    }
}
