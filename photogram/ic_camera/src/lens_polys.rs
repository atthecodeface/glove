use serde::{Deserialize, Serialize};

use ic_base::{JsonParsable, PiecewiseBezier, Result};

use crate::polynomial;

/// Set NaN to be NAN, as Nan is output by Rust in debug for f64 NAN...
#[allow(non_upper_case_globals)]
const NaN: f64 = f64::NAN;

const LP_EQUIDISTANT_WTS: &'static [f64] = &[
    NaN,
    0.3809181092477624,
    1.0,
    2.0,
    0.0,
    0.3338622514162089,
    0.6607020605193412,
    0.7178726975245269,
    NaN,
    0.6666066911835843,
    1.0,
    2.0,
    -0.011770661352866085,
    0.3529683758694251,
    0.6271689158861165,
    0.7805003253260381,
    -0.0428995078513319,
    0.3686032498159363,
    0.618674566935448,
    0.7854070589009521,
];

const LP_EQUIDISTANT_STW: &'static [f64] = &[
    NaN,
    0.4331446047191349,
    1.0,
    4.0,
    NaN,
    0.2475112026966485,
    1.0,
    2.0,
    0.0,
    0.33297370793437503,
    0.6694989068532413,
    1.3259066730451181,
    0.00060276422869876,
    0.3344024348410919,
    0.6554392519561532,
    1.3975850885172036,
    NaN,
    0.7115947077528645,
    1.0,
    2.0,
    -0.06485297981308591,
    0.40905523287145584,
    0.5675427952179675,
    1.5058155828687063,
    NaN,
    0.8508197592697293,
    1.0,
    2.0,
    -0.6361956074736099,
    0.6422759294591174,
    0.4690611340777764,
    1.5486659911981882,
    -2.0263830871426762,
    0.8879411459055575,
    0.42401273464680145,
    1.5571981028368573,
];

const LP_EQUISOLID_WTS: &'static [f64] = &[
    NaN,
    0.3809181092477624,
    1.0,
    2.0,
    0.0,
    0.333965834842076,
    0.6595263573816275,
    0.6864354941809694,
    NaN,
    0.6666066911835843,
    1.0,
    2.0,
    -0.0133596888317431,
    0.35581861596661923,
    0.6208517493888941,
    0.7591879178554635,
    -0.041503319771970304,
    0.37147587226331613,
    0.6118851716614055,
    0.7644129357783873,
];

const LP_EQUISOLID_STW: &'static [f64] = &[
    NaN,
    0.4129723912297557,
    1.0,
    4.0,
    NaN,
    0.23598422355986043,
    1.0,
    2.0,
    0.0,
    0.3328777296467,
    0.6703463112308228,
    1.3646514131444079,
    -0.0003675575091826033,
    0.33700137433804045,
    0.6475623557095105,
    1.4687483137639816,
    NaN,
    0.6784546427345988,
    1.0,
    2.0,
    -0.08058623121267061,
    0.4344663407808156,
    0.5266369639191844,
    1.624020590601365,
    NaN,
    0.8111957684870202,
    1.0,
    2.0,
    -1.0050413959904745,
    0.8675049388721163,
    0.3185393093415456,
    1.7265055005529462,
    -5.234878220621795,
    1.789662937457308,
    0.11238964184787648,
    1.7739150761150029,
];

const LP_STEREOGRAPHIC_WTS: &'static [f64] = &[
    NaN,
    0.39269908169872414,
    1.0,
    2.0,
    0.0,
    0.33371389612897423,
    0.6625332957466958,
    0.7841220822061388,
    NaN,
    0.6872233929727672,
    1.0,
    2.0,
    -0.00945627955571915,
    0.34814949661213923,
    0.6386627362546502,
    0.8262701803131307,
    -0.043541568184132906,
    0.3617735548351235,
    0.6323617229368159,
    0.8296205891271242,
];

const LP_STEREOGRAPHIC_STW: &'static [f64] = &[
    NaN,
    0.6920072846668404,
    1.0,
    4.0,
    NaN,
    0.27680291386673617,
    1.0,
    2.0,
    0.0,
    0.33310164422449345,
    0.6683784040596974,
    1.2456685264835485,
    -0.006168438119716235,
    0.3450871451469215,
    0.6432533546568973,
    1.3071675165077483,
    NaN,
    0.8996094700668925,
    1.0,
    2.0,
    -0.10600191472701681,
    0.3946482406484648,
    0.6162001922934941,
    1.3226385005934782,
    -0.2107337282420758,
    0.4178526953793522,
    0.6121918111065544,
    1.3232590699446545,
];

