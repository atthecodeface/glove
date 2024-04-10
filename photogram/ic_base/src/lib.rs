//a Documentation
/*! Documentation

!*/

//a Modules
pub mod json;
pub mod types;
pub mod utils;

pub use types::{Mat3x3, Point2D, Point3D, Point4D, Quat, RollYaw, TanXTanY};

mod ray;
pub use ray::Ray;

mod mesh;
pub use mesh::Mesh;
pub use utils::Rrc;
