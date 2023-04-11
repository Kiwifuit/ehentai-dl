cfg_if::cfg_if! {
    if #[cfg(feature = "aniyomi")] {
        use std::fs::{OpenOptions};
        use crate::aniyomi::create_dir;
    } else {
        use std::fs::{create_dir, OpenOptions};
    }
}

use std::io::prelude::*;
use std::path::PathBuf;

use futures_util::StreamExt;
use indicatif::ProgressStyle;
use log::debug;
use reqwest::get;

use crate::gallery::{Gallery, Image};
use crate::progress::Progress;

const PROGBAR_STYLE: &str = "{prefix:<50} [{bar:>50}] {msg} {bytes}/{total_bytes}";

#[derive(Debug)]
pub enum DownloadError {
    NetworkError(reqwest::Error),
    FileSystemError(std::io::Error),
    ChunkError(reqwest::Error),
    WriteError(std::io::Error),
}

async fn download_image(
    image: &Image,
    parent_dir: &PathBuf,
    m_prog: &Progress,
) -> Result<usize, DownloadError> {
    let resp = get(image.get_url())
        .await
        .map_err(|e| DownloadError::NetworkError(e))?;

    let content_length = resp.content_length().unwrap();
    let mut resp = resp.bytes_stream();

    let save_path = parent_dir.join(image.get_filename());
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&save_path)
        .map_err(|e| DownloadError::FileSystemError(e))?;
    let mut downloaded = 0;

    let download_prog = m_prog.add_custom_prog(
        content_length,
        format!("Downloading {}", image.get_filename()),
        ProgressStyle::with_template(PROGBAR_STYLE)
            .unwrap()
            .progress_chars("█▓▒░"),
    );

    while let Some(chunk) = resp.next().await {
        let chunk = chunk.map_err(|e| DownloadError::ChunkError(e))?;
        downloaded += chunk.len();

        file.write(&chunk)
            .map_err(|e| DownloadError::WriteError(e))?;
        download_prog.inc(chunk.len() as u64);
    }

    debug!(
        "Written {} bytes total to {}",
        downloaded,
        save_path.file_name().unwrap().to_str().unwrap()
    );
    Ok(downloaded)
}

pub fn download_gallery(gallery: &Gallery, m_prog: &Progress) -> Result<Vec<usize>, DownloadError> {
    let this_dir = PathBuf::from(".");
    let root_dir = this_dir.join(gallery.title());
    let download_prog = m_prog.add_prog(gallery.len() as u64, "Downloading images");
    let mut download_sizes = vec![];

    create_dir(&root_dir).map_err(|e| DownloadError::FileSystemError(e))?;
    for image in gallery.images() {
        let downloaded = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(download_image(image, &root_dir, &m_prog))?;
        download_sizes.push(downloaded);
        download_prog.inc(1);
    }

    Ok(download_sizes)
}