const LP_EQUIANGULAR_WTS: &'static [f64] = &[
    NaN,
    0.3887720908817369,
    1.0,
    2.0,
    0.0,
    0.3336999238156753,
    0.6626383753026297,
    0.7836341632999364,
    NaN,
    0.6803511590430396,
    1.0,
    2.0,
    -0.009125173193686109,
    0.3478390589501448,
    0.6389352921928282,
    0.8260463471259278,
    -0.043438384902371396,
    0.3618042977386561,
    0.6323642881685042,
    0.8296186753991659,
];
const LP_EQUIANGULAR_STW: &'static [f64] = &[
    NaN,
    0.6880616378989937,
    1.0,
    4.0,
    NaN,
    0.2752246551595975,
    1.0,
    2.0,
    0.0,
    0.33311253176075467,
    0.6683091947614725,
    1.2458874707586318,
    -0.005960452842952435,
    0.34479341680383246,
    0.6436226181974165,
    1.3067529818215586,
    NaN,
    0.8944801292686918,
    1.0,
    2.0,
    -0.10147647445967323,
    0.3933185375489927,
    0.6165403810131558,
    1.3225664259453422,
    -0.22441670299429006,
    0.41791872175953415,
    0.6122577182012914,
    1.3232568661534838,
];

const LP_ORTHOGRAPHIC_WTS: &'static [f64] = &[
    NaN,
    0.6391177554646736,
    1.0,
    4.0,
    NaN,
    0.3652101459798135,
    1.0,
    2.0,
    0.0,
    0.3341470736449248,
    0.6570014624940024,
    0.5876086232213922,
    -0.015338058532515575,
    0.36126357386748253,
    0.6052861789086066,
    0.6920488863691894,
    NaN,
    1.0499791696919638,
    1.0,
    2.0,
    -0.043377124814274026,
    0.378566489674653,
    0.5940850491855811,
    0.6995207992235135,
    0.015309039765085686,
    0.38412405811753736,
    0.5947006978206639,
    0.6995772669140814,
];
const LP_ORTHOGRAPHIC_STW: &'static [f64] = &[
    NaN,
    0.5967157722525002,
    1.0,
    10.0,
    NaN,
    0.3422902253125737,
    1.0,
    4.0,
    NaN,
    0.19559441446432785,
    1.0,
    2.0,
    0.0,
    0.33267090648013165,
    0.6726733738414374,
    1.4786873555633946,
    -0.0055400064473545285,
    0.3530183935769955,
    0.5959506570448951,
    1.7781905395128206,
    NaN,
    0.4523120834487581,
    1.0,
    2.0,
    -0.024219104295980287,
    0.38754802059312965,
    0.5356104196893625,
    1.876073576642625,
    NaN,
    0.5348284770508964,
    1.0,
    2.0,
    -0.21051709288065013,
    0.609038194784489,
    0.27097174116215683,
    2.193902549222571,
    -0.647842576390758,
    0.9799538521119757,
    -0.043426031628087,
    2.4602226858123686,
    NaN,
    0.75916992215671,
    1.0,
    6.0,
    NaN,
    0.6895467150549057,
    1.0,
    2.0,
    -3.4536581938411928,
    2.755451717496612,
    -1.168748658590232,
    3.175054569023473,
    NaN,
    0.7359621864561086,
    1.0,
    2.0,
    -28.041747318660782,
    13.74331422587693,
    -6.093793155223921,
    5.389338100953864,
    -184.81687790341675,
    69.71002714083443,
    -26.091009627474705,
    12.540905006138928,
    NaN,
    0.7707737900070106,
    1.0,
    2.0,
    -945.468473863788,
    310.65049510844983,
    -102.43123224523151,
    36.73543775964936,
    NaN,
    0.776575723932161,
    1.0,
    2.0,
    -3880.8557332176715,
    1183.62704383675,
    -362.0722303753719,
    113.9639190499438,
    -23111.26378382556,
    6684.915652276017,
    -1935.8699797703885,
    564.2019697268843,
];

/// A LensPoly is a pair of polynomials that map sensor yaw to and from world
/// yaw; each must be the inverse of the other.
///
/// This provides two main methods that do the mapping; they take an angle from
/// 0 to PI/2 and map it to another angle, presumably in the same range
///
/// The simplest mapping is linear in both directions, for a 'rectilinear' lens.
///
/// They are encoded using a pair of piecewise-cubic-bezier-curve-trees.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LensPolys {
    /// Function of fractional X-offset (0 center, 1 RH of sensor) to angle
    ///
    /// fractional Y-offset is px_rel_y / (px_height/2) / pixel_aspect_ratio
    stw_poly: PiecewiseBezier,

    /// Function of angle to fractional X-offset (0 center, 1 RH of sensor)
    wts_poly: PiecewiseBezier,
}

