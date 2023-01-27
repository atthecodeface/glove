//a Imports
use super::{CameraProjection, CameraSensor, Point2D, RectSensor, TanXTanY};

//a CameraRectilinear
//tp CameraRectilinear
#[derive(Debug, Clone, Default)]
pub struct CameraRectilinear {
    sensor: RectSensor,
    diag_fov_degrees: f64,
    pub x_px_from_tan_sc: f64,
    pub y_px_from_tan_sc: f64,
}

//ip CameraRectilinear
impl CameraRectilinear {
    pub fn new(
        diag_fov_degrees: f64,
        px_width: usize,
        px_height: usize,
        px_aspect_ratio: f64,
        flip_y: bool,
    ) -> Self {
        let mm_sensor_width = 1.;
        let sensor = RectSensor::new(mm_sensor_width, px_width, px_height)
            .set_sensor_size(
                mm_sensor_width,
                mm_sensor_width * (px_height as f64) / (px_width as f64 * px_aspect_ratio),
            )
            .set_flip_y(flip_y);
        let mm_half_sensor_diag = sensor.mm_sensor_diagonal() / 2.0;
        let tan_half_diag_fov = (diag_fov_degrees / 2.).to_radians().tan();
        // For a rectilinear lens the distance from the centre of the sensor
        // to the point a ray strikes is propotional to the tan of the angle
        //
        // mm_half_sensor_diag corresponds to tan_half_diag_fov
        let mm_from_tan_sc = mm_half_sensor_diag / tan_half_diag_fov;

        // Now to pixels
        let x_px_from_tan_sc = px_width as f64 / sensor.mm_sensor_width() * mm_from_tan_sc;
        let y_px_from_tan_sc = px_height as f64 / sensor.mm_sensor_height() * mm_from_tan_sc;

        Self {
            sensor,
            diag_fov_degrees,
            x_px_from_tan_sc,
            y_px_from_tan_sc,
        }
    }

    // want 22.62 * 2 = 45.24
    pub fn x_fov_degrees(&self) -> f64 {
        let tan_half_fov = self.sensor.px_width() / 2. / self.x_px_from_tan_sc;
        (2.0 * tan_half_fov.atan()).to_degrees()
    }

    pub fn new_logitech_c270_640() -> Self {
        Self::new(55.03, 640, 480, 1., true)
    }
}

//ip Display for CameraRectilinear
impl std::fmt::Display for CameraRectilinear {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "CamRect[{}x{} diag fov {:0.4} x fov {:0.4}",
            self.sensor.px_width(),
            self.sensor.px_height(),
            self.diag_fov_degrees,
            self.x_fov_degrees()
        )
    }
}

//ip CameraProjection for CameraRectilinear
impl CameraProjection for CameraRectilinear {
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
        [
            px_xy[0] / self.x_px_from_tan_sc,
            px_xy[1] / self.y_px_from_tan_sc,
        ]
        .into()
    }

    /// Map a tan(x), tan(y) (i.e. x/z, y/z) to a centre-relative XY
    /// pixel in the frame of the camera
    ///
    /// This must apply the lens projection
    fn txty_to_px_rel_xy(&self, txty: TanXTanY) -> Point2D {
        [
            txty[0] * self.x_px_from_tan_sc,
            txty[1] * self.y_px_from_tan_sc,
        ]
        .into()
    }
}
