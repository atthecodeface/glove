//a Imports
use serde::{Deserialize, Serialize};

use ic_base::Point2D;

use crate::CameraSensor;

//a Serialization
//fp serialize_body_name
pub fn serialize_body_name<S: serde::Serializer>(
    body: &CameraBody,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    serializer.serialize_str(body.name())
}

//a CameraBody
//tp CameraBody
/// A rectangular camera sensor
////// This provides an implementation of [CameraSensor], which allows mapping from a known point on an image (captured by the sensor) to relative positions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraBody {
    /// Name
    name: String,

    /// Aliases
    aliases: Vec<String>,

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
    #[serde(skip)]
    mm_single_pixel_width: f64,

    /// Derived width of a sensor pixel in mm
    ///
    /// mm_sensor_height / px_height
    #[serde(skip)]
    mm_single_pixel_height: f64,

    /// Derived non-squareness of sensor pixel - X to Y ratio
    ///
    /// = mm_single_pixel_width / mm_single_pixel_height
    #[serde(skip)]
    pixel_aspect_ratio: f64,
}

//ip Default for CameraBody
/// Sensor sizes:
///   medium format 53.7 by 40.2mm
///   medium format 43.8 by 32.9mm
///   Full frame 35mm is 36.0 by 24.0mm
///   Nikon APS-C 23.6 by 15.6mm
///   Canon APS-C 22.3 by 14.9mm (or 22.2 by 14.8)
///   Canon APS-H 28.7 by 19.0mm
///   Logitech C270 is 3.58 by 2.02mm (1280 x 720 @ 2.8umsq)
impl std::default::Default for CameraBody {
    fn default() -> Self {
        let mut s = Self {
            name: "CameraBody".into(),
            aliases: Vec::new(),
            px_centre: [200., 150.],
            px_width: 400.,
            px_height: 300.,
            flip_y: false,
            mm_sensor_width: 36.,
            mm_sensor_height: 24.,
            pixel_aspect_ratio: 1.,
            mm_single_pixel_width: 1.,
            mm_single_pixel_height: 1.,
        };
        s.derive();
        s
    }
}

//ip Display for CameraBody
impl std::fmt::Display for CameraBody {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "{}: {}x{} @ {} by {}",
            self.name, self.px_width, self.px_height, self.mm_sensor_width, self.mm_sensor_height
        )
    }
}

//ip CameraBody
impl CameraBody {
    //fp new
    pub fn new(mm_sensor_width: f64, px_width: usize, px_height: usize) -> Self {
        let mut s = Self::default()
            .set_px_frame(px_width, px_height)
            .set_sensor_width(mm_sensor_width);
        s.derive();
        s
    }

    //fp new_35mm
    pub fn new_35mm(px_width: usize, px_height: usize) -> Self {
        let mut s = Self::new(36.0, px_width, px_height)
            .set_name("35mm body")
            .set_sensor_size(36.0, 24.0)
            .set_name("35mm body")
            .set_sensor_size(36.0, 24.0)
            .set_flip_y(true);

        s.derive();
        s
    }

    //fp new_logitech_c270_640
    pub fn new_logitech_c270_640() -> Self {
        // diag fov 55.03
        let mut s = Self::new(3.58, 640, 480) // ignore first arg
            .set_name("Logitech C270 @ 640x480")
            .set_sensor_size(640.0 * 2.8, 480.0 * 2.8) // 2.8umsq pixels
            .set_flip_y(true);

        s.derive();
        s
    }

    //cp set_name
    pub fn set_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
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

    //mp derive
    pub fn derive(&mut self) {
        self.mm_single_pixel_width = self.mm_sensor_width / self.px_width;
        self.mm_single_pixel_height = self.mm_sensor_height / self.px_height;
        self.pixel_aspect_ratio = self.mm_single_pixel_width / self.mm_single_pixel_height;
    }

    //mp has_name
    pub fn has_name(&self, name: &str) -> bool {
        if name == self.name {
            true
        } else {
            for a in self.aliases.iter() {
                if name == a {
                    return true;
                }
            }
            false
        }
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

    //ap mm_single_pixel_width
    pub fn mm_single_pixel_width(&self) -> f64 {
        self.mm_single_pixel_width
    }

    //ap mm_single_pixel_height
    pub fn mm_single_pixel_height(&self) -> f64 {
        self.mm_single_pixel_height
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

//ip CameraSensor for CameraBody
impl CameraSensor for CameraBody {
    //fp name
    fn name(&self) -> &str {
        &self.name
    }

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
}
