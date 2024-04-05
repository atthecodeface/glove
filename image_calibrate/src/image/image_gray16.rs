//a Imports
use std::io::Cursor;
use std::path::Path;

use crate::image::{Color, Image, ImageRgb8};
use image::io::Reader as ImageReader;
pub use image::DynamicImage;

use image::{GenericImage, GenericImageView, ImageBuffer, Luma};

//a ImageGray16
#[derive(Debug)]
pub struct ImageGray16(DynamicImage);

//a ip ImageGray16
impl ImageGray16 {
    //cp read_image
    pub fn read_image<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let img: ImageBuffer<Luma<u16>, Vec<u16>> = ImageReader::open(path)
            .map_err(|e| format!("Failed to open file {}", e))?
            .decode()
            .map_err(|e| format!("Failed to decode image {}", e))?
            .into_luma16();
        Ok(Self(img.into()))
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
            Ok(Self(DynamicImage::new_luma16(width, height)))
        }
    }

    //ac buffer
    pub(crate) fn buffer(&self) -> &image::DynamicImage {
        &self.0
    }

    //cp of_rgb
    pub fn of_rgb(image: &ImageRgb8) -> Self {
        let image = image.buffer().to_luma16();
        Self(image.into())
    }
}

//ip Image for ImageGray16
impl Image for ImageGray16 {
    type Pixel = u16;
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
    fn put(&mut self, x: u32, y: u32, color: &Self::Pixel) {
        let img = self.0.as_mut_luma16().unwrap();
        image::GenericImage::put_pixel(img, x, y, [*color].into());
    }
    fn get(&self, x: u32, y: u32) -> Self::Pixel {
        let img = self.0.as_luma16().unwrap();
        img.get_pixel(x, y).0[0]
    }
    fn size(&self) -> (u32, u32) {
        (self.0.width(), self.0.height())
    }
}
