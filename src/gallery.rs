// This one is for when we start communicating in channels
#![allow(unused_must_use)]

use std::fmt::Debug;
use std::fs::read_to_string;
use std::io::{ErrorKind, Write};
use std::path::Path;
use std::thread::spawn;

use crate::parsers::{get_galleries, get_images, get_tags, Tags};

use log::{debug, info, trace};
use reqwest::blocking::Client;
use tempfile::NamedTempFile;
use termprogress::prelude::*;

const CHUNK_SIZE: usize = 1024;

#[derive(Debug)]
pub enum GalleryError {
    WriteError(ErrorKind),
    SearchError(regex::Error),

    TempDirError(ErrorKind),
    NetworkError(reqwest::Error),
}

#[derive(Debug)]
pub struct Gallery {
    images: Vec<String>,
    tags: Tags,
    client: Client,
}

impl Gallery {
    pub fn new() -> Self {
        Self {
            tags: Tags::new(),
            images: vec![],
            client: Client::new(),
        }
    }

    pub fn download_images(&self) -> Result<usize, GalleryError> {
        let mut total = 0;
        let mut progress = Bar::default();

        progress.set_title(format!("Downloading {} images", self.images.len()).as_str());
        for image in &self.images {
            let resp = self
                .client
                .get(image)
                .send()
                .map_err(|e| GalleryError::NetworkError(e))?;

            let content = resp.text().map_err(|e| GalleryError::NetworkError(e))?;
            let images = get_images(&content).map_err(|e| GalleryError::SearchError(e))?;

            for image in images {
                info!("Downloading {:?}", image);

                total += self.download_image(&image)?;
            }
        }

        Ok(total)
    }

    fn download_image(&self, image: &String) -> Result<usize, GalleryError> {
        let mut savefile = NamedTempFile::new_in(
            self.tags
                .get_tag("name")
                .unwrap_or(&String::from("unknown")),
        )
        .map_err(|e| GalleryError::TempDirError(e.kind()))?;

        info!("Created file on {:?}", savefile);

        let resp = self
            .client
            .get(image)
            .send()
            .map_err(|e| GalleryError::NetworkError(e))?;

        debug!("DOWNLOAD GET {} => {}", resp.url(), resp.status());

        let len = resp.content_length().unwrap_or(0) as usize;
        let bytes = resp.bytes().map_err(|e| GalleryError::NetworkError(e))?;
        let mut progress = Bar::default();
        let mut downloaded = 0;

        debug!("{len} bytes to write to {savefile:?}");
        for chunk in bytes.chunks(CHUNK_SIZE) {
            downloaded += savefile
                .write(chunk)
                .map_err(|e| GalleryError::WriteError(e.kind()))?;

            progress.set_progress(downloaded as f64 / bytes.len() as f64);
            trace!("{}%", downloaded as f64 / bytes.len() as f64)
        }

        Ok(len)
    }
}

pub fn make_galleries<StrPath>(path: &StrPath) -> Result<Vec<Gallery>, String>
where
    StrPath: AsRef<Path> + Debug + ?Sized,
{
    let buf = read_to_string(path).map_err(|e| e.kind().to_string())?;

    let galleries = buf
        .split("\n")
        .map(|i| i.to_string())
        .collect::<Vec<String>>();

    debug!("{} galleries to get", galleries.len());
    let client = Client::new();
    let mut threads = vec![];
    for gallery in galleries {
        if gallery == String::new() {
            continue;
        }

        debug!("On gallery {:?}", gallery);
        let client = client.clone();

        threads.push(spawn(move || {
            let gallery_content = client.get(gallery.clone()).send().unwrap();
            let mut res = Gallery::new();

            debug!("GET {} => {}", gallery, gallery_content.status());
            let html = gallery_content.text().unwrap();

            debug!("Extracting tags");
            let tags = get_tags(&html).unwrap();

            for tag in tags.keys() {
                let val = tags.get(tag).unwrap();

                debug!("Tag {}: {}", tag, val);
                res.tags.add_tag(tag.clone(), val.clone());
            }

            debug!("Extracting Images");

            let images = get_galleries(&html).unwrap();

            for image in images {
                debug!("Image {:?}", image);
                res.images.push(image);
            }

            res
        }));
    }

    let mut galleries = vec![];
    for thread in threads {
        if let Ok(gallery) = thread.join() {
            galleries.push(gallery);
        }
    }
    Ok(galleries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gallery_works() {
        let gallery = make_galleries("./res/galleries.txt");

        assert!(gallery.is_ok());
        let gallery = gallery.unwrap();

        assert_eq!(gallery.len(), 55);
    }
}
