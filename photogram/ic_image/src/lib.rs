use image::{ColorType, ImageReader};
use std::path::PathBuf;

mod color;
mod image_luma;
mod image_pt;
mod image_rgb;
mod image_square;
mod line_iter;
mod regions;
mod traits;

mod patch;

pub use patch::{FromPatchFn, ImagePatch};

pub use color::{Color8, Gray16};
pub use image_pt::ImagePt;
pub(crate) use line_iter::LineIter;
pub use traits::{Image, ImageColor, ImageConvert, ImageDraw};

pub use image_luma::{ImageGray16, Luma16Image, LumaF32Image};
pub use image_rgb::ImageRgb8;

pub use image_square::{ImageSquareSet, ImageSquares};
pub use regions::Region;

use ic_base::PathSet;

mod image_cache;
pub use image_cache::{ImageCache, ImageCacheEntry};

pub use image::{GenericImage, GenericImageView};

/// Read a path - relative to a [PathSet] - as an image, returning it as either
/// an ImageRgb8 or an ImageGray16 depending on the kind of file.
///
/// The full pathname of the image read is also returned
pub fn read_image<P: AsRef<std::path::Path> + std::fmt::Display>(
    path_set: &PathSet,
    path: P,
) -> ic_base::Result<(PathBuf, Option<ImageRgb8>, Option<ImageGray16>)> {
    if let Some(path) = path_set.find_file(&path) {
        let img = ImageReader::open(&path)?.with_guessed_format()?.decode()?;
        let path = path.to_owned();
        match img.color() {
            ColorType::Rgb8 => Ok((path, Some(img.into_rgb8()), None)),
            ColorType::L16 => Ok((path, None, Some(img.into_luma16()))),
            _ => Ok((path, Some(img.into_rgb8()), None)),
        }
    } else {
        Err(format!("Failed to find image file {path}").into())
    }
}
