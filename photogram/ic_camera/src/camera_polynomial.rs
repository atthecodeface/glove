//a Imports
use serde::{Deserialize, Serialize};

use geo_nd::quat;

use ic_base::json;
use ic_base::{Point2D, Point3D, Quat, RollYaw, TanXTanY};

use crate::{serialize_body_name, serialize_lens_name};
use crate::{CameraBody, CameraDatabase, CameraLens};
use crate::{CameraProjection, CameraSensor};

//a CameraPolynomialDesc
//tp CameraPolynomialDesc
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CameraPolynomialDesc {
    /// Name of the camera body
    pub body: String,
    /// The spherical lens mapping polynomial
    pub lens: String,
    /// The distance the lens if focussed on - make it 1E6*mm_focal_length  for infinity
    pub mm_focus_distance: f64,
    /// Position in world coordinates of the camera
    pub position: Point3D,
    /// Orientation to be applied to camera-relative world coordinates
    /// to convert to camera-space coordinates
    pub orientation: Quat,
}

//a CameraPolynomial
//tp CameraPolynomial
#[derive(Debug, Clone, Default, Serialize)]
pub struct CameraPolynomial {
    /// Description of the camera body
    #[serde(serialize_with = "serialize_body_name")]
    body: CameraBody,
    /// The spherical lens mapping polynomial
    #[serde(serialize_with = "serialize_lens_name")]
    lens: CameraLens,
    /// The distance the lens if focussed on - make it 1E6*mm_focal_length  for infinity
    ///
    /// Note 1/f = 1/u + 1/v; hence u = 1/(1/f - 1/v) = fv / v-f
    ///
    /// the polynomial is calibrated at infinity then it is set for u = f
    ///
    /// For an actual 'd' we have u' = fd/(f-d); the image is magnified on the sensor by u'/u,
    /// which is u'/f or d/(d-f)
    mm_focus_distance: f64,
    /// Position in world coordinates of the camera
    #[serde(default)]
    position: Point3D,
    /// Orientation to be applied to camera-relative world coordinates
    /// to convert to camera-space coordinates
    #[serde(default)]
    orientation: Quat,
    /// Derived magnification due to focus distance
    #[serde(skip)]
    maginification_of_focus: f64,
    /// Convert from tan(angle) to x pixel
    ///
    /// This is sensor.mm_single_pixel_width / sensor.mm_sensor_width * mm_focal_length
    #[serde(skip)]
    x_px_from_tan_sc: f64,
    /// Convert from tan(angle) to y pixel
    #[serde(skip)]
    y_px_from_tan_sc: f64,
}

//ip CameraPolynomial
impl CameraPolynomial {
    //ap lens
    pub fn lens(&self) -> &CameraLens {
        &self.lens
    }

    //ap body
    pub fn body(&self) -> &CameraBody {
        &self.body
    }

    //cp new
    pub fn new(
        body: CameraBody,
        lens: CameraLens,
        mm_focus_distance: f64,
        position: Point3D,
        orientation: Quat,
    ) -> Self {
        let mut cp = Self {
            body,
            lens,
            mm_focus_distance,
            position,
            orientation,
            maginification_of_focus: 1., // derived
            x_px_from_tan_sc: 1.,        // derived
            y_px_from_tan_sc: 1.,        // derived
        };
        cp.derive();
        cp
    }

    //cp from_desc
    pub fn from_desc(cdb: &CameraDatabase, desc: CameraPolynomialDesc) -> Result<Self, String> {
        let body = cdb.get_body_err(&desc.body)?.clone();
        let lens = cdb.get_lens_err(&desc.lens)?.clone();
        Ok(Self::new(
            body,
            lens,
            desc.mm_focus_distance,
            desc.position,
            desc.orientation,
        ))
    }

    //cp from_json
    pub fn from_json(cdb: &CameraDatabase, json: &str) -> Result<Self, String> {
        let desc: CameraPolynomialDesc = json::from_json("camera instance descriptor", json)?;
        Self::from_desc(cdb, desc)
    }

    //fp to_json
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("{}", e))
    }

    //mp set_lens
    pub fn set_lens(&mut self, lens: CameraLens) {
        self.lens = lens;
        self.derive();
    }

    //mp derive
    pub fn derive(&mut self) {
        let mm_focal_length = self.lens.mm_focal_length();
        self.maginification_of_focus =
            self.mm_focus_distance / (self.mm_focus_distance - mm_focal_length);
        let scale = mm_focal_length * self.maginification_of_focus;
        // mm_sensor height/width / scale is a tan
        // We want x_px = x_px_from_tan_sc * tan
        // But tan = x_px * mm_single_pixel_width / scale
        // hence x_px = tan * scale / mm_single_pixel_width
        self.x_px_from_tan_sc = scale / self.body.mm_single_pixel_width();
        self.y_px_from_tan_sc = scale / self.body.mm_single_pixel_height();
    }

    //fp map_model
    /// Map a model coordinate to an absolute XY camera coordinate
    #[inline]
    pub fn map_model(&self, model: Point3D) -> Point2D {
        self.world_xyz_to_px_abs_xy(model)
    }

    //zz All done
}

