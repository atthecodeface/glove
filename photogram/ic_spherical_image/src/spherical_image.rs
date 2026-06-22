use crate::{GreatCircleTriangleIndex, SdIndex, SphericalData, SphericalImageError};
use ic_base::JsonParsable;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{SphericalImageShape, SphericalPatch, SphericalPatchDescriptor};

/// A descriptor of a spherical image that accompanies the actual bitmap that
/// contains the pixels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphericalImageDescriptor {
    /// The image size
    img_wh: (u32, u32),
    /// Toplevel shape
    shape: SphericalImageShape,
    /// Patch hierarchy description
    patches: Vec<SphericalPatchDescriptor>,
}

impl JsonParsable for SphericalImageDescriptor {
    type PostParseArg = ();
    type PostParseResult = SphericalImage;
    fn reason() -> &'static str {
        "SphericalImagehDescriptor"
    }
    fn post_parse(self, _args: &()) -> ic_base::Result<SphericalImage> {
        Ok(SphericalImage::of_desc(&self)?)
    }
}

impl SphericalImageDescriptor {
    /// Create a [SphericalImageDescriptor] for the toplevel triangles of the given shape
    pub fn of_shape_toplevel(
        shape: SphericalImageShape,
        img_wh: (u32, u32),
        patch_size: u32,
    ) -> Result<Self, SphericalImageError> {
        let mut patches = vec![];
        let sd = shape.to_spherical_data()?;
        let mut img_x = 0;
        let mut img_y = 0;
        for (i, (_t0, _t1)) in sd
            .iter_triangle_indicess()
            .zip(sd.iter_triangle_indicess().skip(1))
            .enumerate()
            .filter(|(i, _)| (i & 1) == 0)
        {
            let spd = SphericalPatchDescriptor {
                patch_size,
                img_xy: (img_x, img_y),
                toplevel_t0: i as u16,
                toplevel_t1: (i + 1) as u16,
                subdivision_to_patch: 0,
                t0_subdivision_hierarchy: 0,
                t1_subdivision_hierarchy: 0,
                patch_subdivision: 0,
            };
            img_x += patch_size;
            if img_x >= img_wh.0 {
                img_x = 0;
                img_y += patch_size;
            }
            patches.push(spd);
        }
        Ok(Self {
            shape,
            patches,
            img_wh,
        })
    }
}

/// A descriptor of a spherical image that accompanies the actual bitmap that
/// contains the pixels
#[derive(Debug)]
pub struct SphericalImage {
    /// The image size
    img_wh: (u32, u32),
    /// Toplevel shape
    shape: SphericalImageShape,
    /// The triangles, normals, etc in the sphere surface
    sd: SphericalData,
    /// GreatCircleTriangleIndex for sd for triangles in the image that are
    /// toplevel or toplevel subdivide by one
    sd_index: SdIndex,
    /// Patches that make up the surface
    patches: Vec<SphericalPatch>,
    patch_map: HashMap<GreatCircleTriangleIndex, usize>,
}

impl SphericalImage {
    fn of_desc(desc: &SphericalImageDescriptor) -> Result<Self, SphericalImageError> {
        let mut sd = desc.shape.to_spherical_data()?;
        let mut patches = vec![];
        for p in &desc.patches {
            patches.push(SphericalPatch::of_desc(&mut sd, p)?);
        }
        let mut patch_map = HashMap::new();
        for (i, p) in patches.iter().enumerate() {
            patch_map.insert(p.t0, i);
            patch_map.insert(p.t1, i);
        }
        let sd_index = SdIndex::new(
            &sd,
            sd.iter_triangle_indicess()
                .filter(|t| sd[*t].subdivision_path().subdivision() <= 1),
        );

        Ok(Self {
            img_wh: desc.img_wh,
            shape: desc.shape.clone(),
            sd,
            sd_index,
            patches,
            patch_map,
        })
    }
    /*
    //    pub fn of_desc(desc: SphericalImageDescriptor) -> Self {}
    pub fn of_shape(pts: &[[f64; 3]], triangle_indices: &[(usize, usize, usize)]) -> Self {
        let mut patches = vec![];
        for ti in triangle_indices.chunks(2) {
            assert_eq!(ti.len(), 2, "An even number of faces is required");
            let gc0 = GCTriangle::of_points(
                &pts[ti[0].0].into(),
                &pts[ti[0].1].into(),
                &pts[ti[0].2].into(),
            );
            let gc1 = GCTriangle::of_points(
                &pts[ti[1].0].into(),
                &pts[ti[1].1].into(),
                &pts[ti[1].2].into(),
            );
            patches.push(ImagePatch::of_gc_triangles(&gc0, &gc1));
        }
        Self {
            img_wh: (0, 0),
            patches,
        }
    }
    */
}
