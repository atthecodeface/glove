//a Imports
use std::rc::Rc;

use geo_nd::quat;
use serde::{Deserialize, Serialize};

use crate::{
    json, CameraBody, CameraDatabase, CameraLens, CameraPolynomial, CameraPolynomialDesc,
    CameraProjection, CameraView, Point2D, Point3D, Quat, TanXTanY,
};

//a CameraInstance
//tp CameraInstanceDesc
/// A struct that contains the fields required for deserializing a
/// CameraInstance in conjnunction with a CameraDatabase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraInstanceDesc {
    camera: CameraPolynomialDesc,
    /// Position in world coordinates of the camera
    position: Point3D,
    /// Direction to be applied to camera-relative world coordinates
    /// to convert to camera-space coordinates
    direction: Quat,
}

//tp CameraInstance
/// A camera that allows mapping a world point to camera relative XYZ,
/// and then it can be mapped to tan(x) / tan(y) to roll/yaw or pixel
/// relative XY (relative to the center of the camera sensor)
#[derive(Debug, Clone, Default, Serialize)]
pub struct CameraInstance {
    /// Map from tan(x), tan(y) to Roll/Yaw or even to pixel relative
    /// XY
    camera: Rc<CameraPolynomial>,
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

//ip Display for CameraInstance
impl std::fmt::Display for CameraInstance {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let dxyz = quat::apply3(&quat::conjugate(self.direction.as_ref()), &[0., 0., 1.]);
        // First rotation around Y axis (yaw)
        let yaw = dxyz[0].atan2(dxyz[2]).to_degrees();
        let (axis, angle) = quat::as_axis_angle(self.direction.as_ref());
        // Then rotation around X axis (elevation)
        let pitch = dxyz[1]
            .atan2((dxyz[0] * dxyz[0] + dxyz[2] * dxyz[2]).sqrt())
            .to_degrees();
        write!(
            fmt,
            "@[{:.2},{:.2},{:.2}] yaw {:.4} pitch {:.4} unit dir [{:.2},{:.2},{:.2}] (q axis {:.3},{:.3},{:.3} angle {:.2}",
            self.position[0],
            self.position[1],
            self.position[2],
            yaw,
            pitch,
            dxyz[0],
            dxyz[1],
            dxyz[2],
            axis[0],
            axis[1],
            axis[2],
            angle.to_degrees()
        )
    }
}

//ip CameraView for CameraInstance
impl CameraView for CameraInstance {
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
    fn px_abs_xy_to_camera_txty(&self, px_abs_xy: Point2D) -> TanXTanY {
        let px_rel_xy = self.camera.px_abs_xy_to_px_rel_xy(px_abs_xy);
        self.camera.px_rel_xy_to_txty(px_rel_xy)
    }

    //fp camera_txty_to_px_abs_xy
    /// Map a tan(x)/tan(y) to screen Point2D coordinate
    fn camera_txty_to_px_abs_xy(&self, txty: &TanXTanY) -> Point2D {
        let px_rel_xy = self.camera.txty_to_px_rel_xy(*txty);
        self.camera.px_rel_xy_to_px_abs_xy(px_rel_xy)
    }
}

//ip CameraInstance
impl CameraInstance {
    //ap camera
    pub fn camera(&self) -> &Rc<CameraPolynomial> {
        &self.camera
    }

    //ap body
    pub fn body(&self) -> &Rc<CameraBody> {
        self.camera.body()
    }

    //ap lens
    pub fn lens(&self) -> &Rc<CameraLens> {
        self.camera.lens()
    }

    //fp new
    pub fn new(camera: Rc<CameraPolynomial>, position: Point3D, direction: Quat) -> Self {
        Self {
            camera,
            position,
            direction,
        }
    }

    //cp from_desc
    pub fn from_desc(cdb: &CameraDatabase, desc: CameraInstanceDesc) -> Result<Self, String> {
        let camera = CameraPolynomial::from_desc(cdb, desc.camera)?;
        Ok(Self::new(camera.into(), desc.position, desc.direction))
    }

    //cp from_json
    pub fn from_json(cdb: &CameraDatabase, json: &str) -> Result<Self, String> {
        let desc: CameraInstanceDesc = json::from_json("camera instance descriptor", json)?;
        Self::from_desc(cdb, desc)
    }

    //fp to_json
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("{}", e))
    }

    //mp set_projection
    pub fn set_projection(&mut self, camera: Rc<CameraPolynomial>) {
        self.camera = camera;
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
    pub fn moved_by(mut self, dp: Point3D) -> Self {
        self.position += dp;
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

    //mp place_at
    pub fn place_at(&mut self, p: Point3D) {
        self.position = p;
    }

    //mp set_direction
    pub fn set_direction(&mut self, q: Quat) {
        self.direction = q;
    }

    //mp clone_placed_at
    pub fn clone_placed_at(&self, location: Point3D) -> Self {
        self.clone().placed_at(location)
    }

    //mp clone_with_direction
    pub fn clone_with_direction(&self, direction: Quat) -> Self {
        self.clone().with_direction(direction)
    }

    //mp clone_moved_by
    pub fn clone_moved_by(&self, dp: Point3D) -> Self {
        self.clone().moved_by(dp)
    }

    //mp clone_rotated_by
    pub fn clone_rotated_by(&self, q: &Quat) -> Self {
        self.clone().rotated_by(q)
    }

    //fp map_model
    /// Map a model coordinate to an absolute XY camera coordinate
    #[inline]
    pub fn map_model(&self, model: Point3D) -> Point2D {
        self.world_xyz_to_px_abs_xy(model)
    }

    //zz All done
}
