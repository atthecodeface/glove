use serde::{Deserialize, Serialize};

use crate::SphericalImageError;

/// A subdivision of a (toplevel) triangle, which is a depth of subdivision and
/// a hierarchy path consisting of 2 bits per hierarchy level
///
/// To indicate no subdivision the level is 0, and the path is 0. This is the default().
///
/// Given the triangle is P0, P1, P2, the subdivision of the triangle into 4
/// smaller subtriangles T0, T1, T2, T3 is:
///
///
/// ```text
///            P0
///           /  \
///          / T0 \
///        M01----M20
///        /  \T3/  \
///       / T1 \/ T2 \
///      P1----M12----P2
/// ```
///
/// The hierarchy path uses 0 for 'into T0', 1 for 'into T1', etc
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SubdivisionPath {
    /// Depth of subdivision - the path must not have any bits above 2*subdivision set
    subdivision: u8,
    /// The path to reach this level of subdivision
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
    pub fn subdivision_is_legal_for_patch_size(subdivision: u8, patch_size: u32) -> bool {
        if subdivision >= 32 {
            false
        } else if (1_u32 << (subdivision as u32)) > patch_size {
            false
        } else {
            true
        }
    }
}
