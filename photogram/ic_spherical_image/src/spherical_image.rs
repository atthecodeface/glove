use crate::{
    GreatCircleTriangleIndex, ImageFile, ImageFileDesc, SdIndex, SphericalData,
    SphericalImageError, SubdivisionPath,
};
use ic_base::{JsonParsable, PathSet, Point3D, Result};
use ic_image::{Image, ImageGray16, ImageRgb8};
use indexed::{Idx, IndexedVec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::{SphericalImageShape, SphericalPatch, SphericalPatchDescriptor};

indexed::make_index!(
    /// An index into the patches for a spherical file
    PatchIndex, usize, true);

indexed::make_index!(
    /// An index into the image files that contain the data for a spherical image
    ImageFileIndex, usize, true);

/// A descriptor of a spherical image that accompanies the actual bitmap that
/// contains the pixels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SphericalImageDescriptor {
    /// The toplevel shape of the spherical image - Tetrahedron, Octahedron, Icosahedron
    shape: SphericalImageShape,
    /// Image files that contain data for the spherical image
    #[serde(default)]
    files: Vec<ImageFileDesc>,
    /// Patch hierarchy description
    ///
    /// The patches *must* be in order of lowest resolution to highest
    /// resolution, if they overlap at all
    ///
    /// A patch contains a square of image data for an adjacent pair of
    /// triangles that form part of the spherical image; it is within one of the
    /// image files at a specific offset, of a specified size
    patches: Vec<SphericalPatchDescriptor>,
}

impl JsonParsable for SphericalImageDescriptor {
    type PostParseArg = ();
    type PostParseResult = Self;
    fn reason() -> &'static str {
        "SphericalImageDescriptor"
    }
    fn post_parse(self, _args: &()) -> ic_base::Result<Self> {
        Ok(self)
    }
}

impl SphericalImageDescriptor {
    /// Create a [SphericalImageDescriptor] for the toplevel triangles of the given shape
    pub fn of_shape_toplevel(
        shape: SphericalImageShape,
        img_wh: (u32, u32),
        patch_size: u32,
    ) -> Result<Self> {
        let mut patches = vec![];
        let sd = shape.to_spherical_data()?;
        let mut img_x = 0;
        let mut img_y = 0;
        for (i, (_t0, _t1)) in sd
            .iter_triangle_indices()
            .zip(sd.iter_triangle_indices().skip(1))
            .enumerate()
            .filter(|(i, _)| (i & 1) == 0)
        {
            let spd = SphericalPatchDescriptor {
                file_number: 0,
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
            files: vec![],
            shape,
            patches,
        })
    }
}

/// The data required for a spherical image that accompanies the actual bitmaps that
/// contains the pixel data
///
///
#[derive(Debug)]
pub struct SphericalImage<I: Image> {
    /// The toplevel shape of the spherical image - Tetrahedron, Octahedron, Icosahedron
    shape: SphericalImageShape,
    /// Path set used for files
    path_set: PathSet,
    /// Images that contain the pixel data for the image, at various resolutions potentially
    image_files: IndexedVec<ImageFileIndex, ImageFile<I>, true>,
    /// The GC triangles, normals, etc in the sphere surface
    sd: SphericalData,
    /// An index that allows a vector to be mapped to a
    /// GreatCircleTriangleIndex; this is derived from the SphericalData element
    /// to provide for fast determination as to which GC triangle a specific
    /// vector lies within
    ///
    /// The vector will lie on either a positive or negative side of every great
    /// circle normal at a patch level of 0/1; a bitmask is generated for these,
    /// i.e. vector maps to a bitmask of up to 64 bits; this is a sparse result
    /// (i.e. not all 2^64 values are possible). The index maps the bitmask
    /// value to a specific GreateCircleTriangleIndex, which is the GC Triangle
    /// that the vector lies within.
    sd_index: SdIndex,
    /// Patches that make up the surface
    ///
    /// The patches *must* be in order of lowest resolution to highest
    /// resolution, if they overlap at all
    patches: IndexedVec<PatchIndex, SphericalPatch, true>,
    /// Map from great circle index to the element of patches
    ///
    /// Should use PatchIndex
    patch_map: HashMap<GreatCircleTriangleIndex, usize>,
}

impl<I: Image> std::ops::Index<PatchIndex> for SphericalImage<I> {
    type Output = SphericalPatch;
    fn index(&self, index: PatchIndex) -> &Self::Output {
        &self.patches[index]
    }
}

