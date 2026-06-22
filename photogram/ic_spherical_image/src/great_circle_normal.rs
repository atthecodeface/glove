use crate::NormalIndex;
use ic_base::Point3D;
use indexed::Idx;

#[derive(Debug)]
pub struct GcNormal(NormalIndex, Point3D);

impl GcNormal {
    pub fn new(idx: NormalIndex, vec: Point3D) -> Self {
        Self(idx, vec)
    }
    pub fn index(&self) -> NormalIndex {
        self.0
    }
    pub fn inv_index(&self) -> NormalIndex {
        let i = self.0.index();
        NormalIndex::from_usize(i ^ 1)
    }
    pub fn lower_index(&self) -> NormalIndex {
        let i = self.0.index();
        NormalIndex::from_usize(i & !1)
    }
    pub fn vector(&self) -> &Point3D {
        &self.1
    }
}
