use std::{io::ErrorKind, num::ParseIntError};

use log::{debug, info};
use regex::Regex;
use reqwest::blocking::{Client, Response};
use scraper::{error::SelectorErrorKind, ElementRef, Html, Selector};

const PAGINATION_REGEX: &str = r"Showing 1 - (\d+) of (\d+)";

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

pub struct Gallery {
    name: String,
    total: usize,
    gallery: Vec<String>,
    client: Client,
}

impl Gallery {
    pub fn new<'a>(url: String) -> Result<Self, GalleryError<'a>> {
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

        let images = get_images(&total_pages, &url, &gallery, &client)?;

        Ok(Self {
            client,
            gallery: images,
            total: total_pages,
            name: title,
        })
    }
}

fn get_images<'a>(
    pages: &usize,
    base_url: &String,
    selector: &Selector,
    client: &Client,
) -> Result<Vec<String>, GalleryError<'a>> {
    let mut res = vec![];

    for i in 0..pages.clone() {
        let resp = get_page(client, &format!("{}?p={}", base_url, i))?;
        debug!("GET {:?} => {}", resp.url(), resp.status());

        let html = Html::parse_fragment(
            &resp
                .text()
                .map_err(|e| GalleryError::EmptyResponseError(e))?,
        );

        res.extend(
            html.select(selector)
                .map(|elem| elem.value().attr("href").unwrap().to_string()),
        );
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
        let (rendered, total) = parse_pagination(raw)?;

        Ok(rendered / total)
    } else {
        Err(GalleryError::EmptyError)
    }
}

fn compile_selector<'a>(sel: &'a str) -> Result<Selector, GalleryError<'a>> {
    Selector::parse(sel).map_err(|e| GalleryError::SelectorParseError(e))
}

fn parse_pagination<'a>(raw: String) -> Result<(usize, usize), GalleryError<'a>> {
    let parser = Regex::new(PAGINATION_REGEX).map_err(|e| GalleryError::RegexError(e))?;
    let caps = parser
        .captures(raw.as_str())
        .ok_or(GalleryError::NoCapture)?;

    Ok((
        caps[1]
            .parse::<usize>()
            .map_err(|e| GalleryError::ParseError(e))?,
        caps[2]
            .parse::<usize>()
            .map_err(|e| GalleryError::ParseError(e))?,
    ))
}