impl<I: Image> std::ops::Index<ImageFileIndex> for SphericalImage<I> {
    type Output = ImageFile<I>;
    fn index(&self, index: ImageFileIndex) -> &Self::Output {
        &self.image_files[index]
    }
}

impl<I: Image> SphericalImage<I> {
    /// Create the SdIndex, which maps vectors to GC Triangles, and the patch
    /// map, which maps GC triangle index to highest resolution patch
    fn create_indices(&mut self) {
        self.patch_map.clear();
        for (i, p) in self.patches.iter().enumerate() {
            self.patch_map.insert(p.t0, i);
            self.patch_map.insert(p.t1, i);
        }
        self.sd_index = SdIndex::new(
            &self.sd,
            self.sd
                .iter_triangle_indices()
                .filter(|t| self.sd[*t].subdivision_path().subdivision() <= 1),
        );
    }

    /// Create a new spherical image with empty data, given a toplevel shape
    ///
    /// The path set will be empty
    pub fn of_shape(shape: SphericalImageShape) -> Self {
        let path_set = PathSet::default();
        let image_files = IndexedVec::default();
        let sd = shape.to_spherical_data().unwrap();
        let patch_map = HashMap::new();
        let sd_index = SdIndex::default();
        let patches = IndexedVec::default();
        let mut s = Self {
            path_set,
            shape: shape.clone(),
            image_files,
            sd,
            sd_index,
            patches,
            patch_map,
        };
        s.create_indices();
        s
    }

    /// Create a spherical image from a descriptor, which has presumably been read from a file
    ///
    /// This includes reading the image pixel data; it will fail if the images
    /// cannot be found
    ///
    /// To enable files to be found, the PathSet must be supplied; this is then
    /// kept in the spherical image itself
    pub fn of_desc(path_set: &PathSet, desc: &SphericalImageDescriptor) -> ic_base::Result<Self> {
        let mut image_files = IndexedVec::default();
        for id in desc.files.iter() {
            image_files.push(ImageFile::of_desc(path_set, id)?);
        }
        let mut sd = desc.shape.to_spherical_data()?;
        let mut patches = IndexedVec::default();
        for p in &desc.patches {
            patches.push(SphericalPatch::of_desc(&mut sd, p)?);
        }
        let path_set = path_set.clone();

        let patch_map = HashMap::new();
        let sd_index = SdIndex::default();
        let mut s = Self {
            path_set,
            shape: desc.shape.clone(),
            image_files,
            sd,
            sd_index,
            patches,
            patch_map,
        };
        s.create_indices();
        Ok(s)
    }

    /// Create a [SphericalImageDescriptor] for this spherical image, usually
    /// for saving in a file
    pub fn to_desc(&self) -> SphericalImageDescriptor {
        let files: Vec<_> = self.image_files.iter().map(|f| f.to_desc()).collect();
        let shape = self.shape;
        let patches: Vec<_> = self.patches.iter().map(|p| p.to_desc(&self.sd)).collect();
        SphericalImageDescriptor {
            files,
            shape,
            patches,
        }
    }

    /// Get a reference to the [PathSet] for the image
    pub fn path_set(&self) -> &PathSet {
        &self.path_set
    }

    /// Set the PathSet
    pub fn set_path_set(&mut self, path_set: PathSet) {
        self.path_set = path_set;
    }

    /// Add a blank new image of certain dimensions to the image data for the spherical image
    pub fn add_new_image(&mut self, width: u32, height: u32) -> ImageFileIndex {
        self.image_files.push(ImageFile::new(width, height))
    }

    /// Add an image file (a JPEG usually) to the spherical image, given its path
    pub fn add_image_file<A: AsRef<Path>>(
        &mut self,
        filename: A,
        img_wh: Option<(u32, u32)>,
    ) -> ic_base::Result<ImageFileIndex> {
        Ok(self
            .image_files
            .push(ImageFile::of_file(&self.path_set, filename, img_wh)?))
    }

    /// Set the path of a specific image file within the spherical image
    pub fn set_image_path<P: AsRef<Path>>(
        &mut self,
        image_file: ImageFileIndex,
        path: P,
    ) -> Result<()> {
        if let Some(f) = self.image_files.get_mut(image_file) {
            f.set_path(path);
            Ok(())
        } else {
            Err(format!("Image file index {image_file:?} out of range").into())
        }
    }

    /// Write a specific image to its given file path
    pub fn write_image(&self, image_file: ImageFileIndex) -> ic_base::Result<()> {
        self.image_files[image_file]
            .image()
            .write(self.image_files[image_file].path())
    }

