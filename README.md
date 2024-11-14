
# Spriterator

[![build](https://github.com/krchmkn/spriterator/actions/workflows/build.yml/badge.svg)](https://github.com/krchmkn/spriterator/actions/workflows/build.yml)

[Spriterator](https://crates.io/crates/spriterator) is a Rust library that creates optimized sprite sheets by combining multiple images from a specified directory into a compact format. It arranges images row by row to minimize gaps, creating multiple sheets if necessary when images exceed defined maximum dimensions. The library supports popular image formats and offers parallel processing to speed up large tasks.

## Features

- **Recursive Directory Scanning**: Finds all images in nested directories.
- **Compact Layout**: Places images row by row without extra spacing.
- **Automatic Sheet Splitting**: Creates multiple sprite sheets if images exceed specified dimensions.
- **Transparent Padding Removal**: Trims transparent edges to reduce unused space.
- **Supported Formats**: Accepts `png`, `jpg`, `jpeg`, `webp`.

## Example

The following example demonstrates how to use `Spriterator` to create sprite sheets from images in a directory.

```rust
use spriterator::Spriterator;
use std::fs;
use std::path::Path;

fn prepare_directory(path: &str) -> std::io::Result<()> {
    let dir_path = Path::new(path);

    if dir_path.exists() {
        fs::remove_dir_all(dir_path)?;
    }

    fs::create_dir_all(dir_path)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_dir = "/path/to/sprites";

    prepare_directory(output_dir)?;

    let spriterator = Spriterator::new("/path/to/images", 1200, 2048);
    let sprites = spriterator.generate()?;

    for (index, sprite) in sprites.iter().enumerate() {
        sprite.save(format!("{}/{}.webp", output_dir, index))?;
    }

    Ok(())
}
```
