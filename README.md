# THL Tools

CLI tool to extract and repack files from the "The Hundred Line" game

## Usage

You need to have Rust installed on your computer. Due to the difference of path formatting, I don't know if this program works on Windows.

### Extraction

To extract the content of a `.mvgl` archive:
```sh
cargo x path/to/the/archive.mvgl path/to/the/extracted/directory
```

### Packing

To repack the content of a folder to a `.mvgl` archive:
```sh
cargo p path/to/the/directory path/to/the/created/archive
```

### Reading lines

To read every dialogue lines of a `.mbe` file:
```sh
cargo rl path/to/the/file.mbe [-p PREFIX]
```

The default prefix is "> ".

### Extracting all dialogues into a single CSV

```
cargo xd path/to/the/game/directory comma,separated,languages,to,use
```

Available languages being 'japanese', 'english', 'traditional-chinese' and 'simplified-chinese'

### Repacking all dialogues into the game

```
cargo rd path/to/the/csv path/to/the/mvgl/file/to/patch path/to/the/destination
```

This will patch a mvgl file, putting your own text in its place. The mvgl files are located in `gamedata/app_text[LANGUAGE].dx11.mvgl`, where LANGUAGE is 00 for Japanese, 01 for English, 02 for Traditional Chinese, and 03 for Simplified Chinese.

## How the files are composed.

In the `resources/` folder, I put the `.ksy` files describing the file format of `.mvgl` files and `.mbe` files.

## As for images

Most of images are situated in the `gamedata/app_0.dx11.mvgl`. Some of these images are overwrote in the corresponding `gamedata/app_*.dx11.mvgl` for each languages.
