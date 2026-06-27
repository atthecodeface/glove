//a Imports
use wasm_bindgen::prelude::*;

use ic_base::JsonParsable;
use ic_mapping::PointMapping;
use ic_project::Project;

use crate::{
    WasmCameraDatabase, WasmCip, WasmNamedPointSet, err_to_string,
};

//a WasmProject
#[wasm_bindgen]
pub struct WasmProject {
    project: Project,
}

//ip WasmProject
#[wasm_bindgen]
impl WasmProject {
    //cp new
    /// Create a new WasmGraphCanvas attached to a Canvas HTML element,
    /// adding events to the canvas that provide the paint program
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmProject {
        let project = Project::default();
        Self { project }
    }

    //mp to_json
    #[wasm_bindgen]
    pub fn to_json(&self, pretty: bool) -> Result<String, JsValue> {
        Ok(self.project.to_json(pretty).map_err(err_to_string)?)
    }

    //cp read_json
    #[wasm_bindgen]
    pub fn read_json(&mut self, json: &str) -> Result<(), JsValue> {
        self.project = Project::load_json(json, &()).map_err(err_to_string)?;
        Ok(())
    }

    //ap cdb
    #[wasm_bindgen(getter)]
    pub fn cdb(&self) -> WasmCameraDatabase {
        WasmCameraDatabase::of_cdb(self.project.cdb().clone())
    }

    //ap set_cdb
    #[wasm_bindgen(setter)]
    pub fn set_cdb(&mut self, wcdb: WasmCameraDatabase) {
        // unsafe {
        // crate::console_log!("Log {:?}", wcdb.cdb);
        // }
        let cdb = wcdb.cdb().clone();
        drop(wcdb);
        let cdb = cdb.take().unwrap();
        self.project.set_cdb(cdb);
    }

    //ap nps
    #[wasm_bindgen(getter)]
    pub fn nps(&self) -> WasmNamedPointSet {
        WasmNamedPointSet::of_nps(self.project.nps().clone())
    }

    //ap set_nps
    #[wasm_bindgen(setter)]
    pub fn set_nps(&mut self, wnps: &WasmNamedPointSet) {
        // unsafe {
        // crate::console_log!("Log {:?}", wnps.nps);
        // }
        self.project.set_nps(wnps.nps().clone());
    }

    //mp add_cip
    pub fn add_cip(&mut self, cip: &WasmCip) -> String {
        let cip_name = cip.image().clone();
        self.project.add_cip(cip.cip().clone());
        cip_name
    }

    //ap ncips
    pub fn ncips(&self) -> usize {
        self.project.ncips()
    }

    //mp cip_name
    pub fn cip_name(&self, n: usize) -> Option<String> {
        self.project.cip_name(n)
    }

    //mp cip
    pub fn cip(&self, name: String) -> Result<WasmCip, String> {
        if self.project.cip(&name).is_none() {
            Err(format!("Cip {name} not found"))
        } else {
            Ok(WasmCip::of_cip(self.project.cip(&name).unwrap().clone()))
        }
    }

    //mp cip_read_json
    pub fn cip_read_json(
        &self,
        name: String,
        camera_json: &str,
        pms_json: &str,
    ) -> Result<String, String> {
        if self.project.cip(&name).is_none() {
            Err(format!("Cip {name} not found"))
        } else {
            self.project
                .cip(&name)
                .unwrap()
                .borrow_mut()
                .read_json(&self.project, camera_json, pms_json)
                .map_err(err_to_string)
        }
    }

    //mp locate_all
    pub fn locate_all(&self, max_pairs: usize) {
        let filter = |_n, _pm: &PointMapping| true; //pms_n.contains(&n) && pm.model_error() < max_np_error);
        self.project.locate_all(filter, max_pairs);
    }

    //mp derive_nps_location
    /// Returns [x,y,z,e]
    pub fn derive_nps_location(&self, name: &str) -> Option<Box<[f64]>> {
        self.project
            .derive_nps_location(name)
            .map(|(a, e)| [a[0], a[1], a[2], e].into())
    }

    //zz All done
}