//ip Display for CameraPolynomial
impl std::fmt::Display for CameraPolynomial {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "CamPoly[{}x{} lens {} @ {}mm]",
            self.body.px_width(),
            self.body.px_height(),
            self.lens.name(),
            self.lens.mm_focal_length(),
        )?;

        let dxyz = quat::apply3(&quat::conjugate(self.orientation.as_ref()), &[0., 0., 1.]);
        write!(
            fmt,
            "   @[{:.2},{:.2},{:.2}] in dir [{:.2},{:.2},{:.2}]",
            self.position[0], self.position[1], self.position[2], dxyz[0], dxyz[1], dxyz[2],
        )
    }
}

//ip CameraProjection for CameraPolynomial
impl CameraProjection for CameraPolynomial {
    /// Get name of camera
    fn camera_name(&self) -> String {
        self.body.name().into()
    }

    /// Get name of lens
    fn lens_name(&self) -> String {
        self.lens.name().into()
    }

    // focus_distance
    fn focus_distance(&self) -> f64 {
        self.mm_focus_distance
    }

    //mp position
    fn position(&self) -> Point3D {
        self.position
    }

    //mp orientation
    fn orientation(&self) -> Quat {
        self.orientation
    }

    //mp set_position
    fn set_position(&mut self, p: Point3D) {
        self.position = p;
    }

    //mp set_orientation
    fn set_orientation(&mut self, q: Quat) {
        self.orientation = q;
    }

    // set_focus_distance
    fn set_focus_distance(&mut self, mm_focus_distance: f64) {
        self.mm_focus_distance = mm_focus_distance;
        self.derive()
    }

    //fp px_abs_xy_to_camera_txty
    /// Map a screen Point2D coordinate to tan(x)/tan(y)
    fn px_abs_xy_to_camera_txty(&self, px_abs_xy: Point2D) -> TanXTanY {
        let px_rel_xy = self.px_abs_xy_to_px_rel_xy(px_abs_xy);
        self.px_rel_xy_to_txty(px_rel_xy)
    }

    //fp camera_txty_to_px_abs_xy
    /// Map a tan(x)/tan(y) to screen Point2D coordinate
    fn camera_txty_to_px_abs_xy(&self, txty: &TanXTanY) -> Point2D {
        let px_rel_xy = self.txty_to_px_rel_xy(*txty);
        self.px_rel_xy_to_px_abs_xy(px_rel_xy)
    }

    /// Map from centre-relative to absolute pixel
    fn px_rel_xy_to_px_abs_xy(&self, xy: Point2D) -> Point2D {
        self.body.px_rel_xy_to_px_abs_xy(xy)
    }

    /// Map from absolute to centre-relative pixel
    fn px_abs_xy_to_px_rel_xy(&self, xy: Point2D) -> Point2D {
        self.body.px_abs_xy_to_px_rel_xy(xy)
    }

    /// Map an actual centre-relative XY pixel in the frame of the
    /// camera to a Roll/Yaw
    fn px_rel_xy_to_txty(&self, px_xy: Point2D) -> TanXTanY {
        let txty_frame: TanXTanY = [
            px_xy[0] / self.x_px_from_tan_sc,
            px_xy[1] / self.y_px_from_tan_sc,
        ]
        .into();
        let ry_frame: RollYaw = txty_frame.into();
        let ry_camera = RollYaw::from_roll_tan_yaw(
            ry_frame.sin_roll(),
            ry_frame.cos_roll(),
            self.lens.sensor_to_world(ry_frame.tan_yaw()),
        );
        ry_camera.into()
    }

    /// Map a tan(x), tan(y) (i.e. x/z, y/z) to a centre-relative XY
    /// pixel in the frame of the camera
    ///
    /// This must apply the lens projection
    fn txty_to_px_rel_xy(&self, txty: TanXTanY) -> Point2D {
        let ry_camera: RollYaw = txty.into();
        let ry_frame = RollYaw::from_roll_tan_yaw(
            ry_camera.sin_roll(),
            ry_camera.cos_roll(),
            self.lens.world_to_sensor(ry_camera.tan_yaw()),
        );
        let txty_frame: TanXTanY = ry_frame.into();
        [
            txty_frame[0] * self.x_px_from_tan_sc,
            txty_frame[1] * self.y_px_from_tan_sc,
        ]
        .into()
    }

    //mp px_rel_xy_to_ry
    /// Map an actual centre-relative XY pixel in the frame of the
    /// camera to a Roll/Yaw
    fn px_rel_xy_to_ry(&self, px_xy: Point2D) -> RollYaw {
        let txty_frame: TanXTanY = [
            px_xy[0] / self.x_px_from_tan_sc,
            px_xy[1] / self.y_px_from_tan_sc,
        ]
        .into();
        txty_frame.into()
    }
}
