
# Spriterator

[![build](https://github.com/krchmkn/spriterator/actions/workflows/build.yml/badge.svg)](https://github.com/krchmkn/spriterator/actions/workflows/build.yml)

[Spriterator](https://crates.io/crates/spriterator) is a Rust library that creates sprite sheets by combining multiple images from a specified directory into a compact format.

## Example

The following example demonstrates how to use `Spriterator` to create sprite sheets from images in a directory.

```rust
use spriterator::Spriterator;
use std::fs;
use std::path::Path;
use std::time::Instant;

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

    let size = 1024;
    let spriterator = Spriterator::new(
        "/path/to/images",
        size,
        size,
        Some(64),
        Some(64),
    );

    let sprites = spriterator.generate()?;

    for (index, sprite) in sprites.iter().enumerate() {
        sprite.save(format!("{}/{}.{}", output_dir, index, "webp"))?;
    }

    Ok(())
}
```
