use std::collections;

const TAG_PARSER: &str = r"https://e-hentai\.org/tag/(?P<name>.*):(?P<value>.*)";

pub enum TagError {
    RegExParseError(regex::Error),
}

pub struct Tags {
    tags: collections::HashMap<String, String>,
}

impl Tags {
    pub fn new() -> Self {
        Self {
            tags: collections::HashMap::new(),
        }
    }

    pub fn feed_tags(&mut self, content: &String) -> Result<(), TagError> {
        let parser = regex::Regex::new(TAG_PARSER).map_err(|e| TagError::RegExParseError(e))?;

        for cap in parser.captures_iter(content) {
            self.tags
                .insert(cap["name"].to_string(), cap["value"].to_string());
        }

        Ok(())
    }
}
