use ic_base::Point2D;

use crate::CameraSensor;

#[derive(Debug, Clone, Copy, Default)]
pub struct SizedSensor {
    pub width: u32,
    pub height: u32,
    pub mm_width: f64,
    pub mm_height: f64,
}

impl CameraSensor for SizedSensor {
    /// Get the size of the sensor in pixels
    fn sensor_px_size(&self) -> (f64, f64) {
        (self.width as f64, self.height as f64)
    }

    fn sensor_px_abs_to_optical_xy(&self, px_xy: Point2D) -> Point2D {
        [
            px_xy[0] - (self.width as f64 / 2.0),
            (self.height as f64 / 2.0) - px_xy[1],
        ]
        .into()
    }

    fn optical_xy_to_px_abs(&self, px_xy: Point2D) -> Point2D {
        [
            (self.width as f64 / 2.0) + px_xy[0],
            (self.height as f64 / 2.0) - px_xy[1],
        ]
        .into()
    }

    fn sensor_mm_single_pixel_height(&self) -> f64 {
        self.mm_height / (self.height as f64)
    }

    fn sensor_mm_single_pixel_width(&self) -> f64 {
        self.mm_width / (self.width as f64)
    }
}
