//a Imports
use image::io::Reader as ImageReader;
use image::GenericImage;

use crate::Point2D;

//a Public functions
//fp read_image
pub fn read_image(filename: &str) -> Result<image::DynamicImage, String> {
    let img = ImageReader::open(filename)
        .map_err(|e| format!("Failed to open file {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode jpeg {}", e))?;
    Ok(img)
}

//fp write_image
pub fn write_image(img: &mut image::DynamicImage, filename: &str) -> Result<(), String> {
    image::save_buffer(
        filename,
        img.as_bytes(),
        img.width(),
        img.height(),
        image::ColorType::Rgb8,
    )
    .map_err(|e| format!("Failed to encode jpeg {}", e))?;
    Ok(())
}

//fp draw_cross
pub fn draw_cross(img: &mut image::DynamicImage, p: &Point2D, size: f64, color: &[u8; 4]) {
    let color = image::Rgba(*color);
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
