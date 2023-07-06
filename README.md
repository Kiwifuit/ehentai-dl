# ehentai-dl
A program to download your galleries 🦀

*This project is **not yet complete**, and so release builds wont be released just yet*

## TODO
- [ ] Faster Downloads
  - Multithreaded downloading?

## Compilation
The pre-compiled binaries ***do not contain features***, it is only for the bare minimum of downloading an e-hentai gallery.

The features listed below must be compiled manually:
```
git clone https://github.com/Kiwifuit/ehentai-dl
cd ehentai-dl
cargo build --features <config,aniyomi,etc>
```

## Compilation: Features

### `cli`
**This is the default feature**. This adds support for configuring via command-line.

This feature takes precedence over `config`, so if both features are enabled, only `cli`'s values are respected, and there will be no `config.toml` file.

All build features are assumed to be enabled, unlike
`config` where these features can be enabled or disabled.

### `aniyomi`
Writes the necessary metadata and file structure for Aniyomi to parse.

### `zip`
Zips the whole gallery and deletes the original. ***Does not work with Aniyomi's way of parsing zip files***, so this feature is only intended for storage and/or data transfer

### `config`
Compiled features can be turned on and off here. The program will error out when you try to set for a feature that wasn't compiled with the binary.

***This feature does not work well with `cli` as its values
take precedence over `config`'s***

**NOTE**: This feature is *sorta* broken in the fact that it needs all features to be turned on. **Enabling this feature means you should enable `zip`, `aniyomi`, and `metrics`**

### `metrics`
Reports metrics data/which galleries were the heaviest