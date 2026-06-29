use std::borrow::Borrow;

use ic_base::Rrc;
use ic_mapping::PointMapping;
use ic_project::{Cip, CipDesc};
use wasm_bindgen::prelude::*;

use crate::{WasmCameraInstance, WasmPointMappingSet, err_to_string};

#[wasm_bindgen]
#[derive(Debug)]
pub struct WasmCipDesc(CipDesc);

#[wasm_bindgen]
impl WasmCipDesc {
    /// Try to parse a Json string as a CipDesc without a Project
    pub fn try_json(json: &str) -> Result<Self, JsValue> {
        let json = ic_base::JsonSrc::<ic_project::CipDesc>::of_json(json).map_err(err_to_string)?;
        let (_, cip_desc) = json
            .deserialize_as::<ic_project::CipDesc>("Cip")
            .map_err(err_to_string)?;
        Ok(Self(cip_desc))
    }

    #[wasm_bindgen(getter)]
    pub fn num_mappings(&self) -> usize {
        self.0.num_mappings()
    }

    #[wasm_bindgen(getter)]
    pub fn image(&self) -> String {
        let t: &String = self.0.image().borrow();
        t.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn camera_body(&self) -> String {
        self.0.camera().body().to_owned()
    }

    #[wasm_bindgen(getter)]
    pub fn camera_lens(&self) -> String {
        self.0.camera().lens().to_owned()
    }
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct WasmCip {
    cip: Rrc<Cip>,
}

impl WasmCip {
    pub fn of_cip(cip: Rrc<Cip>) -> Self {
        Self { cip }
    }

    pub fn cip(&self) -> &Rrc<Cip> {
        &self.cip
    }
}

#[wasm_bindgen]
impl WasmCip {
    /// Create a new WasmGraphCanvas attached to a Canvas HTML element,
    /// adding events to the canvas that provide the paint program
    #[wasm_bindgen(constructor)]
    pub fn new(cam_file: &str, image: &str, pms_file: &str) -> WasmCip {
        let mut cip = Cip::default();
        cip.set_camera_filename(cam_file);
        cip.set_image_filename(image);
        cip.set_pms_filename(pms_file);
        let cip = cip.into();
        Self { cip }
    }

    #[wasm_bindgen(getter)]
    pub fn cam_file(&self) -> String {
        self.cip.borrow().camera_filename().into()
    }

    #[wasm_bindgen(getter)]
    pub fn image(&self) -> String {
        self.cip.borrow().image().to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn image_filename(&self) -> String {
        self.cip.borrow().image_filename().to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn pms_file(&self) -> String {
        self.cip.borrow().pms_filename().into()
    }

    #[wasm_bindgen(getter)]
    pub fn camera(&self) -> WasmCameraInstance {
        WasmCameraInstance::of_camera(self.cip.borrow().camera().clone())
    }

    //ap set_camera
    #[wasm_bindgen(setter)]
    pub fn set_camera(&mut self, wcamera: &WasmCameraInstance) {
        self.cip.borrow_mut().set_camera(wcamera.camera().clone());
    }

    //ap pms
    #[wasm_bindgen(getter)]
    pub fn pms(&self) -> WasmPointMappingSet {
        WasmPointMappingSet::of_pms(self.cip.borrow().pms().clone())
    }

    //mp locate
    pub fn locate(&self, max_np_error: f64, max_pairs: usize) {
        let filter = |_, pm: &PointMapping| pm.model_error() < max_np_error;
        self.cip.borrow_mut().locate(filter, max_pairs);
    }

    //mp orient_camera_using_model_directions
    pub fn orient_camera_using_model_directions(&self, max_np_error: f64) {
        let filter = |_, pm: &PointMapping| pm.model_error() < max_np_error;
        self.cip
            .borrow_mut()
            .orient_camera_using_model_directions(filter);
    }

    //zz All done
}
