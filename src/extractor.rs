use scraper::{Html, Selector};

const TITLE_TAG: &str = "div.gm div#gd2 h1#gn";
const FILENAME_TAG: &str = "div#i1.sni div#i4 div";
const IMAGE_TAG: &str = "div#i1.sni div#i3 a img#img";
const IMAGE_LINK_EXTRACTOR: &str = r"https://e-hentai\.org/s/([a-z0-9]{10})/([0-9]{7})-(\d+)";
const GALLERY_TAG_TAG: &str = "div#taglist table tbody tr";
