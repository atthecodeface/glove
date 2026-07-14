use geo_nd_wasm::{Quatf64, Vec3f64, WasmQuatf64, WasmVec2f64, WasmVec3f64};
use star_catalog_wasm::star_catalog::Vec3f32;
//a Imports
use wasm_bindgen::prelude::*;

use ic_base::{JsonParsable, Point2D, Point3D, Rrc};
use ic_camera::{
    CameraDatabase, CameraInstance, CameraInstanceDesc, CameraProjection, CameraSensor,
};

use crate::{ToFromWasmArr, WasmPointMappingSet, WasmRay, err_to_string};

#[wasm_bindgen]
pub struct WasmCameraDatabase {
    cdb: Rrc<CameraDatabase>,
}

impl WasmCameraDatabase {
    pub fn of_cdb(cdb: Rrc<CameraDatabase>) -> Self {
        Self { cdb }
    }

    pub fn cdb(&self) -> &Rrc<CameraDatabase> {
        &self.cdb
    }
}

#[wasm_bindgen]
impl WasmCameraDatabase {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let cdb = CameraDatabase::default().into();
        Self { cdb }
    }

    pub fn of_json(json: &str) -> Result<Self, JsValue> {
        let cdb = CameraDatabase::load_json(json, &())
            .map_err(err_to_string)?
            .into();
        Ok(Self { cdb })
    }
    pub fn to_json(&self, pretty: bool) -> Result<String, JsValue> {
        Ok(self.cdb.borrow().to_json(pretty).map_err(err_to_string)?)
    }
    pub fn num_bodies(&self) -> usize {
        self.cdb.borrow().bodies().len()
    }
    pub fn num_lenses(&self) -> usize {
        self.cdb.borrow().lenses().len()
    }
    pub fn body_name(&self, idx: usize) -> Option<String> {
        self.cdb
            .borrow()
            .bodies()
            .get(idx)
            .map(|c| c.name().to_owned())
    }
    pub fn lens_name(&self, idx: usize) -> Option<String> {
        self.cdb
            .borrow()
            .lenses()
            .get(idx)
            .map(|c| c.name().to_owned())
    }
}

#[wasm_bindgen]
pub struct WasmCameraInstance {
    camera: Rrc<CameraInstance>,
}

//ip WasmCameraInstance
impl WasmCameraInstance {
    pub fn of_camera(camera: Rrc<CameraInstance>) -> Self {
        Self { camera }
    }

    pub fn camera(&self) -> &Rrc<CameraInstance> {
        &self.camera
    }
}

#[wasm_bindgen]
impl WasmCameraInstance {
    #[wasm_bindgen(constructor)]
    pub fn new(cdb: &WasmCameraDatabase, json: &str) -> Result<WasmCameraInstance, JsValue> {
        let camera = CameraInstanceDesc::load_json(json, &cdb.cdb.borrow())
            .map_err(err_to_string)?
            .into();
        Ok(Self { camera })
    }

    #[wasm_bindgen(getter)]
    pub fn body(&self) -> String {
        self.camera.borrow().camera_name().into()
    }

    #[wasm_bindgen(getter)]
    pub fn lens(&self) -> String {
        self.camera.borrow().lens_name().into()
    }

    #[wasm_bindgen(getter)]
    pub fn focal_length(&self) -> f64 {
        self.camera.borrow().focal_length()
    }

    #[wasm_bindgen(getter)]
    pub fn tan_hfovd(&self) -> f64 {
        let txty = self.camera.borrow().tan_hfov();
        (txty.0 * txty.0 + txty.1 * txty.1).sqrt()
    }

    #[wasm_bindgen(getter)]
    pub fn tan_hfovh(&self) -> f64 {
        self.camera.borrow().tan_hfov().0
    }

    #[wasm_bindgen(getter)]
    pub fn tan_hfovv(&self) -> f64 {
        self.camera.borrow().tan_hfov().1
    }

    #[wasm_bindgen(getter)]
    pub fn position(&self) -> WasmVec3f64 {
        self.camera.borrow().position().into()
    }

    #[wasm_bindgen(setter)]
    pub fn set_position(&mut self, position: &WasmVec3f64) {
        let position: Vec3f64 = position.into();
        self.camera.borrow_mut().set_position(&position);
    }

    #[wasm_bindgen(getter)]
    pub fn orientation(&self) -> WasmQuatf64 {
        self.camera.borrow().orientation().into()
    }

    /// Set a WasmQuatf64 to the camera's orientation
    pub fn orientation_set_quat(&self, q: &mut WasmQuatf64) {
        *q.as_mut() = self.camera.borrow().orientation();
    }

    #[wasm_bindgen(setter)]
    pub fn set_orientation(&mut self, orientation: &WasmQuatf64) {
        let position: Quatf64 = orientation.into();
        self.camera.borrow_mut().set_orientation(&position);
    }

