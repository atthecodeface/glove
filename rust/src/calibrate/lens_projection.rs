//tp RollYaw
/// To 'look at' a point in a framed image, roll around the -ve 'z'
/// axis (in/out of the frame), putting the point on the +ve X axis,
/// then yaw (rotate about +ve Y)
#[derive(Debug, Clone, Copy)]
pub struct RollYaw {
    roll: f64,
    yaw: f64,
}

//tt CalcPoly
pub trait CalcPoly {
    fn calc(&self, x: f64) -> f64;
}

//ip CalcPoly for &[f64]
impl CalcPoly for &[f64] {
    fn calc(&self, mut x: f64) {
        let mut r = 0.;
        let mut xn = 1.0;
        for p in self.iter() {
            r += p * xn;
            xn *= x;
        }
        r
    }
}

//tt LensProjection
pub trait LensProjection {
    /// Map a Roll/Yaw to an actual XY in the frame of the camera
    fn ry_to_xy(&self, ry: RollYaw) -> [f64; 2];

    /// Map an actual XY in the frame of the camera to a Roll/Yaw
    fn xy_to_ry(&self, xy: [f64; 2]) -> RollYaw;
}

//ip Polynomial
#[derive(Default, Clone)]
pub struct Polynomial {
    centre: [f64; 2],
    px_width: f64,
    px_height: f64,
    px_eccentricity: f64,
    // The width of the sensor in mm
    //
    // Assuming that the camera changes focal length purely by moving the lens
    // away from the sensor, the offset onto the sensor will scale purely by
    // mm_focal_length / mm_sensor_width
    mm_sensor_width: f64,
    // The focal length in mm
    mm_focal_length: f64,
    // Real lenses have a mapping from angle-from-centre to a
    // distance-from-center on the sensor that is notionally standard
    // (such as stereographic); however, the actual mapping is
    // lens-specific, and so this provides a polynomial mapping which
    // can be generated from taking pictures of a grid
    //
    // For a stereographic lens the polynomial is offset = 2 tan(angle/2) ( * fl / sw)
    //
    // For an 'equidistant' or 'equiangular' lens the polynomial is offset = angle ( * fl / sw)
    //
    // For an 'equisolid' lens the polynomial is offset =  2 sin(angle/2) ( * fl / sw)
    //
    // For an 'orthographic' lens the polynomial is offset =  sin(angle) ( * fl / sw)
    //
    // For a rectilinear lens the polynomial is offset = tan(angle) ( * fl / sw); this keeps lines straight
    poly: Vec<f64>,
    inv_poly: Vec<f64>,
}

impl Polynomial {
    fn new(mm_focal_length: f64, mm_sensor_width: f64, width: f64, height: f64) -> Self {
        Self::default()
            .set_sensor_width(mm_sensor_width)
            .set_focal_length(mm_focal_length)
            .set_frame(width, height, 1.)
            .set_poly(&[0., 1.])
            .set_inv_poly(&[0., 1.])
    }
    fn set_focal_length(mut self, mm_focal_length: f64) -> Self {
        self.mm_focal_length = mm_focal_length;
        self
    }
    fn set_sensor_width(mut self, mm_sensor_width: f64) -> Self {
        self.mm_sensor_width = mm_sensor_width;
        self
    }
    fn set_frame(mut self, px_width: f64, px_height: f64, px_eccentricity: f64) -> Self {
        self.px_width = px_width;
        self.px_height = px_height;
        self.px_eccentricity = px_eccentricity;
        self.centre = [px_width / 2.0, px_height / 2.0];
        self
    }
    fn set_eccentricity(mut self, px_eccentricity: f64) -> Self {
        self.px_eccentricity = px_eccentricity;
        self
    }
    fn set_poly(mut self, poly: &[f64]) -> Self {
        self.poly = poly.iter.collect();
        self
    }
    fn set_inv_poly(mut self, inv_poly: &[f64]) -> Self {
        self.inv_poly = poly.iter.collect();
        self
    }
    /// Map an off-axis angle in radians to an offset in pixels from
    /// the center in the X axis
    ///
    /// If the pixels or not quite square then the Y offset will be
    /// slightly different
    fn angle_to_offset(&self, angle: f64) -> f64 {
        self.inv_poly.calc(angle) // *focal_length/frame_width;
    }

    /// Map an offset in pixels from
    /// the center in the X axis to an off-axis angle in radians
    fn offset_to_angle(&self, offset: f64) -> f64 {
        offset
    }
}

impl LensProjection for Polynomial {
    fn ry_to_xy(&self, ry: RollYaw) -> [f64; 2];
    fn xy_to_ry(&self, xy: [f64; 2]) -> RollYaw;
}

//a Lens polynomial
/// Function of X-offset / (px_width/2) to angle
///
/// From data captured on the 20-35mm lens at 20mm on Rebel2Ti
const CANON_20_35_REBEL2TI_AT_20_POLY: [f64; 5] = [
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
const CANON_20_35_REBEL2TI_AT_20_INV_POLY: [f64; 5] = [
    7.3033194404697801e-05,
    1.0573355058624287,
    0.031719783990272865,
    0.25078854816002316,
    -0.55629963398692217,
    0.87342620262394521,
];
