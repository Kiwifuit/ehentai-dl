use log::{debug, info};
use regex::Regex;
use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io::{self, prelude::*};
use std::path::Path;
use std::sync::mpsc::{self, Sender};
use std::{string, thread};

const GALLERY_PARSER: &str = r"https://e-hentai\.org/g/(\d{7})/([a-z0-9]{10})";

#[derive(Debug)]
/// Wraps various errors into one. `chunk_size` is generally used for
/// wrapping `std::mpsc::SendError`, and a value of `0` automatically
/// signifies that the function does not do any I/O operations
/// *(technically it means that it wont be wrapping `std::mpsc::SendError`,
/// and therefore, no I/O)*
pub enum ParseError<const chunk_size: usize> {
    IoError(io::Error),
    ChannelError(mpsc::SendError<([u8; chunk_size], usize)>),
    RegexParseError(regex::Error),
    StringEncodeError(string::FromUtf8Error),
}

/// Loads a file chunk by chunk and sends it via `tx`, along with how many
/// bytes it has read.
///
/// Right before the function finishes, it sends an empty buffer, along
/// with how many bytes it has read total
fn load_file<const chunk_size: usize>(
    path: &Path,
    tx: Sender<([u8; chunk_size], usize)>,
) -> Result<usize, ParseError<chunk_size>> {
    let mut buf = [0; chunk_size];

    let mut file = OpenOptions::new()
        .read(true)
        .open(path)
        .map_err(|e| ParseError::IoError(e))?;

    let mut bytes_read_total = 0;
    while let Ok(bytes_read) = file.read(&mut buf) {
        bytes_read_total += bytes_read;

        let payload = (buf, bytes_read);
        tx.send(payload).map_err(|e| ParseError::ChannelError(e))?;

        buf.fill(0);
    }
    tx.send((buf, bytes_read_total))
        .map_err(|e| ParseError::ChannelError(e))?;

    Ok(bytes_read_total)
}

pub fn read_file<const chunk_size: usize, F>(file: &'static F) -> Result<String, ParseError<0>>
where
    F: AsRef<OsStr> + ?Sized,
{
    let file = Path::new(file);
    let (file_tx, file_rx) = mpsc::channel();

    thread::spawn(move || load_file::<chunk_size>(&file, file_tx));

    let mut res = String::new();
    while let Ok(payload) = file_rx.recv() {
        match payload {
            // TODO: Log how many bytes were read
            (chunk, read_total) if chunk.iter().map(|i| *i as usize).sum::<usize>() == 0 => {
                info!("Read {} bytes", read_total)
            }
            (chunk, read_total) => {
                let bytes = chunk
                    .iter()
                    .take(read_total)
                    .map(|i| *i)
                    .collect::<Vec<u8>>();
                res += &String::from_utf8(bytes).map_err(|e| ParseError::StringEncodeError(e))?;
            }
        }
    }

    Ok(res)
}

pub fn get_all_galleries(raw: &String) -> Result<Vec<String>, ParseError<0>> {
    let parser = Regex::new(GALLERY_PARSER).map_err(|e| ParseError::RegexParseError(e))?;

    Ok(parser
        .captures_iter(raw)
        // Here its alright to .unwrap() because we can 100% guarantee
        // that there'll be a 0th element
        .map(|e| e.get(0).unwrap().as_str().to_string())
        .collect::<Vec<String>>())
}
