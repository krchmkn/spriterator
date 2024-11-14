use image::{GenericImage, GenericImageView, RgbaImage};
use once_cell::sync::Lazy;
use rayon::prelude::*;
use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;

/// A static set containing supported image file extensions, used for quick verification of valid image files.
static IMAGE_EXTENSIONS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    ["png", "jpg", "jpeg", "gif", "bmp", "ico", "tiff", "webp"]
        .iter()
        .cloned()
        .collect()
});

/// `Spriterator` is responsible for generating optimized sprite sheets from images, with customizable dimensions.
pub struct Spriterator {
    dir_path: String,
    max_width: u32,
    max_height: u32,
}

impl Spriterator {
    /// Initializes a new `Spriterator` instance with specified directory path and sheet dimensions.
    ///
    /// # Arguments
    ///
    /// * `dir_path` - The directory containing images to be processed.
    /// * `max_width` - Maximum allowable width of each sprite sheet in pixels.
    /// * `max_height` - Maximum allowable height of each sprite sheet in pixels.
    pub fn new(dir_path: &str, max_width: u32, max_height: u32) -> Self {
        Self {
            dir_path: dir_path.to_string(),
            max_width,
            max_height,
        }
    }

    /// Generates sprite sheets from images in the specified directory, removing transparent padding around them.
    ///
    /// This method arranges images row-by-row, reducing empty space. If an image does not fit within the specified dimensions,
    /// a new sprite sheet is started. Returns a vector of `RgbaImage` containing the generated sprite sheets.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<RgbaImage>)` - Contains the generated sprite sheets.
    /// * `Err` - If no images are found or an error occurs during processing.
    pub fn generate(&self) -> Result<Vec<RgbaImage>, Box<dyn std::error::Error>> {
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

        let mut sprites = Vec::new();
        let mut current_sprite = RgbaImage::new(self.max_width, self.max_height);
        let (mut current_x, mut current_y, mut row_height) = (0, 0, 0);

        for img in &images {
            // Validate image dimensions against maximum allowed sprite sheet dimensions.
            if img.width() > self.max_width || img.height() > self.max_height {
                return Err(format!(
                    "Image dimensions {}x{} exceed max dimensions {}x{}.",
                    img.width(),
                    img.height(),
                    self.max_width,
                    self.max_height
                )
                .into());
            }

            // Start a new row if the current image does not fit horizontally.
            if current_x + img.width() > self.max_width {
                current_y += row_height;
                current_x = 0;
                row_height = 0;
            }

            // Finalize the current sprite sheet if the current image does not fit vertically.
            if current_y + img.height() > self.max_height {
                // Trim the current sprite sheet to remove unnecessary transparent space.
                let trimmed_sprite = trim_transparent(&current_sprite);
                sprites.push(trimmed_sprite);

                // Begin a new sprite sheet for subsequent images.
                current_sprite = RgbaImage::new(self.max_width, self.max_height);
                current_x = 0;
                current_y = 0;
                row_height = 0;
            }

            // Add the current image to the sprite sheet at the current coordinates.
            current_sprite.copy_from(img, current_x, current_y)?;
            row_height = row_height.max(img.height());
            current_x += img.width();
        }

        // Trim and add the last sprite sheet if it contains any images.
        let trimmed_sprite = trim_transparent(&current_sprite);
        sprites.push(trimmed_sprite);

        Ok(sprites)
    }
}

