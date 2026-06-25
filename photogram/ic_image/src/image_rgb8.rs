use std::io::Cursor;
use std::path::Path;

use image::{DynamicImage, GenericImageView, ImageReader};

use ic_base::Result;

use crate::{Color, Image, ImageDrawable, ImageGray16};

//a ImageRbg8
#[derive(Debug, Clone)]
pub struct ImageRgb8(DynamicImage);

//ip Deref for ImageRgb8
impl std::ops::Deref for ImageRgb8 {
    type Target = DynamicImage;
    fn deref(&self) -> &DynamicImage {
        &self.0
    }
}

impl std::ops::DerefMut for ImageRgb8 {
    fn deref_mut(&mut self) -> &mut DynamicImage {
        &mut self.0
    }
}

//ip ImageRgb8
impl ImageRgb8 {
    pub(crate) fn buffer(&self) -> &image::DynamicImage {
        &self.0
    }

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

    pub fn of_gray(image: &ImageGray16) -> Self {
        let image = image.buffer().to_rgb8();
        Self(image.into())
    }

    pub fn from_image(image: &image::DynamicImage) -> Self {
        let image = image.to_rgb8();
        Self(image.into())
    }

    pub fn of_image(i: DynamicImage) -> std::result::Result<Self, DynamicImage> {
        if i.as_rgba8().is_some() {
            Ok(Self(i))
        } else {
            Err(i)
        }
    }
}

//ip ImageDrawable for ImageRgb8
impl ImageDrawable for ImageRgb8 {
    type Pixel = Color;

    fn put(&mut self, x: u32, y: u32, color: &Color) {
        image::GenericImage::put_pixel(&mut self.0, x, y, color.0);
    }
    fn get(&self, x: u32, y: u32) -> Color {
        Color(self.0.get_pixel(x, y))
    }
    fn blend(&mut self, x: u32, y: u32, blend: f64, color: &Color) {
        let img = self.0.as_mut_rgb8().unwrap();
        let pixel = img.get_pixel_mut(x, y);
        let r = (color.0[0] as f64) * (1.0 - blend) + (blend * (pixel[0] as f64));
        let g = (color.0[1] as f64) * (1.0 - blend) + (blend * (pixel[1] as f64));
        let b = (color.0[2] as f64) * (1.0 - blend) + (blend * (pixel[2] as f64));
        pixel[0] = r as u8;
        pixel[1] = g as u8;
        pixel[2] = b as u8;
    }
    fn size(&self) -> (u32, u32) {
        (self.0.width(), self.0.height())
    }
}

//ip Image for ImageRgb8
impl Image for ImageRgb8 {
    fn new(width: u32, height: u32) -> Self {
        Self(DynamicImage::new_rgb8(width, height))
    }

    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.0
            .save(path)
            .map_err(|e| format!("Failed to encode image {e}"))?;
        Ok(())
    }

    fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let img = ImageReader::open(&path)?.decode()?.into_rgb8();
        Ok(Self(img.into()))
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
}
