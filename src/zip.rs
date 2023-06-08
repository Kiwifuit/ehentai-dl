use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::{self, prelude::*};
use std::path::{Path, PathBuf};

use crate::sanitize::sanitize_title;
use log::{debug, info, trace};

#[cfg(feature = "zip")]
use zip::{write::*, CompressionMethod};

#[cfg(feature = "zip")]
const COMPRESSION_MODE: CompressionMethod = CompressionMethod::BZIP2;

#[cfg(feature = "zip")]
type ZipFile = ZipWriter<File>;

#[derive(Debug)]
#[cfg(feature = "zip")]
pub enum ZipError {
    ZipOpenError(io::Error),
    AddDirError(zip::result::ZipError),
    ReadError(io::Error),
    WriteError(io::Error),
    StartFileError {
        error: zip::result::ZipError,
        compression: CompressionMethod,
    },
}

#[cfg(feature = "zip")]
impl Display for ZipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "error while {}",
            match self {
                Self::ZipOpenError(e) => format!("opening zip file: {}", e),
                Self::AddDirError(e) => format!("adding directory: {}", e),
                Self::ReadError(e) => format!("reading file: {}", e),
                Self::WriteError(e) => format!("writing file: {}", e),
                Self::StartFileError {
                    error: e,
                    compression: _,
                } => format!("compressing file: {}", e),
            }
        )
    }
}

#[cfg(feature = "zip")]
pub fn make_zip<P: AsRef<Path>>(file: &P) -> Result<ZipFile, ZipError> {
    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(file)
        .map_err(|e| ZipError::ZipOpenError(e))?;

    Ok(ZipWriter::new(file))
}

#[cfg(feature = "zip")]
pub fn add_file<P, const CHUNK_SIZE: usize>(arch: &mut ZipFile, path: &P) -> Result<usize, ZipError>
where
    P: AsRef<Path>,
{
    trace!("Chunk size provided is {}", CHUNK_SIZE);

    let path = path
        .as_ref()
        .components()
        .filter_map(|c| {
            if c.as_os_str().to_string_lossy() == "." {
                None
            } else {
                Some(sanitize_title(&c.as_os_str().to_string_lossy()))
            }
        })
        .collect::<PathBuf>();

    let compression_opts = FileOptions::default()
        .compression_method(COMPRESSION_MODE)
        .compression_level(Some(9));

    info!("Adding file {:?}", path.display());

    if path.is_file() {
        let path = path.to_str().unwrap();

        let mut file = OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|e| ZipError::ReadError(e))?;

        let mut buf = [0; CHUNK_SIZE];
        arch.start_file(path, compression_opts)
            .map_err(|e| ZipError::StartFileError {
                error: e,
                compression: COMPRESSION_MODE,
            })?;

        let mut written_bytes = 0;
        while let Ok(read) = file.read(&mut buf) {
            // We have to manually check if we finished writing
            // because EOF only returns Ok(0)
            if read == 0 {
                break;
            }

            let written = arch
                .write(
                    &buf.iter()
                        .take(read)
                        .map(|e| e.to_owned())
                        .collect::<Vec<u8>>(),
                )
                .map_err(|e| ZipError::WriteError(e))?;

            written_bytes += written;
            trace!("Written Total: {}", written_bytes);

            trace!("Clearing buffer");
            trace!(
                "Buffer dump: {}",
                buf.iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            );
            buf.fill(0);
            trace!("Buffer cleared");
        }

        debug!("Written {:?} to archive", path);
        return Ok(written_bytes);
    } else if !path.starts_with("./") {
        arch.add_directory(path.as_os_str().to_string_lossy(), compression_opts)
            .map_err(|e| ZipError::AddDirError(e))?;
    }

    Ok(0)
}
