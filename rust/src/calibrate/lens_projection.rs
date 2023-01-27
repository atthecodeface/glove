use super::CalcPoly;
use super::CameraProjection;
use super::RollYaw;
use super::{Point2D, Point3D, TanXTanY};

//a Trait
//tp Polynomial
/// A lens projection implemented with a polynomial mapping of radial offset to off-axis angle of the ray
///
/// Real lenses have a mapping from angle-from-centre to a
/// distance-from-center on the sensor that is notionally standard
/// (such as stereographic); however, the actual mapping is
/// lens-specific, and so this provides a polynomial mapping which
/// can be generated from taking pictures of a grid
///
/// For a stereographic lens the polynomial is offset = 2 tan(angle/2) ( * fl / sw)
///
/// For an 'equidistant' or 'equiangular' lens the polynomial is offset = angle ( * fl / sw)
///
/// For an 'equisolid' lens the polynomial is offset =  2 sin(angle/2) ( * fl / sw)
///
/// For an 'orthographic' lens the polynomial is offset =  sin(angle) ( * fl / sw)
///
/// For a rectilinear lens the polynomial is offset = tan(angle) ( * fl / sw); this keeps lines straight
#[derive(Debug, Clone)]
pub struct Polynomial {
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

    // The focal length in mm
    mm_focal_length: f64,

    /// Function of fractional X-offset (0 center, 1 RH of sensor) to angle
    ///
    /// fractional Y-offset is px_rel_y / (px_height/2) / pixel_aspect_ratio
    poly: Vec<f64>,

    /// Function of angle to fractional X-offset (0 center, 1 RH of sensor)
    inv_poly: Vec<f64>,

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

    /// Fractional X to relative pixel X scaling
    frac_x_to_px_x: f64,

    /// Fractional X to relative pixel Y scaling
    frac_x_to_px_y: f64,
}

//ip default for Polynomial
/// Sensor sizes:
///   medium format 53.7 by 40.2mm
///   medium format 43.8 by 32.9mm
///   Full frame 35mm is 36.0 by 24.0mm
///   Nikon APS-C 23.6 by 15.6mm
///   Canon APS-C 22.3 by 14.9mm (or 22.2 by 14.8)
///   Canon APS-H 28.7 by 19.0mm
impl std::default::Default for Polynomial {
    fn default() -> Self {
        (Self {
            px_centre: [200., 150.],
            px_width: 400.,
            px_height: 300.,
            flip_y: false,
            mm_sensor_width: 36.,
            mm_sensor_height: 24.,
            mm_focal_length: 20.,
            poly: vec![0., 1.],
            inv_poly: vec![0., 1.],
            pixel_aspect_ratio: 1.,
            mm_single_pixel_width: 1.,
            mm_single_pixel_height: 1.,
            frac_x_to_px_x: 0.,
            frac_x_to_px_y: 0.,
        })
        .derive()
    }
}

//ip Polynomial
impl Polynomial {
    //fp new
    pub fn new(
        mm_focal_length: f64,
        mm_sensor_width: f64,
        px_width: usize,
        px_height: usize,
    ) -> Self {
        Self::default()
            .set_px_frame(px_width, px_height)
            .set_sensor_width(mm_sensor_width)
            .set_focal_length(mm_focal_length)
            .derive()
    }

    //cp set_flip_y
    pub fn set_flip_y(mut self, flip_y: bool) -> Self {
        self.flip_y = flip_y;
        self
    }

