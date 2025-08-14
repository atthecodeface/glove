//a Imports
use std::default::Default;

use serde::{Deserialize, Serialize};


//a LineIndex
//tp LineIndex
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct LineIndex {
    l: usize,
}

//ip From<usize> for LineIndex {
impl From<usize> for LineIndex {
    fn from(l: usize) -> LineIndex {
        LineIndex { l }
    }
}

//ip std::fmt::Display for LineIndex {
impl std::fmt::Display for LineIndex {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "Ln({})", self.l)
    }
}

//ip PartialEq<usize> for LineIndex
impl std::cmp::PartialEq<usize> for LineIndex {
    fn eq(&self, l: &usize) -> bool {
        self.l == *l
    }
}

//ip PartialEq<&usize> for LineIndex
impl std::cmp::PartialEq<&usize> for LineIndex {
    fn eq(&self, l: &&usize) -> bool {
        self.l == **l
    }
}

//ip LineIndex
impl LineIndex {
    pub fn as_usize(self) -> usize {
        self.l
    }
}
