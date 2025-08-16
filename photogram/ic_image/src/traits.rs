//a Imports
use std::path::Path;

use image::DynamicImage;

use ic_base::{Point2D, Result};

use crate::LineIter;

//a Image trait
pub trait ImageColor: From<u8> {
    fn grey(x: u8) -> Self {
        x.into()
    }
    fn rgb(r: u8, g: u8, b: u8) -> Self {
        ((((r as u16) + (g as u16) + (b as u16)) / 3) as u8).into()
    }
}

//tt ImageDrawable
pub trait ImageDrawable {
    type Pixel: ImageColor;
    fn get(&self, x: u32, y: u32) -> Self::Pixel;
    fn put(&mut self, x: u32, y: u32, color: &Self::Pixel);
    fn size(&self) -> (u32, u32);

    //mp Provided functions
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

//tt Image
pub trait Image: ImageDrawable {
    fn new(width: usize, height: usize) -> Self;
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()>;
    fn encode(&self, extension: &str) -> Result<Vec<u8>>;
}
