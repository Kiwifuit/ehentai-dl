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
