//a Imports
use std::path::Path;

use ic_base::{Point2D, Result};

use crate::LineIter;

pub trait ImageColor: From<u8> {
    fn grey(x: u8) -> Self {
        x.into()
    }
    fn rgb(r: u8, g: u8, b: u8) -> Self {
        ((((r as u16) + (g as u16) + (b as u16)) / 3) as u8).into()
    }
}
pub trait ImageDrawable {
    type Pixel: ImageColor;
    fn get(&self, x: u32, y: u32) -> Self::Pixel;
    fn put(&mut self, x: u32, y: u32, color: &Self::Pixel);
    fn size(&self) -> (u32, u32);

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

pub trait Image: Sized + ImageDrawable {
    fn new(width: u32, height: u32) -> Self;
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()>;
    fn read<P: AsRef<Path>>(path: P) -> Result<Self>;
    fn encode(&self, extension: &str) -> Result<Vec<u8>>;
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
