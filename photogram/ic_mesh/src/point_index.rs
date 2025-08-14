//a Imports
use std::default::Default;

use serde::{Deserialize, Serialize};

//a PointIndex
//tp PointIndex
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct PointIndex {
    p: usize,
}

//ip From<usize> for PointIndex
impl From<usize> for PointIndex {
    fn from(p: usize) -> PointIndex {
        PointIndex { p }
    }
}

//ip From<PointIndex> for usize
impl From<PointIndex> for usize {
    fn from(pt: PointIndex) -> usize {
        pt.p
    }
}

//ip Display for PointIndex
impl std::fmt::Display for PointIndex {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "Pt({})", self.p)
    }
}

//ip PartialEq<usize> for PointIndex
impl std::cmp::PartialEq<usize> for PointIndex {
    fn eq(&self, p: &usize) -> bool {
        self.p == *p
    }
}

//ip PartialEq<&usize> for PointIndex
impl std::cmp::PartialEq<&usize> for PointIndex {
    fn eq(&self, p: &&usize) -> bool {
        self.p == **p
    }
}

//ip PointIndex
impl PointIndex {
    pub fn as_usize(self) -> usize {
        self.p
    }
}
