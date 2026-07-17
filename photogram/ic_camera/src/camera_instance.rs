use serde::Serialize;

use geo_nd::quat;

use ic_base::{Point2D, Point3D, Quat, Result, RollYaw, TanXTanY};

use crate::{CameraBody, CameraDatabase, CameraLens};
use crate::{CameraInstanceDesc, CameraProjection, CameraSensor};
use crate::{serialize_body_name, serialize_lens_name};

/// An instance of a camera - a body, lens, focus distance, position and orientation
///
/// The sensor has a pixel count (width and height), size in mm (width
/// and height), and a center pixel
///
/// The lens has a focal length and a single bijective mapping from
/// RollYaw in world space to/from RollYaw in sensor space
///
/// The instance has the lens focussed at a certain distance
///
/// The standard lens equation applies: 1/f = 1/u + 1/v
///
/// Here, f is the focal length of the lens, u the focus distance from
/// the lens to the object that it is focused on.
///
/// v is the distance from the lens to the sensor; v = u*f/(u-f)
///
/// An absolute PXY position on the sensor can be mapped to one relative to the
/// center pixel (+Y is up); this can be mapped to an X and Y mm.
///
/// The XYZ camera space is +X right, +Y up, +Z *out*, i.e. the centre pixel in
/// an image is in the direction *(0,0,-1)*.
///
/// Then the direction vector for a sensor pixel is (X, Y, -v)
///
/// A TanXTanY represents a vector (tx,ty,-1); hence tx = X/v, ty=Y/v.
///
/// The scaling for pixel x-to-tx is:
///
///   (px - center) * sensor.mm_single_pixel_width / v
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
    /// Note the thin lens equation of 1/f = 1/u + 1/v
    ///
    /// This is the 'u' in the thin lens equation
    mm_focus_distance: f64,

    /// The distance from the lens to the sensor given
    /// mm_focus_distance and the lens focal length
    ///
    /// This is the 'v' in the thin lens equation
    ///
    /// v = u*f/(u-f)
    #[serde(default)]
    lens_sensor_distance: f64,

    /// Position in world coordinates of the camera
    ///
    /// World/Model XYZ  = Camera relative XYZ + camera position
    #[serde(default)]
    position: Point3D,

    /// Orientation to be applied to camera-relative world coordinates
    /// to convert to camera-space coordinates
    ///
    /// camera space direction = orientation * (world posn - cam posn)
    #[serde(default)]
    orientation: Quat,

    /// Convert to tan(angle) from relative x pixel
    ///
    /// This is sensor.mm_single_pixel_width / lens_sensor_distance
    #[serde(skip)]
    tx_from_px_sc: f64,

    /// Convert to tan(angle) from relative y pixel
    ///
    /// This is sensor.mm_single_pixel_height / lens_sensor_distance
    #[serde(skip)]
    ty_from_py_sc: f64,
}

impl CameraInstance {
    /// Get a reference to the camera lens
    pub fn lens(&self) -> &CameraLens {
        &self.lens
    }

    /// Get a reference to the camera body
    pub fn body(&self) -> &CameraBody {
        &self.body
    }
}

impl CameraInstance {
    /// Get a new camera with a given body and lens, focussed at a particular
    /// distance with a given position and orientation
    ///
    /// Orientation is the world-to-camera quaternion; its conjugate is camera-to-world
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
            lens_sensor_distance: 1., // derived
            tx_from_px_sc: 1.,        // derived
            ty_from_py_sc: 1.,        // derived
        };
        cp.derive();
        cp
    }

    /// Build a [CameraInstance] from a descriptor, given a camera database to retrieve the lens and body from (by name)
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

    /// Deconstruct the [CameraInstance] into a descripton which can be serialized
    pub fn to_desc(self) -> CameraInstanceDesc {
        CameraInstanceDesc::new(
            self.body.name().to_owned(),
            self.lens.name().to_owned(),
            self.mm_focus_distance,
            self.position,
            self.orientation,
        )
    }

    /// Deconstruct the [CameraInstance] into a descripton which can be serialized
    pub fn to_desc_json(self) -> Result<String> {
        self.to_desc().to_json()
    }

    /// Generate the json of the [CameraInstance]
    pub fn to_json(&self, pretty: bool) -> Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            Ok(serde_json::to_string(self)?)
        }
    }
}

