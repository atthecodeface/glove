use ic_base::{Point2D, Point3D, Quat, TanXTanY};

use crate::{
    AdjustableCameraProjection, BaseCamera, CameraLensProjection, CameraProjection, CameraSensor,
    LensProjection, SizedSensor,
};

/// This type provides a simple rectangular sensor with an arbitrary lens,
/// giving a camera with a position and orientation.
///
/// Its purpose is to allow images to be used with a lens mapping; for example,
/// this can be used to create a panoramic camera mapping onto a backing image
/// using a cylindrical lens projection. It is not aimed for modeling a
/// real-world camera.
#[derive(Debug, Clone, Copy, Default)]
pub struct SimpleSensorCamera<L: LensProjection + std::default::Default + Clone + std::fmt::Debug> {
    pub sensor: SizedSensor,
    pub lens: L,
    pub camera: BaseCamera,
    pub mm_focal_length: f64,
    pub mm_focus_distance: f64,
}

impl<L> SimpleSensorCamera<L>
where
    L: LensProjection + std::default::Default + Clone + std::fmt::Debug,
{
    /// Create a new 'camera' at the origin, looking along -Z with X to the right, focussed at (near) infinity with given sensor and lens details
    pub fn new(width: u32, height: u32, mm_per_pixel: f64, lens: L, mm_focal_length: f64) -> Self {
        let sensor = SizedSensor {
            width,
            height,
            mm_height: (height as f64 * mm_per_pixel),
            mm_width: (width as f64 * mm_per_pixel),
        };
        let camera = BaseCamera::default();
        Self {
            sensor,
            lens,
            camera,
            mm_focal_length,
            mm_focus_distance: 1.0E7,
        }
    }
}

impl<L> AdjustableCameraProjection for SimpleSensorCamera<L>
where
    L: LensProjection + std::default::Default + Clone + std::fmt::Debug,
{
    fn set_position(&mut self, position: &Point3D) {
        self.camera.set_position(position);
    }
    fn set_orientation(&mut self, orientation: &Quat) {
        self.camera.set_orientation(orientation);
    }
    fn set_optical_axis_offset(&mut self, _p: &Point2D) {}
    fn set_focus_distance(&mut self, mm_focus_distance: f64) {
        self.mm_focus_distance = mm_focus_distance;
    }
}

impl<L> CameraSensor for SimpleSensorCamera<L>
where
    L: LensProjection + std::default::Default + Clone + std::fmt::Debug,
{
    fn sensor_px_size(&self) -> (f64, f64) {
        self.sensor.sensor_px_size()
    }

    fn sensor_px_abs_to_optical_xy(&self, px_xy: Point2D) -> Point2D {
        self.sensor.sensor_px_abs_to_optical_xy(px_xy)
    }

    fn optical_xy_to_px_abs(&self, px_xy: Point2D) -> Point2D {
        self.sensor.optical_xy_to_px_abs(px_xy)
    }

    fn sensor_mm_single_pixel_height(&self) -> f64 {
        self.sensor.sensor_mm_single_pixel_height()
    }

    fn sensor_mm_single_pixel_width(&self) -> f64 {
        self.sensor.sensor_mm_single_pixel_width()
    }
}
impl<L> LensProjection for SimpleSensorCamera<L>
where
    L: LensProjection + std::default::Default + Clone + std::fmt::Debug,
{
    fn optical_txty_to_camera_txty(&self, optical_txty: TanXTanY) -> TanXTanY {
        self.lens.optical_txty_to_camera_txty(optical_txty)
    }
    fn camera_txty_to_optical_txty(&self, camera_txty: TanXTanY) -> TanXTanY {
        self.lens.camera_txty_to_optical_txty(camera_txty)
    }
}
impl<L> CameraProjection for SimpleSensorCamera<L>
where
    L: LensProjection + std::default::Default + Clone + std::fmt::Debug,
{
    fn position(&self) -> Point3D {
        self.camera.position()
    }
    fn orientation(&self) -> Quat {
        self.camera.orientation()
    }
}
impl<L> CameraLensProjection for SimpleSensorCamera<L>
where
    L: LensProjection + std::default::Default + Clone + std::fmt::Debug,
{
    fn optical_axis_offset(&self) -> Point2D {
        Point2D::default()
    }
    fn lens_sensor_distance(&self) -> f64 {
        1.0 / (1.0 / self.mm_focal_length - 1.0 / self.mm_focus_distance)
    }
}
