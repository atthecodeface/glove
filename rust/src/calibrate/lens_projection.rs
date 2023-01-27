use super::CalcPoly;
use super::OldLensProjection;
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
}

//ip OldLensProjection for Polynomial
impl OldLensProjection for Polynomial {
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
    //fp px_rel_xy_to_ry
    #[inline]
    fn px_rel_xy_to_ry(&self, xy: Point2D) -> RollYaw {
        let frac_xy_x = xy[0] / self.frac_x_to_px_x;
        let frac_xy_y = xy[1] / self.frac_x_to_px_y;
        let r = (frac_xy_x * frac_xy_x + frac_xy_y * frac_xy_y).sqrt();
        let roll = frac_xy_y.atan2(frac_xy_x);
        let yaw = self.poly.calc(r);
        RollYaw { roll, yaw }
    }

    //fp ry_to_px_rel_xy
    #[inline]
    fn ry_to_px_rel_xy(&self, ry: RollYaw) -> Point2D {
        let x_frac = self.inv_poly.calc(ry.yaw);
        let s = ry.roll.sin();
        let c = ry.roll.cos();
        [
            x_frac * c * self.frac_x_to_px_y,
            x_frac * s * self.frac_x_to_px_y,
        ]
        .into()
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
//a Lens linear FOV
fn new_lens_linear_fov() -> Polynomial {
    Polynomial::default()
}

//tp Blah
#[derive(Debug, Clone, Default)]
pub struct Blah {}
impl Blah {
    fn centre_xy(&self) -> Point2D {
        [320., 240.].into()
    }
    fn screen_size(&self) -> Point2D {
        [640., 480.].into()
    }
    fn aspect_ratio(&self) -> f64 {
        640. / 480.
    }
    fn tan_fov_y(&self) -> f64 {
        -self.tan_fov_x() / self.aspect_ratio()
    }
    // tan_fov_x is the x to z ratio that makes a right-most pixel map to the camera space for the edge of the camera view
    // So it is tan of *half* the full camera FOV width
    //
    // The spec diagonal FOV is probably 55.0 degrees
    //
    // This yields an X FOV of 2.0 * atan(4/5 * tan(55.0/2.0)) = 45.2 degrees
    //
    // Hence 22.6 would seem to be the correct number
    fn tan_fov_x(&self) -> f64 {
        // With MIN_ERROR = 2.0
        // 20.9 Lowest WE 85 27.96 Camera @[-191.72,-247.43,472.45] yaw -18.19 pitch -19.85 + [-0.29,-0.34,0.89]
        // 21.9 Lowest WE 10 26.74 Camera @[-180.39,-208.51,469.58] yaw -17.10 pitch -16.33 + [-0.28,-0.28,0.92]
        // 22.9 Lowest WE 17 9.51 Camera @[-177.09,-202.00,441.55] yaw -17.65 pitch -16.40 + [-0.29,-0.28,0.91]
        // 23.9 Lowest WE 88 5.36 Camera @[-183.55,-190.09,409.24] yaw -19.55 pitch -16.17 + [-0.32,-0.28,0.91]
        // 24.9 Lowest WE 235 6.36 Camera @[-173.57,-175.53,395.57] yaw -18.95 pitch -14.92 + [-0.31,-0.26,0.91]
        // 25.9 Lowest WE 247 7.25 Camera @[-165.02,-173.48,376.42] yaw -18.66 pitch -15.36 + [-0.31,-0.26,0.91]
        // 26.9 Lowest WE 297 64.51 Camera @[-121.16,-187.45,367.38] yaw -13.56 pitch -17.81 + [-0.22,-0.31,0.93]
        // 27,6 WE 74.49 Camera @[-118.03,-134.71,404.21] yaw -11.56 pitch -9.87 + [-0.20,-0.17,0.97]
        // 28.6 WE 82.28 Camera @[-122.58,-123.63,388.92] yaw -12.39 pitch -8.26 + [-0.21,-0.14,0.97]
        // 29.1 WE 83.41 Camera @[-103.61,-132.34,374.19] yaw -10.41 pitch -10.21 + [-0.18,-0.18,0.97]
        // 29.6 WE 68.79 Camera @[-110.52,-137.75,353.28] yaw -11.92 pitch -11.80 + [-0.20,-0.20,0.96]

        // With MIN_ERROR = 0.5
        // 22.9 Lowest WE 77 4.20 Camera @[-190.81,-194.42,434.13] yaw -19.44 pitch -15.79 + [-0.32,-0.27,0.91]
        // 23.4 Lowest WE 74 6.08 Camera @[-180.90,-186.53,431.35] yaw -18.37 pitch -15.04 + [-0.30,-0.26,0.92]
        // 23.5 Lowest WE 57 11.82 Camera @[-173.70,-202.87,424.04] yaw -17.78 pitch -17.10 + [-0.29,-0.29,0.91]
        // 23.6 Lowest WE 15 12.30 Camera @[-168.33,-193.25,428.91] yaw -17.15 pitch -15.95 + [-0.28,-0.27,0.92]
        // 23.7 Lowest WE 56 5.81 Camera @[-182.86,-183.68,420.27] yaw -19.07 pitch -15.03 + [-0.32,-0.26,0.91]
        // 23.9 Lowest WE 92 4.77 Camera @[-182.12,-185.26,414.33] yaw -19.18 pitch -15.42 + [-0.32,-0.27,0.91]
        // 24.9 Lowest WE 251 16.39 Camera @[-173.77,-186.47,396.40] yaw -18.77 pitch -16.02 + [-0.31,-0.28,0.91]

        // With MIN_ERROR = 0.5, pos in out adj by 0.01
        // 22.5  Lowest WE 54 10.98 38.10 Camera @[-184.62,-210.27,440.32] yaw -18.49 pitch -17.35 + [-0.30,-0.30,0.91]
        // 22.55 Lowest WE 83 3.69 15.41 Camera @[-195.99,-196.84,442.77] yaw -19.65 pitch -15.73 + [-0.32,-0.27,0.91]
        // 22.57 Lowest WE 83 4.57 23.53 Camera @[-199.16,-201.65,435.92] yaw -20.21 pitch -16.46 + [-0.33,-0.28,0.90]
        // 22.58 Lowest WE 122 19.63 87.42 Camera @[-175.07,-223.47,436.55] yaw -17.63 pitch -18.90 + [-0.29,-0.32,0.90]
        // 22.59 Lowest WE 53 8.44 28.08 Camera @[-185.74,-209.59,439.91] yaw -18.68 pitch -17.23 + [-0.31,-0.30,0.90]
        // 22.6  Lowest WE 92 4.02 17.16 Camera @[-195.55,-199.31,439.53] yaw -19.69 pitch -16.11 + [-0.32,-0.28,0.90]
        // 22.61 Lowest WE 77 3.66 16.26 Camera @[-196.20,-200.80,435.82] yaw -19.93 pitch -16.39 + [-0.33,-0.28,0.90]
        // 22.62 Lowest WE 65 3.63 15.87 Camera @[-195.54,-198.09,439.14] yaw -19.75 pitch -15.99 + [-0.32,-0.28,0.90]
        // 22.65 Lowest WE 124 4.91 19.63 Camera @[-195.62,-205.03,432.49] yaw -19.98 pitch -16.94 + [-0.33,-0.29,0.90]
        // 23.3  Lowest WE 152 4.92 23.04 Camera @[-190.61,-193.61,421.99] yaw -19.82 pitch -16.09 + [-0.33,-0.28,0.90]
        // 23.35 Lowest WE 117 5.39 24.82 Camera @[-189.70,-198.25,418.95] yaw -19.87 pitch -16.67 + [-0.33,-0.29,0.90]
        // 23.4  Lowest WE 120 4.35 22.65 Camera @[-190.19,-194.27,417.75] yaw -19.97 pitch -16.29 + [-0.33,-0.28,0.90]
        // 23.45 Lowest WE 100 4.62 19.28 Camera @[-187.44,-188.69,423.31] yaw -19.44 pitch -15.50 + [-0.32,-0.27,0.91]
        // 23.5  Lowest WE 84 3.96 20.11 Camera @[-187.12,-186.96,421.74] yaw -19.47 pitch -15.38 + [-0.32,-0.27,0.91]
        // 23.51 Lowest WE 102 5.50 26.89 Camera @[-186.52,-198.07,413.55] yaw -19.69 pitch -16.90 + [-0.32,-0.29,0.90]
        // 23.52 Lowest WE 81 4.80 19.55 Camera @[-187.01,-186.60,421.22] yaw -19.48 pitch -15.32 + [-0.32,-0.26,0.91]
        // 23.53 Lowest WE 69 4.62 19.32 Camera @[-185.16,-185.59,423.40] yaw -19.20 pitch -15.15 + [-0.32,-0.26,0.91]
        // 23.54 Lowest WE 101 6.20 29.70 Camera @[-185.01,-195.70,414.41] yaw -19.48 pitch -16.67 + [-0.32,-0.29,0.90]
        // 23.55 Lowest WE 145 4.68 23.16 Camera @[-188.06,-188.81,418.34] yaw -19.68 pitch -15.69 + [-0.32,-0.27,0.91]
        // 23.6  Lowest WE 104 4.55 22.78 Camera @[-187.47,-188.88,417.79] yaw -19.63 pitch -15.69 + [-0.32,-0.27,0.91]
        // 23.65 Lowest WE 66 4.25 21.91 Camera @[-185.98,-185.40,418.98] yaw -19.42 pitch -15.28 + [-0.32,-0.26,0.91]
        // 23.7  Lowest WE 87 4.73 22.97 Camera @[-186.76,-185.65,415.75] yaw -19.65 pitch -15.40 + [-0.32,-0.27,0.91]
        // 23.75 Lowest WE 41 11.28 54.03 Camera @[-169.76,-202.04,419.17] yaw -17.58 pitch -17.20 + [-0.29,-0.30,0.91]
        // 23.8  Lowest WE 113 4.95 Camera @[-183.80,-184.33,415.98] yaw -19.32 pitch -15.27 + [-0.32,-0.26,0.91]
        // 23.85 Lowest WE 133 5.57 Camera @[-185.01,-194.11,408.40] yaw -19.75 pitch -16.60 + [-0.32,-0.29,0.90]
        // 23.95 Lowest WE 96 5.05 Camera @[-183.71,-183.40,412.44] yaw -19.42 pitch -15.30 + [-0.32,-0.26,0.91]
        // 24.05 Lowest WE 246 6.71 Camera @[-185.75,-179.94,408.13] yaw -19.85 pitch -15.00 + [-0.33,-0.26,0.91]
        // 27.05 Lowest WE 251 9.70 Camera @[-157.94,-162.90,358.31] yaw -18.47 pitch -14.75 + [-0.31,-0.25,0.92]
        // 28.05 Lowest WE 175 12.26 Camera @[-147.95,-157.90,346.69] yaw -17.63 pitch -14.44 + [-0.29,-0.25,0.92]
        // 29.05 Lowest WE 213 14.73 66.67 Camera @[-142.12,-150.93,332.64] yaw -17.39 pitch -14.02 + [-0.29,-0.24,0.93]

        // With new rotation adjustment to 'worst case of all' by spinnning aroud all of them one by one
        // 22.57 WE 5.18 Camera @[-195.94,-203.95,434.83] yaw -20.00 pitch -16.72 + [-0.33,-0.29,0.90]
        // 22.58 Lowest WE 2 4.77 18.39 Camera @[-195.95,-203.95,434.86] yaw -19.97 pitch -16.73 + [-0.33,-0.29,0.90]
        // 22.59 Lowest WE 1 4.65 17.13 Camera @[-195.98,-203.99,434.95] yaw -19.96 pitch -16.74 + [-0.33,-0.29,0.90]
        // 22.6  Lowest WE 1 4.55 20.35 Camera @[-196.02,-204.02,435.05] yaw -19.91 pitch -16.76 + [-0.33,-0.29,0.90]
        // 22.61 Lowest WE 11 4.57 20.24 Camera @[-195.94,-203.95,434.85] yaw -19.91 pitch -16.76 + [-0.33,-0.29,0.90]
        // 22.62 Lowest WE 1 4.92 20.56 Camera @[-196.02,-204.01,435.04] yaw -19.97 pitch -16.74 + [-0.33,-0.29,0.90]
        // 22.63 Lowest WE 3 4.87 19.99 Camera @[-195.96,-203.97,434.89] yaw -19.93 pitch -16.74 + [-0.33,-0.29,0.90]
        // 22.64 Lowest WE 2 4.97 21.15 Camera @[-195.95,-203.96,434.86] yaw -19.91 pitch -16.74 + [-0.33,-0.29,0.90]
        // 22.65 Lowest WE 3 5.16 20.74 Camera @[-195.98,-203.98,434.94] yaw -19.92 pitch -16.73 + [-0.33,-0.29,0.90]

        // CDATA 1
        // 22.62 Lowest WE 6 12.06 73.79 Camera @[54.87,-29.79,781.31] yaw 3.00 pitch -6.00 + [0.05,-0.10,0.99]
        // 22.7  Lowest WE 5 11.76 75.93 Camera @[54.27,-29.57,777.99] yaw 2.96 pitch -6.00 + [0.05,-0.10,0.99]
        22.62_f64.to_radians().tan()
    }
}

//ip OldLensProjection for Blah
impl OldLensProjection for Blah {
    fn px_rel_xy_to_px_abs_xy(&self, xy: Point2D) -> Point2D {
        xy + self.centre_xy()
    }

    /// Map from absolute to centre-relative pixel
    fn px_abs_xy_to_px_rel_xy(&self, xy: Point2D) -> Point2D {
        xy - self.centre_xy()
    }

    /// Map an actual centre-relative XY pixel in the frame of the
    /// camera to a Roll/Yaw
    fn px_rel_xy_to_ry(&self, xy: Point2D) -> RollYaw {
        let txty = self.px_rel_xy_to_txty(xy);
        let r = (txty[0] * txty[0] + txty[1] * txty[1]).sqrt();
        let roll = txty[1].atan2(txty[0]);
        let yaw = r.atan();
        RollYaw { roll, yaw }
    }

    /// Map a Roll/Yaw to a centre-relative XY pixel in the frame of
    /// the camera
    fn ry_to_px_rel_xy(&self, ry: RollYaw) -> Point2D {
        let offset = ry.yaw.tan();
        let s = ry.roll.sin();
        let c = ry.roll.cos();
        let txty = [offset * c, offset * s].into();
        self.txty_to_px_rel_xy(txty)
    }

    /// Map an actual centre-relative XY pixel in the frame of the
    /// camera to a Roll/Yaw
    fn px_rel_xy_to_txty(&self, xy: Point2D) -> TanXTanY {
        let wh = self.screen_size();
        [
            xy[0] / (wh[0] / 2.0) * self.tan_fov_x(),
            xy[1] / (wh[1] / 2.0) * self.tan_fov_y(),
        ]
        .into()
    }

    /// Map a tan(x), tan(y) (i.e. x/z, y/z) to a centre-relative XY
    /// pixel in the frame of the camera
    ///
    /// This must apply the lens projection
    fn txty_to_px_rel_xy(&self, txty: TanXTanY) -> Point2D {
        let wh = self.screen_size();
        [
            txty[0] * wh[0] / 2.0 / self.tan_fov_x(),
            txty[1] * wh[1] / 2.0 / self.tan_fov_y(),
        ]
        .into()
    }
}
