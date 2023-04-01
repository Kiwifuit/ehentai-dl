use std::{path::Path, sync::mpsc::channel, thread};

mod extractor;
mod logger;
mod parser;

const CHUNK_SIZE: usize = 2048;

fn main() {
    logger::setup_logger(logger::LogLevel::Debug)
        .expect("unexpected error whie starting the logger");

    let a = parser::read_file::<CHUNK_SIZE, str>("res/galleries.txt");

    dbg!(a);
}
