pub use image::{Luma, Rgba};
use serde::{Deserialize, Serialize};

use crate::ImageColor;

//a ImageColor for u16
impl ImageColor for u16 {}

//a Gray16
//tp Gray16
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Gray16(Luma<u16>);

//ip Display for Gray16 {
impl std::fmt::Display for Gray16 {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.as_string())
    }
}

//ip From<&u16> for Gray16
impl From<&u16> for Gray16 {
    fn from(c: &u16) -> Gray16 {
        Gray16([(*c)].into())
    }
}

//ip From<u16> for Gray16
impl From<u16> for Gray16 {
    fn from(c: u16) -> Gray16 {
        Gray16([c].into())
    }
}

//ip TryFrom<&str> for Gray16
impl TryFrom<&str> for Gray16 {
    type Error = String;
    fn try_from(s: &str) -> Result<Gray16, String> {
        if s == "None" {
            Ok(Gray16::none())
        } else if s.starts_with('#') {
            let l = s.len();
            if l != 3 && l != 5 {
                Err(format!("Expected #GGGG or #GG for Gray16, got {s}"))
            } else {
                let short_gray = s.len() < 5;
                match u16::from_str_radix(s.split_at(1).1, 16) {
                    Ok(gray) => {
                        if short_gray {
                            Ok((gray * 0x101).into())
                        } else {
                            Ok(gray.into())
                        }
                    }
                    Err(e) => Err(format!("Expected #GGGG or #GG for Gray16, got {s} : {e}")),
                }
            }
        } else {
            Err(format!("Expected #GGGG or #GG for Gray16, got {s}"))
        }
    }
}

//ip Gray16
impl Gray16 {
    //cp none
    #[inline]
    pub fn none() -> Self {
        0.into()
    }

    //cp black
    #[inline]
    pub fn black() -> Self {
        0.into()
    }

    //cp color_eq
    #[inline]
    pub fn color_eq(&self, other: &Self) -> bool {
        self.0[0] == other.0[0]
    }

    //cp brightness
    #[inline]
    pub fn brightness(&self) -> f32 {
        (self.0[0] as f32) / 65536.0
    }

    //cp as_string
    pub fn as_string(&self) -> String {
        format!("#{:04x}", self.0[0])
    }
}

//ip Serialize for Gray16
impl Serialize for Gray16 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_string().serialize(serializer)
    }
}

//ip Deserialize for Gray16
impl<'de> Deserialize<'de> for Gray16 {
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let color_str = String::deserialize(deserializer)?;
        color_str.as_str().try_into().map_err(DE::Error::custom)
    }
}

impl ImageColor for Color8 {
    fn rgb(r: u8, g: u8, b: u8) -> Self {
        [r, g, b, 255].into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color8(pub Rgba<u8>);

impl std::default::Default for Color8 {
    fn default() -> Self {
        Color8([0, 0, 0, 0].into())
    }
}

//ip Display for Color {
impl std::fmt::Display for Color8 {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}", self.as_string())
    }
}

//ip From<&[u8; 4]> for Color
impl From<&[u8; 4]> for Color8 {
    fn from(c: &[u8; 4]) -> Color8 {
        Color8((*c).into())
    }
}

//ip From<[u8; 4]> for Color
impl From<[u8; 4]> for Color8 {
    fn from(c: [u8; 4]) -> Color8 {
        Color8(c.into())
    }
}

//ip From<u8> for Color
impl From<u8> for Color8 {
    fn from(c: u8) -> Color8 {
        [c, c, c, 255].into()
    }
}

//ip TryFrom<&str> for Color
impl TryFrom<&str> for Color8 {
    type Error = String;
    fn try_from(s: &str) -> Result<Color8, String> {
        if s == "None" {
            Ok(Color8::none())
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
                            let a = { if has_alpha { (rgb >> 12) & 0xf } else { 15 } };
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
impl Color8 {
    //cp none
    #[inline]
    pub fn none() -> Self {
        Color8([0, 0, 0, 0].into())
    }

    //cp black
    #[inline]
    pub fn black() -> Self {
        Color8([0, 0, 0, 255].into())
    }

    //cp color_eq
    #[inline]
    pub fn color_eq(&self, other: &Self) -> bool {
        self.0[0] == other.0[0] && self.0[1] == other.0[1] && self.0[2] == other.0[2]
    }

    //cp brightness
    #[inline]
    pub fn brightness(&self) -> f32 {
        ((self.0[0] as f32) + (self.0[1] as f32) + (self.0[2] as f32)) / 768.0
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

    pub fn to_hls(&self) -> (f32, f32, f32) {
        let red = (self.0[0] as f32) / 255.0;
        let green = (self.0[1] as f32) / 255.0;
        let blue = (self.0[2] as f32) / 255.0;
        let rgb_min = red.min(green.min(blue));
        let rgb_max = red.max(green.max(blue));

        let lightness = (rgb_max + rgb_min) / 2.0;
        if rgb_min == rgb_max {
            (0.0, lightness, 0.0)
        } else {
            let chroma = rgb_max - rgb_min;
            let saturation = chroma / (1.0 - (lightness * 2.0 - 1.0).abs());
            let mut hue = {
                if rgb_max == red {
                    (green - blue) / chroma
                } else if rgb_max == green {
                    (blue - red) / chroma + 2.0
                } else {
                    (red - green) / chroma + 4.0
                }
            };
            hue = (hue + 6.0) * 60.0;
            if hue > 360.0 {
                hue -= 360.0;
            }
            (hue, lightness, saturation)
        }
    }

    pub fn of_hls((hue, lightness, saturation): (f32, f32, f32)) -> Self {
        let chroma = saturation * (1.0 - (lightness * 2.0 - 1.0).abs());
        if chroma < 1E-6 {
            return ((lightness * 255.0) as u8).into();
        }
        // lightness = (Max + Min)/2
        // Chroma = Max - min
        let rgb_max = lightness + chroma * 0.5;
        // sector_hue is 1..7
        let sector_hue = hue / 60.0 + 1.0;
        // sector is 0/3 (red is max), 1 (green is max), 2 (blue is max)
        let sector = (sector_hue * 0.5).floor();
        // subhue is -0.5 to 0.5
        let subhue = (sector_hue * 0.5).fract() - 0.5;
        // rgb0 is min -> max as hue increases
        let rgb0 = lightness + subhue * chroma;
        // rgb1 is max -> min as hue increases
        let rgb1 = lightness - subhue * chroma;

        let (red, green, blue) = {
            match sector as u8 {
                1 => (rgb1, rgb_max, rgb0),
                2 => (rgb0, rgb1, rgb_max),
                _ => (rgb_max, rgb0, rgb1),
            }
        };
        let red = (red * 255.0) as u8;
        let green = (green * 255.0) as u8;
        let blue = (blue * 255.0) as u8;
        [red, green, blue, 255].into()
    }
}

impl Serialize for Color8 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_string().serialize(serializer)
    }
}

//ip Deserialize for Color
impl<'de> Deserialize<'de> for Color8 {
    fn deserialize<DE>(deserializer: DE) -> Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let color_str = String::deserialize(deserializer)?;
        color_str.as_str().try_into().map_err(DE::Error::custom)
    }
}
