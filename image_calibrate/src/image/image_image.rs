//a Imports
use std::io::Cursor;
use std::path::Path;

use crate::image::{Color, Image};
use image::io::Reader as ImageReader;
pub use image::DynamicImage;

use image::{GenericImage, GenericImageView};

//a ImageBuffer
pub struct ImageBuffer(DynamicImage);

//a Public functions
impl ImageBuffer {
    //cp read_image
    pub fn read_image<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let img = ImageReader::open(path)
            .map_err(|e| format!("Failed to open file {}", e))?
            .decode()
            .map_err(|e| format!("Failed to decode image {}", e))?;
        Ok(Self(img))
    }

    //fp read_or_create_image
    pub fn read_or_create_image(
        width: usize,
        height: usize,
        opt_filename: Option<&str>,
    ) -> Result<Self, String> {
        let width = width as u32;
        let height = height as u32;
        if let Some(filename) = opt_filename {
            let img = Self::read_image(filename)?;
            let (w, h) = img.size();
            if w != width || h != height {
                Err(format!(
                    "Image read has incorrect dimensions of ({},{}) instead of ({width},{height})",
                    w, h,
                ))
            } else {
                Ok(img)
            }
        } else {
            Ok(Self(DynamicImage::new_rgb8(width, height)))
        }
    }
}

//ip Image for ImageBuffer
impl Image for ImageBuffer {
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        self.0
            .save(path)
            .map_err(|e| format!("Failed to encode image {}", e))?;
        Ok(())
    }
    fn encode(&self, extension: &str) -> Result<Vec<u8>, String> {
        let format = {
            match extension {
                "jpg" => image::ImageFormat::Jpeg,
                "jpeg" => image::ImageFormat::Jpeg,
                "png" => image::ImageFormat::Png,
                _ => Err(format!("Unknown image format {extension}"))?,
            }
        };

        let mut bytes: Vec<u8> = Vec::new();
        self.0
            .write_to(&mut Cursor::new(&mut bytes), format)
            .map_err(|e| format!("Failed to encode image {}", e))?;
        Ok(bytes)
    }
    fn put(&mut self, x: u32, y: u32, color: &Color) {
        image::GenericImage::put_pixel(&mut self.0, x, y, color.0);
    }
    fn get(&self, x: u32, y: u32) -> Color {
        Color(self.0.get_pixel(x, y))
    }
    fn size(&self) -> (u32, u32) {
        (self.0.width(), self.0.height())
    }
}
