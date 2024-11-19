use image::{imageops::FilterType, RgbaImage};
use std::error::Error;
use walkdir::WalkDir;

use crate::sprite::Sprite;

const SUPPORTED_EXTENSIONS: [&str; 2] = ["png", "webp"];

/// Represents a spritesheet generator.
#[derive(Debug)]
pub struct Spriterator {
    dir_path: String,
    max_width: u32,
    max_height: u32,
    image_width: Option<u32>,
    image_height: Option<u32>,
}

impl Spriterator {
    /// Creates a new `Spriterator` instance.
    ///
    /// # Arguments
    /// - `dir_path`: Path to the directory containing images.
    /// - `max_width`: Maximum width of the spritesheet.
    /// - `max_height`: Maximum height of the spritesheet.
    /// - `image_width`: Optional target width for resizing images.
    /// - `image_height`: Optional target height for resizing images.
    ///
    /// # Returns
    /// A new `Spriterator` instance.
    pub fn new(
        dir_path: &str,
        max_width: u32,
        max_height: u32,
        image_width: Option<u32>,
        image_height: Option<u32>,
    ) -> Self {
        Self {
            dir_path: dir_path.to_string(),
            max_width,
            max_height,
            image_width,
            image_height,
        }
    }

    /// Generates a list of sprites from the images in the specified directory.
    ///
    /// # Returns
    /// A `Result` containing a vector of `Sprite` instances on success, or an error on failure.
    pub fn generate(&self) -> Result<Vec<Sprite>, Box<dyn Error>> {
        let images = self.get_images()?;

        let mut sprites: Vec<Sprite> = Vec::new();
        let mut current_sprite = RgbaImage::new(self.max_width, self.max_height);
        let mut current_frames: Vec<(u32, u32, u32, u32)> = Vec::new();
        let (mut current_x, mut current_y, mut row_height) = (0, 0, 0);

        for img in images {
            if current_x + img.width() > self.max_width {
                current_y += row_height;
                current_x = 0;
                row_height = 0;
            }

            if current_y + img.height() > self.max_height {
                if !current_frames.is_empty() {
                    let mut trimmed_sprite = Sprite::new(self.trim_transparent(&current_sprite));
                    for &(x, y, width, height) in &current_frames {
                        trimmed_sprite.add_frame(x, y, width, height);
                    }
                    sprites.push(trimmed_sprite);
                }

                current_sprite = RgbaImage::new(self.max_width, self.max_height);
                current_frames.clear();
                current_x = 0;
                current_y = 0;
                row_height = 0;
            }

            image::imageops::overlay(
                &mut current_sprite,
                &img,
                current_x as i64,
                current_y as i64,
            );
            current_frames.push((current_x, current_y, img.width(), img.height()));

            row_height = row_height.max(img.height());
            current_x += img.width();
        }

        if !current_frames.is_empty() {
            let mut trimmed_sprite = Sprite::new(self.trim_transparent(&current_sprite));
            for &(x, y, width, height) in &current_frames {
                trimmed_sprite.add_frame(x, y, width, height);
            }
            sprites.push(trimmed_sprite);
        }

        Ok(sprites)
    }

    fn get_images(&self) -> Result<Vec<RgbaImage>, Box<dyn Error>> {
        let images: Vec<RgbaImage> = WalkDir::new(&self.dir_path)
            .into_iter()
            .filter_map(|entry| {
                let path = entry.ok()?.path().to_path_buf();

                let is_image = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
                    .unwrap_or(false);

                if path.is_file() && is_image {
                    let img = image::open(&path).ok()?.to_rgba8();

                    if (self.image_width.is_none() && img.width() > self.max_width)
                        || (self.image_height.is_none() && img.height() > self.max_height)
                    {
                        return Some(Err::<RgbaImage, Box<dyn Error>>(
                            format!(
                                "Image {} dimensions {}x{} exceed max dimensions {}x{}.",
                                path.display(),
                                img.width(),
                                img.height(),
                                self.max_width,
                                self.max_height
                            )
                            .into(),
                        ));
                    } else {
                        Some(Ok(self.resize_image(img)))
                    }
                } else {
                    None
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        if images.is_empty() {
            return Err(format!(
                "No images with supported extensions {:?} were found in the specified directory: {}",
                SUPPORTED_EXTENSIONS,
                self.dir_path
            )
            .into());
        }

        Ok(images)
    }

    fn trim_transparent(&self, image: &RgbaImage) -> RgbaImage {
        let (mut max_x, mut max_y) = (0, 0);
        let mut min_x = image.width();
        let mut min_y = image.height();
        let mut is_completely_transparent = true;

        for (x, y, pixel) in image.enumerate_pixels() {
            if pixel[3] > 0 {
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                is_completely_transparent = false;
            }
        }

        if is_completely_transparent {
            return RgbaImage::new(1, 1);
        }

        image::imageops::crop_imm(image, min_x, min_y, max_x - min_x + 1, max_y - min_y + 1)
            .to_image()
    }

    fn resize_image(&self, img: RgbaImage) -> RgbaImage {
        let (original_width, original_height) = img.dimensions();

        match (self.image_width, self.image_height) {
            (Some(width), Some(height)) => {
                image::imageops::resize(&img, width, height, FilterType::Lanczos3)
            }
            (Some(width), None) => {
                let height = (original_height * width) / original_width;
                image::imageops::resize(&img, width, height, FilterType::Lanczos3)
            }
            (None, Some(height)) => {
                let width = (original_width * height) / original_height;
                image::imageops::resize(&img, width, height, FilterType::Lanczos3)
            }
            (None, None) => img,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn test_spriterator_creation() {
        let spriterator = Spriterator::new("test_dir", 1024, 1024, None, None);
        assert_eq!(spriterator.dir_path, "test_dir");
        assert_eq!(spriterator.max_width, 1024);
        assert_eq!(spriterator.max_height, 1024);
    }

    #[test]
    fn test_empty_directory_error() {
        let spriterator = Spriterator::new("empty_dir", 1024, 1024, None, None);
        let result = spriterator.generate();
        assert!(result.is_err());
    }

    #[test]
    fn test_trim_transparent() {
        let spriterator = Spriterator::new("test_dir", 1024, 1024, None, None);
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

    #[test]
    fn test_resize_image1() {
        let spriterator = Spriterator::new("test_dir", 100, 100, Some(10), Some(10));
        let resized = spriterator.resize_image(RgbaImage::new(10, 10));
        assert_eq!(resized.width(), 10);
        assert_eq!(resized.height(), 10);
    }

    #[test]
    fn test_resize_image2() {
        let spriterator = Spriterator::new("test_dir", 100, 100, Some(10), None);
        let resized = spriterator.resize_image(RgbaImage::new(20, 20));
        assert_eq!(resized.width(), 10);
        assert_eq!(resized.height(), (20 * 10) / 20);
    }

    #[test]
    fn test_resize_image3() {
        let spriterator = Spriterator::new("test_dir", 100, 100, None, Some(10));
        let resized = spriterator.resize_image(RgbaImage::new(30, 30));
        assert_eq!(resized.width(), (30 * 10) / 30);
        assert_eq!(resized.height(), 10);
    }
}