//ip Display for LensPolys
impl std::fmt::Display for LensPolys {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "wts:{:0.4}; stw:{:0.4}", self.wts_poly, self.stw_poly)
    }
}

//ip JsonParsable for LensPolys
impl JsonParsable for LensPolys {
    type PostParseArg = ();
    type PostParseResult = Self;
    fn reason() -> &'static str {
        "lens polynomials"
    }
    fn post_parse(self, _args: &Self::PostParseArg) -> Result<Self> {
        self.stw_poly.validate()?;
        self.wts_poly.validate()?;
        Ok(self)
    }
}

impl LensPolys {
    /// Create a rectilinear lens mapping
    ///
    /// This is valid for 0 <= world,sensor <= 90
    pub fn rectilinear() -> Self {
        Self::default()
    }

    /// Create a stereographic lens mapping
    ///
    /// tan(sensor) = 2 tan(world/2)
    ///
    /// This is valid for 0 <= world <= 90, 0 <= sensor<= 63.4
    ///
    /// Note that the fundamental lens equation allows 0<=world<=180, i.e. a
    /// full 360 degree mapping; however, this is not realistic, and not
    /// supported by most of this library
    pub fn stereographic() -> Self {
        let wts_poly = PiecewiseBezier::of_f64s(LP_STEREOGRAPHIC_WTS).unwrap();
        let stw_poly = PiecewiseBezier::of_f64s(LP_STEREOGRAPHIC_STW).unwrap();
        Self::new(stw_poly, wts_poly)
    }

    /// Create an equiangular lens mapping (same as stereographic)
    ///
    /// tan(sensor) = 2 tan(world/2)
    ///
    /// This is valid for 0 <= world <= 90, 0 <= sensor<= 63.4
    ///
    /// Note that the fundamental lens equation allows 0<=world<=180, i.e. a
    /// full 360 degree mapping; however, this is not realistic, and not
    /// supported by most of this library
    pub fn equiangular() -> Self {
        let wts_poly = PiecewiseBezier::of_f64s(LP_EQUIANGULAR_WTS).unwrap();
        let stw_poly = PiecewiseBezier::of_f64s(LP_EQUIANGULAR_STW).unwrap();
        Self::new(stw_poly, wts_poly)
    }

    /// Create an equidistant lens mapping
    ///
    /// tan(sensor) = world
    ///
    /// This is valid for 0 <= world <= 89, 0 <= sensor <= 57.2
    pub fn equidistant() -> Self {
        let wts_poly = PiecewiseBezier::of_f64s(LP_EQUIDISTANT_WTS).unwrap();
        let stw_poly = PiecewiseBezier::of_f64s(LP_EQUIDISTANT_STW).unwrap();
        Self::new(stw_poly, wts_poly)
    }

    /// Create an equisolid lens mapping
    ///
    /// tan(sensor) = 2 sin(world/2)
    ///
    /// This is valid for 0 <= world <= 87.3, 0 <= sensor <= 54.0
    pub fn equisolid() -> Self {
        let wts_poly = PiecewiseBezier::of_f64s(LP_EQUISOLID_WTS).unwrap();
        let stw_poly = PiecewiseBezier::of_f64s(LP_EQUISOLID_STW).unwrap();
        Self::new(stw_poly, wts_poly)
    }

    /// Create an orthographic lens mapping
    ///
    /// tan(sensor) = sin(world)
    ///
    /// This is valid for 0 <= world <= 83.7, 0 <= sensor <= 44.8
    pub fn orthographic() -> Self {
        let wts_poly = PiecewiseBezier::of_f64s(LP_ORTHOGRAPHIC_WTS).unwrap();
        let stw_poly = PiecewiseBezier::of_f64s(LP_ORTHOGRAPHIC_STW).unwrap();
        Self::new(stw_poly, wts_poly)
    }

    /// Map from sensor angle to world angle
    ///
    /// Use the fact that P(yaw) = yaw * poly(yaw^2)
    pub fn map_sensor_to_world(&self, angle: f64) -> f64 {
        self.stw_poly.evaluate(angle)
    }

