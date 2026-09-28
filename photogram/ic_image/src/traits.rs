//a Imports
use std::{io::Cursor, path::Path};

use ic_base::{Point2D, Result};
use image::{GenericImage, ImageBuffer, Luma, Rgb, RgbImage, RgbaImage};

use crate::{ImageLuma16, ImageLumaF32, ImageRgb8, ImageRgba8, LineIter};

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

pub trait ImageConvert: Sized {
    /// Creeate a new LumaF32Image of this image resized, using 'scale' to map components to the F32 value
    fn as_luma_f32(&self, as_width: Option<u32>, scale: f32) -> ImageLumaF32;
    fn as_rgb8(&self, as_width: Option<u32>, scale: f32) -> ImageRgb8;
    fn as_rgba8(&self, as_width: Option<u32>, scale: f32) -> ImageRgba8;
    fn as_luma16(&self, as_width: Option<u32>, scale: f32) -> ImageLuma16;
}
