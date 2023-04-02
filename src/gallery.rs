use scraper::ElementRef;

pub struct Gallery {
    title: String,
    pages: u8,
    images: Vec<Image>,
}

pub struct Image {
    url: String,
    file: String,
}

impl Gallery {
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn add_image(&mut self, image: Image) {
        self.pages += 1;
        self.images.push(image);
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