    /// Map from world angle to sensor angle
    ///
    /// Use the fact that P(yaw) = yaw * poly(yaw^2)
    pub fn map_world_to_sensor(&self, angle: f64) -> f64 {
        self.wts_poly.evaluate(angle)
    }

    pub fn new(stw_poly: PiecewiseBezier, wts_poly: PiecewiseBezier) -> Self {
        Self { stw_poly, wts_poly }
    }

    pub fn to_json(&self, pretty: bool) -> Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            Ok(serde_json::to_string(self)?)
        }
    }

    pub fn stw_poly_as_f64s(&self) -> Vec<f64> {
        self.stw_poly.as_f64s()
    }

    pub fn wts_poly_as_f64s(&self) -> Vec<f64> {
        self.wts_poly.as_f64s()
    }

    //cp calibration
    /// Calculate polynomials of best-fit for a given set of sensor
    /// and world yaws
    ///
    /// Most lenses map world 0 to 90 to sensor 0 to <90; the calibration is
    /// also likely to have come from a sensor, with a map to the world yaw
    ///
    /// As such we can filter the values to remove outliers; generate a set of
    /// points for sensor values of 0, and yaw_min to yaw_max
    ///
    /// The sets are ensured to be monotonic by separately sorting the world and
    /// sensor yaws in ascending order; then when they are paired they will be monotoinc.
    /// Duplicates or near duplicates are also removed.
    ///
    /// This list of pairs of points form a set of line segments, which are then
    /// approximated to with a certain degree of accuracy using
    /// piecewise-cubic-Bezier curves
    pub fn calibration(
        sensor_yaws: &[f64],
        world_yaws: &[f64],
        yaw_range_min: f64,
        yaw_range_max: f64,
        apply_filter: bool,
    ) -> Result<Self> {
        // Create a vec of (world, sensor) yaw pairs where sensor yaw is > yaw_range_min
        let mut ws_yaws: Vec<_> = sensor_yaws
            .iter()
            .zip(world_yaws.iter())
            .filter(|(s, _)| **s > yaw_range_min)
            .map(|(s, w)| (*w, *s))
            .collect();
        ws_yaws.sort_by(|a, b| (a.1).partial_cmp(&b.1).unwrap());

        // Map vec of (world,sensor) yaw pairs to (local mean world, sensor)
        // values using a windowed filter
        let mean_median_ws_yaws = {
            if apply_filter {
                polynomial::filter_ws_yaws(&ws_yaws, 8)
            } else {
                ws_yaws.clone()
            }
        };

        let mut mm_w_yaws: Vec<_> = mean_median_ws_yaws.iter().map(|(w, _s)| *w).collect();
        let mut mm_s_yaws: Vec<_> = mean_median_ws_yaws.iter().map(|(_w, s)| *s).collect();
        mm_w_yaws.push(0.0);
        mm_s_yaws.push(0.0);
        mm_w_yaws.sort_by(|a, b| (a).partial_cmp(&b).unwrap());
        mm_s_yaws.sort_by(|a, b| (a).partial_cmp(&b).unwrap());
        let min_yaw_step = 0.01;
        let mut last_ok_sw = (0.0, 0.0);
        let mut mm_sw_yaws = vec![];
        for (i, sw) in mm_s_yaws.into_iter().zip(mm_w_yaws.into_iter()).enumerate() {
            if i == 0 {
                mm_sw_yaws.push(last_ok_sw);
            } else {
                if sw.0 < last_ok_sw.0 + min_yaw_step {
                    continue;
                }
                if sw.1 < last_ok_sw.1 + min_yaw_step {
                    continue;
                }
                last_ok_sw = sw;
                mm_sw_yaws.push(sw);
            }
        }

        let stw_poly = PiecewiseBezier::of_x_y_pairs(&mm_sw_yaws, 0.0, yaw_range_max, 1E-4, 1000)?;
        let wts_poly = stw_poly.inv(0.0, yaw_range_max, 1E-5, 1000, 100)?;

        Ok(Self { wts_poly, stw_poly })
    }

    pub fn of_wts_fn<F>(wts_fn: &F, yaw_range_min: f64, yaw_range_max: f64) -> Result<Self>
    where
        F: Fn(f64) -> f64,
    {
        let wts_poly = PiecewiseBezier::of_fn(yaw_range_min, yaw_range_max, wts_fn, 1E-4, 100)?;
        let stw_poly = wts_poly.inv(yaw_range_min, yaw_range_max, 1E-6, 10000, 100)?;

        Ok(Self { wts_poly, stw_poly })
    }
}
