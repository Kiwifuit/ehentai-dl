use std::process;

use log::info;

#[macro_use]
mod macros;

cfg_if::cfg_if! {
    if #[cfg(not(feature = "aniyomi"))] {
        #[allow(unused_imports)]
        mod aniyomi;

        #[allow(dead_code)]
        mod gallery;
    } else {
        mod aniyomi;
        mod gallery;
    }
}

cfg_if::cfg_if! {
    if #[cfg(not(feature = "zip"))] {
        #[allow(unused_imports)]
        mod zip;

        #[allow(dead_code)]
        mod downloader;
    } else {
        mod zip;
        mod downloader;
    }
}

mod extractor;
mod logger;
mod parser;
mod progress;

const CHUNK_SIZE: usize = 1024;

fn main() {
    logger::setup_logger(logger::LogLevel::Debug)
        .expect("unexpected error whie starting the logger");
    let m_prog = progress::Progress::new();

    let raw = parser::read_file::<CHUNK_SIZE, str>("res/galleries.txt").unwrap();
    let galleries = parser::get_all_galleries(&raw).unwrap();
    let gallery_prog = m_prog.add_prog(galleries.len() as u64, "Getting Galleries");

    info!("{} galleries to download", galleries.len());
    for gallery in galleries {
        info!("fetching data for {:?}", gallery);

        let gallery = extractor::get_gallery(&gallery, &m_prog)
            .map_err(|e| {
                eprintln!("Error while extracting gallery:\n{:#?}", e);
                process::exit(1)
            })
            .unwrap();
        info!("downloading gallery {:?}", gallery.title());

        let download_averages = downloader::download_gallery(&gallery, &m_prog)
            .map_err(|e| {
                eprintln!("Error while downloading gallery:\n{:#?}", e);
                process::exit(1)
            })
            .unwrap();

        gallery_prog.inc(1);
    }
    gallery_prog.finish();
}
