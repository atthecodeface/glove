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

!*/

//a Imports
use serde::{Deserialize, Serialize};

use ic_base::json;
use ic_base::{Error, Result};

use crate::polynomial;
use crate::polynomial::CalcPoly;

//a Serialization
//fp serialize_lens_name
pub fn serialize_lens_name<S: serde::Serializer>(
    lens: &CameraLens,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    serializer.serialize_str(lens.name())
}

//a Constants for standard lens types
//cp LP_EQUISOLID
pub const LP_EQUISOLID: ([f64; 8], [f64; 8]) = (
    [
        -0.4950859499527951,
        -0.15013246274611447,
        0.4607994472316932,
        -1.0120615505147725,
        1.1602022431325167,
        -0.7109380326000974,
        0.22122976594255306,
        -0.02746571348689031,
    ],
    [
        1.0000020622392185,
        1.0000532319536433,
        0.7467438690364361,
        0.7625024914741516,
        -0.005959510803222656,
        4.911521911621094,
        -10.391387939453125,
        15.370040893554688,
    ],
);

//cp LP_STEREOGRAPHIC
pub const LP_STEREOGRAPHIC: ([f64; 7], [f64; 7]) = (
    [
        -0.000015352771598031723,
        -0.24938816116127782,
        0.12021242600530968,
        -0.058944843673089053,
        0.02249782230501296,
        -0.0052735659719473915,
        0.0005416791311745328,
    ],
    [
        2.455288012015444e-6,
        0.2499975276623445,
        0.06248133700864855,
        0.011589706235099584,
        0.00011813757009804249,
        -0.00009418785339221358,
        -0.0008547995239496231,
    ],
);

//cp LP_EQUIANGULAR
pub const LP_EQUIANGULAR: ([f64; 9], [f64; 9]) = (
    [
        -5.491853016792447e-6,
        -0.3331545291439397,
        0.19781916053034365,
        -0.13177851983346045,
        0.08084792597219348,
        -0.03880021348595619,
        0.012711396208032966,
        -0.0024505546898581088,
        0.00020687170035671443,
    ],
    [
        3.3091159821196925e-6,
        0.33332521450938657,
        0.13340936787426472,
        0.05325421690940857,
        0.025347352027893066,
        -0.0005129575729370117,
        0.017993569374084473,
        -0.010650575160980225,
        0.005239516496658325,
    ],
);

//cp LP_ORTHOGRAPHIC
pub const LP_ORTHOGRAPHIC: ([f64; 9], [f64; 9]) = (
    [
        -0.00010724715571086563,
        -0.4939921871846309,
        0.3082124108914286,
        -0.05700050573796034,
        -0.2811782229691744,
        0.4273285996168852,
        -0.2843161644414067,
        0.0922634624876082,
        -0.011822454980574548,
    ],
    [
        0.0007894445770944003,
        0.32264182274229825,
        7.223541863262653,
        -100.59029793739319,
        729.6587820053101,
        -2855.4288902282715,
        6187.322406768799,
        -6975.573333740234,
        3200.9576625823975,
    ],
);

