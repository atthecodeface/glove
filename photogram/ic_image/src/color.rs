pub use image::{Luma, Rgba};
use serde::{Deserialize, Serialize};

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

//a Color
//tp Color
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub Rgba<u8>);
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
