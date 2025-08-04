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
use ic_base::Result;

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
pub const LP_EQUISOLID: ([f64; 7], [f64; 7]) = (
    [
        0.0,
        0.4997494436465786,
        0.0020654588006436825,
        -0.06921000644797459,
        0.009734337567351758,
        0.0066660778247751296,
        -0.00195744882512372,
    ],
    [
        0.0,
        1.987718482123455,
        0.24317235918715596,
        -0.9263885319232941,
        7.228096961975098,
        -12.305673584342003,
        9.575857356190681,
    ],
);

pub const LP_STEREOGRAPHIC: ([f64; 7], [f64; 7]) = (
    [
        0.0,
        0.9963953544211108,
        0.03322175214998424,
        -0.3751326131168753,
        0.22490806435234845,
        -0.05375197483226657,
        0.0039019385731080547,
    ],
    [
        0.0,
        1.001861843658844,
        -0.01971894665621221,
        0.33221330866217613,
        -0.15700633265078068,
        0.19460456538945436,
        -0.028680953895673156,
    ],
);

pub const LP_EQUIANGULAR: ([f64; 9], [f64; 9]) = (
    [
        0.0,
        0.9998419532785192,
        0.0013162735849618912,
        -0.33344507962465286,
        -0.03441770374774933,
        0.3506765365600586,
        -0.28652286529541016,
        0.1022481769323349,
        -0.014303863048553467,
    ],
    [
        0.0,
        0.9982260325923562,
        0.03605024516582489,
        0.02770853042602539,
        1.3280048370361328,
        -3.0907912254333496,
        4.449044227600098,
        -3.244915008544922,
        1.0540469884872437,
    ],
);

pub const LP_ORTHOGRAPHIC: ([f64; 7], [f64; 7]) = (
    [
        0.0,
        0.9941659067408182,
        0.06303225585725158,
        -0.7806377926608548,
        0.6024768308270723,
        -0.20585763768758625,
        0.026198503939667717,
    ],
    [
        0.0,
        0.3191991178318858,
        9.813045187387615,
        -55.521155001595616,
        149.31334675848484,
        -186.43467409163713,
        89.10336443781853,
    ],
);
//a LensPolys
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
            stw_poly: vec![0., 1.],
            wts_poly: vec![0., 1.],
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
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
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

    //ap stw_poly
    pub fn stw_poly(&self) -> &[f64] {
        &self.stw_poly
    }

    //ap wts_poly
    pub fn wts_poly(&self) -> &[f64] {
        &self.wts_poly
    }

    //cp calibration
    pub fn calibration(
        poly_degree: usize,
        sensor_yaws: &[f64],
        world_yaws: &[f64],
        yaw_range_min: f64,
        yaw_range_max: f64,
    ) -> Self {
        //cb Calculate ws_yaws
        let mut ws_yaws: Vec<_> = sensor_yaws
            .iter()
            .zip(world_yaws.iter())
            .filter(|(s, _)| **s > yaw_range_min)
            .map(|(s, w)| (*w, *s))
            .collect();
        ws_yaws.sort_by(|a, b| (a.1).partial_cmp(&b.1).unwrap());

        let mean_median_ws_yaws = polynomial::filter_ws_yaws(&ws_yaws);

        let mut sy_gwy: Vec<_> = mean_median_ws_yaws
            .iter()
            .filter(|(_, s)| *s < yaw_range_max)
            .map(|(w, s)| if *s < 0.001 { (*s, 1.) } else { (*s, w / s) })
            .collect();

        let mut stw;
        loop {
            let n = sy_gwy.len();
            stw = polynomial::min_squares_dyn(
                poly_degree - 1, // note - will multiply the polynomial by 'x'
                sy_gwy.iter().copied(),
            );

            break;
            let (max_sq_err, max_n, mean_err, mean_sq_err, variance_err) =
                polynomial::error_in_y_stats(&stw, sy_gwy.iter().copied());
            let sd_err = variance_err.sqrt();
            let max_err = max_sq_err.sqrt();

            let dmin = sd_err * 3.0;
            let dmax = sd_err * 3.0;
            let outliers = polynomial::find_outliers(&stw, sy_gwy.iter().copied(), dmin, dmax);
            eprintln!(" {n} removing {} err: mean {mean_err:.4e} mean_sq {mean_sq_err:.4e} sd {sd_err:.4e} abs max {max_err:.4e} max_n {max_n}", outliers.len());
            // break;
            if outliers.is_empty() {
                break;
            }
            for n in outliers.iter().rev() {
                sy_gwy.remove(*n);
            }
        }

        // Convert p(s) to (p(s)+1) * s
        stw.insert(0, 0.0);

        let wy_gsy = sensor_yaws.iter().map(|s| {
            let w = stw.calc(*s);
            if w.abs() < 0.001 {
                (w, 1.)
            } else {
                (w, *s / w)
            }
        });

        let mut wts = polynomial::min_squares_dyn(
            poly_degree - 1, // note - will multiply the polynomial by 'x'
            wy_gsy,
        );
        wts.insert(0, 0.0);

        eprintln!("{stw:?} {wts:?}");
        Self::new(stw, wts)
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

    //ap sensor_to_world - map tan to tan
    #[inline]
    pub fn sensor_to_world(&self, tan: f64) -> f64 {
        self.polys.stw_poly.calc(tan.atan()).tan()
    }

    //ap world_to_sensor - map tan to tan
    #[inline]
    pub fn world_to_sensor(&self, tan: f64) -> f64 {
        self.polys.wts_poly.calc(tan.atan()).tan()
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
