mod color;
mod image_gray16;
mod image_pt;
mod image_rgb8;
mod image_square;
mod line_iter;
mod regions;
mod traits;

mod patch;

pub use patch::{FromPatchFn, ImagePatch};

pub use color::{Color8, Gray16};
pub use image_pt::ImagePt;
pub(crate) use line_iter::LineIter;
pub use traits::{Image, ImageColor, ImageDrawable};

pub use image_gray16::ImageGray16;
pub use image_rgb8::ImageRgb8;
pub use image_square::{ImageSquareSet, ImageSquares};
pub use regions::Region;

use ic_base::PathSet;
use image::ImageReader;
use std::path::PathBuf;

mod image_cache;
pub use image_cache::{ImageCache, ImageCacheEntry};

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
        let img = match ImageRgb8::of_image(img) {
            Ok(rgb) => {
                return Ok((path, Some(rgb), None));
            }
            Err(img) => img,
        };
        let img = match ImageGray16::of_image(img) {
            Ok(gray) => {
                return Ok((path, None, Some(gray)));
            }
            Err(img) => img,
        };
        Ok((path, Some(ImageRgb8::from_image(&img)), None))
    } else {
        Err(format!("Failed to find image file {path}").into())
    }
}
