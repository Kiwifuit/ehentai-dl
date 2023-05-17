use std::fs::{File, OpenOptions};
use std::io::{self, prelude::*};
use std::path::Path;

use indicatif::ProgressBar;
use log::{debug, trace};

#[cfg(feature = "zip")]
use zip::{write::*, CompressionMethod};

#[cfg(feature = "zip")]
const COMPRESSION: CompressionMethod = CompressionMethod::DEFLATE;

#[cfg(feature = "zip")]
type ZipFile = ZipWriter<File>;

#[derive(Debug)]
#[cfg(feature = "zip")]
pub enum ZipError {
    ZipOpenError(io::Error),
    AddDirError(zip::result::ZipError),
    ReadError(io::Error),
    WriteError(io::Error),
    StartFileError(zip::result::ZipError),
    GetFileLengthError(io::Error),
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
pub fn add_file<P, const CHUNK_SIZE: usize>(
    arch: &mut ZipFile,
    path: &P,
    prog: &ProgressBar,
) -> Result<usize, ZipError>
where
    P: AsRef<Path>,
{
    trace!("Chunk size provided is {}", CHUNK_SIZE);
    let path = path.as_ref();
    let opts = FileOptions::default()
        .compression_method(COMPRESSION)
        .compression_level(Some(9));

    if path.is_dir() {
        prog.set_length(1);
        arch.add_directory(path.as_os_str().to_str().unwrap(), opts)
            .map_err(|e| ZipError::AddDirError(e))?;
        prog.inc(1);

        Ok(0)
    } else {
        let mut file = OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|e| ZipError::ReadError(e))?;

        prog.set_length(get_length(&mut file)?);

        let mut buf = [0; CHUNK_SIZE];
        arch.start_file(path.as_os_str().to_str().unwrap(), opts)
            .map_err(|e| ZipError::StartFileError(e))?;

        let mut written_bytes = 0;
        while let Ok(read) = file.read(&mut buf) {
            written_bytes += arch.write(&buf).map_err(|e| ZipError::WriteError(e))?;
            debug!("Read/Written: {}/{}", read, written_bytes);

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

        Ok(written_bytes)
    }
}

#[cfg(feature = "zip")]
fn get_length<F: Seek>(file: &mut F) -> Result<u64, ZipError> {
    let len = file
        .seek(io::SeekFrom::End(0))
        .map_err(|e| ZipError::GetFileLengthError(e))?;

    file.seek(io::SeekFrom::Start(0))
        .map_err(|e| ZipError::GetFileLengthError(e))?;
    Ok(len)
}
