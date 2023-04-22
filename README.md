# ehentai-dl
A program to download your galleries 🦀

*This project is not yet complete, and so release builds wont be uploaded just yet*

## TODO
- [x] Image downloader
- [x] JSON Metadata for Ani/Tachiyomi
  - `cargo build --release --features aniyomi`
- [ ] Zip Archiver
  - `cargo build --release --features zip`

## Features
*(For the moment, at least) **YOU CAN ONLY PICK EITHER ONE FEATURE FLAG***. Please read below for details.

### `aniyomi`
Writes an additional `details.json` meta file, along with a `.nomedia` file for every directory.

This feature is meant to go well with Aniyomi, a manga reader and anime watcher. **Does not work with the `zip` feature due to how Aniyomi handles zip files**

### `zip`
Zips the whole gallery and deletes the original. Uses the `bizp2` feature of the `zip` crate.

***This feature does not work with the `aniyomi` feature***