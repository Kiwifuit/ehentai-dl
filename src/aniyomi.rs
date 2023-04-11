use std::path::Path;

/// Overrides `std::fs::create_dir` when the `aniyomi` flag
/// is set.
///
/// The override adds a `.nomedia` file inside `path` after
/// its creation.
pub fn create_dir<P: AsRef<Path>>(path: P) -> Result<(), std::io::Error> {
    std::fs::create_dir(&path)?;
    std::fs::File::create(path.as_ref().join(".nomedia"))?;

    Ok(())
}
