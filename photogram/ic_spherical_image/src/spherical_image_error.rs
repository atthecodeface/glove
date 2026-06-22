use thiserror::Error;

use crate::GreatCircleTriangleIndex;

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
    #[error("bad image size ({0:?} compared to {1:?})")]
    BadImageFileSize((u32, u32), (u32, u32)),
}

impl std::convert::From<SphericalImageError> for ic_base::Error {
    fn from(s: SphericalImageError) -> ic_base::Error {
        ic_base::Error::BoxError("SphericalImage", Box::new(s))
    }
}
