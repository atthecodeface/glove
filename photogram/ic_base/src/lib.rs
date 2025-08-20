//a Documentation
/*! Documentation

!*/

//a Modules
mod error;
pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

mod json;
pub use json::{JsonParsable, JsonSrc};

mod path_set;
pub use path_set::{PathGlob, PathSet};
mod plane;
mod quadtree;
mod tag;
pub mod types;
pub mod utils;
pub use tag::{Tag, TagData, TagMap, TagSet};
mod word_xy;
pub use word_xy::WordXy;

pub use plane::Plane;
pub use types::{Mat3x3, Point2D, Point3D, Point4D, Quat, RollYaw, TanXTanY};

mod ray;
pub use ray::{NamedRayList, Ray};

mod mesh;
pub use mesh::Mesh;
pub use utils::Rrc;

pub use quadtree::{QtPath, Quadtree};

use std::path::Path;
pub fn image_name<S: AsRef<str>>(image_filename: S) -> String {
    let path: &Path = image_filename.as_ref().as_ref();
    if let Some(image) = path.file_stem() {
        image.to_string_lossy().into_owned()
    } else {
        image_filename.as_ref().into()
    }
}
