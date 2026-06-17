use crate::PtIndex;
use ic_base::Point3D;

/// A point on the sphere used in one or more great circle lines
///
/// The `vec` are not meant to be copied or cloned, as they are unique, and so
/// this does not support Clone
#[derive(Debug)]
pub struct ImagePt {
    pt: PtIndex,
    vec: Point3D,
}

impl ImagePt {
    pub fn new(pt: PtIndex, vec: Point3D) -> Self {
        Self { pt, vec }
    }
    pub fn index(&self) -> PtIndex {
        self.pt
    }
    pub fn vector(&self) -> &Point3D {
        &self.vec
    }
}
