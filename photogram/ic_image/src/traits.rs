//a Imports
use std::path::Path;

use ic_base::{Point2D, Result};

use crate::LineIter;

/// Trait for an 8-bit color/greyscale
pub trait ImageColor: From<u8> {
    fn grey(x: u8) -> Self {
        x.into()
    }
    fn rgb(r: u8, g: u8, b: u8) -> Self {
        ((((r as u16) + (g as u16) + (b as u16)) / 3) as u8).into()
    }
    fn grey_u16(x: u16) -> Self {
        Self::grey((x >> 8) as u8)
    }
    fn rgb_u16(r: u16, g: u16, b: u16) -> Self {
        ((((r as u32) + (g as u32) + (b as u32)) / 0x300) as u8).into()
    }
}

/// Trait for a drawable image;
pub trait ImageDrawable {
    /// Pixel type, which must support simple generation from u8/u16 values
    type Pixel: ImageColor;
    /// Get the pixel at an (x,y) location
    fn get(&self, x: u32, y: u32) -> Self::Pixel;
    /// Set the pixel at an (x,y) location
    fn put(&mut self, x: u32, y: u32, color: &Self::Pixel);
    /// Set the pixel at an (x,y) location to a blend of the current value with the new value
    ///
    /// blend is the fraction of the current color to keep: 0.0 to replace the
    /// pixel completely, 1.0 to ignore the new color, and 0.5 for half-old,
    /// half-new
    fn blend(&mut self, x: u32, y: u32, blend: f64, color: &Self::Pixel);
    /// Get the size of the image
    fn size(&self) -> (u32, u32);

    /// Draw a cross on the image (plus sign) at the point of a given size in pixels
    fn draw_cross(&mut self, p: &Point2D, size: f64, color: &Self::Pixel) {
        let s = size.ceil() as u32;
        let cx = p[0] as u32;
        let cy = p[1] as u32;
        let (w, h) = self.size();
        if cx + s >= w || cx < s || cy + s >= h || cy < s {
            return;
        }
        for i in 0..(2 * s + 1) {
            self.put(cx - s + i, cy, color);
            self.put(cx, cy - s + i, color);
        }
    }

    /// Draw a cross on the image (X sign) at the point of a given size in pixels
    fn draw_x(&mut self, p: &Point2D, size: f64, color: &Self::Pixel) {
        let s = size.ceil() as u32;
        let cx = p[0] as u32;
        let cy = p[1] as u32;
        let (w, h) = self.size();
        if cx + s >= w || cx < s || cy + s >= h || cy < s {
            return;
        }
        for i in 0..(2 * s + 1) {
            self.put(cx - s + i, cy - s + i, color);
            self.put(cx + s - i, cy - s + i, color);
        }
    }

    /// Draw a line between the two points on the image
    fn draw_line(&mut self, p0: &Point2D, p1: &Point2D, color: &Self::Pixel) {
        let x0 = p0[0] as i32;
        let y0 = p0[1] as i32;
        let x1 = p1[0] as i32;
        let y1 = p1[1] as i32;
        let (w, h) = self.size();
        if let Some(line) = LineIter::new(x0, y0, x1, y1) {
            for (x, y) in line {
                if x >= w && y >= h {
                    break;
                }
                if x >= w || y >= h {
                    continue;
                }
                self.put(x, y, color);
            }
        }
    }
}

/// Trait for an image
pub trait Image: Sized + ImageDrawable {
    /// Create a new image of the given size
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
                if wh == img.size() {
                    Ok(img)
                } else {
                    let (w, h) = img.size();
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
