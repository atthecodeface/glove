//a Imports
use std::rc::Rc;

use geo_nd::{quat, Quaternion};
use serde::{Deserialize, Serialize};

use crate::{
    json, CameraBody, CameraDatabase, CameraInstance, CameraLens, CameraProjection, CameraSensor,
    CameraView, Point2D, Point3D, Quat, RollYaw, SphericalLensProjection, TanXTanY,
};

//a CameraPolynomialCalibrateDesc
//tp CameraPolynomialCalibrateDesc
/// A description of a calibration for a camera and a lens, for an
/// image of a grid (e.g. graph paper)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraPolynomialCalibrateDesc {
    /// Camera description (body, lens, min focus distance)
    camera: CameraPolynomialDesc,
    /// Distance of the camera from the graph paper for the photograph
    distance: f64,
    /// Grid point the camera is centred on
    centred_on: (f64, f64),
    /// Rotation around the X axis (after accounting for z-rotation),
    /// i.e. how much the camera is off left-right from straight-on
    x_rotation: f64,
    /// Rotation around the X axis (after accounting for z-rotation),
    /// i.e. how much the camera is off left-right from straight-on
    y_rotation: f64,
    /// Rotation around the Z axis, i.e. how much off-horiztonal the
    /// camera was
    z_rotation: f64,
    /// Mappings from grid coordinates to absolute camera pixel values
    mappings: Vec<(isize, isize, usize, usize)>,
}

//a CameraPolynomialCalibrate
//tp CameraPolynomialCalibrate
#[derive(Debug, Clone, Default, Serialize)]
pub struct CameraPolynomialCalibrate {
    /// Description of the camera body
    #[serde(serialize_with = "serialize_body_name")]
    body: Rc<CameraBody>,
    /// The spherical lens mapping polynomial
    #[serde(serialize_with = "serialize_lens_name")]
    lens: Rc<CameraLens>,
    /// Distance of the focus of the camera in the image
    mm_focus_distance: f64,
    /// Distance of the camera from the graph paper for the photograph
    distance: f64,
    /// Grid point the camera is centred on
    centred_on: (f64, f64),
    /// Rotation around the X axis (after accounting for z-rotation),
    /// i.e. how much the camera is off left-right from straight-on
    x_rotation: f64,
    /// Rotation around the X axis (after accounting for z-rotation),
    /// i.e. how much the camera is off left-right from straight-on
    y_rotation: f64,
    /// Rotation around the Z axis, i.e. how much off-horiztonal the
    /// camera was
    z_rotation: f64,
    /// Mappings from grid coordinates to absolute camera pixel values
    mappings: Vec<(isize, isize, usize, usize)>,
    /// Derived camera instance
    #[serde(skip)]
    camera: CameraInstance,
}

//ip CameraPolynomialCalibrate
impl CameraPolynomialCalibrate {
    //ap camera
    pub fn camera(&self) -> &CameraInstance {
        &self.camera
    }

    //ap distance
    pub fn distance(&self) -> f64 {
        self.distance
    }

    //cp from_desc
    pub fn from_desc(
        cdb: &CameraDatabase,
        desc: CameraPolynomialCalibrateDesc,
    ) -> Result<Self, String> {
        let position = [desc.centred_on.0, desc.centred_on.1, desc.distance].into();
        let direction: Quat = quat::look_at(&[0., 0., -1.], &[0., 1., 0.]).into();
        let rotate_x: Quat = quat::of_axis_angle(&[1., 0., 0.], desc.x_rotation).into();
        let rotate_y: Quat = quat::of_axis_angle(&[0., 1., 0.], desc.y_rotation).into();
        let rotate_z: Quat = quat::of_axis_angle(&[0., 0., 1.], desc.z_rotation).into();
        let direction = direction * rotate_z;
        let direction = direction * rotate_x;
        let direction = direction * rotate_y;
        let position: Point3D = rotate_y.conjugate().apply3(&position);
        let position: Point3D = rotate_x.conjugate().apply3(&position);

        let body = cdb.get_body_err(&desc.camera.body)?;
        let lens = cdb.get_lens_err(&desc.camera.lens)?;
        let camera = CameraInstance::new(
            body.clone(),
            lens.clone(),
            desc.camera.mm_focus_distance,
            position,
            direction,
        );
        // eprintln!("{camera}");
        // let m: Point3D = camera.camera_xyz_to_world_xyz([0., 0., -desc.distance].into());
        // eprintln!("Camera {camera} focused on {m}");
        let s = Self {
            body,
            lens,
            camera,
            mm_focus_distance: desc.camera.mm_focus_distance,
            centred_on: desc.centred_on,
            distance: desc.distance,
            x_rotation: desc.x_rotation,
            y_rotation: desc.y_rotation,
            z_rotation: desc.z_rotation,
            mappings: desc.mappings,
        };
        Ok(s)
    }

