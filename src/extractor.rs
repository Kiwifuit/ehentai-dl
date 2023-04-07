use std::{
    fmt::{Debug, Display},
    string,
};

use log::{debug, info};
use reqwest::{blocking::get, IntoUrl};
use scraper::Html;

use crate::{gallery, progress::Progress};

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

pub fn get_gallery<'a, U>(
    url: &'a U,
    progress: &Progress,
) -> Result<gallery::Gallery, ExtractionError<'a>>
where
    U: Debug + Display + ToString + ?Sized,
    &'a U: IntoUrl + 'a,
{
    let mut gallery = gallery::Gallery::new();
    let overall_progress = progress.add_prog(3, "Getting info for gallery");

    info!("Extracting info for {:?}", url);
    let html = get_html(url)?;

    let pages = get_pages(&html)?;
    info!("{} page(s) to download", pages);

    overall_progress.set_message("title");
    get_title(&mut gallery, &html)?;
    overall_progress.inc(1);

    overall_progress.set_message("tags");
    get_tags(&mut gallery, &html)?;
    overall_progress.inc(1);

    overall_progress.set_message("images");
    get_images(&pages, &url.to_string(), &mut gallery, &progress)?;
    overall_progress.inc(1);

    overall_progress.set_message("");
    overall_progress.finish();

    Ok(gallery)
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

fn get_pages<'a>(html: &Html) -> Result<u8, ExtractionError<'a>> {
    let sel = compile!(selector "p.gpc")?;
    let pages_raw = html
        .select(&sel)
        .next()
        .ok_or(ExtractionError::EmptyData("page count"))?;

    let pagination = crate::parser::get_pagination(&pages_raw.text().collect::<String>())
        .map_err(|e| ExtractionError::DataParseError(e))?;

    // we use .ceil instead of .round because we want even 1.1 to
    // get considered as 2.0, .ceil achieves this function while
    // .round, well, rounds it off to 1.0
    Ok(pagination.ceil() as u8)
}

pub fn get_images<'a>(
    pages: &u8,
    gallery_url: &String,
    gallery: &mut gallery::Gallery,
    progress: &Progress,
) -> Result<(), ExtractionError<'a>> {
    let sel = compile!(selector "div#gdt div.gdtm div a")?;

    for i in 0..*pages {
        let url = format!("{}?p={}", gallery_url, i);
        let html = get_html(url)?;
        let images = html.select(&sel).collect::<Vec<scraper::ElementRef>>();

        let prog = progress.add_prog(
            images.len() as u64,
            format!("Extracting image data (page {} of {})", i + 1, pages),
        );

        for image in images {
            let url = image.value().attr("href").unwrap().to_string();
            let mut image = gallery::Image::new(&url);

            get_image_data(&mut image)?;
            gallery.add_image(image);
            prog.inc(1);
        }
        prog.finish();
    }

    Ok(())
}

pub fn get_image_data<'a>(image: &mut gallery::Image) -> Result<(), ExtractionError<'a>> {
    let html = get_html(image.get_url())?;
    let filename = compile! { selector "div#i2 div" }?;
    let image_url = compile! { selector "div#i3 a img" }?;
    let url = html
        .select(&image_url)
        .nth(0)
        .unwrap()
        .value()
        .attr("src")
        .unwrap()
        .to_string();

    image.set_url(url);

    let filename_raw = html
        .select(&filename)
        .nth(2) // the 1st is the nav bar
        .ok_or(ExtractionError::EmptyData("filename"))?
        .text()
        .collect::<String>();

    let filename = crate::parser::get_filename(&filename_raw)
        .map_err(|e| ExtractionError::DataParseError(e))?;

    image.set_filename(filename);
    Ok(())
}

pub fn get_tags<'a>(
    gallery: &mut gallery::Gallery,
    html: &Html,
) -> Result<(), ExtractionError<'a>> {
    let tag_types = compile! { selector "div#taglist table tbody tr" }?;
    let tag_name = compile! { selector "tr td" }?;
    let tag_value = compile! { selector "tr td div a" }?;

    for tag_type in html.select(&tag_types) {
        let mut tag = tag_type.select(&tag_name);
        let tag_name = tag
            .next()
            .unwrap()
            .text()
            .map(|c| &c[..c.len() - 1])
            .collect::<String>();

        for raw in tag {
            let tag_value = raw.select(&tag_value).nth(0).unwrap().text().collect();
            gallery.add_tag(tag_name.clone(), tag_value);
        }
    }

    Ok(())
}
