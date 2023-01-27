//a Imports
use super::{CameraSensor, Point2D, RollDist};

//a RectSensor
//tp RectSensor
/// A rectangular camera sensor
///
#[derive(Debug, Clone)]
pub struct RectSensor {
    /// Centre pixel
    px_centre: [f64; 2],

    /// Width of sensor in pixels (normally an int)
    px_width: f64,

    /// Height of sensor in pixels (normally an int)
    px_height: f64,

    /// Set to true if sensor absolute pixel coords have origin at top left
    flip_y: bool,

    // The width of the sensor in mm
    //
    // Assuming that the camera changes focal length purely by moving the lens
    // away from the sensor, the offset onto the sensor will scale purely by
    // mm_focal_length / mm_sensor_width
    mm_sensor_width: f64,

    // The height of the sensor in mm
    mm_sensor_height: f64,

    /// Derived width of a sensor pixel in mm
    ///
    /// mm_sensor_width / px_width
    mm_single_pixel_width: f64,

    /// Derived width of a sensor pixel in mm
    ///
    /// mm_sensor_height / px_height
    mm_single_pixel_height: f64,

    /// Derived non-squareness of sensor pixel - X to Y ratio
    ///
    /// = mm_single_pixel_width / mm_single_pixel_height
    pixel_aspect_ratio: f64,
}

//ip Default for RectSensor
/// Sensor sizes:
///   medium format 53.7 by 40.2mm
///   medium format 43.8 by 32.9mm
///   Full frame 35mm is 36.0 by 24.0mm
///   Nikon APS-C 23.6 by 15.6mm
///   Canon APS-C 22.3 by 14.9mm (or 22.2 by 14.8)
///   Canon APS-H 28.7 by 19.0mm
impl std::default::Default for RectSensor {
    fn default() -> Self {
        (Self {
            px_centre: [200., 150.],
            px_width: 400.,
            px_height: 300.,
            flip_y: false,
            mm_sensor_width: 36.,
            mm_sensor_height: 24.,
            pixel_aspect_ratio: 1.,
            mm_single_pixel_width: 1.,
            mm_single_pixel_height: 1.,
        })
        .derive()
    }
}

//ip RectSensor
impl RectSensor {
    //fp new
    pub fn new(mm_sensor_width: f64, px_width: usize, px_height: usize) -> Self {
        Self::default()
            .set_px_frame(px_width, px_height)
            .set_sensor_width(mm_sensor_width)
            .derive()
    }

    //cp set_flip_y
    pub fn set_flip_y(mut self, flip_y: bool) -> Self {
        self.flip_y = flip_y;
        self
    }

    //cp set_sensor_size
    /// Set the sensor size
    pub fn set_sensor_size(mut self, mm_sensor_width: f64, mm_sensor_height: f64) -> Self {
        self.mm_sensor_width = mm_sensor_width;
        self.mm_sensor_height = mm_sensor_height;
        self
    }

    //cp set_sensor_width
    /// Set the sensor width, and height assuming pixels are square
    pub fn set_sensor_width(mut self, mm_sensor_width: f64) -> Self {
        self.mm_sensor_width = mm_sensor_width;
        self.mm_sensor_height = mm_sensor_width / self.px_width * self.px_height;
        self
    }

    //cp set_sensor_height
    /// Set the sensor height, and width assuming pixels are square
    pub fn set_sensor_height(mut self, mm_sensor_height: f64) -> Self {
        self.mm_sensor_height = mm_sensor_height;
        self.mm_sensor_width = mm_sensor_height / self.px_height * self.px_width;
        self
    }

    //cp set_px_frame
    /// Set the pixel width and height, and centre to be the half-and-half
    pub fn set_px_frame(mut self, px_width: usize, px_height: usize) -> Self {
        self.px_width = px_width as f64;
        self.px_height = px_height as f64;
        self.px_centre = [self.px_width / 2.0, self.px_height / 2.0];
        self
    }

    //cp set_px_centre
    /// Set the pixel centre; invoke after set_px_frame()
    pub fn set_px_centre(mut self, px_centre: [usize; 2]) -> Self {
        self.px_centre = [px_centre[0] as f64, px_centre[1] as f64];
        self
    }

    //cp derive
    pub fn derive(mut self) -> Self {
        self.mm_single_pixel_width = self.mm_sensor_width / self.px_width;
        self.mm_single_pixel_height = self.mm_sensor_height / self.px_height;
        self.pixel_aspect_ratio = self.mm_single_pixel_width / self.mm_single_pixel_height;
        self
    }

    //ap mm_sensor_width
    pub fn mm_sensor_width(&self) -> f64 {
        self.mm_sensor_width
    }

    //ap mm_sensor_height
    pub fn mm_sensor_height(&self) -> f64 {
        self.mm_sensor_height
    }

    //ap mm_sensor_diagonal
    pub fn mm_sensor_diagonal(&self) -> f64 {
        (self.mm_sensor_height * self.mm_sensor_height
            + self.mm_sensor_width * self.mm_sensor_width)
            .sqrt()
    }

    //ap px_centre
    pub fn px_centre(&self) -> Point2D {
        self.px_centre.into()
    }

    //ap px_width
    pub fn px_width(&self) -> f64 {
        self.px_width
    }
    //ap px_height
    pub fn px_height(&self) -> f64 {
        self.px_height
    }

    //ap mm_aspect_ratio
    pub fn mm_aspect_ratio(&self) -> f64 {
        self.pixel_aspect_ratio
    }
    //zz All done
}

//ip CameraSensor for RectSensor
impl CameraSensor for RectSensor {
    //fp px_abs_xy_to_px_rel_xy
    #[inline]
    fn px_abs_xy_to_px_rel_xy(&self, xy: Point2D) -> Point2D {
        if self.flip_y {
            [xy[0] - self.px_centre[0], -xy[1] + self.px_centre[1]].into()
        } else {
            [xy[0] - self.px_centre[0], xy[1] - self.px_centre[1]].into()
        }
    }

    //fp px_rel_xy_to_px_abs_xy
    #[inline]
    fn px_rel_xy_to_px_abs_xy(&self, xy: Point2D) -> Point2D {
        if self.flip_y {
            [xy[0] + self.px_centre[0], -xy[1] + self.px_centre[1]].into()
        } else {
            [xy[0] + self.px_centre[0], xy[1] + self.px_centre[1]].into()
        }
    }

    //fp px_rel_xy_to_rd
    /// Map an actual centre-relative XY pixel in the frame of the
    /// camera to a RollDist (mmm from centre and angle)
    #[inline]
    fn px_rel_xy_to_rd(&self, xy: Point2D) -> RollDist {
        let mm_xy = [
            xy[0] * self.mm_single_pixel_width,
            xy[1] * self.mm_single_pixel_height,
        ];
        RollDist::from_mm_xy(mm_xy.into())
    }

    //fp rd_to_px_rel_xy
    /// Map a RollDist to a centre-relative XY pixel in the frame of
    /// the camera
    #[inline]
    fn rd_to_px_rel_xy(&self, rd: RollDist) -> Point2D {
        let mm_xy = rd.to_mm_xy();
        [
            mm_xy[0] / self.mm_single_pixel_width,
            mm_xy[1] / self.mm_single_pixel_height,
        ]
        .into()
    }
}