//a LensPolys
//tp LensPolys
/// Polynomials that map (in some manner) sensor yaw to and from world
/// yaw for a spherical lens
///
/// The simplest polynomial mapping P(yaw) is from yaw angle to yaw
/// angle (in radians). However, the mapping has two properties: it
/// maps yaw of 0 to yaw of 0, and it is antisymmetric. i.e. P(0)=0,
/// P(-yaw)=-P(yaw).
///
/// If this is encoded as a polynomial (as it is here) this means that
/// the *even* coefficients *must* be zero.
///
///  i.e. P(x) = p1.x + p3.x^3 + p5.x^5 + ... (p even are zero)
///
/// So encoding this as a simple polynomial will waste half of the
/// coefficients
///
/// Hence this actually encodes the polynomial as Q(x), where:
///
///    P(yaw) = yaw*Q(yaw^2)
///
///    x * Q(x^2) = p1.x + p3.x^3 + p5.x^5 + ...
///
///    Q(x^2) = p1 + p3.x^2 + p5.x^4 + ...
///
///    q0 = p1, q1 = p3, q2 = p5, q3 = p7, ...
///
/// To calculate P(yaw), then, this is just yaw * Q(yaw^2)
///
/// The default *linear* polynomial is P(yaw) = yaw => Q(yaw^2)=1
///
/// Now, for most lenses P(x) is near x for most reasonable x; hence Q(yaw^2) = 1+R(yaw^2)
///
///    R(x^2) = p1-1 + p3.x^2 + p5.x^4 + ...
///
///    r0 = p1-1, r1 = p3, r2 = p5, r3 = p7, ...
///
/// The calibration could take advantage of this
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LensPolys {
    /// Function of fractional X-offset (0 center, 1 RH of sensor) to angle
    ///
    /// fractional Y-offset is px_rel_y / (px_height/2) / pixel_aspect_ratio
    stw_poly: Vec<f64>,

    /// Function of angle to fractional X-offset (0 center, 1 RH of sensor)
    wts_poly: Vec<f64>,
}

//ip Default for LensPolys
impl std::default::Default for LensPolys {
    fn default() -> Self {
        Self {
            stw_poly: vec![0.],
            wts_poly: vec![0.],
        }
    }
}

//ip Display for LensPolys
impl std::fmt::Display for LensPolys {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "wts:{:0.4?}; stw:{:0.4?}",
            self.wts_poly, self.stw_poly
        )
    }
}

//ip LensPolys
impl LensPolys {
    //cp stereographic
    pub fn stereographic() -> Self {
        let wts_poly = LP_STEREOGRAPHIC.0.to_vec();
        let stw_poly = LP_STEREOGRAPHIC.1.to_vec();
        Self::new(stw_poly, wts_poly)
    }

    //cp equisolid
    pub fn equisolid() -> Self {
        let wts_poly = LP_EQUISOLID.0.to_vec();
        let stw_poly = LP_EQUISOLID.1.to_vec();
        Self::new(stw_poly, wts_poly)
    }

    //cp equiangular
    pub fn equiangular() -> Self {
        let wts_poly = LP_EQUIANGULAR.0.to_vec();
        let stw_poly = LP_EQUIANGULAR.1.to_vec();
        Self::new(stw_poly, wts_poly)
    }

    //cp orthographic
    pub fn orthographic() -> Self {
        let wts_poly = LP_ORTHOGRAPHIC.0.to_vec();
        let stw_poly = LP_ORTHOGRAPHIC.1.to_vec();
        Self::new(stw_poly, wts_poly)
    }

    //cp new
    pub fn new(stw_poly: Vec<f64>, wts_poly: Vec<f64>) -> Self {
        Self { stw_poly, wts_poly }
    }

    //cp from_json`
    pub fn from_json(json: &str) -> Result<Self> {
        json::from_json("lens polynomials", json)
    }

