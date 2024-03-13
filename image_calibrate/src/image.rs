//a Imports
use image::io::Reader as ImageReader;
pub use image::{DynamicImage, GenericImage, GenericImageView, Rgba};

use crate::Point2D;

//a Public functions
//fp read_image
pub fn read_image(filename: &str) -> Result<DynamicImage, String> {
    let img = ImageReader::open(filename)
        .map_err(|e| format!("Failed to open file {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image {}", e))?;
    Ok(img)
}

//fp write_image
pub fn write_image(img: &mut DynamicImage, filename: &str) -> Result<(), String> {
    img.save(filename)
        .map_err(|e| format!("Failed to encode image {}", e))?;
    Ok(())
}

//fp draw_cross
pub fn draw_cross(img: &mut DynamicImage, p: Point2D, size: f64, color: &[u8; 4]) {
    let color = Rgba(*color);
    let s = size.ceil() as u32;
    let cx = p[0] as u32;
    let cy = p[1] as u32;
    if cx + s >= img.width() || cx < s || cy + s >= img.height() || cy < s {
        return;
    }
    for i in 0..(2 * s + 1) {
        img.put_pixel(cx - s + i, cy, color);
        img.put_pixel(cx, cy - s + i, color);
    }
}
