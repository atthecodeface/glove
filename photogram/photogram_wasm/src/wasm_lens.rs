use wasm_bindgen::prelude::*;

use ic_camera::LensPolys;

use crate::WasmBezier1f64;
use crate::console_log;
use crate::err_to_string;

/*
 * A WasmLensPoly is a specific lens calibration
 *
 */
#[wasm_bindgen]
#[derive(Debug, Clone, Default)]
pub struct WasmLensPoly(LensPolys);

impl std::convert::From<LensPolys> for WasmLensPoly {
    fn from(lp: LensPolys) -> Self {
        Self(lp)
    }
}

impl WasmLensPoly {
    pub fn polys(&self) -> &LensPolys {
        &self.0
    }
}

#[wasm_bindgen]
impl WasmLensPoly {
    /// Create a new WasmPointMapping from a WasmNamedPoint with no mapping
    #[wasm_bindgen(constructor)]
    pub fn new(kind: &str) -> Self {
        match kind {
            "equisolid" => Self(LensPolys::equisolid()),
            "equidistant" => Self(LensPolys::equidistant()),
            "equiangular" => Self(LensPolys::equiangular()),
            "stereographic" => Self(LensPolys::stereographic()),
            "orthographic" => Self(LensPolys::orthographic()),
            _ => Self(LensPolys::rectilinear()),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn max_world(&self) -> f64 {
        self.0.max_world_yaw()
    }

    #[wasm_bindgen(getter)]
    pub fn max_sensor(&self) -> f64 {
        self.0.max_sensor_yaw()
    }

    pub fn to_json(&self) -> Result<String, String> {
        self.0.to_json(true).map_err(err_to_string)
    }

    pub fn of_calibration(
        sensor: &[f64],
        world: &[f64],
        yaw_min: f64,
        yaw_max: f64,
    ) -> Result<Self, String> {
        // If filter is enabled then we lose the left and right hand points - which kills the behavior required
        let polys = LensPolys::calibration(sensor, world, yaw_min, yaw_max, false)
            .map_err(err_to_string)?;
        //        console_log!("{}", polys.to_json(true).map_err(err_to_string)?);
        Ok(Self(polys))
    }

    pub fn wts(&self, world: f64) -> f64 {
        self.0.map_world_to_sensor(world)
    }
    pub fn stw(&self, sensor: f64) -> f64 {
        self.0.map_sensor_to_world(sensor)
    }
    pub fn num_beziers(&self, world: bool) -> usize {
        self.0.iter_beziers(world).count()
    }
    pub fn bezier_end_range(&self, world: bool, idx: usize) -> f64 {
        let Some(b) = self.0.iter_beziers(world).skip(idx).next() else {
            return f64::NAN;
        };
        b.1
    }

    pub fn set_bezier(&self, world: bool, idx: usize, bezier: &mut WasmBezier1f64) -> f64 {
        let Some(b) = self.0.iter_beziers(world).skip(idx).next() else {
            return f64::NAN;
        };
        while bezier.num_control_pts() < 4 {
            bezier.bezier_mut().push([0.0]);
        }
        for (d, s) in bezier.bezier_mut().iter_mut().zip(b.2.iter()) {
            *d = *s;
        }
        b.0
    }
}
