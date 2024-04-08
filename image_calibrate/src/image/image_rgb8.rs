//a Imports
use std::io::Cursor;
use std::path::Path;

use crate::image::{Color, Image, ImageGray16};
use image::io::Reader as ImageReader;
pub use image::DynamicImage;

use image::{GenericImage, GenericImageView};

//a ImageRbg8
#[derive(Debug, Clone)]
pub struct ImageRgb8(DynamicImage);

//a Public functions
impl ImageRgb8 {
    //cp read_image
    pub fn read_image<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let img = ImageReader::open(path)
            .map_err(|e| format!("Failed to open file {}", e))?
            .decode()
            .map_err(|e| format!("Failed to decode image {}", e))?
            .into_rgb8();
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
            Ok(Self(DynamicImage::new_rgb8(width, height)))
        }
    }

    //ac buffer
    pub(crate) fn buffer(&self) -> &image::DynamicImage {
        &self.0
    }

    //mp as_vec_gray_u32
    pub fn as_vec_gray_u32(&self, as_width: Option<usize>) -> (usize, usize, Vec<u32>) {
        let size = self.size();
        let size = (size.0 as usize, size.1 as usize);
        let (width, height) = as_width.map(|w| (w, w * size.1 / size.0)).unwrap_or(size);
        let mut result: Vec<u32> = vec![0; width * height];
        let mut i = 0;
        let r_sc = 52;
        let g_sc = 177;
        let b_sc = 18;
        let img = self.0.as_rgb8().unwrap();
        for y in 0..height {
            let sy = y * size.1 / height;
            for x in 0..width {
                let sx = x * size.0 / width;
                let rgba = img[(sx as u32, sy as u32)];
                let l = (rgba[0] as u32) * r_sc + (rgba[1] as u32) * g_sc + (rgba[2] as u32) * b_sc;
                result[i] = l;
                i += 1;
            }
        }
        (width, height, result)
    }

    //cp of_gray
    pub fn of_gray(image: &ImageGray16) -> Self {
        let image = image.buffer().to_rgb8();
        Self(image.into())
    }
}

//ip Image for ImageRgb8
impl Image for ImageRgb8 {
    type Pixel = Color;

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
