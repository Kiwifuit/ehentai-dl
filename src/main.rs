use std::process;

use log::info;

#[macro_use]
mod macros;
mod extractor;
mod gallery;
mod logger;
mod parser;
mod progress;

const CHUNK_SIZE: usize = 1024;

fn main() {
    logger::setup_logger(logger::LogLevel::Debug)
        .expect("unexpected error whie starting the logger");
    let m_prog = progress::Progress::new();

    // let raw = parser::read_file::<CHUNK_SIZE, str>("res/galleries.txt").unwrap();
    // let galleries = parser::get_all_galleries(&raw).unwrap();

    // info!("{} galleries to download", galleries.len());
    // for gallery in galleries {
    //     info!("fetching data for {:?}", gallery);
    //     let gallery = extractor::get_gallery(&gallery)
    //         .map_err(|e| {
    //             eprintln!("Error while extracting gallery:\n{:#?}", e);
    //             process::exit(1)
    //         })
    //         .unwrap();

    //     dbg!(gallery);
    // }

    let g = extractor::get_gallery("https://e-hentai.org/g/2492813/42631dcf66/", &m_prog).unwrap();

    dbg!(g);
}