    //cp set_focal_length
    pub fn set_focal_length(mut self, mm_focal_length: f64) -> Self {
        self.mm_focal_length = mm_focal_length;
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
    /// Set the sensor width
    pub fn set_sensor_width(mut self, mm_sensor_width: f64) -> Self {
        self.mm_sensor_width = mm_sensor_width;
        self.mm_sensor_height = mm_sensor_width / self.px_width * self.px_height;
        self
    }

    //cp set_sensor_height
    /// Set the sensor height, setting the width also
    pub fn set_sensor_height(mut self, mm_sensor_height: f64) -> Self {
        self.mm_sensor_height = mm_sensor_height;
        self.mm_sensor_width = mm_sensor_height / self.px_height * self.px_width;
        self
    }

    //cp set_px_frame
    pub fn set_px_frame(mut self, px_width: usize, px_height: usize) -> Self {
        self.px_width = px_width as f64;
        self.px_height = px_height as f64;
        self.px_centre = [self.px_width / 2.0, self.px_height / 2.0];
        self
    }

    //cp set_px_centre
    pub fn set_px_centre(mut self, px_centre: [usize; 2]) -> Self {
        self.px_centre = [px_centre[0] as f64, px_centre[1] as f64];
        self
    }

    //cp set_poly
    pub fn set_poly(mut self, poly: &[f64]) -> Self {
        self.poly = poly.iter().map(|x| *x).collect();
        self
    }

    //cp set_inv_poly
    pub fn set_inv_poly(mut self, inv_poly: &[f64]) -> Self {
        self.inv_poly = inv_poly.iter().map(|x| *x).collect();
        self
    }

    //cp derive
    pub fn derive(mut self) -> Self {
        self.mm_single_pixel_width = self.mm_sensor_width / self.px_width;
        self.mm_single_pixel_height = self.mm_sensor_height / self.px_height;
        self.pixel_aspect_ratio = self.mm_single_pixel_width / self.mm_single_pixel_height;
        self.frac_x_to_px_x = self.px_width / 2.;
        self.frac_x_to_px_y = self.px_height / 2. * self.pixel_aspect_ratio;
        self
    }

    //mp abs_px_x_to_mm
    /// Convert an absolute X pixel (0 to width) to mm offset form centre
    #[inline]
    pub fn abs_px_x_to_mm(&self, px_x: usize) -> f64 {
        self.rel_px_x_to_mm(px_x as f64 - self.px_centre[0])
    }

    //mp abs_px_y_to_mm
    /// Convert an absolute Y pixel (0 to height) to mm offset form centre
    #[inline]
    pub fn abs_px_y_to_mm(&self, px_y: usize) -> f64 {
        self.rel_px_y_to_mm(px_y as f64 - self.px_centre[1])
    }

    //mp rel_px_x_to_mm
    /// Convert an absolute X pixel (0 to width) to mm offset form centre
    #[inline]
    pub fn rel_px_x_to_mm(&self, px_x: f64) -> f64 {
        px_x * self.mm_single_pixel_width
    }

    //mp rel_px_y_to_mm
    /// Convert an absolute Y pixel (0 to height) to mm offset form centre
    #[inline]
    pub fn rel_px_y_to_mm(&self, px_y: f64) -> f64 {
        px_y * self.mm_single_pixel_height
    }
    //fp px_rel_xy_to_ry
    #[inline]
    fn px_rel_xy_to_ry(&self, xy: Point2D) -> RollYaw {
        let frac_xy_x = xy[0] / self.frac_x_to_px_x;
        let frac_xy_y = xy[1] / self.frac_x_to_px_y;
        let r = (frac_xy_x * frac_xy_x + frac_xy_y * frac_xy_y).sqrt();
        let sin_roll = frac_xy_y / r;
        let cos_roll = frac_xy_x / r;
        let yaw = self.poly.calc(r);
        // Deprecated use of from_roll_yaw
        RollYaw::from_roll_yaw(sin_roll, cos_roll, yaw)
    }

    //fp ry_to_px_rel_xy
    #[inline]
    fn ry_to_px_rel_xy(&self, ry: RollYaw) -> Point2D {
        // Deprecated use of ry.yaw()
        let x_frac = self.inv_poly.calc(ry.yaw());
        let s = ry.sin_roll();
        let c = ry.cos_roll();
        [
            x_frac * c * self.frac_x_to_px_y,
            x_frac * s * self.frac_x_to_px_y,
        ]
        .into()
    }
    //zz All done
}

//ip CameraProjection for Polynomial
impl CameraProjection for Polynomial {
    //fp px_rel_xy_to_px_abs_xy
    #[inline]
    fn px_rel_xy_to_px_abs_xy(&self, xy: Point2D) -> Point2D {
        if self.flip_y {
            [xy[0] + self.px_centre[0], -xy[1] + self.px_centre[1]].into()
        } else {
            [xy[0] + self.px_centre[0], xy[1] + self.px_centre[1]].into()
        }
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

    //fp px_rel_xy_to_txty
    /// Map an actual centre-relative XY pixel in the frame of the
    /// camera to a tan(x), tan(y)
    ///
    /// This must apply the lens projection
    ///
    /// The default functions combines other mapping functions, so may
    /// not be fully optimized
    #[inline]
    fn px_rel_xy_to_txty(&self, xy: Point2D) -> TanXTanY {
        let ry = self.px_rel_xy_to_ry(xy);
        ry.into()
    }

    //fp txty_to_px_rel_xy
    /// Map a tan(x), tan(y) (i.e. x/z, y/z) to a centre-relative XY
    /// pixel in the frame of the camera
    ///
    /// This must apply the lens projection
    ///
    /// An implementation can improve the performance for some lenses
    /// where this is a much simpler mapping than the two stages combined
    #[inline]
    fn txty_to_px_rel_xy(&self, txty: TanXTanY) -> Point2D {
        let ry: RollYaw = txty.into();
        self.ry_to_px_rel_xy(ry)
    }
}

//a Lens polynomial
/// Function of X-offset / (px_width/2) to angle
///
/// From data captured on the 20-35mm lens at 20mm on Rebel2Ti
pub const CANON_20_35_REBEL2TI_AT_20_POLY: [f64; 6] = [
    -7.9469597546248994e-05,
    0.94840242521694607,
    -0.072647540142068201,
    0.078163335684369506,
    -0.26366976024895566,
    0.064274376512159836,
];

/// Function of angle to X-offset / (px_width/2)
///
/// Inverse function
pub const CANON_20_35_REBEL2TI_AT_20_INV_POLY: [f64; 6] = [
    7.3033194404697801e-05,
    1.0573355058624287,
    0.031719783990272865,
    0.25078854816002316,
    -0.55629963398692217,
    0.87342620262394521,
];
