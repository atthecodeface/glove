use ic_base::Point2D;

mod traits;
pub use traits::Image;
mod color;
pub use color::{Color, Gray16};

mod image_gray16;
mod image_rgb8;
pub use image_gray16::ImageGray16;
pub use image_rgb8::ImageRgb8;

mod patch;
pub use patch::Patch;

mod regions;
pub use regions::Region;

//a ImagePt
//tp ImagePt
#[derive(Debug, Default, Clone, Copy)]
pub struct ImagePt {
    px: f32,
    py: f32,
    style: u8,
}
impl std::convert::From<(usize, usize, u8)> for ImagePt {
    fn from((px, py, style): (usize, usize, u8)) -> ImagePt {
        ImagePt {
            px: px as f32,
            py: py as f32,
            style,
        }
    }
}
impl std::convert::From<(isize, isize, u8)> for ImagePt {
    fn from((px, py, style): (isize, isize, u8)) -> ImagePt {
        ImagePt {
            px: px as f32,
            py: py as f32,
            style,
        }
    }
}
impl std::convert::From<(Point2D, u8)> for ImagePt {
    fn from((pt, style): (Point2D, u8)) -> ImagePt {
        ImagePt {
            px: pt[0] as f32,
            py: pt[1] as f32,
            style,
        }
    }
}

impl ImagePt {
    pub fn draw(&self, img: &mut ImageRgb8) {
        match self.style {
            3 => {
                let (w, h) = img.size();
                if self.px < 0.0 || self.py < 0.0 || self.px >= w as f32 || self.py >= h as f32 {
                    return;
                }
                let color = [40, 255, 40, 255].into();
                img.put(self.px as u32, self.py as u32, &color);
            }
            0 => {
                let color = [255, 0, 255, 255].into();
                img.draw_cross([self.px as f64, self.py as f64].into(), 10.0, &color);
            }
            1 => {
                let color = [0, 255, 255, 255].into();
                img.draw_cross([self.px as f64, self.py as f64].into(), 20.0, &color);
            }
            _ => {
                let color = [255, 255, 125, 255].into();
                img.draw_x([self.px as f64, self.py as f64].into(), 20.0, &color);
            }
        };
    }
}
