//a Documentation
/*!

 Lens polynomial
 Notes Jan 2023

 On the Canon 50mm lens on the 5d mark IV (2160 == 4v3a6028). The 5D mark IV has a 35mm sensor 6720x4480

 Calibration images (2157 - 2160) taken at a distance of roughly 43cm to focal plane from object

 The four photos taken use focus on 4 steps from close up (minimum focus distance about 40cm) to infinity
 Hence the lens-to-frame distance is (see below) 57.5mm for 2157 to 50.0mm for 2160

 The test photos of noughts and crosses (2161-2163, 2164-2166) are
 3 images focused at infinity and 3 focused as close up as possible
 (presumably at 40cm)

 Minimum focusing is 450mm!

 Note 1/f = 1/u + 1/v or u = 1/(1/f - 1/v) = fv / v-f

 Infinity is lens at 50mm from sensor; minimum focusing distance
 (object to focal plane) is about 44cm = 440mm; that is u+v

 f = uv / u+v => uv = f*(u+v) = 50*440 = u*(440-u) =>
   u^2 - 440u + 50*440 = 0 -> u=1/2(440-sqrt(440^2-4*50*440))
   u = 57.5mm, v=402.5mm

 [ it seems from the Rust program that u is probably 57.212 for min focus, which means
   50 = 57.212*v / (57.212 + v) => 50v + 50*57.212 = 57.212*v => v = 396.644,
   hence u+v = 453mm and not 440mm. ]

 [ If it is 440mm and u=57.212 then v=382.788 and f = 57.212*382.788 / 440 = 49.77mm ]
 [ If it is 445mm and u=57.212 then v=387.788 and f = 57.212*387.788 / 445 = 49.85mm ]
 [ If it is 450mm and u=57.212 then v=392.788 and f = 57.212*387.788 / 450 = 49.93mm ]

 If the object is K mm away then on from the sensor the sensor image size is propotional to u/K-u,
 and for two different sensor to lens distances u1 and u2 there is a relative scaling of

 u1/u2 * (K-u2)/(K-u1)

 Function of X-offset / (px_width/2) to angle

 For 2160 (aka 4v3a6028) - focus at infinity, lens 50mm from fame -
 the lines (every centimeter in the source) are at X offsets of
 (with +-5 error):

 246, 457, 675, 892, 1110, 1331, 1554, 1777, 2003, 2229, 2456, 2680, 2910, 3140, 3368, 3597, 3825, 4057, 4287, 4513, 4745, 4973, 5202, 5430, 5660, 5884, 6111, 6336, 6560

 The lines (every centimeter in the source) in 2156 (slightly better than 2157) (aka 4v3a604) are at X offsets of (with +-5 error) (note should be zoom by about 400/350):

 245, 495, 750, 1007, 1264, 1523, 1785, 2049, 2312, 2577, 2844, 3111, 3378, 3645, 3914, 4181, 4449, 4716, 4985, 5250, 5516, 5781, 6045, 6306, 6567,

 At the centre (about 3360) the 1cm separation is 228 for '2160' and 267 for '2156', or a scaling of 1.17

 If the object were K=450mm from the focus plane then u1/u2 * (K-u2)/(K-u1), with u1=50mm, u2=57.5mm,
 yields 1.1719

 The separation on the source is 10mmm; this should map to
 10*u/(K-u)mm on the sensor, which is 36mm wide for 6720 pixels
 (186.6px/mm), so it should be 10*50/(450-50)*186.6 = 233.25, and
 10*57.5/(450-57.5)=273.36. These are somewhat larger than what was
 captured; that indicates an actual distance of K=460mm; this still
 works: 10.0*50/(460-50)*186.6 = 227.6 and 10.0*57.5/(460-57.5)*186.6=266.6

 Assuming the center of the source is then indeed 460mm from the
 camera sensor, then we have to account for the source not being
 perfectly parallel to the plane of the camera - it was flat
 though, so the difference between the left and right in terms of
 distance from the camera should be a linear mapping from amount
 off-center. Hence distance from camera to source = 460+c*distance.

 For the far left we have 13 to 14 cm from the centre as (457-246)=211px;
 on the right we have (6560-6336)=224px. Assuming the *same* lens mapping for both
 we know that px is proportional to mm, hence:

 57.5/(460+cd-57.5) is proportional to 224, and 57.5/(460-cd-57.5) is proportionaal to 211,

 hence (460+cd-57.5)/(460-cd-57.5) = 211/224
 (402.5+cd)*224 = 211*(402.5-cd)
  402.5*224 + 224cd = 211*402.5 - 211cd
  402.5*(224-211) = -(211 + 224)cd
  cd = -402.5*(224-211) / (224+211)
  cd = -12
 Now d = 135mm (approx) hence c = -0.089mm / mm

 as a check, at 65mm to the left we have a K of 460+-0.089*65 = 454.2mm to 465.8mm

 A 10mm separation will therefore be expected to have a ratio of:

 (454.2-57.5)/(465.8-57.5) = 0.972; we actually see 2003-1777=226 on the left and 4973-4745=228 on the right.
 We would expect to see 226 and 232.6 or 221.5 and 228 (basically a difference of about 6.5 pixels, not 2).

 That is what we get for 7-8mm (223 on left, 229 on right).

 Maybe the best approach is to try various values of cd in a best fit lens projection for the points,
 where we determine that, relative to the lens, we have:

 z = 460 - distance*sin(angle of door); x = point num * 10mm * cos(angle of door) (possibly +0.3mm offset);
 distance from lens to frame = 57.5

 The door looks to be at an angle of 1.88 degrees


//tt SphericalLensProjection
/// The concept is that there are absolute pixel positions within a
/// sensor, which can be converted to relative, which can be converted
/// to a tan(x)/tan(y) - in world space X/Z and Y/Z, which can be
/// mapped from (but not really to) xyz
///
/// The projection of the lens maps a world-angle to a sensor-angle,
/// and vice-versa
///
/// A particular lens may be focused on infinity, or closer; the
/// closer the focus, the larger the image on the sensor (as the lens
/// is further from the sensor). To allow for this a client requires
/// the knowledge of the focal length of the lens; the projection
/// mapping is not impacted by moving the lens, of course.
pub trait SphericalLensProjection: std::fmt::Debug {
    fn mm_focal_length(&self) -> f64;
    fn sensor_to_world(&self, tan: f64) -> f64;
    fn world_to_sensor(&self, tan: f64) -> f64;
}

!*/

