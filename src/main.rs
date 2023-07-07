use std::path::PathBuf;
use std::process::exit;

#[cfg(any(feature = "config", feature = "cli"))]
use std::sync::Arc;

use log::{debug, error, info};

cfg_if::cfg_if! {
    if #[cfg(feature = "metrics")] {
        use std::collections::HashMap;
        use humansize::{FormatSize, DECIMAL};
        use stybulate::{Cell, Headers, Style, Table};
    }
}

#[cfg(feature = "cli")]
use clap::Parser;
#[cfg(not(feature = "cli"))]
use std::env::args;

#[macro_use]
mod macros;

#[cfg_attr(not(feature = "aniyomi"), allow(unused_imports))]
mod aniyomi;
#[cfg_attr(not(feature = "cli"), allow(unused_imports))]
mod cli;
#[cfg_attr(not(feature = "config"), allow(unused_imports))]
mod config;
#[cfg_attr(not(feature = "zip"), allow(dead_code))]
mod downloader;
#[cfg_attr(not(feature = "aniyomi"), allow(dead_code))]
mod gallery;
#[cfg_attr(not(feature = "zip"), allow(unused_imports))]
mod zip;

mod extractor;
#[cfg_attr(not(feature = "config"), allow(dead_code))]
mod logger;
mod parser;
mod progress;
mod sanitize;
mod version;

const CHUNK_SIZE: usize = 1024;

cfg_if::cfg_if! {
    if #[cfg(feature = "config")] {
        lazy_static::lazy_static! {
            static ref CONFIG: Arc<config::Config> = Arc::new(
                config::read_config()
                    .map_err(|e| {
                        eprintln!("error while loading config: {}", e);
                        exit(-1);
                    })
                    .unwrap(),
            );
        }
    } else if #[cfg(feature = "cli")] {
        lazy_static::lazy_static! {
            static ref ARGS: Arc<cli::Args> = Arc::new(cli::Args::parse());
        }
    }
}

#[tokio::main]
async fn main() {
    let version = version::get_version();
    let mut errs = 0;

    cfg_if::cfg_if! {
        if #[cfg(feature = "cli")] {
            let log_level = ARGS.log_level.unwrap_or_default();
        } else if #[cfg(feature = "config")] {
            let log_level = CONFIG.app.log_level;
        } else {
            let log_level = logger::LogLevel::default();
        }
    }

    logger::setup_logger(log_level).expect("unexpected error while starting the logger");

    info!("{}", version);
    info!("Using log level {:?}", log_level);

    let m_prog = progress::Progress::new();

    let file = get_file();
    let raw = parser::read_file::<CHUNK_SIZE, PathBuf>(&file).unwrap();
    let galleries = parser::get_all_galleries(&raw).unwrap();
    let gallery_prog = m_prog.add_prog(galleries.len() as u64, "Getting Galleries");

    #[cfg(feature = "metrics")]
    let mut download_totals = HashMap::new();

    info!("{} galleries to download", galleries.len());
    for gallery in galleries {
        info!("fetching data for {:?}", gallery);
        let gallery = extractor::get_gallery(&gallery, &m_prog).await;

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

        #[cfg(feature = "metrics")]
        match downloader::download_gallery::<CHUNK_SIZE>(&gallery, &m_prog).await {
            Ok(downloads) => {
                download_totals.insert(gallery, downloads);
            }
            Err(err) => {
                error!(
                    "Error while downloading gallery {1:?}: {0}\nFull Error:\n{0:#?}",
                    err,
                    gallery.title()
                );
                errs += 1;
            }
        }

        #[cfg(not(feature = "metrics"))]
        if let Err(err) = downloader::download_gallery::<CHUNK_SIZE>(&gallery, &m_prog) {
            error!(
                "Error while downloading gallery {1:?}: {0}\nFull Error:\n{0:#?}",
                err,
                gallery.title()
            );
            errs += 1;
        }

        gallery_prog.inc(1);
    }
    gallery_prog.finish_and_clear();

    if errs < 0 {
        eprintln!(
            "{} error(s) have occurred while downloading, check the logs for more info",
            errs
        );
        exit(errs);
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "metrics")] {
            let table = Table::new(
                Style::FancyPresto,
                download_totals.iter().map(|(gallery, sizes)| {
                    let sizes_total: usize = sizes.into_iter().sum();

                    vec![Cell::from(gallery.title()), Cell::from(&sizes_total.format_size(DECIMAL))]
                }).collect(),
                Some(Headers::from(vec!["Title", "Download Size"]))
            ).tabulate();

            println!("Downloaded the following Galleries:\n\n{}", table);
        }
    }
}

#[cfg(feature = "cli")]
fn get_file() -> PathBuf {
    let file = ARGS.links_file.clone();

    debug!("File Path is {}", &file.as_path().display());

    file
}

#[cfg(not(feature = "cli"))]
fn get_file() -> PathBuf {
    let raw_path = match args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("No file to read from was provided");
            exit(0x404)
        }
    };

    let file = PathBuf::from(raw_path);

    debug!("File Path is {}", &file.as_path().display());

    file
}
