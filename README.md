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

### Translation

To translate the lines of a file, saving the translation in another:

```sh
cargo tl path/to/the/file.mbe path/to/the/translated/file
```

To translate a file, overwriting it:

```sh
cargo etl path/to/the/file.mbe
```

### Reading lines

To read every dialogue lines of a `.mbe` file:
```sh
cargo rl path/to/the/file.mbe [-p PREFIX]
```

The default prefix is "> ".
