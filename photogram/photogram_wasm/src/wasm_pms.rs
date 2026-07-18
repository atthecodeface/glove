use wasm_bindgen::prelude::*;

use ic_base::{JsonParsable, JsonSrc, Point2D, Rrc};
use ic_mapping::PointMappingSet;

use crate::{WasmNamedPointSet, err_to_string};

#[wasm_bindgen]
pub struct WasmPointMappingSet {
    pms: Rrc<PointMappingSet>,
}

//ip WasmPointMappingSet
impl WasmPointMappingSet {
    //ap pms
    pub fn pms(&self) -> &Rrc<PointMappingSet> {
        &self.pms
    }
    //cp of_pms
    pub fn of_pms(pms: Rrc<PointMappingSet>) -> Self {
        Self { pms }
    }
}

#[wasm_bindgen]
impl WasmPointMappingSet {
    /// Create a new WasmPointMappingSet from a camera database and a Json file
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmPointMappingSet {
        let pms = PointMappingSet::default().into();
        Self { pms }
    }

    /// Try to parse a Json file as a PointMappingSet, returning the number of points
    pub fn try_json(json: &str) -> Result<usize, JsValue> {
        let json = JsonSrc::<PointMappingSet>::of_json(json).map_err(err_to_string)?;
        let (_, pms) = json
            .deserialize_as::<PointMappingSet>("Pms")
            .map_err(err_to_string)?;
        Ok(pms.mappings().len())
    }

    /// Read a json file to add to the points
    pub fn read_json(&mut self, wnps: &WasmNamedPointSet, json: &str) -> Result<(), JsValue> {
        let (_pms, _pms_not_found) = PointMappingSet::load_json(json, &wnps.nps.borrow())
            .map_err(err_to_string)?
            .into();
        // if !nf.is_empty() {
        // eprintln!("Warning: {}", nf);
        // }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<String, JsValue> {
        Ok(self.pms.borrow().to_json(false).map_err(err_to_string)?)
    }

    /// Get the number of mappings
    #[wasm_bindgen(getter)]
    pub fn length(&self) -> usize {
        self.pms.borrow().mappings().len()
    }

    /// Get the name of the nth point mapping
    pub fn get_name(&self, n: usize) -> Option<String> {
        self.pms
            .borrow()
            .mappings()
            .get(n)
            .map(|m| m.named_point().ref_tag().as_str().to_owned())
    }

    /// Get the XY coords
    pub fn get_xy(&self, n: usize) -> Option<Box<[f64]>> {
        self.pms
            .borrow()
            .mappings()
            .get(n)
            .map(|m| [m.screen()[0], m.screen()[1]].into())
    }

    /// Get the XY coords and error
    pub fn get_xy_err(&self, n: usize) -> Option<Box<[f64]>> {
        self.pms
            .borrow()
            .mappings()
            .get(n)
            .map(|m| [m.screen()[0], m.screen()[1], m.error()].into())
    }

    pub fn set_xy(&mut self, n: usize, x: f64, y: f64) -> Result<(), String> {
        self.pms
            .borrow_mut()
            .mappings_mut()
            .get_mut(n)
            .map(|m| m.set_screen([x, y].into()))
            .ok_or("Index out of range".into())
    }

    /// Find the index of the first mapping that matches the name
    pub fn mapping_of_name(&self, name: &str) -> Option<usize> {
        self.pms
            .borrow()
            .mappings()
            .iter()
            .enumerate()
            .find(|(_, m)| m.named_point().has_name(name))
            .map(|(n, _)| n)
            .into()
    }

    /// Add a mapping - this permits multiple mappings to the same np
    pub fn add_mapping(&mut self, wnps: &WasmNamedPointSet, name: &str) -> Option<usize> {
        self.pms
            .borrow_mut()
            .add_mapping(&wnps.nps.borrow(), name, &Point2D::default(), 0.0)
    }

    /// Remove the 'nth' mapping
    pub fn remove_mapping(&mut self, n: usize) -> Result<(), String> {
        if !self.pms.borrow_mut().remove_mapping(n) {
            Err("Index out of range".into())
        } else {
            Ok(())
        }
    }
}
