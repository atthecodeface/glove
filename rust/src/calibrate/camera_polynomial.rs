//a Imports
use super::{
    CameraProjection, CameraSensor, LensProjection, Point2D, Polynomial, RectSensor, RollYaw,
    TanXTanY,
};

//a CameraPolynomial
//tp CameraPolynomial
#[derive(Debug, Clone, Default)]
pub struct CameraPolynomial {
    sensor: RectSensor,
    lens: Polynomial,
    mm_focal_length: f64,
    /// Convert from tan(angle) to x pixel
    ///
    /// This is sensor.mm_single_pixel_width / sensor.mm_sensor_width * mm_focal_length
    pub x_px_from_tan_sc: f64,
    pub y_px_from_tan_sc: f64,
}

//ip CameraPolynomial
impl CameraPolynomial {
    pub fn new(sensor: RectSensor, lens: Polynomial, mm_focal_length: f64) -> Self {
        let x_px_from_tan_sc =
            sensor.mm_single_pixel_width() / sensor.mm_sensor_width() * mm_focal_length;
        let y_px_from_tan_sc =
            sensor.mm_single_pixel_height() / sensor.mm_sensor_height() * mm_focal_length;

        Self {
            sensor,
            lens,
            mm_focal_length,
            x_px_from_tan_sc,
            y_px_from_tan_sc,
        }
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
            self.mm_focal_length,
        )
    }
}

//ip CameraProjection for CameraPolynomial
impl CameraProjection for CameraPolynomial {
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
            self.lens.frame_to_camera(ry_frame.tan_yaw()),
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
            self.lens.camera_to_frame(ry_camera.tan_yaw()),
        );
        let txty_frame: TanXTanY = ry_frame.into();
        [
            txty_frame[0] * self.x_px_from_tan_sc,
            txty_frame[1] * self.y_px_from_tan_sc,
        ]
        .into()
    }
}
