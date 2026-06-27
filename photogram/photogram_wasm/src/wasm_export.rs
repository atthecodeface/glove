//a Imports
use std::default::Default;

use js_sys::Array;
use wasm_bindgen::prelude::*;

use photogram::json::remove_comments as json_remove_comments;
use photogram::prelude::*;

//a Helpful functions
//fi point3d
fn point3d(pt: &[f64]) -> Result<Point3D, String> {
    if pt.len() != 3 {
        Err("Expected model point (x,y,z)".into())
    } else {
        Ok([pt[0], pt[1], pt[2]].into())
    }
}

//fi point2d
fn point2d(pt: &[f64]) -> Result<Point2D, String> {
    if pt.len() != 2 {
        Err("Expected point (x,y)".into())
    } else {
        Ok([pt[0], pt[1]].into())
    }
}

//a WasmCameraDatabase
//tp WasmCameraDatabase
#[wasm_bindgen]
pub struct WasmCameraDatabase {
    cdb: Rrc<CameraDatabase>,
}

//ip WasmCameraDatabase
#[wasm_bindgen]
impl WasmCameraDatabase {
    //fp new
    /// Create a new WasmCameraDatabase from a Json file
    #[wasm_bindgen(constructor)]
    pub fn new(json: &str) -> Result<WasmCameraDatabase, JsValue> {
        let json = json_remove_comments(json);
        let cdb = CameraDatabase::from_json(&json)
            .map_err(|e| e.to_string())?
            .into();
        Ok(Self { cdb })
    }

    //cp of_cdb
    fn of_cdb(cdb: Rrc<CameraDatabase>) -> Self {
        Self { cdb }
    }
}

//a WasmRay
//tp WasmRay
#[wasm_bindgen]
pub struct WasmRay {
    ray: Ray,
}

//ip WasmRay
#[wasm_bindgen]
impl WasmRay {
    //fp new
    /// Create a new WasmRay
    #[wasm_bindgen(constructor)]
    pub fn new(start: &[f64], dirn: &[f64], tan_error: Option<f64>) -> Result<WasmRay, String> {
        let tan_error = tan_error.unwrap_or(0.01);
        let ray = Ray::default()
            .set_start(point3d(start)?)
            .set_direction(point3d(dirn)?)
            .set_tan_error(tan_error);
        Ok(Self { ray })
    }

    //mp model_at_distance
    pub fn model_at_distance(&self, distance: f64) -> Result<Box<[f64]>, String> {
        let model: [f64; 3] = (self.ray.start + self.ray.direction * distance).into();
        Ok(Box::new(model))
    }

    //mp closest_model_to_intersection
    pub fn closest_model_to_intersection(rays: Vec<WasmRay>) -> Option<Box<[f64]>> {
        let ray_list: Vec<Ray> = rays.into_iter().map(|r| r.ray).collect();
        if let Some(model) = Ray::closest_point(&ray_list, &|_r| 1.0) {
            let model: [f64; 3] = model.into();
            Some(Box::new(model))
        } else {
            None
        }
    }
}

//a WasmCameraInstance
//tp WasmCameraInstance
#[wasm_bindgen]
pub struct WasmCameraInstance {
    camera: Rrc<CameraInstance>,
}

//ip WasmCameraInstance
#[wasm_bindgen]
impl WasmCameraInstance {
    //fp new
    /// Create a new WasmCameraInstance from a camera database and a Json file
    #[wasm_bindgen(constructor)]
    pub fn new(cdb: &WasmCameraDatabase, json: &str) -> Result<WasmCameraInstance, JsValue> {
        let json = json_remove_comments(json);
        let camera = CameraInstance::from_json(&cdb.cdb.borrow(), &json)?.into();
        Ok(Self { camera })
    }

