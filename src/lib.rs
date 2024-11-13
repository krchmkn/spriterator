use image::{GenericImage, RgbaImage};
use once_cell::sync::Lazy;
use rayon::prelude::*;
use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;

/// A static set of supported image file extensions for quick lookup.
static IMAGE_EXTENSIONS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let exts = ["png", "jpg", "jpeg", "gif", "bmp", "ico", "tiff", "webp"];
    exts.iter().cloned().collect()
});

/// A struct representing the configuration and functionality of a sprite generator.
pub struct Spriterator {
    dir_path: String,
    output_path: String,
    max_size: u32,
}

impl Spriterator {
    /// Creates a new `Spriterator` instance.
    ///
    /// # Arguments
    ///
    /// * `dir_path` - Path to the directory containing images.
    /// * `output_path` - Path where the generated sprite sheet will be saved.
    /// * `max_size` - Maximum width of the sprite sheet in pixels.
    pub fn new(dir_path: &str, output_path: &str, max_size: u32) -> Self {
        Self {
            dir_path: dir_path.to_string(),
            output_path: output_path.to_string(),
            max_size,
        }
    }

    /// Generates a sprite sheet from images in the specified directory.
    ///
    /// This method arranges images into a grid layout and saves the resulting sprite sheet.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the sprite sheet was successfully generated.
    /// * `Err` if no images were found or if an error occurred while processing.
    pub fn generate(&self) -> Result<(), Box<dyn std::error::Error>> {
        let images: Vec<RgbaImage> = WalkDir::new(&self.dir_path)
            .into_iter()
            .par_bridge()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file() && is_image(entry.path()))
            .filter_map(|entry| image::open(entry.path()).ok().map(|img| img.to_rgba8()))
            .collect();

        if images.is_empty() {
            return Err("No images found in the specified directory.".into());
        }

        let max_image_width = images.iter().map(|img| img.width()).max().unwrap_or(0);
        let max_image_height = images.iter().map(|img| img.height()).max().unwrap_or(0);

        let max_columns = (self.max_size / max_image_width).max(1) as usize;
        let columns = images.len().min(max_columns);
        let rows = (images.len() + columns - 1) / columns;

        let sprite_width = max_image_width * columns as u32;
        let sprite_height = max_image_height * rows as u32;

        let mut sprite = RgbaImage::new(sprite_width, sprite_height);

        for (i, img) in images.iter().enumerate() {
            let x = (i % columns) as u32 * max_image_width;
            let y = (i / columns) as u32 * max_image_height;
            sprite.copy_from(img, x, y)?;
        }

        sprite.save(&self.output_path)?;
        Ok(())
    }
}

/// Checks if a file has an extension that matches common image formats.
fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| IMAGE_EXTENSIONS.contains(ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_is_image() {
        // Positive cases
        let png = Path::new("image.png");
        let jpeg = Path::new("photo.jpeg");
        let webp = Path::new("graphic.webp");

        assert!(is_image(png));
        assert!(is_image(jpeg));
        assert!(is_image(webp));

        // Negative cases
        let txt = Path::new("document.txt");
        let pdf = Path::new("report.pdf");

        assert!(!is_image(txt));
        assert!(!is_image(pdf));
    }

    #[test]
    fn test_generate_sprite() -> Result<(), Box<dyn std::error::Error>> {
        // Set up a temporary directory with sample images
        let temp_dir = tempdir()?;
        let output_file = temp_dir.path().join("sprite.png");

        // Create a few sample images
        for i in 1..=3 {
            let img_path = temp_dir.path().join(format!("img{}.png", i));
            let mut img = RgbaImage::new(100, 100);
            img.put_pixel(50, 50, image::Rgba([255, 0, 0, 255])); // Add a red dot
            img.save(&img_path)?;
        }

        // Initialize Spriterator and generate the sprite
        let spriterator = Spriterator::new(
            temp_dir.path().to_str().unwrap(),
            output_file.to_str().unwrap(),
            300,
        );
        spriterator.generate()?;

        // Check that the output file was created
        assert!(output_file.exists());

        // Load the sprite to verify dimensions
        let sprite = image::open(&output_file)?.to_rgba8();
        assert_eq!(sprite.width(), 300); // max_size specified as 300
        assert!(sprite.height() >= 100); // At least enough height for one row

        Ok(())
    }

    #[test]
    fn test_generate_with_empty_directory() {
        let temp_dir = tempdir().unwrap();
        let output_file = temp_dir.path().join("sprite.png");

        let spriterator = Spriterator::new(
            temp_dir.path().to_str().unwrap(),
            output_file.to_str().unwrap(),
            300,
        );

        let result = spriterator.generate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "No images found in the specified directory."
        );
    }
}
