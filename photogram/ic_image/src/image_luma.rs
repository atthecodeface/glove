use std::{io::Cursor, path::Path};

use ic_base::Result;
use image::{ImageBuffer, Luma, RgbaImage};

use crate::{Image, ImageConvert, ImageDraw, ImageRgb8};

pub type ImageLuma16 = ImageBuffer<Luma<u16>, Vec<u16>>;
pub type ImageLumaF32 = ImageBuffer<Luma<f32>, Vec<f32>>;

impl Image for ImageLuma16 {
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

impl ImageConvert for ImageLuma16 {
    fn as_luma_f32(&self, as_width: Option<u32>, scale: f32) -> ImageLumaF32 {
        let (width, height) = self.resized(as_width);
        let mut img = ImageLumaF32::new(width, height);
        let width = width as usize;
        let height = height as usize;
        let orig_width = self.dimensions().0 as usize;
        let orig_height = self.dimensions().1 as usize;
        let scale = scale / 65536.0;
        for (x, y, p) in img.enumerate_pixels_mut() {
            let sy = (y as usize) * orig_height / height;
            let sx = (x as usize) * orig_width / width;
            let l = scale * (self[(sx as u32, sy as u32)][0] as f32);
            *p = [(l.max(0.0).min(1.0))].into();
        }
        img
    }
    fn as_rgb8(&self, as_width: Option<u32>, scale: f32) -> ImageRgb8 {
        todo!();
    }
    fn as_rgba8(&self, as_width: Option<u32>, scale: f32) -> RgbaImage {
        todo!();
    }
    fn as_luma16(&self, as_width: Option<u32>, scale: f32) -> ImageLuma16 {
        if as_width.is_none() && scale == 1.0 {
            return self.clone();
        }
        let (width, height) = self.resized(as_width);
        let mut img = ImageLuma16::new(width, height);
        let width = width as usize;
        let height = height as usize;
        let orig_width = self.dimensions().0 as usize;
        let orig_height = self.dimensions().1 as usize;
        for (x, y, p) in img.enumerate_pixels_mut() {
            let sy = (y as usize) * orig_height / height;
            let sx = (x as usize) * orig_width / width;
            let l = scale * (self[(sx as u32, sy as u32)][0] as f32);
            *p = [(l.max(0.0).min(65535.0)) as u16].into();
        }
        img
    }
}

impl Image for ImageLumaF32 {
    fn new(width: u32, height: u32) -> Self {
        Self::new(width, height)
    }

    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        Err(format!(
            "Cannot save {:?} as LumaF32 has no save mechanism",
            path.as_ref(),
        )
        .into())
    }

    fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        Err(format!(
            "Cannot load {:?} as LumaF32 has no read mechanism",
            path.as_ref(),
        )
        .into())
    }

    fn encode(&self, _extension: &str) -> Result<Vec<u8>> {
        Err(format!("Cannot enode LumaF32 has no save mechanism",).into())
    }
}

impl ImageConvert for ImageLumaF32 {
    fn as_luma_f32(&self, as_width: Option<u32>, scale: f32) -> ImageLumaF32 {
        if as_width.is_none() && scale == 1.0 {
            return self.clone();
        }
        let (width, height) = self.resized(as_width);
        let mut img = ImageLumaF32::new(width, height);
        let width = width as usize;
        let height = height as usize;
        let orig_width = self.dimensions().0 as usize;
        let orig_height = self.dimensions().1 as usize;
        for (x, y, p) in img.enumerate_pixels_mut() {
            let sy = (y as usize) * orig_height / height;
            let sx = (x as usize) * orig_width / width;
            let l = self[(sx as u32, sy as u32)][0] * scale;
            *p = [l].into();
        }
        img
    }
    fn as_rgb8(&self, as_width: Option<u32>, scale: f32) -> ImageRgb8 {
        todo!();
    }
    fn as_rgba8(&self, as_width: Option<u32>, scale: f32) -> RgbaImage {
        todo!();
    }
    fn as_luma16(&self, as_width: Option<u32>, scale: f32) -> ImageLuma16 {
        let (width, height) = self.resized(as_width);
        let mut img = ImageLuma16::new(width, height);
        let width = width as usize;
        let height = height as usize;
        let orig_width = self.dimensions().0 as usize;
        let orig_height = self.dimensions().1 as usize;
        let scale = 65536.0 * scale;
        for (x, y, p) in img.enumerate_pixels_mut() {
            let sy = (y as usize) * orig_height / height;
            let sx = (x as usize) * orig_width / width;
            let l = scale * self[(sx as u32, sy as u32)][0];
            *p = [(l.max(0.0).min(65535.0)) as u16].into();
        }
        img
    }
}
