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

/// `Spriterator` is a struct that provides functionality to generate compact sprite sheets from images.
/// It allows specifying maximum width and height, creating multiple sheets if necessary.
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
    /// * `max_width` - Maximum width of each sprite sheet in pixels.
    /// * `max_height` - Maximum height of each sprite sheet in pixels.
    pub fn new(dir_path: &str, max_width: u32, max_height: u32) -> Self {
        Self {
            dir_path: dir_path.to_string(),
            max_width,
            max_height,
        }
    }

    /// Generates multiple sprite sheets from images in the specified directory with no spacing.
    ///
    /// This method arranges images row by row, minimizing empty space. If images exceed the specified
    /// maximum height, a new sprite sheet is created. Returns a vector of `RgbaImage` sprites.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<RgbaImage>)` containing the generated sprite sheets.
    /// * `Err` if no images are found or an error occurs during processing.
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
        let mut current_x = 0;
        let mut current_y = 0;
        let mut row_height = 0;

        for img in &images {
            if current_x + img.width() > self.max_width {
                // Start a new row if the image does not fit in the current row
                current_y += row_height;
                current_x = 0;
                row_height = 0;
            }

            if current_y + img.height() > self.max_height {
                // Save the current sprite and start a new one if the image does not fit in the current sprite
                sprites.push(current_sprite);
                current_sprite = RgbaImage::new(self.max_width, self.max_height);
                current_y = 0;
            }

            // Copy the image into the current sprite
            current_sprite.copy_from(img, current_x, current_y)?;
            row_height = row_height.max(img.height());
            current_x += img.width();
        }

        // Add the last sprite to the vector if it's not empty
        sprites.push(current_sprite);

        Ok(sprites)
    }
}

/// Checks if a file has an extension that matches common image formats.
fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| IMAGE_EXTENSIONS.contains(ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}
