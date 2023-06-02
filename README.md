# ehentai-dl
A program to download your galleries 🦀

*This project is not yet complete, and so release builds wont be uploaded just yet*

## TODO
- [x] Image downloader
- [x] JSON Metadata for Ani/Tachiyomi
  - `cargo build --release --features aniyomi`
- [x] Zip Archiver
  - `cargo build --release --features zip`
- [ ] Setting/Config
  - `cargo build --release --features config`
- [ ] Very lightweight CLI/Argument parser
  - Verbosity with `-v` controls the verbosity of the log files
  - Point to a config file with `-c`
  - Show version with `--version`
    - [x] Versioning scheme
     ```
      e-hentai_dl v3.0.2 w/aniyomi,zip,config
      |^^^^^^^^^| |^^^^|  |^^^^^^^^^^^^^^^^^|
       Program    Version     Compiler
        Name                  Features
      ```

## Features
### `aniyomi`
Writes an additional `details.json` meta file, along with a `.nomedia` file for every directory.

This feature is meant to go well with Aniyomi, a manga reader and anime watcher.

### `zip`
Zips the whole gallery and deletes the original. Uses the `bizp2` feature of the `zip` crate.

### `config`
Compiled features can be turned on and off here. The program will error out when you try to set for a feature that wasn't compiled with the binary

**This is a work in progress**