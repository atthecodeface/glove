//a Imports
use std::rc::Rc;

use super::{
    CameraProjection, CameraView, NamedPointSet, Point2D, Point3D, PointMapping, Quat, Rotations,
    TanXTanY,
};

use geo_nd::{quat, vector};

//a Camera
//tp Camera
/// A camera that allows mapping a world point to camera relative XYZ,
/// and then it can be mapped to tan(x) / tan(y) to roll/yaw or pixel
/// relative XY (relative to the center of the camera sensor)
#[derive(Debug, Clone)]
pub struct Camera {
    /// Map from tan(x), tan(y) to Roll/Yaw or even to pixel relative
    /// XY
    projection: Rc<dyn CameraProjection>,
    /// Position in world coordinates of the camera
    ///
    /// Subtract from world coords to get camera-relative world coordinates
    position: Point3D,
    /// Direction to be applied to camera-relative world coordinates
    /// to convert to camera-space coordinates
    ///
    /// Camera-space XYZ = direction applied to (world - positionn)
    direction: Quat,
}

//ip Display for Camera
impl std::fmt::Display for Camera {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let dxyz = quat::apply3(&quat::conjugate(self.direction.as_ref()), &[0., 0., 1.]);
        // First rotation around Y axis (yaw)
        let yaw = dxyz[0].atan2(dxyz[2]).to_degrees();
        // Then rotation around X axis (elevation)
        let pitch = dxyz[1]
            .atan2((dxyz[0] * dxyz[0] + dxyz[2] * dxyz[2]).sqrt())
            .to_degrees();
        write!(
            fmt,
            "@[{:.2},{:.2},{:.2}] yaw {:.2} pitch {:.2} + [{:.2},{:.2},{:.2}]",
            self.position[0],
            self.position[1],
            self.position[2],
            yaw,
            pitch,
            dxyz[0],
            dxyz[1],
            dxyz[2]
        )
    }
}

//ip CameraView for Camera
impl CameraView for Camera {
    //fp location
    fn location(&self) -> Point3D {
        self.position
    }

    //fp direction
    fn direction(&self) -> Quat {
        self.direction
    }

    //fp px_abs_xy_to_camera_txty
    /// Map a screen Point2D coordinate to tan(x)/tan(y)
    fn px_abs_xy_to_camera_txty(&self, px_abs_xy: &Point2D) -> TanXTanY {
        let px_rel_xy = self.projection.px_abs_xy_to_px_rel_xy(*px_abs_xy);
        self.projection.px_rel_xy_to_txty(px_rel_xy)
    }

    //fp camera_txty_to_px_abs_xy
    /// Map a tan(x)/tan(y) to screen Point2D coordinate
    fn camera_txty_to_px_abs_xy(&self, txty: &TanXTanY) -> Point2D {
        let px_rel_xy = self.projection.txty_to_px_rel_xy(*txty);
        self.projection.px_rel_xy_to_px_abs_xy(px_rel_xy)
    }
}

//ip Camera
impl Camera {
    //fp new
    pub fn new(projection: Rc<dyn CameraProjection>, position: Point3D, direction: Quat) -> Self {
        Self {
            projection,
            position,
            direction,
        }
    }

    //cp placed_at
    pub fn placed_at(mut self, p: Point3D) -> Self {
        self.position = p;
        self
    }

    //cp with_direction
    pub fn with_direction(mut self, q: Quat) -> Self {
        self.direction = q;
        self
    }

    //cp moved_by
    pub fn moved_by(mut self, dp: [f64; 3]) -> Self {
        self.position = self.position + Point3D::from(dp);
        self
    }

    //cp rotated_by
    pub fn rotated_by(mut self, q: &Quat) -> Self {
        self.direction = *q * self.direction;
        self
    }

    //cp normalize
    pub fn normalize(&mut self) {
        self.direction = quat::normalize(*self.direction.as_ref()).into();
    }

    //zz All done
}
