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
#[ignore] // Ignored due to CI requirements (a directory with images is necessary for it to function).
fn spriterator_test() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let input_dir = env::var("INPUT_DIR").expect("INPUT_DIR must be set");
    let output_dir = env::var("OUTPUT_DIR").expect("OUTPUT_DIR must be set");
    let ext = env::var("FILE_EXTENSION").expect("FILE_EXTENSION must be set");
    let large_image_size: u32 = env::var("LARGE_IMAGE_SIZE")
        .expect("LARGE_IMAGE_SIZE must be set")
        .parse()
        .unwrap();
    let medium_image_size: u32 = env::var("MEDIUM_IMAGE_SIZE")
        .expect("MEDIUM_IMAGE_SIZE must be set")
        .parse()
        .unwrap();
    let small_image_size: u32 = env::var("SMALL_IMAGE_SIZE")
        .expect("SMALL_IMAGE_SIZE must be set")
        .parse()
        .unwrap();

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
        None,
        None,
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

        let expected_frames = vec![
            (0, 0, medium_image_size, medium_image_size),
            (medium_image_size, 0, small_image_size, small_image_size),
            (
                medium_image_size + small_image_size,
                0,
                large_image_size,
                large_image_size,
            ),
        ];

        for (i, frame) in frames.iter().enumerate() {
            let (expected_x, expected_y, expected_width, expected_height) =
                expected_frames.get(i).expect("Unexpected frame count");
            assert_eq!(frame.get_x(), *expected_x, "Frame {} X mismatch", i);
            assert_eq!(frame.get_y(), *expected_y, "Frame {} Y mismatch", i);
            assert_eq!(
                frame.get_width(),
                *expected_width,
                "Frame {} width mismatch",
                i
            );
            assert_eq!(
                frame.get_height(),
                *expected_height,
                "Frame {} height mismatch",
                i
            );
        }

        let sprite_path = format!("{}/{}.{}", output_dir, index + 1, ext);
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
