use std::collections;

const TAG_PARSER: &str = r"https://e-hentai\.org/tag/(?P<name>\w*):(?P<value>[\w+]*)";
const IMAGE_PARSER: &str = r"https://e-hentai\.org/s/([a-z0-9]{10})/([0-9]{7})-(\d+)";

// Don't really know how to name this constant.
// The one above filters the "image viewer", while
// this one filters the image itself
//
// The way of extracting the image is this:
// Gallery -> Gallery Image Viewer -> Image
const IMAGE_FILE_PARSER: &str = r"(?P<image>https://\w{7}.\w{12}.hath.network(:\d{5}/|/)h/.*\.jpg)";

#[derive(Debug)]
pub struct Tags {
    tags: collections::HashMap<String, String>,
}

impl Tags {
    pub fn new() -> Self {
        Self {
            tags: collections::HashMap::new(),
        }
    }

    pub fn add_tag(&mut self, name: String, value: String) {
        self.tags.insert(name, value);
    }
}

pub fn get_tags(content: &String) -> Result<collections::HashMap<String, String>, String> {
    let parser =
        regex::Regex::new(TAG_PARSER).map_err(|e| format!("Unable to compile parser: {}", e))?;
    let mut tags = collections::HashMap::new();

    for cap in parser.captures_iter(content) {
        tags.insert(cap["name"].to_string(), cap["value"].replace("+", " "));
    }

    Ok(tags)
}

pub fn get_galleries(content: &String) -> Result<Vec<String>, regex::Error> {
    let parser = regex::Regex::new(IMAGE_PARSER)?;
    let mut images = vec![];

    for cap in parser.captures_iter(content) {
        images.push(cap.get(0).unwrap().as_str().to_string())
    }

    Ok(images)
}

pub fn get_images(content: &String) -> Result<Vec<String>, regex::Error> {
    let parser = regex::Regex::new(IMAGE_FILE_PARSER)?;
    let mut images = vec![];

    for cap in parser.captures_iter(content) {
        images.push(cap.get(0).unwrap().as_str().to_string())
    }

    Ok(images)
}
