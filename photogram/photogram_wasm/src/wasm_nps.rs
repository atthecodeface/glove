use wasm_bindgen::prelude::*;

use ic_base::{JsonParsable, Point3D, Rrc};
use ic_image::Color8;
use ic_mapping::NamedPointSet;

use crate::WasmNamedPoint;
use crate::{ToFromWasmArr, err_to_string};

/*
 * A WasmNamedPointSet contains a reference to the contents of the actual named
 * point set in the database
 *
 * Modifying the WasmNamedPointSet modifies the project
 *
 */
#[wasm_bindgen]
pub struct WasmNamedPointSet {
    pub(crate) nps: Rrc<NamedPointSet>,
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
    pub fn new() -> WasmNamedPointSet {
        let nps = Rrc::<NamedPointSet>::default();
        Self { nps }
    }

    pub fn read_json(&mut self, json: &str) -> Result<(), JsValue> {
        let nps = NamedPointSet::load_json(json, &())
            .map_err(err_to_string)?
            .into();

        self.nps.borrow_mut().merge(nps);
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, JsValue> {
        Ok(self.nps.borrow().to_json(false).map_err(err_to_string)?)
    }

    pub fn num_points(&mut self) -> usize {
        self.nps.borrow().len()
    }

    /** Returns the point it replaced, if any */
    pub fn add_pt(&mut self, wnp: &WasmNamedPoint) -> Option<WasmNamedPoint> {
        let mut nps = self.nps.borrow_mut();

        // add_pt returns the *old* point if there was one there already.
        let opt_replaced_point = nps.add_pt(&wnp.name, Color8::black(), false, None, 0.0);

        wnp.set_np(&*nps.get_rc_np(&wnp.name).unwrap());
        opt_replaced_point.map(|np| (&*np).into())
    }

    pub fn get_pt(&self, name: &str) -> Option<WasmNamedPoint> {
        if let Some(np) = self.nps.borrow().get_rc_np(name) {
            Some((&*np).into())
        } else {
            None
        }
    }

    pub fn set_pt(&mut self, name: &str, wnp: &WasmNamedPoint) -> bool {
        if let Some(np) = self.nps.borrow().get_rc_np(name) {
            wnp.set_np(&np);
            true
        } else {
            false
        }
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
        let color: Color8 = color.try_into()?;

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
}