//a Imports
use serde::{Deserialize, Serialize};

use crate::polynomial::CalcPoly;

//a Serialization
//fp serialize_lens_name
pub fn serialize_lens_name<S: serde::Serializer>(
    lens: &CameraLens,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(lens.name())
}

//a CameraLens
//tp CameraLens
/// A lens projection implemented with a polynomial mapping of
/// tan(incoming angle) to tan(outgoing angle) of the ray
///
/// Real lenses have a mapping from angle-from-centre to a
/// distance-from-center on the sensor that is notionally standard
/// (such as stereographic); however, the actual mapping is
/// lens-specific, and so this provides a polynomial mapping which
/// can be generated from taking pictures of a grid
///
/// For a rectilinear lens the polynomial is tan(output) = tan(angle);
/// this keeps lines straight, and is the standard 3D computer
/// projection
///
/// For a stereographic lens the polynomial is tan(output) = 2 tan(angle/2)
///
/// For an 'equidistant' or 'equiangular' lens the polynomial is tan(output) = angle
///
/// For an 'equisolid' lens the polynomial is tan(output) =  2 sin(angle/2)
///
/// For an 'orthographic' lens the polynomial is tan(output) =  sin(angle)
///
/// If a lens is rectilinear with a field-of-view of N degrees for
/// a sensor that is S mm for that angle, then the N/2 degrees corresponds
/// to S/2 mm on the sensor, and hence tan(N/2) = S/2 / mm_focal_length
///
/// Hence mm_focal_length =  S / (2tan(N/2)) = 2.1515mm
///
/// e.g. for N=55 degrees, S=2.24mm we have mm_focal_length =
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraLens {
    /// Name
    name: String,

    /// Aliases
    aliases: Vec<String>,

    /// Focal length of the lens
    mm_focal_length: f64,

    /// Function of fractional X-offset (0 center, 1 RH of sensor) to angle
    ///
    /// fractional Y-offset is px_rel_y / (px_height/2) / pixel_aspect_ratio
    stw_poly: Vec<f64>,

    /// Function of angle to fractional X-offset (0 center, 1 RH of sensor)
    wts_poly: Vec<f64>,
}

//ip Default for CameraLens
impl std::default::Default for CameraLens {
    fn default() -> Self {
        Self {
            name: String::new(),
            aliases: Vec::new(),
            mm_focal_length: 20.,
            stw_poly: vec![0., 1.],
            wts_poly: vec![0., 1.],
        }
    }
}

//ip Display for CameraLens
impl std::fmt::Display for CameraLens {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}: {}mm", self.name, self.mm_focal_length)
    }
}

//ip CameraLens
impl CameraLens {
    //fp new
    pub fn new(name: &str, mm_focal_length: f64) -> Self {
        Self::default()
            .set_name(name)
            .set_focal_length(mm_focal_length)
    }

    //mp set_polys
    pub fn set_polys(&mut self, stw_poly: Vec<f64>, wts_poly: Vec<f64>) {
        self.stw_poly = stw_poly;
        self.wts_poly = wts_poly;
    }

    //cp set_name
    pub fn set_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self
    }

    //cp set_focal_length
    pub fn set_focal_length(mut self, mm_focal_length: f64) -> Self {
        self.mm_focal_length = mm_focal_length;
        self
    }

    //cp set_stw_poly
    pub fn set_stw_poly(mut self, poly: &[f64]) -> Self {
        self.stw_poly = poly.to_vec();
        self
    }

    //cp set_wts_poly
    pub fn set_wts_poly(mut self, poly: &[f64]) -> Self {
        self.wts_poly = poly.to_vec();
        self
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

    //ap name
    pub fn name(&self) -> &str {
        &self.name
    }

    //ap mm_focal_length
    #[inline]
    pub fn mm_focal_length(&self) -> f64 {
        self.mm_focal_length
    }

    //ap sensor_to_world
    #[inline]
    pub fn sensor_to_world(&self, tan: f64) -> f64 {
        self.stw_poly.calc(tan)
    }

    //ap world_to_sensor
    #[inline]
    pub fn world_to_sensor(&self, tan: f64) -> f64 {
        self.wts_poly.calc(tan)
    }
    //zz All done
}
