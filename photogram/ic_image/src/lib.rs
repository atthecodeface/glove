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
        let (width, color) = match self.style {
            0 => (10.0, &[255, 0, 255, 255].into()),
            1 => (20.0, &[0, 255, 255, 255].into()),
            _ => (30.0, &[125, 125, 125, 255].into()),
        };
        img.draw_cross([self.px as f64, self.py as f64].into(), width, color);
    }
}
