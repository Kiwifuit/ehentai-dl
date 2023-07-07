cfg_if::cfg_if! {
    if #[cfg(feature = "aniyomi")] {
        use std::fs::{OpenOptions};
        use crate::aniyomi::*;
    } else {
        use std::fs::{create_dir, OpenOptions};
    }
}

use std::fmt::Display;
use std::io::prelude::*;
use std::path::PathBuf;

use futures_util::StreamExt;
use indicatif::ProgressStyle;
use log::{debug, info};
use reqwest::get;

use crate::gallery::{Gallery, Image};
use crate::progress::Progress;

#[cfg(feature = "zip")]
use crate::zip;

#[cfg(feature = "zip")]
use std::fs::remove_dir_all;

const PROGBAR_STYLE: &str = "{prefix:<50} [{bar:>50}] {msg} {bytes}/{total_bytes}";
const TITLE_DISPLAY_LENGTH: usize = 16;

cfg_if::cfg_if! {
    if #[cfg(feature = "metrics")] {
        type DownloadedImage = (usize, PathBuf);
        type DownloadResponse = Vec<usize>;
    } else {
        type DownloadedImage = PathBuf;
        type DownloadResponse = ();
    }
}

#[derive(Debug)]
pub enum DownloadError {
    NetworkError(reqwest::Error),
    FileSystemError(std::io::Error),
    ChunkError(reqwest::Error),
    WriteError(std::io::Error),
    AddDirError(std::io::Error),

    #[cfg(feature = "zip")]
    ZipError(zip::ZipError),

    #[cfg(feature = "zip")]
    RemoveDirError(PathBuf, std::io::Error),
}

impl Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "error while {}",
            match self {
                Self::NetworkError(e) => format!("performing request: {}", e),
                Self::FileSystemError(e) => format!("reading/writing to the filesystem: {}", e),
                Self::ChunkError(e) => format!("awaiting next chunk: {}", e),
                Self::WriteError(e) => format!("writing to file: {}", e),
                Self::AddDirError(e) => format!("while creating directory: {}", e),

                #[cfg(feature = "zip")]
                Self::ZipError(e) => format!("zipping content: {}", e),
                #[cfg(feature = "zip")]
                Self::RemoveDirError(p, e) => format!("removing directory {:?}: {}", p, e),
            }
        )
    }
}

async fn download_image(
    image: &Image,
    parent_dir: &PathBuf,
    m_prog: &Progress,
) -> Result<DownloadedImage, DownloadError> {
    let resp = get(image.get_url())
        .await
        .map_err(|e| DownloadError::NetworkError(e))?;

    let content_length = resp.content_length().unwrap();
    let mut stream = resp.bytes_stream();

    let save_path = parent_dir.join(image.get_filename());
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&save_path)
        .map_err(|e| DownloadError::FileSystemError(e))?;
    let mut downloaded = 0;

    let download_prog = m_prog.add_custom_prog(
        content_length,
        format!("Downloading {}", try_truncate(image.get_filename())),
        ProgressStyle::with_template(PROGBAR_STYLE).unwrap(),
    );

    while let Some(chunk) = stream.next().await {
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

    cfg_if::cfg_if! {
        if #[cfg(feature = "metrics")] {
            Ok((downloaded, save_path))
        } else {
            Ok(save_path)
        }
    }
}

