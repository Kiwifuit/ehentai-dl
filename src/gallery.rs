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
    pub fn new<StrPath: AsRef<Path> + ?Sized>(path: &StrPath) -> Result<Self, GalleryError> {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|err| GalleryError::OpenError(err.kind()))?;

        let mut buf = vec![];
        file.read(&mut buf)
            .map_err(|err| GalleryError::ReadError(err.kind()))?;

        let images = String::from_utf8(buf)
            .map_err(|err| GalleryError::DecodeError(err))?
            .split('\n')
            .map(|i| i.to_string())
            .collect::<Vec<String>>();

        Ok(Self {
            images,
            client: reqwest::Client::new(),
        })
    }

    pub fn download_all(&self) -> Result<usize, ()> {}

    async fn download_image(&self, img: &String) -> Result<usize, GalleryError> {
        let mut total_downloaded: usize = 0;
        let content = self
            .client
            .get(img)
            .send()
            .await
            .map_err(|e| GalleryError::DownloadError(e))?;

        total_downloaded += content.content_length().or(Some(0)).unwrap() as usize;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gallery_works() {
        let gallery = Gallery::new("./res/galleries.txt");

        assert!(gallery.is_ok())
    }
}