impl CameraInstance {
    /// Set the body of the camera instance
    pub fn set_body(&mut self, body: CameraBody) {
        self.body = body;
        self.derive();
    }

    /// Set the lens of the camera instance
    pub fn set_lens(&mut self, lens: CameraLens) {
        self.lens = lens;
        self.derive();
    }

    /// Set the distance of focus for the camera instance
    pub fn set_mm_focus_distance(&mut self, mm_focus_distance: f64) {
        self.mm_focus_distance = mm_focus_distance;
        self.derive();
    }

    /// Derive the extra data required for the [CameraInstance], from the body, lens and focus distance
    pub fn derive(&mut self) {
        let mm_focal_length = self.lens.mm_focal_length();

        self.lens_sensor_distance =
            self.mm_focus_distance * mm_focal_length / (self.mm_focus_distance - mm_focal_length);

        self.tx_from_px_sc = self.body.mm_single_pixel_width() / self.lens_sensor_distance;
        self.ty_from_py_sc = self.body.mm_single_pixel_height() / self.lens_sensor_distance;
    }
}

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

impl CameraProjection for CameraInstance {
    fn camera_name(&self) -> String {
        self.body.name().into()
    }

    fn lens_name(&self) -> String {
        self.lens.name().into()
    }

    fn focal_length(&self) -> f64 {
        self.lens.mm_focal_length()
    }

    fn focus_distance(&self) -> f64 {
        self.mm_focus_distance
    }

    fn position(&self) -> Point3D {
        self.position
    }

    fn orientation(&self) -> Quat {
        self.orientation
    }

    fn set_position(&mut self, p: &Point3D) {
        self.position = *p;
    }

    fn set_orientation(&mut self, q: &Quat) {
        self.orientation = *q;
    }

    fn set_focus_distance(&mut self, mm_focus_distance: f64) {
        self.mm_focus_distance = mm_focus_distance;
        self.derive()
    }

    fn sensor_mm_size(&self) -> (f64, f64) {
        (self.body.mm_sensor_width(), self.body.mm_sensor_height())
    }

    fn sensor_px_size(&self) -> (f64, f64) {
        self.body.sensor_px_size()
    }

    fn sensor_px_center(&self) -> Point2D {
        self.body.sensor_px_center()
    }

    #[inline]
    fn sensor_ry_to_camera_ry(&self, ry: &RollYaw) -> RollYaw {
        let tan_yaw = ry.tan_yaw();
        ry.with_tan_yaw(self.lens.tan_sensor_to_tan_world(tan_yaw))
    }

    #[inline]
    fn camera_ry_to_sensor_ry(&self, ry: &RollYaw) -> RollYaw {
        let tan_yaw = ry.tan_yaw();
        ry.with_tan_yaw(self.lens.tan_world_to_tan_sensor(tan_yaw))
    }

    fn sensor_txty_to_px_abs_xy(&self, txty: &TanXTanY) -> Point2D {
        let pxy_rel = [txty[0] / self.tx_from_px_sc, txty[1] / self.ty_from_py_sc].into();
        self.body.px_rel_xy_to_px_abs_xy(&pxy_rel)
    }

    fn px_abs_xy_to_sensor_txty(&self, pxy_abs: &Point2D) -> TanXTanY {
        let pxy_rel = self.body.px_abs_xy_to_px_rel_xy(pxy_abs);
        TanXTanY::of_tx_ty(
            pxy_rel[0] * self.tx_from_px_sc,
            pxy_rel[1] * self.ty_from_py_sc,
        )
    }
}
