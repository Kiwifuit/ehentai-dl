use std::slice::Iter;

use scraper::ElementRef;

type Images<'a> = Iter<'a, Image>;

#[derive(Debug)]
pub struct Gallery {
    title: String,
    image_count: u8,
    images: Vec<Image>,
    tags: Tags,
}

#[derive(Debug)]
pub struct Image {
    url: String,
    file: String,
}

#[derive(Debug, Clone)]
pub struct Tag {
    t_type: TagType,
    t_val: String,
}

#[derive(Debug, Clone)]
pub struct Tags {
    _inner: Vec<Tag>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TagType {
    ReClass,
    Parody,
    Character,
    Language,
    Artist,
    Male,
    Female,
    Other,
}

impl Gallery {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            image_count: 0,
            images: vec![],
            tags: Tags::new(),
        }
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn add_image(&mut self, image: Image) {
        self.image_count += 1;
        self.images.push(image);
    }

    pub fn add_tag(&mut self, name: String, value: String) {
        let tag = Tag {
            t_type: name.into(),
            t_val: value,
        };

        self.tags.push(tag);
    }

    pub fn title(&self) -> &String {
        &self.title
    }

    pub const fn len(&self) -> u8 {
        self.image_count
    }

    pub fn images(&self) -> Images<'_> {
        self.images.iter()
    }

    pub fn tags(&self) -> Tags {
        self.tags.clone()
    }
}

impl Image {
    pub fn new(url: &String) -> Self {
        Self {
            url: url.clone(),
            file: String::new(),
        }
    }

    pub fn set_filename(&mut self, file: String) {
        self.file = file;
    }

    pub fn get_filename(&self) -> &String {
        &self.file
    }

    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }

    pub fn get_url(&self) -> &String {
        &self.url
    }
}

impl<'a> From<ElementRef<'a>> for Image {
    fn from(value: ElementRef) -> Self {
        let url = value.value().attr("href").unwrap().to_string();
        Self::new(&url)
    }
}

impl From<String> for TagType {
    fn from(value: String) -> Self {
        match value.to_lowercase().as_str() {
            "reclass" => Self::ReClass,
            "parody" => Self::Parody,
            "character" => Self::Character,
            "language" => Self::Language,
            "artist" => Self::Artist,
            "male" => Self::Male,
            "female" => Self::Female,
            "other" | _ => Self::Other,
        }
    }
}

impl ToString for TagType {
    fn to_string(&self) -> String {
        match self {
            Self::ReClass => String::from("reclass"),
            Self::Parody => String::from("parody"),
            Self::Character => String::from("character"),
            Self::Language => String::from("language"),
            Self::Artist => String::from("artist"),
            Self::Male => String::from("male"),
            Self::Female => String::from("female"),
            Self::Other => String::from("other"),
        }
    }
}

impl Tag {
    pub fn tag_value(&self) -> &String {
        &self.t_val
    }

    pub fn tag_type(&self) -> &TagType {
        &self.t_type
    }
}

impl Tags {
    fn new() -> Self {
        Self { _inner: vec![] }
    }

    pub fn inner(&self) -> &Vec<Tag> {
        &self._inner
    }

    fn push(&mut self, tag: Tag) {
        self._inner.push(tag);
    }

    pub fn get<P>(&self, predicate: P) -> Option<Tag>
    where
        P: Fn(&Tag) -> bool,
    {
        for tag in &self._inner {
            if predicate(tag) {
                return Some(tag.clone());
            }
        }

        None
    }
}

impl IntoIterator for Tags {
    type IntoIter = std::vec::IntoIter<Tag>;
    type Item = Tag;

    fn into_iter(self) -> Self::IntoIter {
        self._inner.into_iter()
    }
}
