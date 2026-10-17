use ic_base::{Point2D, Point3D};
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
#[derive(Clone)]
pub struct ProjectedCameraImage<'a, C, I>
where
    C: CameraLensProjection,
    I: ImageDraw,
{
    /// The underlying image which provides pixels
    pub image: &'a I,
    /// The camera which provides world-direction-to-sensor-absolute mapping
    pub camera: &'a C,
    /// The transformation is image = translate(scale(rotate(sensor)))
    sensor_to_image_scale: f64,
    sensor_to_image_angle: f64,
    sensor_to_image_translation: Point2D,
    /// The width of the image region in pixels; this will be positive or zero
    w: f64,
    /// The height of the image region in pixels; this will be positive or zero
    h: f64,
    /// The X-offset into the image that of the left-most pixels of the region used for the sensor rectangle
    x_ofs: u32,
    /// The Y-offset into the image that of the top-most pixels of the region used for the sensor rectangle
    y_ofs: u32,
}

impl<'a, C, I> std::fmt::Debug for ProjectedCameraImage<'a, C, I>
where
    C: CameraLensProjection,
    I: ImageDraw,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ProjectedCameraImage {{ {}x{}@({},{}) : {}+{} rot {}deg : camera:{:?} : image dims {}x{} }}",
            self.w,
            self.h,
            self.x_ofs,
            self.y_ofs,
            self.sensor_to_image_scale,
            self.sensor_to_image_translation,
            self.sensor_to_image_angle.to_degrees(),
            self.camera,
            self.image.dimensions().0,
            self.image.dimensions().1
        )
    }
}
impl<'a, C, I> ProjectedCameraImage<'a, C, I>
where
    C: CameraLensProjection,
    I: ImageDraw,
{
    pub fn width(&self) -> u32 {
        self.w as u32
    }
    pub fn height(&self) -> u32 {
        self.h as u32
    }

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
            sensor_to_image_scale: 1.0,
            sensor_to_image_angle: 0.0,
            sensor_to_image_translation: Point2D::default(),
            x_ofs: 0,
            y_ofs: 0,
        }
    }

    /// Set the region of the sensor using the centre spot on the sensor and the
    /// width in pixels of the sensor that the image region corresponds to
    ///
    /// (sensor_cxy[0] - sensor_px_width/2) will map to image region x of 0.0
    /// (sensor_cxy[0] + sensor_px_width/2) will map to image region x of 1.0
    ///
    /// scale = w / sensor_px_width
    ///
    /// image = translate(scale(sensor))
    /// Hence 0.0 = translate(w / sensor_px_width * sensor_cxy[0] - w/2)
    ///  0.0 = w / sensor_px_width * sensor_cxy[0] - w/2) + sensor_to_image_translation
    ///  sensor_to_image_translation = w/2 - w / sensor_px_width * sensor_cxy[0]
    pub fn of_sensor_region(mut self, sensor_cxy: Point2D, sensor_px_width: f64) -> Self {
        self.sensor_to_image_scale = self.w / sensor_px_width;
        self.sensor_to_image_translation = [
            self.w / 2.0 - self.sensor_to_image_scale * sensor_cxy[0],
            self.h / 2.0 - self.sensor_to_image_scale * sensor_cxy[1],
        ]
        .into();
        eprintln!("{self:?}");
        self
    }

    /// Apply a rotation by 'angle', not changing the translation
    pub fn with_rotation(mut self, angle: f64) -> Self {
        self.sensor_to_image_angle = angle;
        self
    }

    /// Set the top-left pixel of the sensor that the image region contains
    pub fn with_sensor_translation(mut self, tl: Point2D) -> Self {
        self.sensor_to_image_translation = tl;
        self
    }

    /// Set the scaling factor (amount to multiply sensor pxy to reach image region pxy)
    pub fn with_scaling(mut self, scale: f64) -> Self {
        self.sensor_to_image_scale = scale;
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

    /// Map an image pixel xy to a sensor pixel xy
    pub fn map_image_region_pxy_to_sensor_abs_pxy(&self, image_pxy: Point2D) -> Point2D {
        let ca = self.sensor_to_image_angle.cos() / self.sensor_to_image_scale;
        let sa = self.sensor_to_image_angle.sin() / self.sensor_to_image_scale;
        let unrotated_pxy = image_pxy - self.sensor_to_image_translation;
        [
            unrotated_pxy[0] * ca - unrotated_pxy[1] * sa,
            unrotated_pxy[1] * ca + unrotated_pxy[0] * sa,
        ]
        .into()
    }

    /// Map a sensor pixel xy to an image pixel xy
    pub fn map_sensor_abs_pxy_to_image_region_pxy(&self, sensor_pxy: Point2D) -> Point2D {
        let ca = self.sensor_to_image_angle.cos() * self.sensor_to_image_scale;
        let sa = self.sensor_to_image_angle.sin() * self.sensor_to_image_scale;
        let rotated_pxy: Point2D = [
            sensor_pxy[0] * ca + sensor_pxy[1] * sa,
            sensor_pxy[1] * ca - sensor_pxy[0] * sa,
        ]
        .into();
        rotated_pxy + self.sensor_to_image_translation
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
