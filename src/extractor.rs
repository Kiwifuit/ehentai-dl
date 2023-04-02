use std::{fmt::Display, string};

use log::debug;
use reqwest::{blocking::get, IntoUrl};
use scraper::{Html, Selector};

const TITLE_TAG: &str = "div.gm div#gd2 h1#gn";
const FILENAME_TAG: &str = "div#i1.sni div#i4 div";
const IMAGE_TAG: &str = "div#i1.sni div#i3 a img#img";
const IMAGE_LINK_EXTRACTOR: &str = r"https://e-hentai\.org/s/([a-z0-9]{10})/([0-9]{7})-(\d+)";
const GALLERY_TAG_TAG: &str = "div#taglist table tbody tr";

#[derive(Debug)]
pub enum ExtractionError {
    NetworkError(reqwest::Error),
    BytesDecodeError(reqwest::Error),
    StringDecodeError(string::FromUtf8Error),
}

pub fn get_html<U>(url: U) -> Result<Html, ExtractionError>
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
