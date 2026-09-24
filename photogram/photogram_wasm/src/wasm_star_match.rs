use geo_nd_wasm::{WasmQuatf64, WasmVec3f64};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

use ic_photogram::Cip;
use ic_photogram::Point3D;

use crate::console_log;
use crate::star_catalog::{Catalog, StarMatchMapping, StarMatchMappingSet, Subcube};

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
            let v = p.sensor_as_unit_camera_dir(camera);
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
