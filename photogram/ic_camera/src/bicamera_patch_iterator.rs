use ic_image::FromPatchFn;

use crate::{CameraImageProjection, CameraLensProjection};

/// A type the can be used as a [FromPatchFn] to allow filling an image in the
/// sensor which is in 'camera', using a 'projected_image' that is a camera (and
/// an image of its sensor) which can map world rays (for the 'camera' image)
/// onto the projected image sensor contents, should that ray coincide with the
/// projected image camera's field of view.
///
/// The type takes a *mutable* projected_image, so that complex 'world direction
/// to pixel content' operations can be performed by the projected image
/// utilizing 'this direction is likely to be almost the same as the last
/// direction that was asekd for' logic; this is especiallly useful for
/// spherical image, or other hierarchical image storage types.
pub struct BicameraPatchIterator<'a, C1, C2>
where
    C1: CameraLensProjection,
    C2: CameraImageProjection,
{
    camera: &'a C1,
    projected_image: &'a mut C2,
}

impl<'a, C1, C2> BicameraPatchIterator<'a, C1, C2>
where
    C1: CameraLensProjection,
    C2: CameraImageProjection,
{
    pub fn new(camera: &'a C1, projected_image: &'a mut C2) -> Self {
        Self {
            camera,
            projected_image,
        }
    }
}

impl<'a, C1, C2> FromPatchFn for BicameraPatchIterator<'a, C1, C2>
where
    C1: CameraLensProjection,
    C2: CameraImageProjection,
{
    type Pixel = C2::Pixel;
    fn set_mapping(&mut self, _patch_x: u32, _patch_y: u32) {}
    fn map_from_patch(&mut self, patch_x: u32, patch_y: u32) -> Option<Self::Pixel> {
        let world_dir = self
            .camera
            .sensor_px_abs_xy_to_world_dir([patch_x as f64, patch_y as f64].into());
        let camera_dir = self.projected_image.world_dir_to_camera_dir(world_dir);
        self.projected_image.opt_pixel_of_camera_dir(camera_dir)
    }
}
