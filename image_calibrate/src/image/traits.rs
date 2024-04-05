//a Imports
use std::path::Path;

pub use crate::Color;

use crate::Point2D;

//a Image trait
pub trait Image {
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), String>;
    fn encode(&self, extension: &str) -> Result<Vec<u8>, String>;
    fn get(&self, x: u32, y: u32) -> Color;
    fn put(&mut self, x: u32, y: u32, color: &Color);
    fn size(&self) -> (u32, u32);
    fn draw_cross(&mut self, p: Point2D, size: f64, color: &Color) {
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
