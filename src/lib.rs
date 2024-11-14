use image::imageops::crop_imm;
use image::{DynamicImage, GenericImage, RgbaImage};
use std::error::Error;
use walkdir::WalkDir;

/// Supported file extensions for image files.
const SUPPORTED_EXTENSIONS: [&str; 2] = ["png", "webp"];

/// A struct that generates sprite sheets by arranging images in a grid layout
/// and trimming transparent areas from each sprite.
pub struct Spriterator {
    dir_path: String,
    max_width: u32,
    max_height: u32,
}

impl Spriterator {
    /// Creates a new `Spriterator` instance.
    ///
    /// # Arguments
    ///
    /// * `dir_path` - Path to the directory containing images.
    /// * `max_width` - Maximum width for each sprite.
    /// * `max_height` - Maximum height for each sprite.
    pub fn new(dir_path: &str, max_width: u32, max_height: u32) -> Self {
        Self {
            dir_path: dir_path.to_string(),
            max_width,
            max_height,
        }
    }

    /// Generates a vector of sprites by arranging images in rows, respecting the specified maximum
    /// width and height. Each sprite is trimmed to remove transparent areas.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `RgbaImage` objects on success, or an error if image
    /// loading or processing fails.
    pub fn generate(&self) -> Result<Vec<RgbaImage>, Box<dyn Error>> {
        let images = self.get_images()?;

        let mut sprites = Vec::new();
        let mut current_sprite = RgbaImage::new(self.max_width, self.max_height);
        let (mut current_x, mut current_y, mut row_height) = (0, 0, 0);

        for img in &images {
            // Move to the next row if the current image exceeds max width
            if current_x + img.width() > self.max_width {
                current_y += row_height;
                current_x = 0;
                row_height = 0;
            }

            // Start a new sprite if the current image exceeds max height
            if current_y + img.height() > self.max_height {
                let trimmed_sprite = self.trim_transparent(&current_sprite);
                sprites.push(trimmed_sprite);

                current_sprite = RgbaImage::new(self.max_width, self.max_height);
                current_x = 0;
                current_y = 0;
                row_height = 0;
            }

            current_sprite.copy_from(img, current_x, current_y)?;
            row_height = row_height.max(img.height());
            current_x += img.width();
        }

        let trimmed_sprite = self.trim_transparent(&current_sprite);
        sprites.push(trimmed_sprite);

        Ok(sprites)
    }

    /// Retrieves images from the specified directory that match the supported file extensions
    /// and checks their dimensions.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `DynamicImage` objects on success, or an error if
    /// no valid images are found or loading fails.
    fn get_images(&self) -> Result<Vec<DynamicImage>, Box<dyn Error>> {
        let images: Vec<DynamicImage> = WalkDir::new(&self.dir_path)
            .into_iter()
            .filter_map(|entry| {
                let path = entry.ok()?.path().to_path_buf();

                let is_image = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
                    .unwrap_or(false);

                if path.is_file() && is_image {
                    let img = image::open(&path)
                        .map_err(|e| {
                            eprintln!("Error opening image {}: {}", path.display(), e);
                            e
                        })
                        .ok()?;

                    if img.width() > self.max_width || img.height() > self.max_height {
                        eprintln!(
                            "Error: Image {} dimensions {}x{} exceed max dimensions {}x{}.",
                            path.display(),
                            img.width(),
                            img.height(),
                            self.max_width,
                            self.max_height
                        );
                        return None;
                    }

                    Some(img)
                } else {
                    None
                }
            })
            .collect();

        if images.is_empty() {
            return Err(format!(
                "No images with supported extensions {:?} were found in the specified directory.",
                SUPPORTED_EXTENSIONS
            )
            .into());
        }

        Ok(images)
    }

    /// Trims transparent areas from the sprite by cropping to the smallest non-transparent area.
    ///
    /// # Arguments
    ///
    /// * `sprite` - The sprite image to trim.
    ///
    /// # Returns
    ///
    /// An `RgbaImage` containing the trimmed sprite.
    fn trim_transparent(&self, sprite: &RgbaImage) -> RgbaImage {
        let (mut max_x, mut max_y) = (0, 0);
        let mut min_x = sprite.width();
        let mut min_y = sprite.height();

        for (x, y, pixel) in sprite.enumerate_pixels() {
            if pixel[3] > 0 {
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                min_x = min_x.min(x);
                min_y = min_y.min(y);
            }
        }

        let is_completely_transparent = max_x == 0 && max_y == 0 && sprite.get_pixel(0, 0)[3] == 0;
        if is_completely_transparent {
            return RgbaImage::new(1, 1);
        }

        crop_imm(sprite, min_x, min_y, max_x - min_x + 1, max_y - min_y + 1).to_image()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_spriterator_creation() {
        let spriterator = Spriterator::new("test_dir", 1024, 1024);
        assert_eq!(spriterator.dir_path, "test_dir");
        assert_eq!(spriterator.max_width, 1024);
        assert_eq!(spriterator.max_height, 1024);
    }

    #[test]
    fn test_empty_directory_error() {
        let spriterator = Spriterator::new("empty_dir", 1024, 1024);
        let result = spriterator.generate();
        assert!(result.is_err());
    }

    #[test]
    fn test_trim_transparent() {
        let spriterator = Spriterator::new("test_dir", 1024, 1024);
        let mut image = RgbaImage::new(10, 10);
        for x in 2..8 {
            for y in 2..8 {
                image.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        let trimmed = spriterator.trim_transparent(&image);
        assert_eq!(trimmed.width(), 6);
        assert_eq!(trimmed.height(), 6);
    }
}
