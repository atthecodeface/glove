use std::borrow::Borrow;

use geo_nd_wasm::WasmVec3f64;
use star_catalog_wasm::WasmCatalog;
use star_catalog_wasm::star_catalog::StarFilter;
use wasm_bindgen::prelude::*;

use ic_photogram::PointMapping;
use ic_photogram::Rrc;
use ic_photogram::{Cip, CipDesc, JsonSrc};

use crate::{WasmCameraInstance, WasmPointMappingSet, WasmStarMatchSet, err_to_string};

#[wasm_bindgen]
#[derive(Debug)]
pub struct WasmCipDesc(CipDesc);

#[wasm_bindgen]
impl WasmCipDesc {
    /// Try to parse a Json string as a CipDesc without a Project
    pub fn try_json(json: &str) -> Result<Self, JsValue> {
        let json = JsonSrc::<CipDesc>::of_json(json).map_err(err_to_string)?;
        let (_, cip_desc) = json
            .deserialize_as::<CipDesc>("Cip")
            .map_err(err_to_string)?;
        Ok(Self(cip_desc))
    }

    #[wasm_bindgen(getter)]
    pub fn num_mappings(&self) -> usize {
        self.0.num_mappings()
    }

    #[wasm_bindgen(getter)]
    pub fn image(&self) -> String {
        let t: &str = self.0.image().borrow();
        t.to_owned()
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
    pub fn new(image: &str) -> WasmCip {
        let mut cip = Cip::new(image);
        cip.set_image_filename(image);
        let cip = cip.into();
        Self { cip }
    }

    #[wasm_bindgen(getter)]
    pub fn image(&self) -> String {
        self.cip.borrow().name_as_tag().to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn image_filename(&self) -> String {
        self.cip.borrow().image_filename().to_string()
    }

    #[wasm_bindgen(getter)]
    pub fn camera(&self) -> WasmCameraInstance {
        self.cip.borrow().camera().clone().into()
    }

    //ap set_camera
    #[wasm_bindgen(setter)]
    pub fn set_camera(&mut self, wcamera: &WasmCameraInstance) {
        self.cip.borrow_mut().set_camera(wcamera.clone_camera());
    }

    //ap pms
    #[wasm_bindgen(getter)]
    pub fn pms(&self) -> WasmPointMappingSet {
        WasmPointMappingSet::of_pms(self.cip.borrow().pms().clone())
    }

    //mp locate
    pub fn locate(&self, max_np_error: f64, max_pairs: usize) -> Result<(), String> {
        let filter = |_, pm: &PointMapping| pm.model_uncertainty() < max_np_error;
        self.cip
            .borrow_mut()
            .locate(filter, max_pairs)
            .map_err(err_to_string)?;
        Ok(())
    }

    //mp orient_camera_using_model_directions
    pub fn orient_camera_using_model_directions(&self, max_np_error: f64) -> Result<(), String> {
        let filter = |_, pm: &PointMapping| pm.model_uncertainty() < max_np_error;
        let weighting = |_pm: &PointMapping| 1.0;
        self.cip
            .borrow_mut()
            .orient_camera_using_model_directions(filter, weighting)
            .map_err(err_to_string)?;
        Ok(())
    }

    pub fn adjust_camera_orientation_using_dxy2(
        &self,
        max_np_error: f64,
        angle: f64,
        num_steps: usize,
    ) -> Result<f64, String> {
        let filter = |_, pm: &PointMapping| pm.model_uncertainty() < max_np_error;
        let weighting = |_pm: &PointMapping| 1.0;
        Ok(self
            .cip
            .borrow_mut()
            .adjust_camera_orientation_using_dxy2(filter, weighting, angle, num_steps)
            .map_err(err_to_string)?)
    }

    /// Generate an array of 3N values, each being a triplet of PM number, world yaw and sensor yaw for that PM
    ///
    /// Only include points mappings that map to named points, and whose named point is placed
    pub fn generate_pm_world_sensor_yaw(&self) -> Vec<f64> {
        let filter = |_, _pm: &PointMapping| true;
        let mut result = vec![];
        for (pm, _, world_yaw, _, sensor_yaw) in
            self.cip.borrow_mut().generate_pm_world_sensor_data(filter)
        {
            result.push(pm as f64);
            result.push(world_yaw);
            result.push(sensor_yaw);
        }
        result
    }

    pub fn stars_of_pms(
        &self,
        catalog: &WasmCatalog,
        max_angle_delta: f64,
        max_candidates: usize,
    ) -> WasmStarMatchSet {
        let max_angle_delta = max_angle_delta.to_radians();
        let cip = self.cip.borrow();
        let mut catalog = catalog.catalog_mut();
        catalog.clear_filter();
        catalog.add_filter(StarFilter::brighter_than(5.0));

        WasmStarMatchSet::new(&cip, &mut catalog, max_angle_delta, max_candidates)
    }

    /// Find the *world* direction of a point mapping, and set the WasmVec3f64 for it
    ///
    /// This applies the Lens Mapping and camera orientation
    pub fn set_pms_world_dir_vec(&self, pm: usize, vec: &mut WasmVec3f64) -> bool {
        let cip = self.cip.borrow();
        let pms = cip.pms().borrow();
        let Some(xy) = pms.mappings().get(pm) else {
            return false;
        };
        let camera = cip.camera().borrow();
        let world_dir = xy.sensor_as_unit_world_dir(&*camera).into();
        *vec = world_dir;
        true
    }
}
