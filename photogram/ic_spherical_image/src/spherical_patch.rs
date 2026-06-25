use crate::{
    GreatCircleTriangleIndex, ImageFileIndex, ImagePatch, SphericalData, SphericalImageError,
    SubdivisionPath,
};
use ic_base::{JsonParsable, Point2D, Point3D};

use indexed::Idx;
use serde::{Deserialize, Serialize};

/// A descriptor of a patch (pair of triangles) on a spherical image
///
/// The two triangles are bottom-left and upper-right; they share a diagonal
///
/// When stored in a file the img_xy could be derived?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphericalPatchDescriptor {
    /// File number within the SphericalImage files that contains the data
    #[serde(default)]
    pub file_number: u32,
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

        let _t0_subdivision_path = SubdivisionPath::of_subdivision_and_mask(
            self.subdivision_to_patch,
            self.t0_subdivision_hierarchy,
        )?;
        let _t1_subdivision_path = SubdivisionPath::of_subdivision_and_mask(
            self.subdivision_to_patch,
            self.t1_subdivision_hierarchy,
        )?;

        Ok(self)
    }
}

#[derive(Debug)]
pub struct SphericalPatch {
    /// File number within the SphericalImage files that contains the data
    pub(crate) file_index: ImageFileIndex,
    /// Size of the patch in the image; if zero, then there is no data for this patch
    pub(crate) patch_size: u32,
    /// Offset in patch from bottom left to bottom left of the patch in the image
    pub(crate) img_xy: (u32, u32),
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
    pub(crate) patch_subdivision: u8,
    pub image_patch: ImagePatch,
}

impl SphericalPatch {
    pub fn new(
        sd: &mut SphericalData,
        toplevel_t0: GreatCircleTriangleIndex,
        toplevel_t1: GreatCircleTriangleIndex,
        t0_subdivision_path: SubdivisionPath,
        t1_subdivision_path: SubdivisionPath,
    ) -> Result<Self, SphericalImageError> {
        let t0 = sd.find_or_subdivide_to_gc_triangle(toplevel_t0, t0_subdivision_path)?;
        let t1 = sd.find_or_subdivide_to_gc_triangle(toplevel_t1, t1_subdivision_path)?;

        let Some(image_patch) = ImagePatch::of_gc_triangles(sd, &sd[t0], &sd[t1]) else {
            return Err(SphericalImageError::TrianglesDoNotShareAnEdge(t0, t1));
        };

        let file_index = ImageFileIndex::default();
        let patch_size = 0;
        let img_xy = (0, 0);
        let patch_subdivision = 0;
        Ok(Self {
            file_index,
            patch_size,
            img_xy,
            patch_subdivision,
            toplevel_t0,
            toplevel_t1,
            t0,
            t1,
            image_patch,
        })
    }

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
        let mut s = Self::new(
            sd,
            toplevel_t0,
            toplevel_t1,
            t0_subdivision_path,
            t1_subdivision_path,
        )?;
        s.file_index = ImageFileIndex::from_usize(patch_desc.file_number as usize);
        s.patch_size = patch_desc.patch_size;
        s.img_xy = patch_desc.img_xy;
        s.patch_subdivision = patch_desc.patch_subdivision;
        Ok(s)
    }

    pub fn to_desc(&self, sd: &SphericalData) -> SphericalPatchDescriptor {
        let subdivision = sd[self.t0].subdivision_path().subdivision();
        let t0_subdivision_hierarchy = sd[self.t0].subdivision_path().path();
        let t1_subdivision_hierarchy = sd[self.t1].subdivision_path().path();
        SphericalPatchDescriptor {
            file_number: self.file_index.opt_index().unwrap() as u32,
            patch_size: self.patch_size,
            img_xy: self.img_xy,
            toplevel_t0: self.toplevel_t0.opt_index().unwrap() as u16,
            toplevel_t1: self.toplevel_t1.opt_index().unwrap() as u16,
            subdivision_to_patch: subdivision,
            t0_subdivision_hierarchy,
            t1_subdivision_hierarchy,
            patch_subdivision: self.patch_subdivision,
        }
    }

    pub fn set_image_data(&mut self, img_xy: (u32, u32), patch_size: u32, patch_subdivision: u8) {
        self.img_xy = img_xy;
        self.patch_size = patch_size;
        self.patch_subdivision = patch_subdivision;
        self.image_patch.set_img_xy(img_xy);
        self.image_patch.set_img_sz(patch_size);
    }

    pub fn contains_direction(&self, sd: &SphericalData, p: &Point3D) -> bool {
        sd[self.t0].point_outside_lines(sd, p) == 0 || sd[self.t1].point_outside_lines(sd, p) == 0
    }

    pub fn image_coords(&self, sd: &SphericalData, p: &Point3D) -> Option<Point2D> {
        self.image_patch.image_coords(p)
    }
}