    /// Add patches for the toplevel triangles of the shape, with a given size
    /// and subdivision patch size
    ///
    /// This must only be invoked if no patches exist on the image
    ///
    /// The image must be able to support (|shape triangles|/2) squares of patch_sz per side
    pub fn add_toplevel_patches(
        &mut self,
        image_file: ImageFileIndex,
        patch_size: u32,
        patch_subdivision: u8,
    ) -> ic_base::Result<()> {
        if !self.patches.is_empty() {
            return Err(format!("adding toplevel patches to a spherical image attempted when the image already had {} patches", self.patches.len()).into());
        }
        if !SubdivisionPath::subdivision_is_legal_for_patch_size(patch_subdivision, patch_size) {
            return Err(format!(
                "subdivision of {patch_subdivision} too larger for patch size {patch_size}",
            )
            .into());
        }
        if self.image_files.get(image_file).is_none() {
            return Err(format!("must have a toplevel image of the appropriate size, but {image_file:?} is out of range for the currently added images").into());
        }
        let toplevel_triangles: Vec<_> = self
            .sd
            .iter_triangle_indices()
            .filter(|t| self.sd[*t].subdivision_path().subdivision_matches(0))
            .collect();
        let num_patches = (toplevel_triangles.len() as u32) / 2;
        let image = self.image_files[image_file].image();
        let (width, height) = image.size();
        let width_patches = width.div_ceil(patch_size);
        let height_patches = height.div_ceil(patch_size);
        let max_patches = width_patches * height_patches;
        if max_patches < num_patches {
            return Err(format!("adding {num_patches} toplevel patches of size {patch_size} attempted when the image can only support {max_patches}").into());
        }

        let subdivision_path = SubdivisionPath::default();
        let mut img_x = 0;
        let mut img_y = 0;
        for (_, (toplevel_t0, toplevel_t1)) in toplevel_triangles
            .iter()
            .zip(toplevel_triangles.iter().skip(1))
            .enumerate()
            .filter(|(i, _ts)| i & 1 == 0)
        {
            let mut patch = SphericalPatch::new(
                &mut self.sd,
                *toplevel_t0,
                *toplevel_t1,
                subdivision_path,
                subdivision_path,
            )?;
            patch.file_index = image_file;
            patch.set_image_data((img_x, img_y), patch_size, patch_subdivision);
            img_x += patch_size;
            if img_x + patch_size > width {
                img_x = 0;
                img_y += patch_size;
            }
            self.patches.push(patch);
        }
        self.create_indices();
        Ok(())
    }

    /// Iterate through the patch indices
    pub fn iter_patch_indices(&self) -> impl Clone + ExactSizeIterator<Item = PatchIndex> {
        self.patches.indices()
    }

    /// Fill a specific patch of the spherical image (which is a square of one
    /// of the image files), using the given function that maps a vector to an
    /// optional pixel value
    pub fn fill_image_patch<F: FnMut(Point3D) -> Option<I::Pixel>>(
        &mut self,
        blend: f64,
        patch: PatchIndex,
        get_pixel: F,
    ) {
        // Assume patch_subdivision is 0 for now
        let patch = &self.patches[patch];
        let image_file = patch.file_index;
        let (x, y) = patch.img_xy;
        let size = patch.patch_size;
        let from_patch = patch.image_patch.clone();
        let from_patch =
            from_patch.map_subsquare::<'_, I, _>(patch.patch_subdivision, 0, 0, get_pixel);
        // Drop the patch
        let _ = patch;
        let mut image_patch = ic_image::ImagePatch::new(
            self.image_files[image_file].image_mut(),
            x,
            y,
            size,
            size,
            blend,
            from_patch,
        );
        image_patch.fill_img();
    }

    /// Get the pixel value at a given vector
    pub fn get_pixel_of_direction(&self, p: &Point3D) -> Option<I::Pixel> {
        for patch in self.patches.iter() {
            if patch.contains_direction(&self.sd, p) {
                if let Some(p) = patch.image_coords(&self.sd, p) {
                    let image = self.image_files[patch.file_index].image();
                    let (w, h) = image.size();
                    let x = (p[0].max(0.0).min((w - 1) as f64)) as u32;
                    let y = (p[1].max(0.0).min((h - 1) as f64)) as u32;
                    return Some(image.get(x, y));
                }
            }
        }
        None
    }
}

pub enum SphericalImageKind {
    Rgb(SphericalImage<ImageRgb8>),
    Gray16(SphericalImage<ImageGray16>),
}
