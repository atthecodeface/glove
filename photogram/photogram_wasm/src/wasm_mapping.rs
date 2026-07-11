use crate::WasmVec3f64;

use wasm_bindgen::prelude::*;

use ic_base::{JsonParsable, JsonSrc, Point2D, Point3D, Rrc};
use ic_image::Color;
use ic_mapping::{NamedPointSet, PointMappingSet};

use crate::{ToFromWasmArr, err_to_string};

//a WasmPointMappingSet
//tp WasmPointMappingSet
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

    //mp get_name
    /// Get the nth point mapping
    pub fn get_name(&self, n: usize) -> Result<String, String> {
        self.pms
            .borrow()
            .mappings()
            .get(n)
            .map(|m| m.named_point().ref_tag().as_str().to_owned())
            .ok_or("Index out of range".into())
    }

    //mp get_xy
    /// Get the XY coords
    pub fn get_xy(&self, n: usize) -> Result<Box<[f64]>, String> {
        self.pms
            .borrow()
            .mappings()
            .get(n)
            .map(|m| [m.screen()[0], m.screen()[1]].into())
            .ok_or("Index out of range".into())
    }

    //mp get_xy_err
    /// Get the XY coords and error
    pub fn get_xy_err(&self, n: usize) -> Result<Box<[f64]>, String> {
        self.pms
            .borrow()
            .mappings()
            .get(n)
            .map(|m| [m.screen()[0], m.screen()[1], m.error()].into())
            .ok_or("Index out of range".into())
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
    pub fn add_mapping(
        &mut self,
        wnps: &WasmNamedPointSet,
        name: &str,
        screen: &[f64],
        error: f64,
    ) -> Result<bool, String> {
        Ok(self.pms.borrow_mut().add_mapping(
            &wnps.nps.borrow(),
            name,
            &Point2D::from_wasm(screen)?,
            error,
        ))
    }

    /// Remove the 'nth' mapping
    pub fn remove_mapping(&mut self, n: usize) -> Result<(), String> {
        if !self.pms.borrow_mut().remove_mapping(n) {
            Err("Index out of range".into())
        } else {
            Ok(())
        }
    }

    //zz All done
}

/*
 * A WasmNamedPoint is a transient structure containing the data that is in the
 * NamedPointSet database
 *
 * It is *not* a mirror onto the actual content, and its properties can be
 * changed without updating the actual database
 *
 * To updated the database use ?
 */
#[wasm_bindgen]
pub struct WasmNamedPoint {
    name: String,
    color: String,
    at_infinity: bool,
    model: [f64; 3],
    error: f64,
}

//ip WasmNamedPoint
#[wasm_bindgen]
impl WasmNamedPoint {
    //fp new
    /// Create a new WasmNamedPoint
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, color: &str) -> Result<WasmNamedPoint, JsValue> {
        let name = name.into();
        let _color: Color = color.try_into()?;
        let color = color.into();
        let model = [0.; 3];
        let error = 0.0;
        let at_infinity = false;
        Ok(Self {
            name,
            color,
            at_infinity,
            model,
            error,
        })
    }

    //mp name
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> JsValue {
        (&self.name).into()
    }

    //mp color
    #[wasm_bindgen(getter)]
    pub fn color(&self) -> JsValue {
        (&self.color).into()
    }

    #[wasm_bindgen(getter)]
    pub fn at_infinity(&self) -> bool {
        self.at_infinity
    }

    #[wasm_bindgen(getter)]
    pub fn model(&self) -> Box<[f64]> {
        Box::new(self.model)
    }

    pub fn set_model_vec(&self, v: &mut WasmVec3f64) {
        v.set_array(&self.model);
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> f64 {
        self.error
    }
}

/*
 * A WasmNamedPointSet contains a reference to the contents of the actual named
 * point set in the database
 *
 * Modifying the WasmNamedPointSet modifies the project
 *
 */
#[wasm_bindgen]
pub struct WasmNamedPointSet {
    nps: Rrc<NamedPointSet>,
}

impl WasmNamedPointSet {
    pub fn of_nps(nps: Rrc<NamedPointSet>) -> Self {
        Self { nps }
    }

    pub fn nps(&self) -> &Rrc<NamedPointSet> {
        &self.nps
    }
}

#[wasm_bindgen]
impl WasmNamedPointSet {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmNamedPointSet, JsValue> {
        let nps = Rrc::<NamedPointSet>::default();
        Ok(Self { nps })
    }

    #[wasm_bindgen]
    pub fn read_json(&mut self, json: &str) -> Result<(), JsValue> {
        let nps = NamedPointSet::load_json(json, &())
            .map_err(err_to_string)?
            .into();

        self.nps.borrow_mut().merge(nps);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<String, JsValue> {
        Ok(self.nps.borrow().to_json(false).map_err(err_to_string)?)
    }

    pub fn num_points(&mut self) -> usize {
        self.nps.borrow().len()
    }

    #[wasm_bindgen]
    pub fn add_pt(&mut self, wnp: WasmNamedPoint) -> Result<(), JsValue> {
        let color: Color = wnp.color.as_str().try_into()?;
        self.nps
            .borrow_mut()
            .add_pt(&wnp.name, color, false, Some(wnp.model.into()), wnp.error);
        Ok(())
    }

    pub fn used_by(&mut self, name: &str) -> usize {
        self.nps.borrow().pt_use_count(name)
    }

    pub fn delete_pt(&mut self, name: &str) -> bool {
        let mut nps = self.nps.borrow_mut();
        if nps.pt_use_count(name) == 0 {
            nps.remove_pt(name);
            true
        } else {
            false
        }
    }

    #[wasm_bindgen]
    pub fn get_pt(&mut self, name: &str) -> Option<WasmNamedPoint> {
        if let Some(np) = self.nps.borrow().get_rc_np(name) {
            let (at_infinity, model, error) = np.model();
            let wnp = WasmNamedPoint {
                name: name.into(),
                color: np.color().as_string(),
                at_infinity,
                model: model.into(),
                error,
            };
            Some(wnp)
        } else {
            None
        }
    }

    pub fn pts(&mut self) -> Result<Vec<String>, JsValue> {
        let mut names = vec![];
        for np in self.nps.borrow().iter() {
            names.push(np.ref_tag().to_string());
        }
        Ok(names)
    }

    pub fn set_direction(&self, name: &str, model: &[f64]) -> Result<(), String> {
        if let Some(np) = self.nps.borrow().get_rc_np(name) {
            np.set_model(Some((true, Point3D::from_wasm(model)?, 0.0)));
            Ok(())
        } else {
            Err("Could not find named point".into())
        }
    }

    pub fn set_color(&self, name: &str, color: &str) -> Result<(), String> {
        let color: Color = color.try_into()?;

        if let Some(np) = self.nps.borrow().get_rc_np(name) {
            np.set_color(color);
            Ok(())
        } else {
            Err("Could not find named point".into())
        }
    }

    pub fn set_model(&self, name: &str, model: &[f64], error: f64) -> Result<(), String> {
        if let Some(np) = self.nps.borrow().get_rc_np(name) {
            np.set_model(Some((false, Point3D::from_wasm(model)?, error)));
            Ok(())
        } else {
            Err("Could not find named point".into())
        }
    }

    pub fn unset_model(&self, name: &str) -> Result<(), String> {
        if let Some(np) = self.nps.borrow().get_rc_np(name) {
            np.set_model(None);
            Ok(())
        } else {
            Err("Could not find named point".into())
        }
    }

    //zz All done
}
