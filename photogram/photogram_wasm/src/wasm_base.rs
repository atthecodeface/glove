//a Imports
use wasm_bindgen::prelude::*;

use photogram::{Point3D, Ray};

use crate::wasm_import::ToFromWasmArr;

//a WasmRay
//tp WasmRay
#[wasm_bindgen]
pub struct WasmRay {
    ray: Ray,
}

//ip WasmRay
impl WasmRay {
    pub fn of_ray(ray: Ray) -> Self {
        Self { ray }
    }
}

//ip WasmRay
#[wasm_bindgen]
impl WasmRay {
    //fp new
    /// Create a new WasmRay
    #[wasm_bindgen(constructor)]
    pub fn new(start: &[f64], dirn: &[f64], tan_error: Option<f64>) -> Result<WasmRay, String> {
        let tan_error = tan_error.unwrap_or(0.01);
        let ray = Ray::default()
            .set_start(Point3D::from_wasm(start)?)
            .set_direction(Point3D::from_wasm(dirn)?)
            .set_tan_error(tan_error);
        Ok(Self { ray })
    }

    //mp model_at_distance
    pub fn model_at_distance(&self, distance: f64) -> Result<Box<[f64]>, String> {
        let model: [f64; 3] = (self.ray.start() + self.ray.direction() * distance).into();
        Ok(Box::new(model))
    }

    //mp closest_model_to_intersection
    pub fn closest_model_to_intersection(rays: Vec<WasmRay>) -> Option<Box<[f64]>> {
        if let Some(model) = Ray::closest_point(rays.iter().map(|r| &r.ray), &|_r| 1.0) {
            let model: [f64; 3] = model.into();
            Some(Box::new(model))
        } else {
            None
        }
    }
}
