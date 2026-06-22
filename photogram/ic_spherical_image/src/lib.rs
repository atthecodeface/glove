mod subdivision;
pub use subdivision::SubdivisionPath;

mod spherical_data;
pub use spherical_data::{
    GreatCircleLineIndex, GreatCircleTriangleIndex, NormalIndex, PtIndex, SphericalData,
};
mod great_circle_line;
pub use great_circle_line::GcLine;
mod great_circle_normal;
pub use great_circle_normal::GcNormal;
mod great_circle_triangle;
pub use great_circle_triangle::GcTriangle;
mod image_pt;
use image_pt::ImagePt;
mod sd_index;
pub use sd_index::SdIndex;
mod sd_vector;
pub use sd_vector::SdSubtriangle;

mod spherical_image_patch;
pub use spherical_image_patch::ImagePatch;
pub mod shapes;
mod spherical_image_shape;
mod spherical_patch;
pub use spherical_image_shape::SphericalImageShape;
pub use spherical_patch::{SphericalPatch, SphericalPatchDescriptor};
mod spherical_image;
pub use spherical_image::{SphericalImage, SphericalImageDescriptor};

use thiserror::Error;
#[derive(Debug, Error)]
pub enum SphericalImageError {
    #[error("bad shape {0}")]
    BadShape(String),
    #[error("bad triangle index {0}")]
    BadTriangleIndex(usize),
    #[error("bad subdivison path (subdivision of {0}, path of {1:#0x}")]
    BadSubdivisionPath(u8, u64),
    #[error("triangles do not share an edge ({0:?} {1:?})")]
    TrianglesDoNotShareAnEdge(GreatCircleTriangleIndex, GreatCircleTriangleIndex),
    #[error("failed to do something: {0}")]
    Something(String),
}
impl std::convert::From<SphericalImageError> for ic_base::Error {
    fn from(s: SphericalImageError) -> ic_base::Error {
        ic_base::Error::BoxError("SphericalImage", Box::new(s))
    }
}
