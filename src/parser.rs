use log::debug;
use std::ffi::OsStr;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{self, prelude::*};
use std::path::Path;
use std::sync::mpsc::{self, SyncSender};
use std::{num, string, thread};

type Pagination = f32;

#[derive(Debug)]
/// Wraps various errors into one. `C` is generally used for
/// wrapping `std::mpsc::SendError`, and a value of `0` automatically
/// signifies that the function does not do any I/O operations
/// *(technically it means that it wont be wrapping `std::mpsc::SendError`,
/// and therefore, no I/O)*
pub enum ParseError<const C: usize> {
    IoError(io::Error),
    ChannelError(mpsc::SendError<([u8; C], usize)>),
    RegexParseError(regex::Error),
    StringEncodeError(string::FromUtf8Error),
    NoCapture,
    IntParseError(num::ParseFloatError),
}

impl<const C: usize> Display for ParseError<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IoError(e) => format!("i/o error: {}", e),
                Self::ChannelError(e) => format!("channel error: {}", e),
                Self::RegexParseError(e) => format!("regex error: {}", e),
                Self::StringEncodeError(e) => format!("error while decoding string: {}", e),
                Self::NoCapture => format!("expected to parse something, got nothing"),
                Self::IntParseError(e) => format!("error while parsing int: {}", e),
            }
        )
    }
}

/// Loads a file chunk by chunk and sends it via `tx`, along with how many
/// bytes it has read.
fn load_file<const CHUNK_SIZE: usize>(
    path: &Path,
    tx: SyncSender<([u8; CHUNK_SIZE], usize)>,
) -> Result<usize, ParseError<CHUNK_SIZE>> {
    let mut buf = [0; CHUNK_SIZE];

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

    Ok(bytes_read_total)
}

pub fn read_file<const CHUNK_SIZE: usize, F>(file: &'static F) -> Result<String, ParseError<0>>
where
    F: AsRef<OsStr> + ?Sized,
{
    let file = Path::new(file);
    let (file_tx, file_rx) = mpsc::sync_channel(100);

    thread::spawn(move || load_file::<CHUNK_SIZE>(&file, file_tx));

    let mut contents = String::new();
    while let Ok((chunk, bytes_read)) = file_rx.recv() {
        if bytes_read == 0 {
            break;
        }

        let bytes = chunk
            .iter()
            .take(bytes_read)
            .map(|i| *i)
            .collect::<Vec<u8>>();
        debug!("Read {}/{} bytes", bytes_read, CHUNK_SIZE);
        contents += &String::from_utf8(bytes).map_err(|e| ParseError::StringEncodeError(e))?;
    }

    Ok(contents)
}

pub fn get_all_galleries(raw: &String) -> Result<Vec<String>, ParseError<0>> {
    let parser = compile! {regex r"https://e-hentai\.org/g/(\d{7})/([a-z0-9]{10})"}?;

    Ok(parser
        .captures_iter(raw)
        // Here its alright to .unwrap() because we can 100% guarantee
        // that there'll be a 0th element
        .map(|e| e.get(0).unwrap().as_str().to_string())
        .collect::<Vec<String>>())
}

pub fn get_pagination(raw: &String) -> Result<Pagination, ParseError<0>> {
    let parser = compile! {regex r"Showing 1 - (\d+) of (\d+)"}?;
    let caps = parser.captures(raw.as_str()).ok_or(ParseError::NoCapture)?;

    let total = caps[2]
        .parse::<Pagination>()
        .map_err(|e| ParseError::IntParseError(e))?;
    let rendered = caps[1]
        .parse::<Pagination>()
        .map_err(|e| ParseError::IntParseError(e))?;

    Ok(total / rendered)
}

pub fn get_filename(raw: &String) -> Result<String, ParseError<0>> {
    let parser = compile! {regex r"([\w\d.]*) :: ([\dx ]*) :: ([\d.]* \w*)"}?;

    Ok(parser.captures(raw).ok_or(ParseError::NoCapture)?[1].to_string())
}
