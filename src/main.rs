use log::info;

mod extractor;
mod logger;
mod parser;

const CHUNK_SIZE: usize = 1024;

fn main() {
    logger::setup_logger(logger::LogLevel::Debug)
        .expect("unexpected error whie starting the logger");

    let a = parser::read_file::<CHUNK_SIZE, str>("res/galleries.txt").unwrap();
    let g = parser::get_all_galleries(&a).unwrap();

    info!("{} galleries to download", g.len());
}