    //cp of_camera
    fn of_camera(camera: Rrc<CameraInstance>) -> Self {
        Self { camera }
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
    #[wasm_bindgen(getter)]
    pub fn rotation(&self) -> usize {
        self.camera.borrow().rotation()
    }

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
        let pxy: [f64; 2] = self.camera.borrow().map_model(point3d(pt)?).into();
        Ok(Box::new(pxy))
    }

    //mp direction_of_pt
    pub fn direction_of_pt(&self, pt: &[f64]) -> Result<Box<[f64]>, String> {
        let txty = self.camera.borrow().px_abs_xy_to_camera_txty(point2d(pt)?);
        let model_dir: [f64; 3] = self.camera.borrow().camera_txty_to_world_dir(&txty).into();
        Ok(Box::new(model_dir))
    }

    //mp get_pm_as_ray
    pub fn get_pm_as_ray(
        &self,
        wpms: &WasmPointMappingSet,
        n: usize,
        from_camera: bool,
    ) -> Result<WasmRay, String> {
        if n < wpms.pms.borrow().mappings().len() {
            let ray = self
                .camera
                .borrow()
                .get_pm_as_ray(&wpms.pms.borrow().mappings()[n], from_camera);
            Ok(WasmRay { ray })
        } else {
            Err("PM index out of range".into())
        }
    }

    //mp model_at_distance
    pub fn model_at_distance(&self, pt: &[f64], distance: f64) -> Result<Box<[f64]>, String> {
        let txty = self.camera.borrow().px_abs_xy_to_camera_txty(point2d(pt)?);
        let model = self.camera.borrow().position()
            - self.camera.borrow().camera_txty_to_world_dir(&txty) * distance;
        let model: [f64; 3] = model.into();
        Ok(Box::new(model))
    }

    //cp to_json
    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<String, JsValue> {
        Ok(self.camera.borrow().to_json()?)
    }

    //mp locate_using_model_lines
    pub fn locate_using_model_lines(
        &mut self,
        wpms: &WasmPointMappingSet,
        max_np_error: f64,
    ) -> f64 {
        self.camera
            .borrow_mut()
            .locate_using_model_lines(&wpms.pms.borrow(), max_np_error)
    }

    //mp orient_using_rays_from_model
    pub fn orient_using_rays_from_model(&mut self, wpms: &WasmPointMappingSet) -> f64 {
        self.camera
            .borrow_mut()
            .orient_using_rays_from_model(wpms.pms.borrow().mappings())
    }

    //mp reorient_using_rays_from_model
    pub fn reorient_using_rays_from_model(&mut self, wpms: &WasmPointMappingSet) -> f64 {
        self.camera
            .borrow_mut()
            .reorient_using_rays_from_model(wpms.pms.borrow().mappings())
    }

    //zz All done
}

//a WasmPointMappingSet
//tp WasmPointMappingSet
#[wasm_bindgen]
pub struct WasmPointMappingSet {
    pms: Rrc<PointMappingSet>,
}

//ip WasmPointMappingSet
#[wasm_bindgen]
impl WasmPointMappingSet {
    //fp new
    /// Create a new WasmPointMappingSet from a camera database and a Json file
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmPointMappingSet {
        let pms = PointMappingSet::default().into();
        Self { pms }
    }

    //cp of_pms
    fn of_pms(pms: Rrc<PointMappingSet>) -> Self {
        Self { pms }
    }

    //mp read_json
    /// Read a json file to add to the points
    pub fn read_json(&mut self, wnps: &WasmNamedPointSet, json: &str) -> Result<(), JsValue> {
        let json = json_remove_comments(json);
        let nf = self
            .pms
            .borrow_mut()
            .read_json(&wnps.nps.borrow(), &json, true)?;
        // if !nf.is_empty() {
        // eprintln!("Warning: {}", nf);
        // }
        Ok(())
    }

    //cp to_json
    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<String, JsValue> {
        Ok(self.pms.borrow().to_json()?)
    }

    //mp length
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
            .map(|m| m.model.name().into())
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

