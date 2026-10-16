use std::io::SeekFrom;

use ic_base::Point3D;
use ic_image::ImageDraw;

use crate::{CameraImageProjection, CameraLensProjection};

/// A type that combines a camera (that provides a means to convert from world
/// direction to camera sensor position appropriately) with an image that
/// provides the contents of the camera's sensor
///
/// The image provides a portion of the sensor from (lx,ty) for 'w' width, and 'h' height pixels.
#[derive(Debug, Clone)]
pub struct ProjectedCameraImage<'a, C, I>
where
    C: CameraLensProjection,
    I: ImageDraw,
{
    pub image: &'a I,
    pub camera: &'a C,
    pub lx: f64,
    pub ty: f64,
    pub w: u32,
    pub h: u32,
}
impl<'a, C, I> ProjectedCameraImage<'a, C, I>
where
    C: CameraLensProjection,
    I: ImageDraw,
{
    pub fn mapping_camera_to_image(image: &'a I, camera: &'a C) -> Self {
        let wh = image.dimensions();
        let w = wh.0;
        let h = wh.1;
        Self {
            image,
            camera,
            w,
            h,
            lx: 0.0,
            ty: 0.0,
        }
    }

    pub fn with_position(mut self, lx: f64, ty: f64) -> Self {
        self.lx = lx;
        self.ty = ty;
        self
    }
}

impl<'a, C, I> CameraImageProjection for ProjectedCameraImage<'a, C, I>
where
    C: CameraLensProjection,
    I: ImageDraw,
{
    type Pixel = I::Pixel;
    fn set_mapping_to_camera_dir(&mut self, _dirn: Point3D) {}
    fn opt_pixel_of_camera_dir(&mut self, dirn: Point3D) -> Option<I::Pixel> {
        let Some(pxy) = self.camera.camera_dir_to_opt_sensor_px_abs_xy(dirn) else {
            return None;
        };
        if pxy[0] < self.lx || pxy[1] < self.ty {
            return None;
        }
        if (pxy[0] >= self.lx + (self.w as f64)) || (pxy[1] >= self.ty + (self.h as f64)) {
            return None;
        }
        Some(
            self.image
                .get_pixel((pxy[0] - self.lx) as u32, (pxy[1] - self.ty) as u32),
        )
    }
    fn world_dir_to_camera_dir(&self, world_dir: Point3D) -> Point3D {
        self.camera.world_dir_to_camera_dir(world_dir)
    }
}
