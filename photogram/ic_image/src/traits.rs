//a Imports
use std::path::Path;

use ic_base::{Point2D, Result};

//a Image trait
pub trait Image {
    type Pixel;
    fn new(width: usize, height: usize) -> Self;
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()>;
    fn encode(&self, extension: &str) -> Result<Vec<u8>>;
    fn get(&self, x: u32, y: u32) -> Self::Pixel;
    fn put(&mut self, x: u32, y: u32, color: &Self::Pixel);
    fn size(&self) -> (u32, u32);
    fn draw_cross(&mut self, p: Point2D, size: f64, color: &Self::Pixel) {
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
}