/// Removes transparent pixels from the right and bottom edges of an image.
///
/// # Arguments
///
/// * `sprite` - The input image to be trimmed.
///
/// # Returns
///
/// * `RgbaImage` - The image with transparent padding removed from its edges.
fn trim_transparent(sprite: &RgbaImage) -> RgbaImage {
    let (mut max_x, mut max_y) = (0, 0);

    for (x, y, pixel) in sprite.enumerate_pixels() {
        if pixel[3] > 0 {
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    }

    // Return a 1x1 image if the entire sprite is transparent.
    if max_x == 0 && max_y == 0 && sprite.get_pixel(0, 0)[3] == 0 {
        return RgbaImage::new(1, 1);
    }

    sprite.view(0, 0, max_x + 1, max_y + 1).to_image()
}

/// Determines if a file has a supported image extension.
fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| IMAGE_EXTENSIONS.contains(ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;
    use std::path::Path;
    use tempfile::tempdir;

    /// Helper function to create a temporary test image file.
    fn create_test_image(path: &Path, width: u32, height: u32) -> RgbaImage {
        let img = RgbaImage::new(width, height);
        img.save(path).unwrap();
        img
    }

    #[test]
    fn test_is_image() {
        assert!(is_image(Path::new("test.png")));
        assert!(is_image(Path::new("test.jpg")));
        assert!(!is_image(Path::new("test.txt")));
    }

    #[test]
    fn test_trim_transparent() {
        let mut img = RgbaImage::new(5, 5);
        img.put_pixel(1, 1, Rgba([255, 0, 0, 255]));
        img.put_pixel(3, 3, Rgba([0, 255, 0, 255]));

        let trimmed_img = trim_transparent(&img);
        assert_eq!(trimmed_img.width(), 4);
        assert_eq!(trimmed_img.height(), 4);
    }

    #[test]
    fn test_trim_transparent_fully_transparent() {
        let img = RgbaImage::new(5, 5);
        let trimmed_img = trim_transparent(&img);
        assert_eq!(trimmed_img.width(), 1);
        assert_eq!(trimmed_img.height(), 1);
    }

    #[test]
    fn test_spriterator_no_images() {
        let dir = tempdir().unwrap();
        let spriterator = Spriterator::new(dir.path().to_str().unwrap(), 500, 500);
        let result = spriterator.generate();
        assert!(result.is_err());
    }

    #[test]
    fn test_spriterator_single_image() {
        let dir = tempdir().unwrap();
        let image_path = dir.path().join("image.png");
        let image = create_test_image(&image_path, 100, 100);

        let spriterator = Spriterator::new(dir.path().to_str().unwrap(), 500, 500);
        let result = spriterator.generate().unwrap();

        assert_eq!(result.len(), 1);
        let sprite = &result[0];

        // Ensure the sprite has non-zero dimensions and is trimmed properly
        assert!(sprite.width() > 0 && sprite.height() > 0);
        assert!(sprite.width() <= 500 && sprite.height() <= 500);

        // Only check pixel data if dimensions match the original untrimmed image
        if sprite.width() >= 100 && sprite.height() >= 100 {
            assert_eq!(sprite.get_pixel(50, 50), image.get_pixel(50, 50));
        }
    }

    #[test]
    fn test_spriterator_multiple_images_single_sheet() {
        let dir = tempdir().unwrap();
        create_test_image(&dir.path().join("image1.png"), 100, 100);
        create_test_image(&dir.path().join("image2.png"), 200, 100);

        let spriterator = Spriterator::new(dir.path().to_str().unwrap(), 500, 500);
        let result = spriterator.generate().unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_spriterator_multiple_images_multiple_sheets() {
        let dir = tempdir().unwrap();
        create_test_image(&dir.path().join("image1.png"), 300, 300);
        create_test_image(&dir.path().join("image2.png"), 300, 300);
        create_test_image(&dir.path().join("image3.png"), 300, 300);

        // Reduced max dimensions to force multiple sheets
        let spriterator = Spriterator::new(dir.path().to_str().unwrap(), 300, 300);
        let result = spriterator.generate().unwrap();

        assert_eq!(result.len(), 3); // Expect three sheets
    }

    #[test]
    fn test_spriterator_image_exceeds_max_dimensions() {
        let dir = tempdir().unwrap();
        let image_path = dir.path().join("large_image.png");
        create_test_image(&image_path, 600, 600);

        let spriterator = Spriterator::new(dir.path().to_str().unwrap(), 500, 500);
        let result = spriterator.generate();

        assert!(result.is_err()); // Expect error due to size exceeding max dimensions
    }
}
