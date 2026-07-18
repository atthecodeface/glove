use wasm_bindgen::prelude::*;

use ic_mapping::{PointMapping};

use crate::{WasmVec3f64, WasmNamedPoint};

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
#[derive(Debug, Clone, Default)]
pub struct WasmPointMapping {
    pub(crate) wasm_np: WasmNamedPoint,
    pub(crate) name_upper: String,
    /// True if the PMS details are valid is has a PMS mapping */
    pub(crate) has_pms: bool,
    /// Screen coordinate - only valid if has_pms
    pub(crate) screen: Point2D,
    /// Error in pixels - only valid if has_pms
    pub(crate) error: f64,
    /// Whether to use for initial orientation or not - only valid if has_pms
    pub(crate) usage: u64,
    /// Named point mapped onto the sensor pxy through the camera
    pub(crate) expected: Point2D,
    /// Distance from focus - only valid if has_pms
    pub(crate) focus_distance: f64,
    ///  - only valid if has_pms
  pub(crate) d_map_yaw_err: f64,
  ///  - only valid if has_pms
  pub(crate) d_map_roll_err: f64,
  ///  - only valid if has_pms
  pub(crate) d_map_distance: f64,
}

impl std::convert::From<&PointMapping> for WasmPointMapping {
    fn from(pm: &PointMapping) -> Self {
        let np = pm.named_point();
        let wasm_np = (&*np).into();
        let name_upper = &np.ref_tag().as_str().to_uppercase();
        Self { wasm_np,
            name_upper,
            has_pms: true,
        }
    }
}

impl WasmPointMapping {
    /// Create a new WasmPointMapping from a WasmNamedPoint with no mapping
    #[wasm_bindgen(constructor)]
    pub fn new(wasm_np: &WasmNamedPoint) {
        let mut s = Self::default();
        s.wasm_np = wasm_np.clone();
        s.name_upper = wasm_np.name.to_uppercase();

    this.mapped_nps = mapped_nps;
    this.wasm_np = wasm_np;
    this.name_upper = this.wasm_np.name.toUpperCase();
    this.expected_pxy = [0, 0];
    this.focus_dsq = 0;
  }

  /** Accessor */
  x(): number {
    return this.expected_pxy[0];
  }
  /** Accessor */
  y(): number {
    return this.expected_pxy[1];
  }
    /** Accessor */
  name(): string {
    return this.wasm_np.name;
  }
    /** Accessor */
  color(): string {
    return this.wasm_np.color;
  }

  color_select(parent: HtmlElement): HtmlElement {
    const div = parent.add_ele("div");
    div.add_input_color({ rgb_string: this.wasm_np.color }, this.set_color.bind(this));
    div.add_ele("br");
    div.add_span(this.wasm_np.color);
    return div;
  }

  set_color(color: string) {
    this.mapped_nps.project.nps_set_color(this.wasm_np.name, color);
  }

  uncertainty(): number {
    return 0;
  }

  map_model_with_camera(camera: WasmCameraInstance, focus: [number, number]) {
    const np_pxy = camera.map_model(this.wasm_np.model);
    this.expected_pxy = [np_pxy[0]!, np_pxy[1]!];
    const dx = this.expected_pxy[0] - focus[0];
    const dy = this.expected_pxy[1] - focus[1];
    this.focus_dsq = Math.sqrt(dx * dx + dy * dy);
  }

  get_pms_mapping(camera: WasmCameraInstance, pms: WasmPointMappingSet, n: number) {
    const pxye = pms.get_xy_err(n)!;
    this.has_pms = true;
    this.pms_x = pxye[0]!;
    this.pms_y = pxye[1]!;
    this.pms_error = pxye[2]!;
    const dx = this.expected_pxy[0] - this.pms_x;
    const dy = this.expected_pxy[1] - this.pms_y;
    this.d_map_sq = Math.sqrt(dx * dx + dy * dy);

    const wasm_vec2 = this.mapped_nps.wasm_vec2;
    const wasm_vec3 = this.mapped_nps.wasm_vec3;
    const wasm_quat = this.mapped_nps.wasm_quat;

    // Convert the placed mapped position to a roll/yaw
    //
    // Note that the sensor_dir_of_pt uses the sensor centre and pixel aspect
    // ratio to map to a pure positionq
    //
    // This does NOT use the lens mapping
    wasm_vec2.x = pxye[0]!;
    wasm_vec2.y = pxye[1]!;
    camera.set_sensor_dir_of_pt(wasm_vec2, wasm_vec3);
    const map_roll = camera.camera_roll_of_dir(wasm_vec3);
    wasm_quat.set_unit();
    wasm_quat.set_mul_rotate_z(-map_roll);
    wasm_vec3.set_apply_q3(wasm_quat);
    const placed_yaw = wasm_vec3.x / wasm_vec3.z;

    // Convert the NP expected position, given orientation and lens calibration,
    // to a yaw for yaw error
    //
    // This does NOT use the lens mapping - but the expected position did
    wasm_vec2.x = this.expected_pxy[0];
    wasm_vec2.y = this.expected_pxy[1];
    camera.set_sensor_dir_of_pt(wasm_vec2, wasm_vec3);

    // Rotate the direction for the NP expected position by -map_roll around -Z to
    // generate an (x,y,z) whose x is 'yaw' error, y is 'roll' error, scaled down by
    // z (which should be 1-epsilon)
    wasm_vec3.set_apply_q3(wasm_quat);

    this.d_map_yaw_err = 1000 * (wasm_vec3.x / wasm_vec3.z - placed_yaw);
    this.d_map_roll_err = 1000 * wasm_vec3.y / wasm_vec3.z;
  }

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
    pub fn get_name(&self, n: usize) -> Option<String> {
        self.pms
            .borrow()
            .mappings()
            .get(n)
            .map(|m| m.named_point().ref_tag().as_str().to_owned())
    }

    //mp get_xy
    /// Get the XY coords
    pub fn get_xy(&self, n: usize) -> Option<Box<[f64]>> {
        self.pms
            .borrow()
            .mappings()
            .get(n)
            .map(|m| [m.screen()[0], m.screen()[1]].into())
    }

    //mp get_xy_err
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
        let _color: Color8 = color.try_into()?;
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
    pub fn name(&self) -> String {
        (&self.name).into()
    }

    //mp color
    #[wasm_bindgen(getter)]
    pub fn color(&self) -> String {
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
        let color: Color8 = wnp.color.as_str().try_into()?;
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

    //zz All done
}