    //cp from_json
    pub fn from_json(cdb: &CameraDatabase, json: &str) -> Result<Self, String> {
        let desc: CameraPolynomialCalibrateDesc =
            json::from_json("camera calibration descriptor", json)?;
        Self::from_desc(cdb, desc)
    }

    //mp grid_as_model
    /*
    pub fn grid_as_model(&self, grid: Point2D) -> Point3D {
        let xy_rel = self.camera_poly.px_abs_xy_to_px_rel_xy(xy);
        let ry = self.camera_poly.px_rel_xy_to_ry(&self, xy_rel);
    }
     */

    //ap get_pairings
    /// Get pairings between grid points, their camera-relative Point3Ds, and the roll-yaw described by
    /// the camera focus distance and lens type (not using its
    /// polynomial)
    pub fn get_pairings(&self) -> Vec<(Point2D, Point3D, RollYaw)> {
        let mut result = vec![];
        for (kx, ky, vx, vy) in &self.mappings {
            let grid: Point2D = [*kx as f64, *ky as f64].into();
            let grid_world: Point3D = [*kx as f64, *ky as f64, 0.].into();
            let grid_camera = self.camera.world_xyz_to_camera_xyz(grid_world);
            let pxy_abs: Point2D = [*vx as f64, *vy as f64].into();
            let pxy_rel = self.camera.px_abs_xy_to_px_rel_xy(pxy_abs);
            // eprintln!("{grid} : {grid_camera} : {pxy_abs} : {pxy_rel}");
            let ry = self.camera.px_rel_xy_to_ry(pxy_rel);
            result.push((grid, grid_camera, ry));
        }
        result
    }

    //ap get_xy_pairings
    /// Get XY pairings
    pub fn get_xy_pairings(&self) -> Vec<(Point2D, Point2D)> {
        let mut result = vec![];
        for (kx, ky, vx, vy) in &self.mappings {
            let grid: Point2D = [*kx as f64, *ky as f64].into();
            let pxy_abs: Point2D = [*vx as f64, *vy as f64].into();
            result.push((grid, pxy_abs));
        }
        result
    }
}

//a CameraPolynomialDesc
//tp CameraPolynomialDesc
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CameraPolynomialDesc {
    /// Name of the camera body
    body: String,
    /// The spherical lens mapping polynomial
    lens: String,
    /// The distance the lens if focussed on - make it 1E6*mm_focal_length  for infinity
    mm_focus_distance: f64,
}

//a CameraPolynomial
//tp CameraPolynomial
#[derive(Debug, Clone, Default, Serialize)]
pub struct CameraPolynomial {
    /// Description of the camera body
    #[serde(serialize_with = "serialize_body_name")]
    body: Rc<CameraBody>,
    /// The spherical lens mapping polynomial
    #[serde(serialize_with = "serialize_lens_name")]
    lens: Rc<CameraLens>,
    /// The distance the lens if focussed on - make it 1E6*mm_focal_length  for infinity
    ///
    /// Note 1/f = 1/u + 1/v; hence u = 1/(1/f - 1/v) = fv / v-f
    ///
    /// the polynomial is calibrated at infinity then it is set for u = f
    ///
    /// For an actual 'd' we have u' = fd/(f-d); the image is magnified on the sensor by u'/u,
    /// which is u'/f or d/(d-f)
    mm_focus_distance: f64,
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

fn serialize_body_name<S: serde::Serializer>(
    body: &Rc<CameraBody>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(body.name())
}

fn serialize_lens_name<S: serde::Serializer>(
    lens: &Rc<CameraLens>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(lens.name())
}

//ip CameraPolynomial
impl CameraPolynomial {
    //ap lens
    pub fn lens(&self) -> &Rc<CameraLens> {
        &self.lens
    }

    //ap body
    pub fn body(&self) -> &Rc<CameraBody> {
        &self.body
    }

    //cp new
    pub fn new(body: Rc<CameraBody>, lens: Rc<CameraLens>, mm_focus_distance: f64) -> Self {
        let mut cp = Self {
            body,
            lens,
            mm_focus_distance,
            maginification_of_focus: 1., // derived
            x_px_from_tan_sc: 1.,        // derived
            y_px_from_tan_sc: 1.,        // derived
        };
        cp.derive();
        cp
    }

    //cp from_desc
    pub fn from_desc(cdb: &CameraDatabase, desc: CameraPolynomialDesc) -> Result<Self, String> {
        let body = cdb.get_body_err(&desc.body)?;
        let lens = cdb.get_lens_err(&desc.lens)?;
        Ok(Self::new(body, lens, desc.mm_focus_distance))
    }

    //mp set_lens
    pub fn set_lens(&mut self, lens: Rc<CameraLens>) {
        self.lens = lens.clone();
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

    fn set_focus_distance(&mut self, mm_focus_distance: f64) {
        self.mm_focus_distance = mm_focus_distance;
        self.derive()
    }
    fn focus_distance(&self) -> f64 {
        self.mm_focus_distance
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
