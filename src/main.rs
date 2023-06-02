use std::process::exit;

use log::{error, info};

#[macro_use]
mod macros;

#[cfg_attr(not(feature = "aniyomi"), allow(unused_imports))]
mod aniyomi;
#[cfg_attr(not(feature = "config"), allow(unused_imports))]
mod config;
#[cfg_attr(not(feature = "zip"), allow(dead_code))]
mod downloader;
#[cfg_attr(not(feature = "aniyomi"), allow(dead_code))]
mod gallery;
#[cfg_attr(not(feature = "zip"), allow(unused_imports))]
mod zip;

mod extractor;
mod logger;
mod parser;
mod progress;
mod version;

const CHUNK_SIZE: usize = 1024;

fn main() {
    cfg_if::cfg_if! {
        if #[cfg(feature = "config")] {
            if let Err(err) = config::read_config() {
                eprintln!("error while loading config: {}", err);
                exit(-1)
            }
        }
    }

    let version = version::get_version();
    let mut errs = 0;
    logger::setup_logger(logger::LogLevel::Debug)
        .expect("unexpected error while starting the logger");
    info!("{}", version);

    let m_prog = progress::Progress::new();

    let raw = parser::read_file::<CHUNK_SIZE, str>("res/galleries.txt").unwrap();
    let galleries = parser::get_all_galleries(&raw).unwrap();
    let gallery_prog = m_prog.add_prog(galleries.len() as u64, "Getting Galleries");

    info!("{} galleries to download", galleries.len());
    for gallery in galleries {
        info!("fetching data for {:?}", gallery);
        let gallery = extractor::get_gallery(&gallery, &m_prog);

        if let Err(ref err) = gallery {
            error!(
                "Error while extracting data for gallery: {0}\nFull Error:\n{0:#?}",
                err
            );
            errs += 1;

            continue;
        }

        let gallery = gallery.unwrap();
        info!("downloading gallery {:?}", gallery.title());
        let download_averages = downloader::download_gallery::<CHUNK_SIZE>(&gallery, &m_prog);

        if let Err(ref err) = download_averages {
            error!(
                "Error while downloading gallery {1:?}: {0}\nFull Error:\n{0:#?}",
                err,
                gallery.title()
            );
            errs += 1;

            continue;
        }
        let download_averages = download_averages.unwrap();

        gallery_prog.inc(1);
    }
    gallery_prog.finish();

    if errs < 0 {
        eprintln!(
            "{} error(s) have occurred while downloading, check the logs for more info",
            errs
        );
        exit(errs);
    }
}
