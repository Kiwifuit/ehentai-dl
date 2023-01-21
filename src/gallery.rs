use std::io::prelude::*;
use std::path::Path;
use std::{fs, io, string};

pub enum GalleryError {
    OpenError(io::ErrorKind),
    ReadError(io::ErrorKind),
    DecodeError(string::FromUtf8Error),

    DownloadError(reqwest::Error),
}

pub struct Gallery {
    images: Vec<String>,
    client: reqwest::Client,
}

impl Gallery {
    pub fn new(images: Vec<String>) -> Result<Self, GalleryError> {
        Ok(Self {
            images,
            client: reqwest::Client::new(),
        })
    }
}

pub fn make_galleries<StrPath>(path: &StrPath) -> Result<Vec<Gallery>, String>
where
    StrPath: AsRef<Path> + ?Sized,
{
    let mut file = fs::OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|err| err.kind().to_string())?;

    let mut buf = vec![];
    file.read(&mut buf).map_err(|err| err.kind().to_string())?;

    let images = String::from_utf8(buf)
        .map_err(|err| err.to_string())?
        .split('\n')
        .map(|i| i.to_string())
        .collect::<Vec<String>>();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gallery_works() {
        let gallery = make_galleries("./res/galleries.txt");

        assert!(gallery.is_ok())
    }
}
