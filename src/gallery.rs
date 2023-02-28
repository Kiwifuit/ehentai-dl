use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::thread::{sleep, spawn};
use std::time::Duration;
use std::{io::ErrorKind, num::ParseIntError};

use log::{debug, info, warn};
use regex::Regex;
use reqwest::blocking::{Client, Response};
use scraper::{error::SelectorErrorKind, ElementRef, Html, Selector};

const PAGINATION_REGEX: &str = r"Showing 1 - (\d+) of (\d+)";

#[derive(Debug)]
pub enum GalleryError<'a> {
    NetworkError(reqwest::Error),
    EmptyResponseError(reqwest::Error),
    IoError(ErrorKind),
    SelectorParseError(SelectorErrorKind<'a>),
    RegexError(regex::Error),
    NoCapture,
    ParseError(ParseIntError),
    EmptyError,
}

#[derive(Debug)]
pub struct Gallery {
    name: String,
    total: usize,
    gallery: Vec<String>,
    client: Client,
}

impl Gallery {
    pub fn new<'a>(url: String, thread_limit: u8) -> Result<Self, GalleryError<'a>> {
        let client = Client::new();
        let resp = get_page(&client, &url)?;

        debug!("GET {:?} => {}", resp.url(), resp.status());

        let content = Html::parse_fragment(
            resp.text()
                .map_err(|e| GalleryError::EmptyResponseError(e))?
                .as_str(),
        );

        let title = compile_selector("h1#gn")?;
        let gallery = compile_selector("#gdt .gdtm div a")?;
        let total_pages_raw = compile_selector("p.gpc")?;

        let title = get_title(&content.select(&title).next())?;
        let total_pages = calculate_total_pages(&content.select(&total_pages_raw).next())?;

        info!(
            "Gallery {:?} has {} page(s) to download",
            title, total_pages
        );

        let images = get_images(&total_pages, &url, &gallery, &client, thread_limit)?;

        Ok(Self {
            client,
            gallery: images.clone(),
            total: images.len(),
            name: title,
        })
    }
}

fn get_images<'a>(
    pages: &usize,
    base_url: &String,
    selector: &Selector,
    client: &Client,
    thread_limit: u8,
) -> Result<Vec<String>, GalleryError<'a>> {
    let mut images = vec![];
    let mut image_workers = vec![];
    let total_workers = Arc::new(AtomicU8::new(0));

    for i in 0..pages.clone() {
        let resp = get_page(client, &format!("{}?p={}", base_url, i))?;
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

    for image in images {
        while total_workers.load(Ordering::Relaxed) >= thread_limit {
            sleep(Duration::from_micros(20));
            warn!("Queue is full! Re-trying in 20ms");
        }

        let tw = total_workers.clone();
        let client = client.clone();
        let image = image.clone();

        image_workers.push(spawn(move || {
            tw.fetch_add(1, Ordering::Acquire);

            let resp = get_page(&client, &image).expect("this URL to respond");
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
        }));
    }

    let mut res = vec![];
    for img_worker in image_workers {
        if let Ok(url) = img_worker.join() {
            res.push(url);
        }
    }

    Ok(res)
}

fn get_page<'a>(client: &Client, url: &String) -> Result<Response, GalleryError<'a>> {
    Ok(client
        .get(url)
        .send()
        .map_err(|e| GalleryError::NetworkError(e))?)
}

fn get_title<'a>(raw: &Option<ElementRef>) -> Result<String, GalleryError<'a>> {
    if let &Some(element) = raw {
        Ok(element.text().collect::<String>())
    } else {
        Err(GalleryError::EmptyError)
    }
}

fn calculate_total_pages<'a>(raw: &Option<ElementRef>) -> Result<usize, GalleryError<'a>> {
    if let &Some(elememt) = raw {
        let raw = elememt.text().collect::<String>();
        let total = parse_pagination(raw)?;

        Ok(total)
    } else {
        Err(GalleryError::EmptyError)
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
