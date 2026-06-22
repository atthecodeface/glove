use crate::{
    GreatCircleTriangleIndex, ImagePatch, SdIndex, SphericalData, SphericalImageError,
    SubdivisionPath,
};
use ic_base::JsonParsable;
use ic_base::{GCTriangle, Point2D, Point3D, Triangle3D};
use indexed::Idx;
use serde::{Deserialize, Serialize};

/// A descriptor of a patch (pair of triangles) on a spherical image
///
/// The two triangles are bottom-left and upper-right; they share a diagonal
///
/// When stored in a file the img_xy could be derived?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphericalPatchDescriptor {
    /// Size of the patch in the image; if zero, then there is no data for this patch
    pub patch_size: u32,
    /// Offset in patch from bottom left to bottom left of the patch in the image
    pub img_xy: (u32, u32),
    /// Toplevel triangle index (into shape) of bottom-left triangle
    pub toplevel_t0: u16,
    /// Toplevel triangle index (into shape) of upper-right triangle
    pub toplevel_t1: u16,
    /// Subdivision depth from toplevel to that of these triangles
    pub subdivision_to_patch: u8,
    /// How to reach t0 from its toplevel - using a depth of subdivision_to_patch
    pub t0_subdivision_hierarchy: u64,
    /// How to reach t1 from its toplevel - using a depth of subdivision_to_patch
    pub t1_subdivision_hierarchy: u64,
    /// Subdivision depth *within* the these triangles (if zero then each
    /// toplevel-triangle-subdivided-by-subdivision_to_patch is linearly mapped
    /// to the image pixels from the barycentric coordinates)
    pub patch_subdivision: u8,
}

impl JsonParsable for SphericalPatchDescriptor {
    type PostParseArg = SphericalData;
    type PostParseResult = SphericalPatchDescriptor;
    fn reason() -> &'static str {
        "SphericalPatchDescriptor"
    }
    fn post_parse(self, sd: &SphericalData) -> ic_base::Result<Self> {
        if self.toplevel_t0 as usize >= sd.num_triangles() {
            return Err(SphericalImageError::BadTriangleIndex(self.toplevel_t0 as usize).into());
        }
        if self.toplevel_t1 as usize >= sd.num_triangles() {
            return Err(SphericalImageError::BadTriangleIndex(self.toplevel_t1 as usize).into());
        }
        let sub = self.subdivision_to_patch;
        let sub_mask = ((!0_u64) << (2 * (sub as u64)));
        let t0_subdivision_hierarchy = self.t0_subdivision_hierarchy;
        let t1_subdivision_hierarchy = self.t1_subdivision_hierarchy;
        if t0_subdivision_hierarchy & sub_mask != 0 {
            return Err(
                SphericalImageError::BadSubdivisionPath(sub, t0_subdivision_hierarchy).into(),
            );
        }
        if t1_subdivision_hierarchy & sub_mask != 0 {
            return Err(
                SphericalImageError::BadSubdivisionPath(sub, t1_subdivision_hierarchy).into(),
            );
        }

        Ok(self)
    }
}

#[derive(Debug)]
pub struct SphericalPatch {
    /// Size of the patch in the image; if zero, then there is no data for this patch
    pub patch_size: u32,
    /// Offset in patch from bottom left to bottom left of the patch in the image
    pub img_xy: (u32, u32),
    /// Toplevel triangle index (into shape) of bottom-left triangle
    ///
    /// The subdivision of `sd[self.toplevel_t0]` will be 0
    pub toplevel_t0: GreatCircleTriangleIndex,
    /// Toplevel triangle index (into shape) of upper-right triangle
    ///
    /// The subdivision of `sd[self.toplevel_t0]` will be 0
    pub toplevel_t1: GreatCircleTriangleIndex,
    pub t0: GreatCircleTriangleIndex,
    pub t1: GreatCircleTriangleIndex,
    /// Subdivision depth *within* the these triangles (if zero then each
    /// toplevel-triangle-subdivided-by-subdivision_to_patch is linearly mapped
    /// to the image pixels from the barycentric coordinates)
    pub patch_subdivision: u8,
    pub image_patch: ImagePatch,
}

impl SphericalPatch {
    pub fn of_desc(
        sd: &mut SphericalData,
        patch_desc: &SphericalPatchDescriptor,
    ) -> Result<Self, SphericalImageError> {
        let toplevel_t0 = GreatCircleTriangleIndex::from_usize(patch_desc.toplevel_t0 as usize);
        let toplevel_t1 = GreatCircleTriangleIndex::from_usize(patch_desc.toplevel_t1 as usize);
        let t0_subdivision_path = SubdivisionPath::of_subdivision_and_mask(
            patch_desc.subdivision_to_patch,
            patch_desc.t0_subdivision_hierarchy,
        )?;
        let t1_subdivision_path = SubdivisionPath::of_subdivision_and_mask(
            patch_desc.subdivision_to_patch,
            patch_desc.t1_subdivision_hierarchy,
        )?;
        let t0 = sd.find_or_subdivide_to_gc_triangle(toplevel_t0, t0_subdivision_path)?;
        let t1 = sd.find_or_subdivide_to_gc_triangle(toplevel_t1, t1_subdivision_path)?;

        let Some(image_patch) = ImagePatch::of_gc_triangles(sd, &sd[t0], &sd[t1]) else {
            return Err(SphericalImageError::TrianglesDoNotShareAnEdge(t0, t1));
        };

        // Validate that t0 and t1 share an edge
        Ok(Self {
            patch_size: patch_desc.patch_size,
            img_xy: patch_desc.img_xy,
            toplevel_t0,
            toplevel_t1,
            t0,
            t1,
            patch_subdivision: patch_desc.patch_subdivision,
            image_patch,
        })
    }
    pub fn to_desc(&self, sd: &SphericalData) -> SphericalPatchDescriptor {
        let subdivision = sd[self.t0].subdivision_path().subdivision();
        let t0_subdivision_hierarchy = sd[self.t0].subdivision_path().path();
        let t1_subdivision_hierarchy = sd[self.t1].subdivision_path().path();
        SphericalPatchDescriptor {
            patch_size: self.patch_size,
            img_xy: self.img_xy,
            toplevel_t0: self.toplevel_t0.index() as u16,
            toplevel_t1: self.toplevel_t1.index() as u16,
            subdivision_to_patch: subdivision,
            t0_subdivision_hierarchy,
            t1_subdivision_hierarchy,
            patch_subdivision: self.patch_subdivision,
        }
    }
}
