use std::collections;

const TAG_PARSER: &str = r"https://e-hentai\.org/tag/(?P<name>.*):(?P<value>.*)";
const IMAGE_PARSER: &str = r"https://e-hentai\.org/s/([a-z0-9]{10})/([0-9]{7})-(\d+)";

pub enum TagError {
    RegExParseError(regex::Error),
}

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

pub fn get_tags(content: &String) -> Result<collections::HashMap<String, String>, regex::Error> {
    let parser = regex::Regex::new(TAG_PARSER)?;
    let mut tags = collections::HashMap::new();

    for cap in parser.captures_iter(content) {
        tags.insert(cap["name"].to_string(), cap["value"].to_string());
    }

    Ok(tags)
}

pub fn get_images(content: &String) -> Result<Vec<String>, regex::Error> {
    let parser = regex::Regex::new(IMAGE_PARSER)?;
    let mut images = vec![];

    for cap in parser.captures_iter(content) {
        images.push(cap.get(0).unwrap().as_str().to_string())
    }

    Ok(images)
}
