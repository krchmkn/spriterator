use dotenv::dotenv;
use spriterator::Spriterator;
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;

/// Test the Spriterator struct
///
/// INPUT_DIR = /path/to/input/images
/// OUTPUT_DIR = /path/to/output/sprites
/// FILE_EXTENSION = png
#[test]
fn spriterator_test() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let input_dir = env::var("INPUT_DIR").expect("INPUT_DIR must be set");
    let output_dir = env::var("OUTPUT_DIR").expect("OUTPUT_DIR must be set");
    let ext = env::var("FILE_EXTENSION").expect("FILE_EXTENSION must be set");

    let output_dir = format!("{}/{}", output_dir, ext);

    let dir_path = Path::new(&output_dir);
    if dir_path.exists() {
        fs::remove_dir_all(dir_path)?;
    }
    fs::create_dir_all(dir_path)?;

    let spriterator = Spriterator::new(
        format!("{}/{}", input_dir, ext).as_str(),
        2048,
        2048,
        Some(64),
        Some(64),
    );

    let sprites = spriterator.generate();

    let sprites = sprites.map_err(|e| {
        eprintln!("Failed to generate sprites: {}", e);
        e
    })?;

    assert!(!sprites.is_empty(), "No sprites were generated");

    for (index, sprite) in sprites.iter().enumerate() {
        let frames = sprite.get_frames();
        assert!(!frames.is_empty(), "Sprite {} has no frames", index);

        let sprite_path = format!("{}/{}.{}", output_dir, index, ext);
        sprite.save(&sprite_path)?;

        assert!(
            Path::new(&sprite_path).exists(),
            "Sprite file was not created: {}",
            sprite_path
        );
    }

    let saved_files: Vec<_> = fs::read_dir(dir_path)?
        .filter_map(|entry| entry.ok())
        .collect();
    assert_eq!(
        saved_files.len(),
        sprites.len(),
        "Mismatch between number of sprites and saved files"
    );

    Ok(())
}
