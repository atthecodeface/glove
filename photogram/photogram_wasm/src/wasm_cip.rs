use std::borrow::Borrow;
use std::rc::Rc;

use geo_nd_wasm::{WasmQuatf64, WasmVec3f64};
use ic_base::{Point3D, Rrc};
use ic_camera::CameraProjection;
use ic_mapping::PointMapping;
use ic_project::{Cip, CipDesc};
use star_catalog_wasm::star_catalog::StarFilter;
use star_catalog_wasm::{WasmCatalog, console_log};
use wasm_bindgen::prelude::*;

use crate::star_catalog::{Catalog, StarMatchMapping, StarMatchMappingSet, Subcube};
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
        let filter = |_, pm: &PointMapping| pm.model_uncertainty() < max_np_error;
        self.cip.borrow_mut().locate(filter, max_pairs);
    }

    //mp orient_camera_using_model_directions
    pub fn orient_camera_using_model_directions(&self, max_np_error: f64) {
        let filter = |_, pm: &PointMapping| pm.model_uncertainty() < max_np_error;
        self.cip
            .borrow_mut()
            .orient_camera_using_model_directions(filter);
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
        let world_dir = xy.get_mapped_world_dir(&*camera).into();
        *vec = world_dir;
        true
    }
}

#[wasm_bindgen]
pub struct WasmStarMatch {
    /// The individual mappings have quaternions mapping image space to star space
    mapping: StarMatchMapping,
}

impl std::convert::From<&StarMatchMapping> for WasmStarMatch {
    fn from(mapping: &StarMatchMapping) -> Self {
        let mapping: StarMatchMapping = (*mapping).clone();
        Self { mapping }
    }
}

#[wasm_bindgen]
impl WasmStarMatch {
    #[wasm_bindgen(getter)]
    pub fn star(&self) -> usize {
        self.mapping.star.as_usize()
    }
    #[wasm_bindgen(getter)]
    pub fn img_index(&self) -> usize {
        self.mapping.img_index
    }
    #[wasm_bindgen(getter)]
    pub fn ordering(&self) -> f64 {
        self.mapping.ordering
    }
    #[wasm_bindgen(getter)]
    pub fn quality(&self) -> f64 {
        self.mapping.quality
    }
    #[wasm_bindgen(getter)]
    pub fn img_vector(&self) -> WasmVec3f64 {
        self.mapping.img_vector.into()
    }
    #[wasm_bindgen(getter)]
    pub fn star_vector(&self) -> WasmVec3f64 {
        self.mapping.star_vector.into()
    }
}

#[derive(Clone)]
#[wasm_bindgen]
pub struct WasmStarMatchMappingSet {
    matches: Rc<StarMatchMappingSet>,
}

impl std::convert::From<StarMatchMappingSet> for WasmStarMatchMappingSet {
    fn from(matches: StarMatchMappingSet) -> Self {
        let matches = matches.into();
        Self { matches }
    }
}

#[wasm_bindgen]
impl WasmStarMatchMappingSet {
    pub fn set_quat(&self, quat: &mut WasmQuatf64) {
        *(quat.as_mut()) = self.matches.q;
    }

    #[wasm_bindgen(getter)]
    pub fn quality(&self) -> f64 {
        self.matches.quality
    }

    #[wasm_bindgen(getter)]
    pub fn angle_mean(&self) -> f64 {
        self.matches.angle_mean.to_degrees()
    }

    #[wasm_bindgen(getter)]
    pub fn num_mappings(&self) -> usize {
        self.matches.mappings.len()
    }

    pub fn mapping(&self, idx: usize) -> Option<WasmStarMatch> {
        self.matches.mappings.get(idx).map(|s| s.into())
    }

    // pub initial_match: StarTriangleMatch,
    // pub mappings: Vec<StarMatchMapping>,
}

#[wasm_bindgen]
pub struct WasmStarMatchSet {
    /// The match sets have quaternions mapping image space to star space
    match_sets: Vec<WasmStarMatchMappingSet>,
    more: bool,
}

impl WasmStarMatchSet {
    pub fn new(cip: &Cip, catalog: &Catalog, max_angle_delta: f64, max_candidates: usize) -> Self {
        let mut img_space_vectors: Vec<[f64; 3]> = vec![];
        let pms = cip.pms().borrow();
        let camera = &*cip.camera().borrow();
        for p in pms.mappings() {
            let v = p.get_mapped_camera_dir(camera);
            // let n = img_space_vectors.len() - 1;
            // img_space_vectors[n][1] *= -1.0;
            //
            console_log!("{v:0.4}");
            img_space_vectors.push(v.into());
        }
        if false {
            use geo_nd::Vector;
            let n = img_space_vectors.len();
            for (i, v) in img_space_vectors.iter().enumerate() {
                let p0: Point3D = v.into();
                let p1: Point3D = img_space_vectors[(i + 1) % n].into();
                let angle = p0.dot(&p1).acos().to_degrees();
                console_log!("angle {angle}");
            }
        }
        let subcube_iter = Subcube::iter_all();
        let (more, mut match_sets) = catalog.find_best_star_mappings(
            subcube_iter,
            &img_space_vectors,
            max_angle_delta,
            max_candidates,
        );
        match_sets.sort_by(|a, b| a.quality.partial_cmp(&b.quality).unwrap());
        for m in match_sets.iter().take(10) {
            console_log!("mean {} quality {}", m.angle_mean.to_degrees(), m.quality);
        }

        let match_sets = match_sets.into_iter().map(|s| s.into()).collect();
        Self { match_sets, more }
    }
}

#[wasm_bindgen]
impl WasmStarMatchSet {
    pub fn has_more(&self) -> bool {
        self.more
    }
    pub fn num_match_sets(&self) -> usize {
        self.match_sets.len()
    }
    pub fn match_sets_num_matches(&self) -> usize {
        self.match_sets.len()
    }
    pub fn get_match(&self, index: usize) -> Option<WasmStarMatchMappingSet> {
        self.match_sets.get(index).cloned()
    }
}
