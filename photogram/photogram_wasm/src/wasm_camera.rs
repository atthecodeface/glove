//a Imports
use wasm_bindgen::prelude::*;

use ic_base::{JsonParsable, Point2D, Point3D, Rrc};
use ic_camera::{CameraDatabase, CameraInstance, CameraInstanceDesc, CameraProjection};

use crate::{ToFromWasmArr, WasmPointMappingSet, WasmRay, err_to_string};

//a WasmCameraDatabase
//tp WasmCameraDatabase
#[wasm_bindgen]
pub struct WasmCameraDatabase {
    cdb: Rrc<CameraDatabase>,
}

//ip WasmCameraDatabase
impl WasmCameraDatabase {
    //cp of_cdb
    pub fn of_cdb(cdb: Rrc<CameraDatabase>) -> Self {
        Self { cdb }
    }
    //cp cdb
    pub fn cdb(&self) -> &Rrc<CameraDatabase> {
        &self.cdb
    }
}
//ip WasmCameraDatabase
#[wasm_bindgen]
impl WasmCameraDatabase {
    //fp new
    /// Create a new WasmCameraDatabase from a Json file
    #[wasm_bindgen(constructor)]
    pub fn new(json: &str) -> Result<WasmCameraDatabase, JsValue> {
        let cdb = CameraDatabase::load_json(json, &())
            .map_err(err_to_string)?
            .into();
        Ok(Self { cdb })
    }
}

//a WasmCameraInstance
//tp WasmCameraInstance
#[wasm_bindgen]
pub struct WasmCameraInstance {
    camera: Rrc<CameraInstance>,
}

//ip WasmCameraInstance
impl WasmCameraInstance {
    //cp of_camera
    pub fn of_camera(camera: Rrc<CameraInstance>) -> Self {
        Self { camera }
    }

    //ap camera
    pub fn camera(&self) -> &Rrc<CameraInstance> {
        &self.camera
    }
}

//ip WasmCameraInstance
#[wasm_bindgen]
impl WasmCameraInstance {
    //fp new
    /// Create a new WasmCameraInstance from a camera database and a Json file
    #[wasm_bindgen(constructor)]
    pub fn new(cdb: &WasmCameraDatabase, json: &str) -> Result<WasmCameraInstance, JsValue> {
        let camera = CameraInstanceDesc::load_json(json, &cdb.cdb.borrow())
            .map_err(err_to_string)?
            .into();
        Ok(Self { camera })
    }

    //mp body
    #[wasm_bindgen(getter)]
    pub fn body(&self) -> String {
        self.camera.borrow().camera_name().into()
    }

    //mp lens
    #[wasm_bindgen(getter)]
    pub fn lens(&self) -> String {
        self.camera.borrow().lens_name().into()
    }

    //mp rotation
    // #[wasm_bindgen(getter)]
    // pub fn rotation(&self) -> usize {
    // self.camera.borrow().rotation()
    // }

    //mp position
    #[wasm_bindgen(getter)]
    pub fn position(&self) -> Result<Box<[f64]>, String> {
        let xyz: [f64; 3] = self.camera.borrow().position().into();
        Ok(Box::new(xyz))
    }

    //mp orientation
    #[wasm_bindgen(getter)]
    pub fn orientation(&self) -> Box<[f64]> {
        let q: [f64; 4] = self.camera.borrow().orientation().into();
        Box::new(q)
    }

    //mp focus_distance
    #[wasm_bindgen(getter)]
    pub fn focus_distance(&self) -> f64 {
        self.camera.borrow().focus_distance()
    }

    //mp set_focus_distance
    #[wasm_bindgen(setter)]
    pub fn set_focus_distance(&mut self, mm_focus_distance: f64) {
        self.camera
            .borrow_mut()
            .set_focus_distance(mm_focus_distance);
    }

    //mp map_model
    pub fn map_model(&self, pt: &[f64]) -> Result<Box<[f64]>, String> {
        Ok(Point2D::to_wasm(
            self.camera
                .borrow()
                .world_xyz_to_px_abs_xy(&Point3D::from_wasm(pt)?),
        ))
    }

    //mp direction_of_pt
    pub fn direction_of_pt(&self, pt: &[f64]) -> Result<Box<[f64]>, String> {
        let txty = self
            .camera
            .borrow()
            .px_abs_xy_to_camera_txty(&Point2D::from_wasm(pt)?);
        Ok(Point3D::to_wasm(
            self.camera.borrow().camera_txty_to_world_dir(&txty),
        ))
    }

    //mp get_pm_as_ray
    pub fn get_pm_as_ray(
        &self,
        wpms: &WasmPointMappingSet,
        n: usize,
        from_camera: bool,
    ) -> Result<WasmRay, String> {
        let pms = wpms.pms().borrow();
        let pms = pms.mappings();
        if let Some(pm) = pms.get(n) {
            let ray = pm.get_mapped_ray(&*self.camera.borrow(), from_camera);
            Ok(WasmRay::of_ray(ray))
        } else {
            Err("PM index out of range".into())
        }
    }

    //mp model_at_distance
    pub fn model_at_distance(&self, pt: &[f64], distance: f64) -> Result<Box<[f64]>, String> {
        let txty = self
            .camera
            .borrow()
            .px_abs_xy_to_camera_txty(&Point2D::from_wasm(pt)?);
        let world_dir = self.camera.borrow().camera_txty_to_world_dir(&txty);
        Ok(Point3D::to_wasm(
            self.camera.borrow().position() - world_dir * distance,
        ))
    }

    //cp to_json
    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<String, JsValue> {
        Ok(self.camera.borrow().to_json(false).map_err(err_to_string)?)
    }

    //zz All done
}
