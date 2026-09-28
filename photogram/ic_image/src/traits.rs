//a Imports
use std::{io::Cursor, path::Path};

use ic_base::{Point2D, Result};
use image::{GenericImage, ImageBuffer, Luma, Rgb, RgbImage, RgbaImage};

use crate::LineIter;

/// Trait for an 8-bit color/greyscale
pub trait ImageColor: Sized {
    fn grey(x: u8) -> Self;
    fn rgb(r: u8, g: u8, b: u8) -> Self;
    fn grey_u16(x: u16) -> Self;
    fn rgb_u16(r: u16, g: u16, b: u16) -> Self;
    fn black() -> Self {
        Self::grey(0)
    }
    fn white() -> Self {
        Self::grey(255)
    }
}

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

pub trait ImageDraw: GenericImage {
    fn resized(&self, as_width: Option<u32>) -> (u32, u32) {
        let size = self.dimensions();
        as_width
            .map(|w| {
                (
                    w,
                    (((w as usize) * (size.1 as usize)) / (size.0 as usize)) as u32,
                )
            })
            .unwrap_or(size)
    }

    /// Draw a cross on the image (plus sign) at the point of a given size in pixels
    fn draw_cross(&mut self, p: &Point2D, size: f64, color: Self::Pixel) {
        let s = size.ceil() as u32;
        let cx = p[0] as u32;
        let cy = p[1] as u32;
        let (w, h) = self.dimensions();
        if cx + s >= w || cx < s || cy + s >= h || cy < s {
            return;
        }
        for i in 0..(2 * s + 1) {
            self.put_pixel(cx - s + i, cy, color);
            self.put_pixel(cx, cy - s + i, color);
        }
    }

    /// Draw a cross on the image (X sign) at the point of a given size in pixels
    fn draw_x(&mut self, p: &Point2D, size: f64, color: Self::Pixel) {
        let s = size.ceil() as u32;
        let cx = p[0] as u32;
        let cy = p[1] as u32;
        let (w, h) = self.dimensions();
        if cx + s >= w || cx < s || cy + s >= h || cy < s {
            return;
        }
        for i in 0..(2 * s + 1) {
            self.put_pixel(cx - s + i, cy - s + i, color);
            self.put_pixel(cx + s - i, cy - s + i, color);
        }
    }

    /// Draw a line between the two points on the image
    fn draw_line(&mut self, p0: &Point2D, p1: &Point2D, color: Self::Pixel) {
        let x0 = p0[0] as i32;
        let y0 = p0[1] as i32;
        let x1 = p1[0] as i32;
        let y1 = p1[1] as i32;
        let (w, h) = self.dimensions();
        if let Some(line) = LineIter::new(x0, y0, x1, y1) {
            for (x, y) in line {
                if x >= w && y >= h {
                    break;
                }
                if x >= w || y >= h {
                    continue;
                }
                self.put_pixel(x, y, color);
            }
        }
    }
}
impl<T> ImageDraw for T where T: GenericImage {}

/// A trait for images in this library
pub trait Image: ImageDraw + Sized {
    /// Create a new image of the given size
    ///
    /// Required in this trait so that 'read_or_create' can invoke it
    fn new(width: u32, height: u32) -> Self;

    /// Write the image to a path using an appropriate encoder
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()>;

    /// Read an image from the given path
    fn read<P: AsRef<Path>>(path: P) -> Result<Self>;

    /// Encode the image to an array of bytes (perhaps to send to a browser, for example)
    fn encode(&self, extension: &str) -> Result<Vec<u8>>;

    /// Read an image if possible, else create it and the given size
    fn read_or_create_image<P: AsRef<Path>>(
        opt_filename: Option<P>,
        opt_img_wh: Option<(u32, u32)>,
    ) -> Result<Self> {
        if let Some(filename) = opt_filename {
            let img = Self::read(filename)?;
            if let Some(wh) = opt_img_wh {
                if wh == img.dimensions() {
                    Ok(img)
                } else {
                    let (w, h) = img.dimensions();
                    let (width, height) = wh;
                    Err(format!(
                        "Image read has incorrect dimensions of ({w},{h}) instead of ({width},{height})",
                    )
                    .into())
                }
            } else {
                Ok(img)
            }
        } else if let Some((width, height)) = opt_img_wh {
            Ok(Self::new(width, height))
        } else {
            panic!("Must provide filename or width+height");
        }
    }
}

pub type Luma16Image = ImageBuffer<Luma<u16>, Vec<u16>>;
pub type LumaF32Image = ImageBuffer<Luma<f32>, Vec<f32>>;

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

pub trait ImageConvert: Sized {
    /// Creeate a new LumaF32Image of this image resized, using 'scale' to map components to the F32 value
    fn as_luma_f32(&self, as_width: Option<u32>, scale: f32) -> LumaF32Image;
    fn as_rgb8(&self, as_width: Option<u32>, scale: f32) -> RgbImage;
    fn as_rgba8(&self, as_width: Option<u32>, scale: f32) -> RgbaImage;
    fn as_luma16(&self, as_width: Option<u32>, scale: f32) -> Luma16Image;
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
    fn as_rgb8(&self, as_width: Option<u32>, scale: f32) -> RgbImage {
        todo!();
    }
    fn as_rgba8(&self, as_width: Option<u32>, scale: f32) -> RgbaImage {
        todo!();
    }
    fn as_luma16(&self, as_width: Option<u32>, scale: f32) -> Luma16Image {
        todo!();
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