    #[wasm_bindgen(getter)]
    pub fn focus_distance(&self) -> f64 {
        self.camera.borrow().focus_distance()
    }

    #[wasm_bindgen(setter)]
    pub fn set_focus_distance(&mut self, mm_focus_distance: f64) {
        self.camera
            .borrow_mut()
            .set_focus_distance(mm_focus_distance);
    }

    #[wasm_bindgen(getter)]
    pub fn sensor_cx(&self) -> f64 {
        self.camera.borrow().sensor_px_center()[0]
    }

    #[wasm_bindgen(getter)]
    pub fn sensor_cy(&self) -> f64 {
        self.camera.borrow().sensor_px_center()[1]
    }

    #[wasm_bindgen(getter)]
    pub fn sensor_width(&self) -> f64 {
        self.camera.borrow().sensor_px_size().0
    }

    #[wasm_bindgen(getter)]
    pub fn sensor_height(&self) -> f64 {
        self.camera.borrow().sensor_px_size().1
    }

    pub fn map_model(&self, pt: &[f64]) -> Result<Box<[f64]>, String> {
        Ok(Point2D::to_wasm(
            self.camera
                .borrow()
                .world_xyz_to_px_abs_xy(&Point3D::from_wasm(pt)?),
        ))
    }

    pub fn direction_of_pt(&self, pt: &[f64]) -> Result<Box<[f64]>, String> {
        let txty = self
            .camera
            .borrow()
            .px_abs_xy_to_camera_txty(&Point2D::from_wasm(pt)?);
        Ok(Point3D::to_wasm(
            self.camera.borrow().camera_txty_to_world_dir(&txty),
        ))
    }

    /// Take a point on the sensor and map it to the direction of a ray relative
    /// to the outside of the camera lens
    ///
    /// This does use the lens mapping
    pub fn set_camera_dir_of_pt(&self, pt: &WasmVec2f64, dir: &mut WasmVec3f64) {
        let pt: Point2D = pt.into();
        let txty = self.camera.borrow().px_abs_xy_to_camera_txty(&pt);
        dir.set_array(txty.to_unit_vector().as_ref());
    }

    /// Take the direction of a ray relative to the outside of the camera lens
    /// and map it to a point on the sensor
    ///
    /// This does use the lens mapping
    pub fn set_pt_of_camera_dir(&self, dir: &WasmVec3f64, pt: &mut WasmVec2f64) {
        let dir: Point3D = dir.into();
        let txty = dir.into();
        let pxy = self.camera.borrow().camera_txty_to_px_abs_xy(&txty);
        pt.set_array(pxy.as_ref());
    }

    /// Take the direction of a ray relative to the outside of the camera lens
    /// and map it to ray relative to the sensor
    ///
    /// This does use the lens mapping
    pub fn set_map_sensor_dir_to_camera_dir(&self, dir: &mut WasmVec3f64) {
        let pt: Point3D = (&*dir).into();
        let txty = pt.into();
        let txty = self.camera.borrow().sensor_txty_to_camera_txty(&txty);
        dir.set_array(txty.to_unit_vector().as_ref());
    }

    /// Take the direction of a ray relative to the outside of the camera lens
    /// and map it to ray relative to the sensor
    ///
    /// This does use the lens mapping
    pub fn set_map_camera_dir_to_sensor_dir(&self, dir: &mut WasmVec3f64) {
        let pt: Point3D = (&*dir).into();
        let txty = pt.into();
        let txty = self.camera.borrow().camera_txty_to_sensor_txty(&txty);
        dir.set_array(txty.to_unit_vector().as_ref());
    }

    /// Take the direction of a ray in world space, accounting for camera orientation,
    /// and map it to ray relative to the sensor
    ///
    /// This does use the lens mapping
    pub fn set_map_world_dir_to_sensor_dir(&self, dir: &mut WasmVec3f64) {
        let pt: Point3D = (&*dir).into();
        let xyz = self.camera.borrow().world_dir_to_camera_xyz(&pt);
        let txty = xyz.into();
        let txty = self.camera.borrow().camera_txty_to_sensor_txty(&txty);
        dir.set_array(txty.to_unit_vector().as_ref());
    }

    pub fn map_yaw_world_to_sensor(&self, yaw: f64) -> f64 {
        let ry = ic_base::RollYaw::of_yaw(yaw);
        self.camera.borrow().camera_ry_to_sensor_ry(&ry).yaw()
    }

    pub fn map_yaw_sensor_to_world(&self, yaw: f64) -> f64 {
        let ry = ic_base::RollYaw::of_yaw(yaw);
        self.camera.borrow().sensor_ry_to_camera_ry(&ry).yaw()
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
