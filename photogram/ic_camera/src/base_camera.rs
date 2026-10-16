use ic_base::{Point2D, Point3D, Quat};

use crate::{AdjustableCameraProjection, CameraProjection};

#[derive(Debug, Clone, Copy, Default)]
pub struct BaseCamera {
    pub position: Point3D,
    pub orientation: Quat,
}

impl CameraProjection for BaseCamera {
    fn orientation(&self) -> Quat {
        self.orientation
    }
    fn position(&self) -> Point3D {
        self.position
    }
}

impl AdjustableCameraProjection for BaseCamera {
    fn set_position(&mut self, position: &Point3D) {
        self.position = *position;
    }
    fn set_orientation(&mut self, orientation: &Quat) {
        self.orientation = *orientation;
    }
    fn set_optical_axis_offset(&mut self, _p: &Point2D) {}
    fn set_focus_distance(&mut self, _mm_focus_distance: f64) {}
}
