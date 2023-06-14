use std::fs;
use std::io::{Error, Write};
use std::path::{Path, PathBuf};

#[cfg(feature = "aniyomi")]
use json_minimal::Json;

use log::info;

use crate::gallery::{Gallery, Tag, TagType};
use crate::version::get_version;

lazy_static::lazy_static! {
    static ref DEFAULT_DESCRIPTION: String = format!("Made with {}", get_version());
}

/// Overrides `std::fs::create_dir` when the `aniyomi` flag
/// is set.
///
/// The override adds a `.nomedia` file inside `path` after
/// its creation.
#[cfg(feature = "aniyomi")]
pub fn create_dir<P: AsRef<Path>>(path: P) -> Result<(), Error> {
    std::fs::create_dir(&path)?;
    std::fs::File::create(path.as_ref().join(".nomedia"))?;

    Ok(())
}

#[cfg(feature = "aniyomi")]
pub struct AniyomiMeta {
    title: String,
    artist: String,
    tags: Vec<Tag>,
}

#[cfg(feature = "aniyomi")]
impl AniyomiMeta {
    pub fn new(title: String, artist: String, tags: Vec<Tag>) -> Self {
        Self {
            title,
            artist,
            tags,
        }
    }
}

#[cfg(feature = "aniyomi")]
impl From<&Gallery> for AniyomiMeta {
    fn from(value: &Gallery) -> Self {
        Self::new(
            value.title().clone(),
            value
                .tags()
                .get(|t| t.tag_type() == &TagType::Artist)
                .unwrap()
                .tag_value()
                .clone(),
            value.tags().inner().clone(),
        )
    }
}

#[cfg(feature = "aniyomi")]
pub fn to_json_file<W: Write>(to: &mut W, meta: &AniyomiMeta) -> Result<usize, Error> {
    let tags = meta
        .tags
        .iter()
        .filter(|&t| t.clone().tag_type() != &TagType::Artist)
        .map(|t| Json::STRING(format!("{}: {}", t.tag_type().to_string(), t.tag_value())))
        .collect();

    let json = Json::new()
        .add(make_object("title", Json::STRING(meta.title.clone())))
        .add(make_object("author", Json::NULL))
        .add(make_object("artist", Json::STRING(meta.artist.clone())))
        .add(make_object(
            "description",
            Json::STRING(
                crate::CONFIG
                    .read()
                    .and_then(|c| {
                        Ok(c.aniyomi
                            .description
                            .clone()
                            .unwrap_or(DEFAULT_DESCRIPTION.to_string()))
                    })
                    .unwrap_or(DEFAULT_DESCRIPTION.to_string()),
            ),
        ))
        .add(make_object("genre", Json::ARRAY(tags)))
        .add(make_object("status", Json::STRING("0".to_string())))
        .add(make_object(
            "_status values",
            Json::ARRAY(vec![Json::STRING("0 = Finished".to_string())]),
        ))
        .print();

    let written = to.write(json.as_bytes())?;

    info!(
        "Dumped Aniyomi Metadata for {:?} ({} bytes written)",
        meta.title, written
    );

    Ok(written)
}

#[cfg(feature = "aniyomi")]
fn make_object<S: ToString>(name: S, value: Json) -> Json {
    Json::OBJECT {
        name: name.to_string(),
        value: Box::new(value),
    }
}

#[cfg(feature = "aniyomi")]
pub fn make_cover<P: AsRef<Path>>(path: P) -> Result<PathBuf, Error> {
    let path = path.as_ref();
    let ext = path.extension().unwrap().to_str().unwrap();
    let cover = path
        .parent()
        .unwrap()
        .with_file_name(format!("cover.{}", ext));

    let written = fs::copy(path, &cover)?;

    info!("Written {:?} ({} bytes written)", cover, written);
    Ok(cover)
}
