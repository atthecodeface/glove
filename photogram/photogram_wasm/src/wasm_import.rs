//a Imports
use wasm_bindgen::prelude::*;

use photogram::{Point2D, Point3D};

pub fn err_to_string(e: photogram::Error) -> String {
    e.to_string()
}

//a Useful macros
//a Log
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

//a Helpful functions
pub trait ToFromWasmArr: Sized {
    fn from_wasm(pt: &[f64]) -> Result<Self, String>;
    fn to_wasm(self) -> Box<[f64]>;
}

impl ToFromWasmArr for Point3D {
    fn from_wasm(pt: &[f64]) -> Result<Point3D, String> {
        if pt.len() != 3 {
            Err("Expected point (x,y,z)".into())
        } else {
            Ok([pt[0], pt[1], pt[2]].into())
        }
    }
    fn to_wasm(self) -> Box<[f64]> {
        let pt: [f64; 3] = self.into();
        Box::new(pt)
    }
}

impl ToFromWasmArr for Point2D {
    fn from_wasm(pt: &[f64]) -> Result<Point2D, String> {
        if pt.len() != 2 {
            Err("Expected point (x,y)".into())
        } else {
            Ok([pt[0], pt[1]].into())
        }
    }
    fn to_wasm(self) -> Box<[f64]> {
        let pt: [f64; 2] = self.into();
        Box::new(pt)
    }
}
