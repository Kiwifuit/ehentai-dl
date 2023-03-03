use std::fmt::Display;
use std::fs::{create_dir, OpenOptions};
use std::io::{self, Write};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::thread::{sleep, Builder};
use std::time::Duration;

use log::{debug, info, warn};
use regex::Regex;
use reqwest::blocking::{Client, Response};
use scraper::{error::SelectorErrorKind, ElementRef, Html, Selector};

const PAGINATION_REGEX: &str = r"Showing 1 - (\d+) of (\d+)";
const THREAD_SLEEP_DURATION: Duration = Duration::from_micros(20);
const CHUNK_SIZE: usize = 2048; // 2KB

#[derive(Debug)]
pub enum GalleryError<'a> {
    NetworkError(reqwest::Error),
    EmptyResponseError(reqwest::Error),
    SelectorParseError(SelectorErrorKind<'a>),
    RegexError(regex::Error),
    IoError(io::Error),
    NoCapture,
    ParseError(ParseIntError),
    EmptyDataError(String),
    ThreadError(usize, io::Error),
}

impl<'a> Display for GalleryError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::NetworkError(e) => format!("an error occurred while downloading data: {}", e),
                Self::EmptyDataError(e) => format!("{:?} was expected to be not empty, but is", e),
                Self::SelectorParseError(e) =>
                    format!("an error occurred while parsing scraper selectors: {}", e),
                Self::RegexError(e) =>
                    format!("an error occurred while trying to parse text: {}", e),
                Self::IoError(e) => format!(
                    "an error occurred while trying to read or write to a file: {}",
                    e
                ),
                Self::NoCapture =>
                    String::from("parser expected to capture something, got nothing"),
                Self::ParseError(e) => format!("the parser returned an error: {}", e),
                Self::EmptyResponseError(e) =>
                    format!("expected to have a response, got nothing but this: {}", e),
                Self::ThreadError(img_no, e) =>
                    format!("thread for image #{} returned an error: {}", img_no, e),
            }
        )
    }
}

#[derive(Debug)]
pub struct Gallery {
    name: String,
    total: usize,
    gallery: Vec<String>,
    client: Arc<Client>,
    thread_limit: u8,
    running_threads: Arc<AtomicU8>,
}

impl Gallery {
    pub fn new(thread_limit: u8) -> Self {
        let client = Arc::new(Client::new());
        let running_threads = Arc::new(AtomicU8::new(0));

        Self {
            client,
            gallery: vec![],
            total: 0,
            name: String::new(),
            thread_limit,
            running_threads,
        }
    }

    pub fn fetch_images<'a, U>(&mut self, url: &U) -> Result<(), GalleryError<'a>>
    where
        U: ToString + ?Sized,
    {
        let resp = get(&self.client, &url.to_string())?;

        debug!("GET {:?} => {}", resp.url(), resp.status());

        let content = Html::parse_fragment(
            resp.text()
                .map_err(|e| GalleryError::EmptyResponseError(e))?
                .as_str(),
        );

        let title = compile_selector("h1#gn")?;
        let gallery = compile_selector("#gdt .gdtm div a")?;
        let total_pages_raw = compile_selector("p.gpc")?;

        self.name = get_title(&content.select(&title).next())?;
        let total_pages = calculate_total_pages(&content.select(&total_pages_raw).next())?;

        info!(
            "Gallery {:?} has {} page(s) to download",
            self.name, total_pages
        );

        self.gallery = get_images(
            &total_pages,
            &url.to_string(),
            &gallery,
            &self.client,
            &self.thread_limit,
            &self.running_threads,
        )?;

        self.total = self.gallery.len();

        Ok(())
    }

    pub fn download_images(&self) -> Result<usize, GalleryError> {
        // TODO: Maybe make this function async? At least make this faster
        //       Multithreading is out of the question, since closure issues
        //       I forgot, but it's probably that.
        let mut total_downloaded = 0;
        let save_dir = PathBuf::from(format!("./{}", self.name));
        // Builder::new()
        //     .prefix(env!("CARGO_BIN_NAME"))
        //     .tempdir()
        //     .map_err(|e| GalleryError::IoError(e))?;

        if !save_dir.exists() {
            warn!("Path {:?} doesn't exist, creating new folder...", save_dir);
            create_dir(&save_dir).map_err(|e| GalleryError::IoError(e))?;
        }

        for image in &self.gallery {
            let resp = get(&self.client, image)?;

            let filename = save_dir.join(get_filename(&resp));

            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(filename.clone())
                .map_err(|e| GalleryError::IoError(e))?;

            for (chunk_no, chunk) in resp.bytes().unwrap().chunks(CHUNK_SIZE).enumerate() {
                total_downloaded += file.write(chunk).map_err(|e| GalleryError::IoError(e))?;

                info!("({}) Written chunk #{}", filename.display(), chunk_no);
            }

            info!("({}) finished downloading", filename.display());
            // resp.copy_to(&mut file);
        }

        info!("Written {} bytes total", total_downloaded);
        Ok(total_downloaded)
    }
}

