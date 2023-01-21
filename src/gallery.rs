// This one is for when we start communicating in channels
#![allow(unused_must_use)]

use std::fmt::Debug;
use std::fs::read_to_string;
use std::io::ErrorKind;
use std::path::Path;
use std::string::FromUtf8Error;
use std::thread::spawn;

use crate::parsers::{get_galleries, get_tags, Tags};

use log::debug;
use reqwest::blocking::Client;

#[derive(Debug)]
pub enum GalleryError {
    OpenError(ErrorKind),
    ReadError(ErrorKind),
    DecodeError(FromUtf8Error),

    DownloadError(reqwest::Error),
}

#[derive(Debug)]
pub struct Gallery {
    url: String,
    images: Vec<String>,
    tags: Tags,
    client: Client,
}

impl Gallery {
    pub fn new(url: String) -> Result<Self, GalleryError> {
        Ok(Self {
            url,
            tags: Tags::new(),
            images: vec![],
            client: Client::new(),
        })
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
            let mut res = Gallery::new(gallery.clone()).unwrap();

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