    //mp to_json
    pub fn to_json(&self, pretty: bool) -> Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            Ok(serde_json::to_string(self)?)
        }
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

    //mp stw
    /// Map from sensor angle to world angle
    ///
    /// Use the fact that P(yaw) = yaw * poly(yaw^2)
    pub fn stw(&self, angle: f64) -> f64 {
        angle * self.stw_poly.calc(angle.powi(2)) + angle
    }

    //mp wts
    /// Map from world angle to sensor angle
    ///
    /// Use the fact that P(yaw) = yaw * poly(yaw^2)
    pub fn wts(&self, angle: f64) -> f64 {
        angle * self.wts_poly.calc(angle.powi(2)) + angle
    }

    //cp calibration
    /// Calculate polynomials of best-fit for a given set of sensor
    /// and world yaws
    ///
    /// This generates first a sensor-to-world mapping, and then a
    /// world-to-sensor mapping that is a good inverse of that.
    ///
    /// The initial generation sorts the sensor/world pairs according
    /// to the sensor value, and then applies a median filter to
    /// remove outliers. This filter consides 2N+1 consecutive
    /// world/sensor values (centred on sensor value S) and ignores
    /// the largest and smallest values (if N were one this would just
    /// taken the median); it uses the mean of the remaining 2N-1
    /// values as the actual world/sensor value for the sensor value
    /// S.
    ///
    /// The polynomial generated is a best fit for X values of
    /// sensor^2 and Y values of world/sensor, as required by the
    /// compressed polynomials used in the LensPoly type.
    pub fn calibration(
        poly_degree: usize,
        sensor_yaws: &[f64],
        world_yaws: &[f64],
        yaw_range_min: f64,
        yaw_range_max: f64,
    ) -> Result<Self> {
        //cb Calculate ws_yaws
        let mut ws_yaws: Vec<_> = sensor_yaws
            .iter()
            .zip(world_yaws.iter())
            .filter(|(s, _)| **s > yaw_range_min)
            .map(|(s, w)| (*w, *s))
            .collect();
        ws_yaws.sort_by(|a, b| (a.1).partial_cmp(&b.1).unwrap());

        let mean_median_ws_yaws = polynomial::filter_ws_yaws(&ws_yaws);

        let sy_gwy: Vec<_> = mean_median_ws_yaws
            .iter()
            .filter(|(_, s)| *s < yaw_range_max)
            .map(|(w, s)| {
                if *s < 0.001 {
                    (s.powi(2), 0.)
                } else {
                    (s.powi(2), (w - s) / s)
                }
            })
            .collect();

        let stw =
            polynomial::min_squares_dyn(poly_degree, sy_gwy.iter().copied()).map_err(|e| {
                Error::SelfError(
                    "failed to derive sensor-to-world polynomial".to_string(),
                    e.into(),
                )
            })?;

        let wy_gsy = sensor_yaws.iter().map(|s| {
            let w = *s * stw.calc(s.powi(2)) + *s;
            if w.abs() < 0.001 {
                (w.powi(2), 0.)
            } else {
                (w.powi(2), (*s - w) / w)
            }
        });

        let wts = polynomial::min_squares_dyn(poly_degree, wy_gsy).map_err(|e| {
            Error::SelfError(
                "failed to derive world-to-sensor polynomial".to_string(),
                e.into(),
            )
        })?;

        for max_rel_err in [0.01_f64, 0.001_f64, 0.0001_f64] {
            let last_coeff = *(stw.last().unwrap());
            let max_angle = (max_rel_err / last_coeff)
                .abs()
                .powf(0.5 / (stw.len() as f64));
            eprintln!(
                "Max usable angle for {}% angle error is stw: {}",
                max_rel_err * 100.0,
                max_angle.to_degrees()
            );
        }
        // eprintln!("{stw:?} {wts:?}");
        Ok(Self::new(stw, wts))
    }
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

    /// Polynomials defining the lens
    #[serde(flatten)]
    polys: LensPolys,
}

//ip Default for CameraLens
impl std::default::Default for CameraLens {
    fn default() -> Self {
        Self {
            name: "".into(),
            aliases: vec![],
            mm_focal_length: 20.,
            polys: LensPolys::default(),
        }
    }
}

//ip Display for CameraLens
impl std::fmt::Display for CameraLens {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
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
    pub fn set_polys(&mut self, polys: LensPolys) {
        self.polys = polys;
    }

    //ap polys
    pub fn polys(&self) -> &LensPolys {
        &self.polys
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
        self.polys = self.polys.set_stw_poly(poly);
        self
    }

