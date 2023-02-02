//a Imports
use serde::{Deserialize, Serialize};

use super::{
    CameraProjection, CameraSensor, Point2D, RectSensor, RollYaw, SphericalLensPoly,
    SphericalLensProjection, TanXTanY,
};

//a CameraPolynomial
//tp CameraPolynomial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraPolynomial {
    /// Description of the (rectangular) sensor of the camera
    sensor: RectSensor,
    /// The spherical lens mapping polynomial
    lens: SphericalLensPoly,
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

//ip CameraPolynomial
impl CameraPolynomial {
    pub fn new(sensor: RectSensor, lens: SphericalLensPoly, mm_focus_distance: f64) -> Self {
        let mut cp = Self {
            sensor,
            lens,
            mm_focus_distance,
            maginification_of_focus: 1., // derived
            x_px_from_tan_sc: 1.,        // derived
            y_px_from_tan_sc: 1.,        // derived
        };
        cp.derive();
        cp
    }
    pub fn derive(&mut self) {
        self.sensor = self.sensor.clone().derive();
        let mm_focal_length = self.lens.mm_focal_length();
        self.maginification_of_focus =
            self.mm_focus_distance / (self.mm_focus_distance - mm_focal_length);
        let scale = mm_focal_length * self.maginification_of_focus;
        // mm_sensor height/width / scale is a tan
        // We want x_px = x_px_from_tan_sc * tan
        // But tan = x_px * mm_single_pixel_width / scale
        // hence x_px = tan * scale / mm_single_pixel_width
        self.x_px_from_tan_sc = scale / self.sensor.mm_single_pixel_width();
        self.y_px_from_tan_sc = scale / self.sensor.mm_single_pixel_height();
    }
}

//ip Display for CameraPolynomial
impl std::fmt::Display for CameraPolynomial {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "CamPoly[{}x{} lens {} @ {}mm]",
            self.sensor.px_width(),
            self.sensor.px_height(),
            self.lens.name(),
            self.lens.mm_focal_length(),
        )
    }
}

//ip CameraProjection for CameraPolynomial
impl CameraProjection for CameraPolynomial {
    /// Get name of camera
    fn camera_name(&self) -> &str {
        self.sensor.name()
    }

    /// Get name of lens
    fn lens_name(&self) -> &str {
        self.lens.name()
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
        self.sensor.px_rel_xy_to_px_abs_xy(xy)
    }

    /// Map from absolute to centre-relative pixel
    fn px_abs_xy_to_px_rel_xy(&self, xy: Point2D) -> Point2D {
        self.sensor.px_abs_xy_to_px_rel_xy(xy)
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
}
