//a Imports
use std::io::Cursor;
use std::path::Path;

use image::io::Reader as ImageReader;
pub use image::{DynamicImage, GenericImage, GenericImageView};
use serde::{Deserialize, Serialize};

use crate::Point2D;

//a Color
//tp Color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(image::Rgba<u8>);
impl std::default::Default for Color {
    fn default() -> Self {
        Color([0, 0, 0, 0].into())
    }
}

//ip Display for Color {
impl std::fmt::Display for Color {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.as_string())
    }
}

//ip From<&[u8; 4]> for Color
impl From<&[u8; 4]> for Color {
    fn from(c: &[u8; 4]) -> Color {
        Color((*c).into())
    }
}

//ip From<[u8; 4]> for Color
impl From<[u8; 4]> for Color {
    fn from(c: [u8; 4]) -> Color {
        Color(c.into())
    }
}

//ip TryFrom<&str> for Color
impl TryFrom<&str> for Color {
    type Error = String;
    fn try_from(s: &str) -> Result<Color, String> {
        if s == "None" {
            Ok(Color::none())
        } else if s.starts_with('#') {
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

//ip Color
impl Color {
    //cp none
    #[inline]
    pub fn none() -> Self {
        Color([0, 0, 0, 0].into())
    }

    //cp black
    #[inline]
    pub fn black() -> Self {
        Color([0, 0, 0, 255].into())
    }

    //cp color_eq
    #[inline]
    pub fn color_eq(&self, other: &Self) -> bool {
        self.0[0] == other.0[0] && self.0[1] == other.0[1] && self.0[2] == other.0[2]
    }

    //cp as_string
    pub fn as_string(&self) -> String {
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
        self.as_string().serialize(serializer)
    }
}

//ip Deserialize for Color
impl<'de> Deserialize<'de> for Color {
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let color_str = String::deserialize(deserializer)?;
        color_str.as_str().try_into().map_err(DE::Error::custom)
    }
}

//a Public functions
//fp read_image
pub fn read_image<P: AsRef<Path>>(path: P) -> Result<DynamicImage, String> {
    let img = ImageReader::open(path)
        .map_err(|e| format!("Failed to open file {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image {}", e))?;
    Ok(img)
}

//fp read_or_create_image
pub fn read_or_create_image(
    width: usize,
    height: usize,
    opt_filename: Option<&str>,
) -> Result<DynamicImage, String> {
    let width = width as u32;
    let height = height as u32;
    if let Some(filename) = opt_filename {
        let img = read_image(filename)?;
        if img.width() != width || img.height() != height {
            Err(format!(
                "Image read has incorrect dimensions of ({},{}) instead of ({width},{height})",
                img.width(),
                img.height()
            ))
        } else {
            Ok(img)
        }
    } else {
        Ok(DynamicImage::new_rgb8(width, height))
    }
}

//a Image trait
pub trait Image: GenericImage + GenericImageView {
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), String>;
    fn encode(&self, extension: &str) -> Result<Vec<u8>, String>;
    fn get(&self, x: u32, y: u32) -> Color;
    fn put(&mut self, x: u32, y: u32, color: &Color);
    fn size(&self) -> (u32, u32);
    fn draw_cross(&mut self, p: Point2D, size: f64, color: &Color) {
        let s = size.ceil() as u32;
        let cx = p[0] as u32;
        let cy = p[1] as u32;
        if cx + s >= self.width() || cx < s || cy + s >= self.height() || cy < s {
            return;
        }
        for i in 0..(2 * s + 1) {
            self.put(cx - s + i, cy, color);
            self.put(cx, cy - s + i, color);
        }
    }
}
impl Image for DynamicImage {
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        self.save(path)
            .map_err(|e| format!("Failed to encode image {}", e))?;
        Ok(())
    }
    fn encode(&self, extension: &str) -> Result<Vec<u8>, String> {
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
            .map_err(|e| format!("Failed to encode image {}", e))?;
        Ok(bytes)
    }
    fn put(&mut self, x: u32, y: u32, color: &Color) {
        image::GenericImage::put_pixel(self, x, y, color.0);
    }
    fn get(&self, x: u32, y: u32) -> Color {
        Color(self.get_pixel(x, y))
    }
    fn size(&self) -> (u32, u32) {
        (self.width(), self.height())
    }
}