    //cp set_wts_poly
    pub fn set_wts_poly(mut self, poly: &[f64]) -> Self {
        self.polys = self.polys.set_wts_poly(poly);
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

    //ap tan_sensor_to_tan_world - map tan to tan
    #[inline]
    pub fn tan_sensor_to_tan_world(&self, tan: f64) -> f64 {
        self.polys.stw(tan.atan()).tan()
    }

    //ap tan_world_to_tan_sensor - map tan to tan
    #[inline]
    pub fn tan_world_to_tan_sensor(&self, tan: f64) -> f64 {
        self.polys.wts(tan.atan()).tan()
    }

    //zz All done
}
//a Plotting
/*
    //cb stw plot
    {
        use poloto::prelude::*;
        use tagu::prelude::*;
        let theme = poloto::render::Theme::light();
        let theme = theme.append(tagu::build::raw(".poloto_scatter{stroke-width:1px;}"));
        let theme = theme.append(tagu::build::raw(
            ".poloto_text.poloto_legend{font-size:10px;}",
        ));
        let theme = theme.append(tagu::build::raw(
            ".poloto_line{stroke-dasharray:1;stroke-width:1px;}",
        ));

        let plots = poloto::build::origin();
        let plots = plots.chain(
            poloto::build::plot("Poly").line(
                (0..300)
                    .map(|n| (n as f64) / 300.0 * yaw_range_max * 1.2)
                    .map(|s| (s.to_degrees(), stw_clone.calc(s.tan()) / s.tan() - 1.0)),
            ),
        );
        let plots = plots.chain(
            poloto::build::plot("Original").scatter(
                world_yaws
                    .iter().zip(sensor_yaws.iter())
                    .map(|(w, s)| (s.to_degrees(), w / s - 1.0)),
            ),
        );
        let plot_initial = poloto::frame_build()
            .data(plots)
            .build_and_label(("Lens Cal Sensor-to-world", "Sensor", "(tan w)/(tan s) - 1"))
            .append_to(poloto::header().append(theme))
            .render_string()
            .map_err(|e| format!("{e:?}"))?;

        let mut f = std::fs::File::create("lc_stw.svg")?;
        f.write_all(plot_initial.to_string().as_bytes())?;
    }

    //cb wts plot
    {
        use poloto::prelude::*;
        use tagu::prelude::*;
        let theme = poloto::render::Theme::light();
        let theme = theme.append(tagu::build::raw(".poloto_scatter{stroke-width:1px;}"));
        let theme = theme.append(tagu::build::raw(
            ".poloto_text.poloto_legend{font-size:10px;}",
        ));
        let theme = theme.append(tagu::build::raw(
            ".poloto_line{stroke-dasharray:1;stroke-width:1px;}",
        ));

        let plots = poloto::build::origin();
        let plots = plots.chain(
            poloto::build::plot("Original").scatter(
                ws_tan_yaws
                    .iter()
                    .map(|(w, s)| (w.atan().to_degrees(), s / w - 1.0)),
            ),
        );
        let plots = plots.chain(
            poloto::build::plot("Poly source").scatter(
                world_tan_yaw
                    .iter()
                    .zip(grad_sensor_tan_yaw.iter())
                    .map(|(w, gs)| (w.atan().to_degrees(), gs)),
            ),
        );
        let plots = plots.chain(
            poloto::build::plot("Poly").line(
                (0..300)
                    .map(|n| (n as f64) / 300.0 * yaw_range_max * 1.2)
                    .map(|w| (w.to_degrees(), wts_clone.calc(w.tan()) / w.tan() - 1.0)),
            ),
        );
        let plot_initial = poloto::frame_build()
            .data(plots)
            .build_and_label(("Lens Cal World-to-sensor", "World", "(tan s)/(tan w) - 1"))
            .append_to(poloto::header().append(theme))
            .render_string()
            .map_err(|e| format!("{e:?}"))?;

        let mut f = std::fs::File::create("lc_wts.svg")?;
        f.write_all(plot_initial.to_string().as_bytes())?;
    }
*/
