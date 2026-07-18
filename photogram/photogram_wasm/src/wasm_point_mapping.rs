use geo_nd::{Quaternion, Vector};
use geo_nd_wasm::WasmVec2f64;
use ic_camera::CameraProjection;
use wasm_bindgen::prelude::*;

use ic_base::{Point2D, RollYaw};
use ic_mapping::PointMapping;

use crate::{
    Quatf64, WasmCameraInstance, WasmNamedPoint, WasmNamedPointSet, WasmPointMappingSet,
    WasmVec3f64,
};

/*
 * A WasmPointMapping is a mapping of a specific WasmNamedPoint
 *
 * The WasmNamedPoint here is immutable - it should perhaps be mutable in the sense that 'set it from the current NamedPointSet'
 *
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
    /// Distance from cursor - only valid if has_pms
    pub(crate) cursor_distance: f64,
    /// Roll/Yaw of mapped point
    pub(crate) screen_ry: RollYaw,
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
        let wasm_np = (&**np).into();
        let name_upper = np.ref_tag().as_str().to_uppercase();
        let mut s = Self::default();
        s.wasm_np = wasm_np;
        s.name_upper = name_upper;
        s.has_pms = false;
        s
    }
}

#[wasm_bindgen]
impl WasmPointMapping {
    /// Create a new WasmPointMapping from a WasmNamedPoint with no mapping
    #[wasm_bindgen(constructor)]
    pub fn new(wasm_np: &WasmNamedPoint) -> Self {
        let mut s = Self::default();
        s.wasm_np = wasm_np.clone();
        s.name_upper = wasm_np.name.to_uppercase();
        s
    }

    /// The name of the NamedPoint
    #[wasm_bindgen(getter)]
    pub fn np_name(&self) -> String {
        (&self.wasm_np.name).into()
    }

    /// The name of the NamedPoint in upper case
    #[wasm_bindgen(getter)]
    pub fn np_name_upper(&self) -> String {
        (&self.name_upper).into()
    }

    /// The color associated with the NamedPoint
    #[wasm_bindgen(getter)]
    pub fn np_color(&self) -> String {
        (&self.wasm_np.color).into()
    }

    /// True if the NamedPoint is mapped
    #[wasm_bindgen(getter)]
    pub fn np_is_mapped(&self) -> bool {
        self.wasm_np.mapped
    }

    /// True if the NamedPoint maps to a direction, not a position in world space
    #[wasm_bindgen(getter)]
    pub fn np_at_infinity(&self) -> bool {
        self.wasm_np.at_infinity
    }

    /// Set a WasmVec3f64 to the model direction/position
    pub fn np_model_set_vec(&self, v: &mut WasmVec3f64) {
        v.set_array(self.wasm_np.model.as_ref());
    }

    /// The uncertainty of the model position/direction
    #[wasm_bindgen(getter)]
    pub fn np_uncertainty(&self) -> f64 {
        self.wasm_np.uncertainty
    }

    /// Expected X coordinate on the image
    #[wasm_bindgen(getter)]
    pub fn expected_x(&self) -> f64 {
        self.expected[0]
    }

    /// Expected Y coordinate on the image
    #[wasm_bindgen(getter)]
    pub fn expected_y(&self) -> f64 {
        self.expected[1]
    }

    /// True if this is actually mapped
    #[wasm_bindgen(getter)]
    pub fn has_pms(&self) -> bool {
        self.has_pms
    }

    /// The uncertainty of the placement on the image
    #[wasm_bindgen(getter)]
    pub fn img_uncertainty(&self) -> f64 {
        self.error
    }

    /// Set the uncertainty of the placement on the image
    #[wasm_bindgen(setter)]
    pub fn set_img_uncertainty(&mut self, v: f64) {
        self.error = v;
    }

    /// Set a WasmVec2f64 to the expected position (given last camera mapping)
    pub fn set_expected_at_vec(&self, v: &mut WasmVec2f64) {
        v.set_array(self.expected.as_ref());
    }

    /// Set a WasmVec2f64 to the mapped image position
    pub fn set_image_vec(&self, v: &mut WasmVec2f64) {
        v.set_array(self.screen.as_ref());
    }

    /// Image X coordinate on the image
    #[wasm_bindgen(getter)]
    pub fn image_x(&self) -> f64 {
        if self.has_pms { self.screen[0] } else { 0.0 }
    }

    /// Image Y coordinate on the image
    #[wasm_bindgen(getter)]
    pub fn image_y(&self) -> f64 {
        if self.has_pms { self.screen[1] } else { 0.0 }
    }

    /// Image Yaw (angle away from center)
    #[wasm_bindgen(getter)]
    pub fn image_yaw(&self) -> f64 {
        if self.has_pms {
            self.screen_ry.yaw()
        } else {
            0.0
        }
    }

    /// Image Roll (anticlockwise angle, 0 being +x)
    #[wasm_bindgen(getter)]
    pub fn image_roll(&self) -> f64 {
        if self.has_pms {
            self.screen_ry.roll()
        } else {
            0.0
        }
    }

    /// Distance from cursor - only valid if has_pms
    #[wasm_bindgen(getter)]
    pub fn cursor_distance(&self) -> f64 {
        self.cursor_distance
    }

    /// The error in 'yaw', which is a distance in unit vector space
    #[wasm_bindgen(getter)]
    pub fn d_map_yaw_err(&self) -> f64 {
        if self.has_pms {
            self.d_map_yaw_err
        } else {
            0.0
        }
    }

    /// The error in 'roll', which is a distance in unit vector space
    #[wasm_bindgen(getter)]
    pub fn d_map_roll_err(&self) -> f64 {
        if self.has_pms {
            self.d_map_roll_err
        } else {
            0.0
        }
    }

    /// The error in 'mapping', which is a distance in image pixels
    #[wasm_bindgen(getter)]
    pub fn d_map_distance(&self) -> f64 {
        if self.has_pms {
            self.d_map_distance
        } else {
            0.0
        }
    }

    /// Update the cursor distance
    pub fn set_cursor(&mut self, x: f64, y: f64) {
        let dx = self.expected[0] - x;
        let dy = self.expected[1] - y;
        self.cursor_distance = (dx * dx + dy * dy).sqrt();
    }

    /// Remap using the camera and point mapping set
    pub fn update(
        &mut self,
        camera: &WasmCameraInstance,
        pms: &WasmPointMappingSet,
        cursor_x: f64,
        cursor_y: f64,
    ) {
        let camera = camera.camera().borrow();
        self.expected = camera.world_xyz_to_px_abs_xy(&self.wasm_np.model);
        self.set_cursor(cursor_x, cursor_y);
        let pms = pms.pms().borrow();
        if let Some(pm) = pms.mapping_of_np_name(&self.wasm_np.name) {
            self.has_pms = true;
            self.screen = *pm.screen();
            self.error = pm.error();
            self.usage = pm.usage();
            self.d_map_distance = self.screen.distance(self.expected);

            // Convert the placed mapped position to a roll/yaw
            //
            // Note that the sensor_dir_of_pt uses the sensor centre and pixel aspect
            // ratio to map to a pure positionq
            //
            // This does NOT use the lens mapping
            let screen_txty = camera.px_abs_xy_to_sensor_txty(&self.screen);
            let screen_sensor_dir = screen_txty.to_unit_vector();
            self.screen_ry = screen_txty.into();
            let roll_quat = Quatf64::default().rotate_z(-self.screen_ry.roll());

            let screen_sensor_on_roll_axis = roll_quat.apply3(&screen_sensor_dir);
            let placed_yaw = screen_sensor_on_roll_axis[0] / screen_sensor_on_roll_axis[2];

            // Convert the NP expected position, given orientation and lens calibration,
            // to a yaw for yaw error
            //
            // This does NOT use the lens mapping - but the expected position did
            let expected_txty = camera.px_abs_xy_to_sensor_txty(&self.expected);
            let expected_sensor_dir = expected_txty.to_unit_vector();

            // Rotate the direction for the NP expected position by -map_roll around -Z to
            // generate an (x,y,z) whose x is 'yaw' error, y is 'roll' error, scaled down by
            // z (which should be 1-epsilon)
            let expected_sensor_on_roll_axis = roll_quat.apply3(&expected_sensor_dir);
            let expected_yaw = expected_sensor_on_roll_axis[0] / expected_sensor_on_roll_axis[2];
            let expected_roll = expected_sensor_on_roll_axis[1] / expected_sensor_on_roll_axis[2];

            self.d_map_yaw_err = expected_yaw - placed_yaw;
            self.d_map_roll_err = expected_roll; // placed_roll is 0 by definition
        } else {
            self.has_pms = false;
        }
    }

    /// Update the NamedPoint details from the *actual* content of the NPS
    ///
    /// This invalidates the mapping
    pub fn update_np(&mut self, nps: WasmNamedPointSet) -> bool {
        let nps = nps.nps().borrow();
        if let Some(np) = nps.get_rc_np(&self.wasm_np.name) {
            self.wasm_np.update_from_np(&*np);
            self.has_pms = false;
            true
        } else {
            false
        }
    }
}
