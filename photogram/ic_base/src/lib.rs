//a Documentation
/*! Documentation

!*/

//a Modules
mod error;
pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub mod json;
mod path_set;
pub use path_set::PathSet;
mod plane;
mod quadtree;
pub mod types;
pub mod utils;
mod word_xy;
pub use word_xy::WordXy;

pub use plane::Plane;
pub use types::{Mat3x3, Point2D, Point3D, Point4D, Quat, RollYaw, TanXTanY};

mod ray;
pub use ray::Ray;

mod mesh;
pub use mesh::Mesh;
pub use utils::Rrc;

pub use quadtree::{QtPath, Quadtree};
