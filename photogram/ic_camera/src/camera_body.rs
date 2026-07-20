use serde::{Deserialize, Serialize};

use ic_base::Point2D;

use crate::CameraSensor;

/// Serialize a [CameraBody] as just its name, so a CameraBody can be reloaded
/// from a JSON file in conjunction with a camera database
pub fn serialize_body_name<S: serde::Serializer>(
    body: &CameraBody,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    serializer.serialize_str(body.name())
}

/// A rectangular camera sensor, within a camera body
///
/// Every digitial camera body contains a rectangular sensor of light sensors,
/// with filters in front of them (for color photography). On a decent camera it
/// will be aligned (perfectly, for the purposes of this) perpendicular to the
/// direction of the lens.
///
/// Sensors tend to have a 4:3 aspect ratio; they will have an integer number of
/// pixels in X and Y, in a 4:3 ratio, but the actual sensor size tends to be
/// such that the pixels are not quite square. The pixel that aligns with the
/// axis of the lens may also not be the middle pixel of the sensor (although it
/// will be close).
///
/// This structure models the camera body, with the sensor size in pixels and
/// its precise physical size; the centre pixel (i,e, the pixel that aligns with
/// the axis of the lens).
///
/// Images taken with the camera are assumed to have pixel coordinates with an
/// origin at the top left (this is really an arbitrary choice); the centre
/// pixel is in this coordinate system.
///
/// ``ignore
///  "mm_sensor_width": 22.67,// 22.65 gives 1.83; 22.66 gives 1.54,6.4; 22.67 gives 1.439, 6.1; 22.68 give 1.5, 5.8; 22.69 gives 1.76,5.6
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraBody {
    /// Name
    name: String,

    /// Aliases
    aliases: Vec<String>,

    /// Centre pixel
    px_center: [f64; 2],

    /// Width of sensor in pixels (normally an int)
    px_width: f64,

    /// Height of sensor in pixels (normally an int)
    px_height: f64,

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
            px_center: [200., 150.],
            px_width: 400.,
            px_height: 300.,
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

impl std::fmt::Display for CameraBody {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "{}: {}x{} @ {} by {}",
            self.name, self.px_width, self.px_height, self.mm_sensor_width, self.mm_sensor_height
        )
    }
}

impl CameraBody {
    /// Createa a new [CameraBody] given a sensor size, assuming pixels are
    /// perfetly recangular and the optical axis is at the centre
    pub fn new(mm_sensor_width: f64, px_width: usize, px_height: usize) -> Self {
        let mut s = Self::default()
            .set_px_frame(px_width, px_height)
            .set_sensor_width_and_height(mm_sensor_width);
        s.derive();
        s
    }

    /// Set the name of the camera body
    pub fn set_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
    }

    /// Set the sensor size
    pub fn set_sensor_size(mut self, mm_sensor_width: f64, mm_sensor_height: f64) -> Self {
        self.mm_sensor_width = mm_sensor_width;
        self.mm_sensor_height = mm_sensor_height;
        self.derive();
        self
    }

    /// Set the sensor width, and height assuming pixels are square
    pub fn set_sensor_width_and_height(mut self, mm_sensor_width: f64) -> Self {
        self.mm_sensor_width = mm_sensor_width;
        self.mm_sensor_height = mm_sensor_width / self.px_width * self.px_height;
        self.derive();
        self
    }

    /// Set the sensor height only
    pub fn set_sensor_height(mut self, mm_sensor_height: f64) -> Self {
        self.mm_sensor_height = mm_sensor_height;
        self.derive();
        self
    }

    /// Set the pixel width and height, and centre to be the half-and-half
    pub fn set_px_frame(mut self, px_width: usize, px_height: usize) -> Self {
        self.px_width = px_width as f64;
        self.px_height = px_height as f64;
        self.px_center = [self.px_width / 2.0, self.px_height / 2.0];
        self.derive();
        self
    }

    /// Set the pixel centre; invoke after set_px_frame()
    pub fn set_px_centre(mut self, px_centre: [usize; 2]) -> Self {
        self.px_center = [px_centre[0] as f64, px_centre[1] as f64];
        self
    }

    /// Derive the data dependent on px size and sensor size (pixel size in mm, pixel aspect ratio)
    pub fn derive(&mut self) {
        self.mm_single_pixel_width = self.mm_sensor_width / self.px_width;
        self.mm_single_pixel_height = self.mm_sensor_height / self.px_height;
        self.pixel_aspect_ratio = self.mm_single_pixel_width / self.mm_single_pixel_height;
    }

    /// Determine if the name or an alias matches a search name
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

    /// Get the sensor width in mm
    pub fn mm_sensor_width(&self) -> f64 {
        self.mm_sensor_width
    }

    /// Get the sensor height in mm
    pub fn mm_sensor_height(&self) -> f64 {
        self.mm_sensor_height
    }

    /// Get the sensor diagonal length in mm
    pub fn mm_sensor_diagonal(&self) -> f64 {
        (self.mm_sensor_height * self.mm_sensor_height
            + self.mm_sensor_width * self.mm_sensor_width)
            .sqrt()
    }

    /// Get the width of a single pixel in mm
    pub fn mm_single_pixel_width(&self) -> f64 {
        self.mm_single_pixel_width
    }

    /// Get the height of a single pixel in mm
    pub fn mm_single_pixel_height(&self) -> f64 {
        self.mm_single_pixel_height
    }

    /// Get the pixel centre
    pub fn px_centre(&self) -> Point2D {
        self.px_center.into()
    }

    /// Get the width of the sensor in pixels
    pub fn px_width(&self) -> f64 {
        self.px_width
    }

    /// Get the height of the sensor in pixels
    pub fn px_height(&self) -> f64 {
        self.px_height
    }

    /// Get the *physical* aspect ratio of the sensor pixels (mm wide / mm high)
    pub fn px_mm_aspect_ratio(&self) -> f64 {
        self.pixel_aspect_ratio
    }
}

impl CameraSensor for CameraBody {
    /// Get the (main) name of the camera body
    fn name(&self) -> &str {
        &self.name
    }

    /// Get the size of the sensor in pixels
    fn sensor_px_size(&self) -> (f64, f64) {
        (self.px_width, self.px_height)
    }

    /// Get the center pixel (the pixel that aligns with the optical axis of the lens)
    fn sensor_px_center(&self) -> Point2D {
        self.px_center.into()
    }

    /// Map an *absolute* pixel value to one relative to the optical axis
    ///
    /// The *relative* pixel coordinates are XY positive as up/right
    #[inline]
    fn px_abs_xy_to_px_rel_xy(&self, xy: &Point2D) -> Point2D {
        [xy[0] - self.px_center[0], -xy[1] + self.px_center[1]].into()
    }

    /// Map a pixel position *relative* to the optical axis to an absolute sensor position
    ///
    /// The *relative* pixel coordinates are XY positive as up/right
    #[inline]
    fn px_rel_xy_to_px_abs_xy(&self, xy: &Point2D) -> Point2D {
        [xy[0] + self.px_center[0], -xy[1] + self.px_center[1]].into()
    }
}
