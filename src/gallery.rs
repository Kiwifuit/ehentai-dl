use scraper::ElementRef;

#[derive(Debug)]
pub struct Gallery {
    title: String,
    pages: u8,
    images: Vec<Image>,
    tags: Vec<Tag>,
}

#[derive(Debug)]
pub struct Image {
    url: String,
    file: String,
}

#[derive(Debug)]
pub struct Tag {
    t_type: TagType,
    t_val: String,
}
#[derive(Debug)]
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
            pages: 0,
            images: vec![],
            tags: vec![],
        }
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn add_image(&mut self, image: Image) {
        self.pages += 1;
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
