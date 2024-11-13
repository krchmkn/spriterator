# Spriterator

[Spriterator](https://crates.io/crates/spriterator) is a Rust library that generates compact sprite sheets from images in a specified directory. It arranges images row by row to minimize empty space and avoid gaps, even if the images are of different sizes. If the images exceed a specified maximum height, the library will create multiple sprite sheets.

This library supports common image formats such as PNG, JPEG, GIF, and WebP, and it can use optional parallel processing (via `rayon`) for efficient image loading.

## Features

- **Recursive Directory Scanning**: Finds all images within nested directories.
- **Compact Layout with No Spacing**: Arranges images tightly in rows without gaps, regardless of size.
- **Multiple Sheets if Necessary**: Generates multiple sprite sheets when images exceed specified height limits.
- **Optional Parallel Processing**: Speeds up loading and processing of images (requires `rayon` feature).
- **Supported Formats**: Handles common image formats (`png`, `jpg`, `jpeg`, `gif`, `bmp`, `ico`, `tiff`, `webp`).

## Usage

Here is an example of using `Spriterator` to generate sprite sheets:

```rust
use spriterator::Spriterator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spriterator = Spriterator::new("path/to/images", 1024, 2048);
    let sprites = spriterator.generate()?;

    // Save each generated sprite sheet
    for (index, sprite) in sprites.iter().enumerate() {
        sprite.save(format!("sprite_sheet_{}.webp", index))?;
    }

    Ok(())
}
```
