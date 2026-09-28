use std::{io::Cursor, path::Path};

use ic_base::Result;
use image::{ImageBuffer, Luma, RgbaImage};

use crate::{Image, ImageConvert, ImageDraw, ImageRgb8};

pub type ImageGray16 = Luma16Image;
pub type Luma16Image = ImageBuffer<Luma<u16>, Vec<u16>>;
pub type LumaF32Image = ImageBuffer<Luma<f32>, Vec<f32>>;

impl Image for Luma16Image {
    fn new(width: u32, height: u32) -> Self {
        Self::new(width, height)
    }

    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.save(path)
            .map_err(|e| format!("Failed to encode image {e}"))?;
        Ok(())
    }

    fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let img = image::ImageReader::open(&path)?.decode()?.into_luma16();
        Ok(img)
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
        self.write_to(&mut Cursor::new(&mut bytes), format)
            .map_err(|e| format!("Failed to encode image {e}"))?;
        Ok(bytes)
    }
}

impl ImageConvert for LumaF32Image {
    fn as_luma_f32(&self, as_width: Option<u32>, scale: f32) -> LumaF32Image {
        let (width, height) = self.resized(as_width);
        let mut img = LumaF32Image::new(width, height);
        let width = width as usize;
        let height = height as usize;
        let orig_width = self.dimensions().0 as usize;
        let orig_height = self.dimensions().0 as usize;
        let r_sc = 52.0 * scale / 65536.0;
        let g_sc = 177.0 * scale / 65536.0;
        let b_sc = 18.0 * scale / 65536.0;
        for (x, y, p) in img.enumerate_pixels_mut() {
            let sy = (y as usize) * orig_height / height;
            for x in 0..width {
                let sx = (x as usize) * orig_width / width;
                let rgba = self[(sx as u32, sy as u32)];
                let l = (rgba[0] as f32) * r_sc + (rgba[1] as f32) * g_sc + (rgba[2] as f32) * b_sc;
                *p = [l].into();
            }
        }
        img
    }
    fn as_rgb8(&self, as_width: Option<u32>, scale: f32) -> ImageRgb8 {
        todo!();
    }
    fn as_rgba8(&self, as_width: Option<u32>, scale: f32) -> RgbaImage {
        todo!();
    }
    fn as_luma16(&self, as_width: Option<u32>, scale: f32) -> Luma16Image {
        todo!();
    }
}
