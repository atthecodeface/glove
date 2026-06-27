//a Imports
use ic_base::Rrc;
use ic_mapping::PointMapping;
use ic_project::Cip;
use wasm_bindgen::prelude::*;

use crate::{WasmCameraInstance, WasmPointMappingSet};

//a WasmCip
#[wasm_bindgen]
#[derive(Debug)]
pub struct WasmCip {
    cip: Rrc<Cip>,
}

//ip WasmCip
impl WasmCip {
    //cp of_cip
    pub fn of_cip(cip: Rrc<Cip>) -> Self {
        Self { cip }
    }
    //cp cip
    pub fn cip(&self) -> &Rrc<Cip> {
        &self.cip
    }
}

//ip WasmCip
#[wasm_bindgen]
impl WasmCip {
    //cp new
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

    //ap cam_file
    #[wasm_bindgen(getter)]
    pub fn cam_file(&self) -> String {
        self.cip.borrow().camera_filename().into()
    }

    //ap image
    #[wasm_bindgen(getter)]
    pub fn image(&self) -> String {
        self.cip.borrow().image().to_string()
    }

    //ap image_filename
    #[wasm_bindgen(getter)]
    pub fn image_filename(&self) -> String {
        self.cip.borrow().image_filename().to_string()
    }

    //ap pms_file
    #[wasm_bindgen(getter)]
    pub fn pms_file(&self) -> String {
        self.cip.borrow().pms_filename().into()
    }

    //ap camera
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
