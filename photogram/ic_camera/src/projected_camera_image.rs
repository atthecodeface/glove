use std::io::SeekFrom;

use geo_nd::{SqMatrix, glsl::Mat3};
use ic_base::{Mat3x3, Point2D, Point3D};
use ic_image::ImageDraw;

use crate::{CameraImageProjection, CameraLensProjection};

/// A type that combines a camera (that provides a means to convert from world
/// direction to camera sensor position appropriately) with a (rectangular
/// portion of an) image that provides the contents of a rectangular portion of
/// the camera's sensor
///
/// The image provides a portion of the sensor from (sensor_tlx) for 'w' width,
/// and 'h' height pixels, with a top-left of (x_ofs, y_ofs).
///
/// [ImageDraw] is used as it requires, and is implemented for, `image::GenericImage`.
///
/// This type supports [CameraImageProjection], which supplies the
/// `opt_pixel_of_camera_dir` and `world_dir_to_camera_dir` methods. These can
/// then be used to say 'get the pixel corresponding to this world direction, if
/// it is in the portion of the sensor'
#[derive(Debug, Clone)]
pub struct ProjectedCameraImage<'a, C, I>
where
    C: CameraLensProjection,
    I: ImageDraw,
{
    /// The underlying image which provides pixels
    pub image: &'a I,
    /// The camera which provides world-direction-to-sensor-absolute mapping
    pub camera: &'a C,
    /// The (row-major) transformation matrix applied to (sensor x, sensor y, 1) to yield an
    /// image-region-relative coordinate providing the pixel value for the sensor
    sensor_to_image_transform: Mat3x3,
    /// The width of the image region in pixels; this will be positive or zero
    w: f64,
    /// The height of the image region in pixels; this will be positive or zero
    h: f64,
    /// The X-offset into the image that of the left-most pixels of the region used for the sensor rectangle
    x_ofs: u32,
    /// The Y-offset into the image that of the top-most pixels of the region used for the sensor rectangle
    y_ofs: u32,
}
impl<'a, C, I> ProjectedCameraImage<'a, C, I>
where
    C: CameraLensProjection,
    I: ImageDraw,
{
    /// Create a [ProjectedCameraImage] with the *whole* of the provided image,
    /// starting at the top-left of the sensor of the provided camera.
    ///
    /// The width and height of the image provide the width and height of the sensor rectangle that the image data provides
    pub fn mapping_camera_to_image(image: &'a I, camera: &'a C) -> Self {
        let wh = image.dimensions();
        let w = wh.0 as f64;
        let h = wh.1 as f64;
        Self {
            image,
            camera,
            w,
            h,
            sensor_to_image_transform: Mat3x3::identity(),
            x_ofs: 0,
            y_ofs: 0,
        }
    }

    /// Set the transformation applied to sensor (px, py, 1) to yield image *region* (px,py,-)
    ///
    /// The image px,py from 0,0 to w,h are supplied by the image starting at (x_ofs,y_ofs)
    pub fn with_transform<A: AsRef<[f64; 9]>>(mut self, transform: A) -> Self {
        self.sensor_to_image_transform = transform.as_ref().into();
        self
    }

    /// Apply a rotation by 'angle', not changing the translation
    pub fn with_rotation(mut self, angle: f64) -> Self {
        let ca = angle.cos();
        let sa = angle.sin();
        let m: Mat3x3 = [ca, sa, 0., -sa, ca, 0., 0., 0., 1.].into();
        self.sensor_to_image_transform = m * self.sensor_to_image_transform;
        self
    }

    /// Set the top-left pixel of the sensor that the image region contains
    ///
    /// If `with_scaling` is also used, issue this call *first*
    pub fn with_sensor_position(mut self, tl: Point2D) -> Self {
        self.sensor_to_image_transform[2] = -tl[0];
        self.sensor_to_image_transform[5] = -tl[1];
        self
    }

    /// Set the scaling factor (amount to multiply sensor pxy to reach image region pxy)
    ///
    /// If `with_sensor_position` is also used, issue this call *afterwards*
    pub fn with_scaling(mut self, scaling: f64) -> Self {
        self.sensor_to_image_transform *= scaling;
        self
    }

    /// Set the rectangular region of the image that provides the sensor data
    ///
    /// If a ray maps outside of this region then its data will not be provided
    pub fn using_image_region(mut self, lx: u32, ty: u32, w: u32, h: u32) -> Self {
        let image_wh = self.image.dimensions();
        assert!(
            lx + w <= image_wh.0,
            "Image region cannot extend beyond width of image"
        );
        assert!(
            ty + h <= image_wh.1,
            "Image region cannot extend beyond height of image"
        );
        self.x_ofs = lx;
        self.y_ofs = ty;
        self.w = w as f64;
        self.h = h as f64;
        self
    }

    /// Map a sensor pixel xy to an image pixel xy
    pub fn map_sensor_abs_pxy_to_image_region_pxy(&self, sensor_pxy: Point2D) -> Point2D {
        let sensor_pxyo: Point3D = [sensor_pxy[0], sensor_pxy[1], 1.0].into();
        let image_pxyo = self.sensor_to_image_transform.mul_vec(&sensor_pxyo);
        [image_pxyo[0], image_pxyo[1]].into()
    }

    /// Map a sensor pixel xy to an image pixel xy
    pub fn map_sensor_abs_pxy_to_opt_image_region_pxy(
        &self,
        sensor_pxy: Point2D,
    ) -> Option<(u32, u32)> {
        let image_pxy = self.map_sensor_abs_pxy_to_image_region_pxy(sensor_pxy);
        if image_pxy[0] < 0.0
            || image_pxy[1] < 0.0
            || image_pxy[0] >= self.w
            || image_pxy[1] >= self.h
        {
            None
        } else {
            Some((
                (image_pxy[0] as u32) + self.x_ofs,
                (image_pxy[1] as u32) + self.y_ofs,
            ))
        }
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
        self.map_sensor_abs_pxy_to_opt_image_region_pxy(pxy)
            .map(|(w, h)| self.image.get_pixel(w, h))
    }
    fn world_dir_to_camera_dir(&self, world_dir: Point3D) -> Point3D {
        self.camera.world_dir_to_camera_dir(world_dir)
    }
}
