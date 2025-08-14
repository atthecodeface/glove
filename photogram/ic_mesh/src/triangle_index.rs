//a Imports
use std::default::Default;

use serde::{Deserialize, Serialize};


//a TriangleIndex
//tp TriangleIndex
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct TriangleIndex {
    t: usize,
}

//ip From<usize> for TriangleIndex
impl From<usize> for TriangleIndex {
    fn from(t: usize) -> TriangleIndex {
        TriangleIndex { t }
    }
}

//ip Display for TriangleIndex
impl std::fmt::Display for TriangleIndex {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "Tr({})", self.t)
    }
}

//ip PartialEq<usize> for TriangleIndex
impl std::cmp::PartialEq<usize> for TriangleIndex {
    fn eq(&self, t: &usize) -> bool {
        self.t == *t
    }
}

//ip PartialEq<&usize> for TriangleIndex
impl std::cmp::PartialEq<&usize> for TriangleIndex {
    fn eq(&self, t: &&usize) -> bool {
        self.t == **t
    }
}

//ip TriangleIndex
impl TriangleIndex {
    pub fn as_usize(self) -> usize {
        self.t
    }
}