    //mp mapping_of_name
    pub fn mapping_of_name(&self, name: &str) -> Option<usize> {
        self.pms
            .borrow()
            .mappings()
            .iter()
            .enumerate()
            .find(|(_, m)| m.name() == name)
            .map(|(n, _)| n)
            .into()
    }

    //mp add_mapping
    pub fn add_mapping(
        &mut self,
        wnps: &WasmNamedPointSet,
        name: &str,
        screen: &[f64],
        error: f64,
    ) -> Result<bool, String> {
        Ok(self
            .pms
            .borrow_mut()
            .add_mapping(&wnps.nps.borrow(), name, &point2d(screen)?, error))
    }

    //mp remove_mapping
    pub fn remove_mapping(&mut self, n: usize) -> Result<(), String> {
        if !self.pms.borrow_mut().remove_mapping(n) {
            Err("Index out of range".into())
        } else {
            Ok(())
        }
    }

    //zz All done
}

//a WasmNamedPoint
//tp WasmNamedPoint
#[wasm_bindgen]
pub struct WasmNamedPoint {
    name: String,
    color: String,
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
        Ok(Self {
            name,
            color,
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

    //mp model
    #[wasm_bindgen(getter)]
    pub fn model(&self) -> Box<[f64]> {
        Box::new(self.model)
    }

    //mp error
    #[wasm_bindgen(getter)]
    pub fn error(&self) -> f64 {
        self.error
    }
}

//a WasmNamedPointSet
//tp WasmNamedPointSet
/// A set of named points
#[wasm_bindgen]
pub struct WasmNamedPointSet {
    nps: Rrc<NamedPointSet>,
}

//ip WasmNamedPointSet
#[wasm_bindgen]
impl WasmNamedPointSet {
    //cp new
    /// Create a new WasmGraphCanvas attached to a Canvas HTML element,
    /// adding events to the canvas that provide the paint program
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmNamedPointSet, JsValue> {
        let nps = Rrc::<NamedPointSet>::default();
        Ok(Self { nps })
    }

    //cp of_nps
    fn of_nps(nps: Rrc<NamedPointSet>) -> Self {
        Self { nps }
    }

    //cp read_json
    #[wasm_bindgen]
    pub fn read_json(&mut self, json: &str) -> Result<(), JsValue> {
        let json = json_remove_comments(json);
        let nps = NamedPointSet::from_json(&json)?;
        self.nps.borrow_mut().merge(&nps);
        Ok(())
    }

    //cp to_json
    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<String, JsValue> {
        Ok(self.nps.borrow().to_json()?)
    }

    //mp add_pt
    #[wasm_bindgen]
    pub fn add_pt(&mut self, wnp: WasmNamedPoint) -> Result<(), JsValue> {
        let color: Color = wnp.color.as_str().try_into()?;
        self.nps
            .borrow_mut()
            .add_pt(&wnp.name, color, Some(wnp.model.into()), wnp.error);
        Ok(())
    }

    //mp get_pt
    #[wasm_bindgen]
    pub fn get_pt(&mut self, name: &str) -> Option<WasmNamedPoint> {
        if let Some(np) = self.nps.borrow().get_pt(name) {
            let (model, error) = np.model();
            let wnp = WasmNamedPoint {
                name: name.into(),
                color: np.color().as_string(),
                model: model.into(),
                error,
            };
            Some(wnp)
        } else {
            None
        }
    }

    //mp pts
    #[wasm_bindgen]
    pub fn pts(&mut self) -> Result<Array, JsValue> {
        let names = js_sys::Array::new();
        for (name, _) in self.nps.borrow().iter() {
            let name: JsValue = name.into();
            names.push(&name);
        }
        Ok(names)
    }

    //mp set_model
    pub fn set_model(&self, name: &str, model: &[f64], error: f64) -> Result<(), String> {
        if let Some(np) = self.nps.borrow().get_pt(name) {
            np.set_model(Some((point3d(model)?, error)));
            Ok(())
        } else {
            Err("Could not find named point".into())
        }
    }

    //mp unset_model
    pub fn unset_model(&self, name: &str) -> Result<(), String> {
        if let Some(np) = self.nps.borrow().get_pt(name) {
            np.set_model(None);
            Ok(())
        } else {
            Err("Could not find named point".into())
        }
    }

    //zz All done
}

//a WasmCip
#[wasm_bindgen]
#[derive(Debug)]
pub struct WasmCip {
    cip: Rrc<Cip>,
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
        cip.set_camera_file(cam_file);
        cip.set_image(image);
        cip.set_pms_file(pms_file);
        let cip = cip.into();
        Self { cip }
    }

    //cp of_cip
    fn of_cip(cip: Rrc<Cip>) -> Self {
        Self { cip }
    }

    //ap cam_file
    #[wasm_bindgen(getter)]
    pub fn cam_file(&self) -> String {
        self.cip.borrow().camera_file().into()
    }

    //ap img
    #[wasm_bindgen(getter)]
    pub fn img(&self) -> String {
        self.cip.borrow().image().into()
    }

    //ap pms_file
    #[wasm_bindgen(getter)]
    pub fn pms_file(&self) -> String {
        self.cip.borrow().pms_file().into()
    }

    //ap camera
    #[wasm_bindgen(getter)]
    pub fn camera(&self) -> WasmCameraInstance {
        WasmCameraInstance::of_camera(self.cip.borrow().camera().clone())
    }

    //ap set_camera
    #[wasm_bindgen(setter)]
    pub fn set_camera(&mut self, wcamera: &WasmCameraInstance) {
        self.cip.borrow_mut().set_camera(wcamera.camera.clone());
    }

    //ap pms
    #[wasm_bindgen(getter)]
    pub fn pms(&self) -> WasmPointMappingSet {
        WasmPointMappingSet::of_pms(self.cip.borrow().pms().clone())
    }

    //zz All done
}

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
        Ok(self.project.to_json(pretty)?)
    }

    //cp read_json
    #[wasm_bindgen]
    pub fn read_json(&mut self, json: &str) -> Result<(), JsValue> {
        let json = json_remove_comments(json);
        self.project = Project::from_json(&json)?;
        Ok(())
    }

    //ap cdb
    #[wasm_bindgen(getter)]
    pub fn cdb(&self) -> WasmCameraDatabase {
        WasmCameraDatabase::of_cdb(self.project.cdb().clone())
    }

    //ap set_cdb
    #[wasm_bindgen(setter)]
    pub fn set_cdb(&mut self, wcdb: &WasmCameraDatabase) {
        // unsafe {
        // crate::console_log!("Log {:?}", wcdb.cdb);
        // }
        self.project.set_cdb(wcdb.cdb.clone());
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
        self.project.set_nps(wnps.nps.clone());
    }

    //mp add_cip
    pub fn add_cip(&mut self, cip: &WasmCip) -> usize {
        self.project.add_cip(cip.cip.clone())
    }

    //ap ncips
    pub fn ncips(&self) -> usize {
        self.project.ncips()
    }

    //mp cip
    pub fn cip(&self, n: usize) -> Result<WasmCip, String> {
        if n >= self.project.ncips() {
            Err("Cip index out of range".into())
        } else {
            Ok(WasmCip::of_cip(self.project.cip(n).clone()))
        }
    }

    //mp cip_read_json
    pub fn cip_read_json(
        &self,
        n: usize,
        camera_json: &str,
        pms_json: &str,
    ) -> Result<String, String> {
        if n >= self.project.ncips() {
            Err("Cip index out of range".into())
        } else {
            self.project
                .cip(n)
                .borrow_mut()
                .read_json(&self.project, camera_json, pms_json)
        }
    }

    //mp locate_all
    pub fn locate_all(&self, max_np_error: f64) {
        self.project.locate_all(max_np_error);
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
