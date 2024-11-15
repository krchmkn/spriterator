
# Spriterator

[![build](https://github.com/krchmkn/spriterator/actions/workflows/build.yml/badge.svg)](https://github.com/krchmkn/spriterator/actions/workflows/build.yml)

[Spriterator](https://crates.io/crates/spriterator) is a Rust library that creates sprite sheets by combining multiple images from a specified directory into a compact format.

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
    let ext = "png";
    let output_dir = format!("/parth/to/sprites/{}", ext);
    
    prepare_directory(output_dir.as_str())?;

    let size = 1024;
    let spriterator = Spriterator::new(
        format!("/parth/to/images/{}", ext).as_str(),
        size,
        size,
        None,
        None,
    );
    let sprites = spriterator.generate()?;

    for (index, sprite) in sprites.iter().enumerate() {
        sprite.save(format!("{}/{}.{}", output_dir, index, ext))?;
    }

    Ok(())
}
```