pub async fn download_gallery<const CHUNK_SIZE: usize>(
    gallery: &Gallery,
    m_prog: &Progress,
) -> Result<DownloadResponse, DownloadError> {
    let cwd = PathBuf::from(".");
    let root_dir = if cfg!(feature = "aniyomi") {
        let cwd = cwd.join(gallery.title());
        create_dir(&cwd).map_err(|e| DownloadError::AddDirError(e))?;

        cwd.join("OneShot")
    } else {
        cwd.join(gallery.title())
    };

    info!("Current Dir: {:?}", root_dir);
    debug!("Gallery: {:?}", &gallery);

    let total = if cfg!(feature = "aniyomi") {
        gallery.len() as u64 + 1
    } else {
        gallery.len() as u64
    };
    let download_prog = m_prog.add_prog(total, "Downloading images");

    #[cfg(feature = "metrics")]
    let mut dl_sizes = vec![];
    let mut dl_files = vec![];

    create_dir(&root_dir).map_err(|e| DownloadError::FileSystemError(e))?;

    for image in gallery.images() {
        let (dl_size, dl_path) = download_image(image, &root_dir, &m_prog).await?;

        #[cfg(feature = "metrics")]
        dl_sizes.push(dl_size);
        dl_files.push(dl_path);
        download_prog.inc(1);
    }

    // cfg! only evaluates to true or false,
    // we're not actually including or excluding
    // code when we use the cfg! macro.
    //
    // "cfg!, unlike #[cfg], does not remove any
    // code and only evaluates to true or false"
    cfg_if::cfg_if! {
        // Very hard to read but I wish that I
        // could make this better
        if #[cfg(feature = "aniyomi")] { // This evaluates on compile time

            // cannot nest cfg_if blocks within cfg_if blocks
            #[cfg(feature = "config")]
            let use_aniyomi = crate::CONFIG.app.features.contains(&String::from("aniyomi"));
            #[cfg(not(feature = "config"))]
            let use_aniyomi = true;

            if use_aniyomi { // This *sorta* evaluates on runtime
                download_prog.set_message("Finishing Touches");
                let meta = AniyomiMeta::from(gallery);
                let meta_path = root_dir.with_file_name("details.json");

                let mut meta_file = OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(&meta_path)
                    .map_err(|e| DownloadError::FileSystemError(e))?;

                info!("Writing aniyomi meta to {:?}", &meta_path);
                to_json_file(&mut meta_file, &meta).map_err(|e| DownloadError::WriteError(e))?;
                let cover_file = make_cover(dl_files.get(0).unwrap()).map_err(|e| DownloadError::WriteError(e))?;

                dl_files.push(meta_path);
                dl_files.push(cover_file);
            }
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "zip")] {
            #[cfg(feature = "config")]
            let use_zip = crate::CONFIG.app.features.contains(&String::from("zip"));
            #[cfg(not(feature = "config"))]
            let use_zip = true;

            #[cfg(feature = "config")]
            let zip_delete_orig = crate::CONFIG.zip.delete_original;
            #[cfg(feature = "cli")]
            let zip_delete_orig = crate::ARGS.delete_original.unwrap_or(false);
            #[cfg(not(any(feature = "config", feature = "cli")))]
            let zip_delete_orig = false;

            if use_zip {
                let zip_prog = m_prog.add_prog(dl_files.len() as u64 + 1, format!("Zipping Gallery {:?}", gallery.title()));
                let mut zip_file = zip::make_zip(&format!("{}.zip", gallery.title())).map_err(|e| DownloadError::ZipError(e))?;

                let rd_prog = m_prog.add_prog(1, "Root directory");
                zip::add_file::<PathBuf, CHUNK_SIZE>(&mut zip_file, &root_dir).map_err(|e| DownloadError::ZipError(e))?;
                rd_prog.finish_and_clear();

                for file in dl_files {
                    // why. just why
                    // what was i trying to achieve by
                    // passing in `&root_dir` in the previous commits
                    let written = zip::add_file::<PathBuf, CHUNK_SIZE>(&mut zip_file, &file).map_err(|e| DownloadError::ZipError(e))?;

                    info!("Written file {:?} to disc ({} bytes written)", file.to_str().unwrap(), written);
                    zip_prog.inc(1);
                }
                zip_prog.finish_and_clear();

                if zip_delete_orig {
                    let root = cwd.join(gallery.title());
                    remove_dir_all(&root).map_err(|e| DownloadError::RemoveDirError(root, e))?;
                }
            }
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "metrics")] {
            Ok(dl_sizes)
        } else {
            Ok(())
        }
    }
}

fn try_truncate(raw: &String) -> String {
    let mut raw = raw.clone();

    if TITLE_DISPLAY_LENGTH <= raw.len() {
        raw.truncate(TITLE_DISPLAY_LENGTH - 3);
    }

    raw
}