fn get_filename(resp: &Response) -> &str {
    resp.url()
        .path_segments()
        .and_then(|seg| seg.last())
        .and_then(|name| if name.is_empty() { None } else { Some(name) })
        .unwrap_or("unknown.bin")
}

fn get_images<'a>(
    pages: &usize,
    base_url: &String,
    selector: &Selector,
    client: &Client,
    thread_limit: &u8,
    thread_counter: &Arc<AtomicU8>,
) -> Result<Vec<String>, GalleryError<'a>> {
    let mut images = vec![];
    let mut image_workers = vec![];
    let total_workers = Arc::clone(thread_counter);

    for i in 0..pages.clone() {
        let resp = get(client, &format!("{}?p={}", base_url, i))?;
        debug!("GET {:?} => {}", resp.url(), resp.status());

        let html = Html::parse_fragment(
            &resp
                .text()
                .map_err(|e| GalleryError::EmptyResponseError(e))?,
        );

        images.extend(
            html.select(selector)
                .map(|elem| elem.value().attr("href").unwrap().to_string()),
        );
    }

    info!("{} images to process", images.len());

    for (img_no, image) in images.iter().enumerate() {
        while total_workers.load(Ordering::Relaxed) >= *thread_limit {
            sleep(THREAD_SLEEP_DURATION);
            warn!(
                "Queue is full! Re-trying in {}ms",
                THREAD_SLEEP_DURATION.as_micros()
            );
        }

        let tw = total_workers.clone();
        let client = client.clone();
        let image = image.clone();
        let image_worker = Builder::new()
            .name(format!("image #{}", img_no))
            .spawn(move || {
                tw.fetch_add(1, Ordering::Acquire);

                let resp = get(&client, &image).expect("this URL to respond");
                debug!("GET {} => {}", resp.url(), resp.status());

                let html = Html::parse_fragment(
                    &resp
                        .text()
                        .expect("the response from the GET to be non-zero"),
                );

                let scraper = Selector::parse("img#img").expect("this selector to parse properly");
                tw.fetch_sub(1, Ordering::Acquire);

                if let Some(img) = html.select(&scraper).next() {
                    img.value().attr("src").unwrap().to_string()
                } else {
                    panic!("Expected to extract img, got nothing")
                }
            })
            .map_err(|e| GalleryError::ThreadError(img_no, e))?;

        image_workers.push(image_worker)
    }

    let mut res = vec![];
    for img_worker in image_workers {
        if let Ok(url) = img_worker.join() {
            res.push(url);
        }
    }

    Ok(res)
}

fn get<'a>(client: &Client, url: &String) -> Result<Response, GalleryError<'a>> {
    Ok(client
        .get(url)
        .send()
        .map_err(|e| GalleryError::NetworkError(e))?)
}

fn get_title<'a>(raw: &Option<ElementRef>) -> Result<String, GalleryError<'a>> {
    if let &Some(element) = raw {
        Ok(element.text().collect::<String>())
    } else {
        Err(GalleryError::EmptyDataError(String::from("title")))
    }
}

fn calculate_total_pages<'a>(raw: &Option<ElementRef>) -> Result<usize, GalleryError<'a>> {
    if let &Some(elememt) = raw {
        let raw = elememt.text().collect::<String>();
        let total = parse_pagination(raw)?;

        Ok(total)
    } else {
        Err(GalleryError::EmptyDataError(String::from("total-pages")))
    }
}

fn compile_selector<'a>(sel: &'a str) -> Result<Selector, GalleryError<'a>> {
    Selector::parse(sel).map_err(|e| GalleryError::SelectorParseError(e))
}

fn parse_pagination<'a>(raw: String) -> Result<usize, GalleryError<'a>> {
    let parser = Regex::new(PAGINATION_REGEX).map_err(|e| GalleryError::RegexError(e))?;
    let caps = parser
        .captures(raw.as_str())
        .ok_or(GalleryError::NoCapture)?;

    Ok(caps[2]
        .parse::<usize>()
        .map_err(|e| GalleryError::ParseError(e))?)
}
