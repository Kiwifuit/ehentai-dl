# ehentai-dl
A program to download your galleries 🦀

*This project is **not yet complete**, and so release builds wont be released just yet*

## TODO
- [x] Image downloader
- [x] JSON Metadata for Ani/Tachiyomi
  - `cargo build --release --features aniyomi`
- [x] Zip Archiver
  - `cargo build --release --features zip`
- [ ] Setting/Config
  - `cargo build --release --features config`
- [ ] Very lightweight CLI/Argument parser
  - Point to a config file with `-c`
  - Show version with `--version`
    - [x] Versioning scheme
     ```
      ehentai_dl v3.0.2 w/aniyomi,zip,config
      |^^^^^^^^^| |^^^^|  |^^^^^^^^^^^^^^^^^|
       Program    Version     Compiler
        Name                  Features
      ```

## Compilation
The pre-compiled binaries ***do not contain features***, it is only for the bare minimum of downloading an e-hentai gallery.

The features listed below must be compiled manually:
```
git clone https://github.com/Kiwifuit/ehentai-dl
cd ehentai-dl
cargo build --features <config,aniyomi,etc>
```

## Compilation: Features
### `aniyomi`
Writes the necessary metadata and file structure for Aniyomi to parse.

### `zip`
Zips the whole gallery and deletes the original. ***Does not work with Aniyomi's way of parsing zip files***, so this feature is only intended for storage and/or data transfer

### `config`
Compiled features can be turned on and off here. The program will error out when you try to set for a feature that wasn't compiled with the binary.

Also contains feature-specific configuration

**This is a work in progress**