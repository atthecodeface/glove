use serde::{Deserialize, Serialize};

use crate::SphericalImageError;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SubdivisionPath {
    subdivision: u8,
    path: u64,
}

impl SubdivisionPath {
    /// The mask of *permitted* bits given a subdivision level
    ///
    /// For 0 this is !0; for 1 this is 0xff...ffc; for 2 this is 0xff..f0, etc
    fn disallowed_bits_of_subdivision(subdivision: u8) -> u64 {
        (!0_u64) << (2 * (subdivision as u64))
    }
    pub fn subdivision(&self) -> u8 {
        self.subdivision
    }
    pub fn path(&self) -> u64 {
        self.path
    }
    pub fn of_subdivision_and_mask(
        subdivision: u8,
        path: u64,
    ) -> Result<Self, SphericalImageError> {
        if path & Self::disallowed_bits_of_subdivision(subdivision) != 0 {
            Err(SphericalImageError::BadSubdivisionPath(subdivision, path))
        } else {
            Ok(Self { subdivision, path })
        }
    }
    pub fn subpath(&self, subpath: u64) -> Self {
        assert!(self.subdivision < 32);
        let subdivision = self.subdivision + 1;

        let path = self.path | (subpath << (2 * (self.subdivision as u64)));
        Self { subdivision, path }
    }
    pub fn parent(&self) -> Self {
        assert!(self.subdivision > 0);
        let subdivision = self.subdivision - 1;
        let path = self.path & !Self::disallowed_bits_of_subdivision(subdivision);
        Self { subdivision, path }
    }
    pub fn subdivision_matches(&self, subdivision: u8) -> bool {
        self.subdivision == subdivision
    }
}
