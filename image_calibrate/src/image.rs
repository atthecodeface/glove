//a Imports
use serde::Serialize;

use image::io::Reader as ImageReader;
pub use image::{DynamicImage, GenericImage, GenericImageView};

use crate::Point2D;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(image::Rgba<u8>);
impl std::default::Default for Color {
    fn default() -> Self {
        Color([0, 0, 0, 0].into())
    }
}

impl From<&[u8; 4]> for Color {
    fn from(c: &[u8; 4]) -> Color {
        Color((*c).into())
    }
}
impl From<[u8; 4]> for Color {
    fn from(c: [u8; 4]) -> Color {
        Color(c.into())
    }
}
impl TryFrom<&str> for Color {
    type Error = String;
    fn try_from(s: &str) -> Result<Color, String> {
        if s == "None" {
            Ok(Color::none())
        } else if s.starts_with("#") {
            let l = s.len();
            if l != 4 && l != 5 && l != 7 && l != 9 {
                Err(format!(
                    "Expected #RGB, #ARGB, #RRGGBB or #AARRGGBB for color, got {s}"
                ))
            } else {
                let short_rgb = s.len() < 7;
                let has_alpha = (s.len() == 5) || (s.len() == 9);
                match u32::from_str_radix(s.split_at(1).1, 16) {
                    Ok(rgb) => {
                        if short_rgb {
                            let a = {
                                if has_alpha {
                                    (rgb >> 12) & 0xf
                                } else {
                                    15
                                }
                            };
                            let r = (rgb >> 8) & 0xf;
                            let g = (rgb >> 4) & 0xf;
                            let b = rgb & 0xf;
                            let r = (r | (r << 4)) as u8;
                            let g = (g | (g << 4)) as u8;
                            let b = (b | (b << 4)) as u8;
                            let a = (a | (a << 4)) as u8;
                            Ok([r, g, b, a].into())
                        } else {
                            let a = {
                                if has_alpha {
                                    ((rgb >> 24) & 0xff) as u8
                                } else {
                                    255
                                }
                            };
                            let r = ((rgb >> 16) & 0xff) as u8;
                            let g = ((rgb >> 8) & 0xff) as u8;
                            let b = (rgb & 0xff) as u8;
                            Ok([r, g, b, a].into())
                        }
                    }
                    Err(e) => Err(format!(
                        "Expected #RGB, #ARGB, #RRGGBB or #AARRGGBB for color, got {s} : {e}"
                    )),
                }
            }
        } else {
            Err(format!(
                "Expected #RGB, #ARGB, #RRGGBB or #AARRGGBB for color, got {s}"
            ))
        }
    }
}
impl Color {
    fn none() -> Self {
        Color([0, 0, 0, 0].into())
    }
    fn to_string(&self) -> String {
        if self.0[3] == 255 {
            format!("#{:02x}{:02x}{:02x}", self.0[0], self.0[1], self.0[2],)
        } else if self.0[3] == 0 {
            "None".into()
        } else {
            format!(
                "#{:02x}{:02x}{:02x}{:02x}",
                self.0[3], self.0[0], self.0[1], self.0[2],
            )
        }
    }
}

//ip Serialize for Color
impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

//a Public functions
//fp read_image
pub fn read_image(filename: &str) -> Result<DynamicImage, String> {
    let img = ImageReader::open(filename)
        .map_err(|e| format!("Failed to open file {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image {}", e))?;
    Ok(img)
}

//a Image trait
pub trait Image: GenericImage + GenericImageView {
    fn write(&self, filename: &str) -> Result<(), String>;
    fn get(&self, x: u32, y: u32) -> Color;
    fn put(&mut self, x: u32, y: u32, color: &Color);
    fn draw_cross(&mut self, p: Point2D, size: f64, color: &[u8; 4]) {
        let color: Color = color.into();
        let s = size.ceil() as u32;
        let cx = p[0] as u32;
        let cy = p[1] as u32;
        if cx + s >= self.width() || cx < s || cy + s >= self.height() || cy < s {
            return;
        }
        for i in 0..(2 * s + 1) {
            self.put(cx - s + i, cy, &color);
            self.put(cx, cy - s + i, &color);
        }
    }
}
impl Image for DynamicImage {
    fn write(&self, filename: &str) -> Result<(), String> {
        self.save(filename)
            .map_err(|e| format!("Failed to encode image {}", e))?;
        Ok(())
    }
    fn put(&mut self, x: u32, y: u32, color: &Color) {
        image::GenericImage::put_pixel(self, x, y, color.0);
    }
    fn get(&self, x: u32, y: u32) -> Color {
        Color(self.get_pixel(x, y))
    }
}
