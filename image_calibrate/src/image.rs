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
