use ic_base::Point2D;

use crate::CameraSensor;

/// A simple rectangle that mimics a camera sensor
///
/// This can be used as a simple 'camera body', and in conjunction with a simple
/// lens can provide a simple camera
#[derive(Debug, Clone, Copy, Default)]
pub struct SizedSensor {
    /// Width of the sensor in pixels
    pub width: u32,
    /// Height of the sensor in pixels
    pub height: u32,
    /// Width of the sensor (notionally) in mm
    pub mm_width: f64,
    /// Height of the sensor (notionally) in mm
    pub mm_height: f64,
}

impl SizedSensor {
    /// Create a new [SizedSensor] with square (in mm) pixels
    pub fn new(width: u32, height: u32, mm_per_pixel: f64) -> Self {
        Self {
            width,
            height,
            mm_width: (width as f64) * mm_per_pixel,
            mm_height: (height as f64) * mm_per_pixel,
        }
    }
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
