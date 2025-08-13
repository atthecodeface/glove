//a Imports
use serde::Serialize;

use geo_nd::quat;

use ic_base::json;
use ic_base::{Point2D, Point3D, Quat, Result, RollYaw, TanXTanY};

use crate::{serialize_body_name, serialize_lens_name};
use crate::{CameraBody, CameraDatabase, CameraLens};
use crate::{CameraInstanceDesc, CameraProjection, CameraSensor};

//a CameraInstance
//tp CameraInstance
#[derive(Debug, Clone, Default, Serialize)]
pub struct CameraInstance {
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

//ip CameraInstance - Accessors
impl CameraInstance {
    //ap lens
    pub fn lens(&self) -> &CameraLens {
        &self.lens
    }

    //ap body
    pub fn body(&self) -> &CameraBody {
        &self.body
    }
}

//ip CameraInstance - Constructors and Destructors
impl CameraInstance {
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
    pub fn from_desc(cdb: &CameraDatabase, desc: CameraInstanceDesc) -> Result<Self> {
        let body = cdb.get_body_err(desc.body())?.clone();
        let lens = cdb.get_lens_err(desc.lens())?.clone();
        let mut camera = Self::new(
            body,
            lens,
            desc.mm_focus_distance(),
            *desc.position(),
            *desc.orientation(),
        );
        camera.derive();
        Ok(camera)
    }

    //cp from_json`
    pub fn from_json(cdb: &CameraDatabase, json: &str) -> Result<Self> {
        let desc: CameraInstanceDesc = json::from_json("camera instance descriptor", json)?;
        Self::from_desc(cdb, desc)
    }

    //dp to_desc
    pub fn to_desc(self) -> CameraInstanceDesc {
        CameraInstanceDesc::new(
            self.body.name().to_owned(),
            self.lens.name().to_owned(),
            self.mm_focus_distance,
            self.position,
            self.orientation,
        )
    }

    //dp to_desc_json
    pub fn to_desc_json(self) -> Result<String> {
        self.to_desc().to_json()
    }

    //fp to_json
    pub fn to_json(&self, pretty: bool) -> Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            Ok(serde_json::to_string(self)?)
        }
    }
}

//ip CameraInstance - Modifiers and other
impl CameraInstance {
    //mp set_body
    pub fn set_body(&mut self, body: CameraBody) {
        self.body = body;
        self.derive();
    }

    //mp set_lens
    pub fn set_lens(&mut self, lens: CameraLens) {
        self.lens = lens;
        self.derive();
    }

    //mp set_mm_focus_distance
    pub fn set_mm_focus_distance(&mut self, mm_focus_distance: f64) {
        self.mm_focus_distance = mm_focus_distance;
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

    //fp xmap_model
    /// Map a model coordinate to an absolute XY camera coordinate
    #[inline]
    pub fn map_model(&self, model: &Point3D) -> Point2D {
        self.world_xyz_to_px_abs_xy(model)
    }

    //zz All done
}

//ip Display for CameraInstance
impl std::fmt::Display for CameraInstance {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "CamPoly[{}x{} lens {} @ {}mm]",
            self.body.px_width(),
            self.body.px_height(),
            self.lens.name(),
            self.mm_focus_distance,
        )?;

        let dxyz = quat::apply3(&quat::conjugate(self.orientation.as_ref()), &[0., 0., -1.]);
        write!(
            fmt,
            "   @[{:.2},{:.2},{:.2}] in dir [{:0.4},{:0.4},{:0.4}]",
            self.position[0], self.position[1], self.position[2], dxyz[0], dxyz[1], dxyz[2],
        )
    }
}

//ip CameraProjection for CameraInstance
impl CameraProjection for CameraInstance {
    //ap camera_name
    /// Get name of camera
    fn camera_name(&self) -> String {
        self.body.name().into()
    }

    //ap lens_name
    /// Get name of lens
    fn lens_name(&self) -> String {
        self.lens.name().into()
    }

    //ap focus_distance
    // focus_distance
    fn focus_distance(&self) -> f64 {
        self.mm_focus_distance
    }

    //ap position
    fn position(&self) -> Point3D {
        self.position
    }

    //ap orientation
    fn orientation(&self) -> Quat {
        self.orientation
    }

    //mp set_position
    fn set_position(&mut self, p: &Point3D) {
        self.position = *p;
    }

    //mp set_orientation
    fn set_orientation(&mut self, q: &Quat) {
        self.orientation = *q;
    }

    //mp set_focus_distance
    fn set_focus_distance(&mut self, mm_focus_distance: f64) {
        self.mm_focus_distance = mm_focus_distance;
        self.derive()
    }

    //mp sensor_size
    fn sensor_size(&self) -> (f64, f64) {
        self.body.sensor_size()
    }

    //mp sensor_center
    fn sensor_center(&self) -> Point2D {
        self.body.sensor_center()
    }

    //mp sensor_ry_to_camera_ry
    /// Apply the lens projection
    #[inline]
    fn sensor_ry_to_camera_ry(&self, ry: &RollYaw) -> RollYaw {
        let tan_yaw = ry.tan_yaw();
        ry.with_tan_yaw(self.lens.tan_sensor_to_tan_world(tan_yaw))
    }

    //mp camera_ry_to_sensor_ry
    /// Apply the lens projection
    #[inline]
    fn camera_ry_to_sensor_ry(&self, ry: &RollYaw) -> RollYaw {
        let tan_yaw = ry.tan_yaw();
        ry.with_tan_yaw(self.lens.tan_world_to_tan_sensor(tan_yaw))
    }

    //mp sensor_txty_to_px_abs_xy
    fn sensor_txty_to_px_abs_xy(&self, txty: &TanXTanY) -> Point2D {
        let pxy_rel = [
            txty[0] * self.x_px_from_tan_sc,
            txty[1] * self.y_px_from_tan_sc,
        ]
        .into();
        self.body.px_rel_xy_to_px_abs_xy(&pxy_rel)
    }

    //mp px_abs_xy_to_sensor_txty
    fn px_abs_xy_to_sensor_txty(&self, pxy_abs: &Point2D) -> TanXTanY {
        let pxy_rel = self.body.px_abs_xy_to_px_rel_xy(pxy_abs);
        TanXTanY {
            data: [
                pxy_rel[0] / self.x_px_from_tan_sc,
                pxy_rel[1] / self.y_px_from_tan_sc,
            ]
            .into(),
        }
        // TanXTanY::of_tx_ty(
        // pxy_rel[0] / self.x_px_from_tan_sc,
        // pxy_rel[1] / self.y_px_from_tan_sc,
        // )
    }
}
