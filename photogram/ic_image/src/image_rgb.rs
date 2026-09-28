use std::{io::Cursor, path::Path};

use ic_base::{Point2D, Result};
use image::{GenericImage, ImageBuffer, Luma, Rgb, RgbImage, RgbaImage};

use crate::{Image, ImageColor, ImageConvert, ImageDraw, Luma16Image, LumaF32Image};

pub type ImageRgb8 = image::RgbImage;

impl ImageColor for Rgb<u8> {
    fn grey(x: u8) -> Self {
        [x, x, x].into()
    }
    fn rgb(r: u8, g: u8, b: u8) -> Self {
        [r, g, b].into()
    }
    fn grey_u16(x: u16) -> Self {
        Self::grey((x >> 8) as u8)
    }
    fn rgb_u16(r: u16, g: u16, b: u16) -> Self {
        Self::rgb((r >> 8) as u8, (g >> 8) as u8, (b >> 8) as u8)
    }
}

impl Image for RgbImage {
    fn new(width: u32, height: u32) -> Self {
        Self::new(width, height)
    }

    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.save(path)
            .map_err(|e| format!("Failed to encode image {e}"))?;
        Ok(())
    }

    fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let img = image::ImageReader::open(&path)?.decode()?.into_rgb8();
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
impl Image for RgbaImage {
    fn new(width: u32, height: u32) -> Self {
        Self::new(width, height)
    }

    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.save(path)
            .map_err(|e| format!("Failed to encode image {e}"))?;
        Ok(())
    }

    fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let img = image::ImageReader::open(&path)?.decode()?.into_rgba8();
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

impl ImageConvert for RgbImage {
    fn as_luma_f32(&self, as_width: Option<u32>, scale: f32) -> LumaF32Image {
        let (width, height) = self.resized(as_width);
        let mut img = LumaF32Image::new(width, height);
        let width = width as usize;
        let height = height as usize;
        let orig_width = self.dimensions().0 as usize;
        let orig_height = self.dimensions().1 as usize;
        let r_sc = 52.0 * scale / 65536.0;
        let g_sc = 177.0 * scale / 65536.0;
        let b_sc = 18.0 * scale / 65536.0;
        for (x, y, p) in img.enumerate_pixels_mut() {
            let sy = (y as usize) * orig_height / height;
            let sx = (x as usize) * orig_width / width;
            let rgba = self[(sx as u32, sy as u32)];
            let l = (rgba[0] as f32) * r_sc + (rgba[1] as f32) * g_sc + (rgba[2] as f32) * b_sc;
            *p = [l].into();
        }
        img
    }
    fn as_luma16(&self, as_width: Option<u32>, scale: f32) -> Luma16Image {
        let (width, height) = self.resized(as_width);
        let mut img = Luma16Image::new(width, height);
        let width = width as usize;
        let height = height as usize;
        let orig_width = self.dimensions().0 as usize;
        let orig_height = self.dimensions().1 as usize;
        let r_sc = 52.0 * scale.max(0.0);
        let g_sc = 177.0 * scale.max(0.0);
        let b_sc = 18.0 * scale.max(0.0);
        for (x, y, p) in img.enumerate_pixels_mut() {
            let sy = (y as usize) * orig_height / height;
            let sx = (x as usize) * orig_width / width;
            let rgba = self[(sx as u32, sy as u32)];
            let l = (rgba[0] as f32) * r_sc + (rgba[1] as f32) * g_sc + (rgba[2] as f32) * b_sc;
            *p = [l.min(65535.0) as u16].into();
        }
        img
    }
    fn as_rgb8(&self, as_width: Option<u32>, scale: f32) -> RgbImage {
        if as_width.is_none() && scale == 1.0 {
            return self.clone();
        }
        let (width, height) = self.resized(as_width);
        let mut img = RgbImage::new(width, height);
        let width = width as usize;
        let height = height as usize;
        let orig_width = self.dimensions().0 as usize;
        let orig_height = self.dimensions().1 as usize;
        let scale = (scale.max(0.0) * 256.0) as usize;
        for (x, y, p) in img.enumerate_pixels_mut() {
            let sy = (y as usize) * orig_height / height;
            let sx = (x as usize) * orig_width / width;
            let rgba = self[(sx as u32, sy as u32)];
            let r = (((rgba[0] as usize) * scale).min(0xffff) >> 8) as u8;
            let g = (((rgba[1] as usize) * scale).min(0xffff) >> 8) as u8;
            let b = (((rgba[2] as usize) * scale).min(0xffff) >> 8) as u8;
            *p = [r, g, b].into();
        }
        img
    }

    fn as_rgba8(&self, as_width: Option<u32>, scale: f32) -> RgbaImage {
        let (width, height) = self.resized(as_width);
        let mut img = RgbaImage::new(width, height);
        let width = width as usize;
        let height = height as usize;
        let orig_width = self.dimensions().0 as usize;
        let orig_height = self.dimensions().1 as usize;
        let scale = (scale.max(0.0) * 256.0) as usize;
        for (x, y, p) in img.enumerate_pixels_mut() {
            let sy = (y as usize) * orig_height / height;
            let sx = (x as usize) * orig_width / width;
            let rgba = self[(sx as u32, sy as u32)];
            let r = (((rgba[0] as usize) * scale).min(0xffff) >> 8) as u8;
            let g = (((rgba[1] as usize) * scale).min(0xffff) >> 8) as u8;
            let b = (((rgba[2] as usize) * scale).min(0xffff) >> 8) as u8;
            *p = [r, g, b, rgba[3]].into();
        }
        img
    }
}
