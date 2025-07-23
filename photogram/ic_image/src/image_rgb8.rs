//a Imports
use std::io::Cursor;
use std::path::Path;

use image::DynamicImage;
use image::GenericImageView;
use image::ImageReader;

use ic_base::Result;

use crate::{Color, Image, ImageGray16};

//a ImageRbg8
#[derive(Debug, Clone)]
pub struct ImageRgb8(DynamicImage);

//a Public functions
impl ImageRgb8 {
    //cp read_image
    pub fn read_image<P: AsRef<Path>>(path: P) -> Result<Self> {
        let img = ImageReader::open(path)?.decode()?.into_rgb8();
        Ok(Self(img.into()))
    }

    //fp read_or_create_image
    pub fn read_or_create_image(
        width: usize,
        height: usize,
        opt_filename: Option<&str>,
    ) -> Result<Self> {
        let width = width as u32;
        let height = height as u32;
        if let Some(filename) = opt_filename {
            let img = Self::read_image(filename)?;
            let (w, h) = img.size();
            if w != width || h != height {
                Err(format!(
                    "Image read has incorrect dimensions of ({w},{h}) instead of ({width},{height})",
                )
                .into())
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

    //mp as_vec_gray_f32
    pub fn as_vec_gray_f32(&self, as_width: Option<usize>) -> (usize, usize, Vec<f32>) {
        let size = self.size();
        let size = (size.0 as usize, size.1 as usize);
        let (width, height) = as_width.map(|w| (w, w * size.1 / size.0)).unwrap_or(size);
        let mut result: Vec<f32> = vec![0.0; width * height];
        let mut i = 0;
        let r_sc = 52.0;
        let g_sc = 177.0;
        let b_sc = 18.0;
        let img = self.0.as_rgb8().unwrap();
        for y in 0..height {
            let sy = y * size.1 / height;
            for x in 0..width {
                let sx = x * size.0 / width;
                let rgba = img[(sx as u32, sy as u32)];
                let l = (rgba[0] as f32) * r_sc + (rgba[1] as f32) * g_sc + (rgba[2] as f32) * b_sc;
                result[i] = l / 65536.0;
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

    fn new(width: usize, height: usize) -> Self {
        Self(DynamicImage::new_rgb8(width as u32, height as u32))
    }
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.0
            .save(path)
            .map_err(|e| format!("Failed to encode image {e}"))?;
        Ok(())
    }
    fn encode(&self, extension: &str) -> Result<Vec<u8>> {
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
            .map_err(|e| format!("Failed to encode image {e}"))?;
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
