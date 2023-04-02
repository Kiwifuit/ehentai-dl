use std::{fmt::Display, string};

use log::debug;
use reqwest::{blocking::get, IntoUrl};
use scraper::{Html, Selector};

use crate::gallery;

const FILENAME_TAG: &str = "div#i1.sni div#i4 div";
const IMAGE_TAG: &str = "div#i1.sni div#i3 a img#img";
const IMAGE_LINK_EXTRACTOR: &str = r"https://e-hentai\.org/s/([a-z0-9]{10})/([0-9]{7})-(\d+)";
const GALLERY_TAG_TAG: &str = "div#taglist table tbody tr";

#[derive(Debug)]
pub enum ExtractionError<'a> {
    NetworkError(reqwest::Error),
    BytesDecodeError(reqwest::Error),
    StringDecodeError(string::FromUtf8Error),
    SelectorParseError(scraper::error::SelectorErrorKind<'a>),
    EmptyData(&'a str),
    DataParseError(crate::parser::ParseError<0>),
}

pub fn get_html<'a, U>(url: U) -> Result<Html, ExtractionError<'a>>
where
    U: IntoUrl + Display + Clone,
{
    let resp = get(url.clone()).map_err(|e| ExtractionError::NetworkError(e))?;
    debug!("GET {} => {}", url, resp.status());

    let bytes = resp
        .bytes()
        .map_err(|e| ExtractionError::BytesDecodeError(e))?
        .iter()
        .map(|i| *i)
        .collect();
    let body = String::from_utf8(bytes).map_err(|e| ExtractionError::StringDecodeError(e))?;
    let html = Html::parse_fragment(body.as_str());

    Ok(html)
}

pub fn get_title<'a>(
    gallery: &mut gallery::Gallery,
    html: &Html,
) -> Result<(), ExtractionError<'a>> {
    let sel = compile!(selector "h1#gn")?;

    if let Some(title) = html.select(&sel).next() {
        gallery.set_title(title.text().collect::<String>());

        Ok(())
    } else {
        Err(ExtractionError::EmptyData("title"))
    }
}

pub fn get_pages<'a>(html: &Html) -> Result<u8, ExtractionError<'a>> {
    let sel = compile!(selector "p.gpc")?;
    let pages_raw = html
        .select(&sel)
        .next()
        .ok_or(ExtractionError::EmptyData("page count"))?;

    Ok(
        crate::parser::get_pagination(&pages_raw.text().collect::<String>())
            .map_err(|e| ExtractionError::DataParseError(e))?,
    )
}

pub fn get_images<'a>(
    pages: &u8,
    gallery_url: &String,
) -> Result<Vec<gallery::Image>, ExtractionError<'a>> {
    let mut images = vec![];
    let sel = compile!(selector "#gdt .gdtm div a")?;

    for i in 1..*pages {
        let url = format!("{}?p={}", gallery_url, i);
        let html = get_html(url)?;

        images.extend(html.select(&sel).map(|e| gallery::Image::from(e)));
    }

    Ok(images)
}

pub fn get_image_filename<'a>(image: &mut gallery::Image) -> Result<(), ExtractionError<'a>> {
    let html = get_html(image.get_url())?;
    let sel = compile! { selector "div#i2 div" }?;

    let filename_raw = html
        .select(&sel)
        .nth(1) // the 0th is the nav bar
        .ok_or(ExtractionError::EmptyData("filename"))?
        .text()
        .collect::<String>();

    let filename = crate::parser::get_filename(&filename_raw)
        .map_err(|e| ExtractionError::DataParseError(e))?;

    image.set_filename(filename);
    Ok(())
}
