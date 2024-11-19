use image::RgbaImage;

use crate::frame::Frame;

#[derive(Debug)]
pub struct Sprite {
    image: RgbaImage,
    frames: Vec<Frame>,
}

impl Sprite {
    pub fn new(image: RgbaImage) -> Self {
        Self {
            image,
            frames: Vec::new(),
        }
    }

    pub fn get_image(&self) -> &RgbaImage {
        &self.image
    }

    pub fn get_frames(&self) -> &Vec<Frame> {
        &self.frames
    }

    pub fn add_frame(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.frames.push(Frame::new(x, y, width, height));
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.image.save(path)?;
        Ok(())
    }
}
