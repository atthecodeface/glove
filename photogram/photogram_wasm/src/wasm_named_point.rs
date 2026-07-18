use wasm_bindgen::prelude::*;

use ic_base::Point3D;
use ic_mapping::NamedPoint;

use crate::WasmVec3f64;

/*
 * A WasmNamedPoint is a transient structure containing the data that is in the
 * NamedPointSet database
 *
 * It is *not* a mirror onto the actual content, and its properties can be
 * changed without updating the actual database
 *
 * To updated the database use add_pt or similar using a WasmNamedPoint
 */
#[wasm_bindgen]
#[derive(Debug, Clone, Default)]
pub struct WasmNamedPoint {
    pub(crate) name: String,
    pub(crate) color: String,
    pub(crate) at_infinity: bool,
    pub(crate) model: Point3D,
    pub(crate) uncertainty: f64,
}

impl std::convert::From<&NamedPoint> for WasmNamedPoint {
    fn from(np: &NamedPoint) -> Self {
        Self {
            name: np.ref_tag().as_str().into(),
            color: np.color().as_string(),
            at_infinity: np.model_is_direction(),
            model: np.model_pt().into(),
            uncertainty: np.model_uncertainty(),
        }
    }
}
#[wasm_bindgen]
impl WasmNamedPoint {
    /// Create a new WasmNamedPoint
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, color: &str) -> WasmNamedPoint {
        let name = name.into();
        let color = color.into();
        let model = Point3D::default();
        let uncertainty = 0.0;
        let at_infinity = false;
        Self {
            name,
            color,
            at_infinity,
            model,
            uncertainty,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        (&self.name).into()
    }

    #[wasm_bindgen(getter)]
    pub fn color(&self) -> String {
        (&self.color).into()
    }

    #[wasm_bindgen(getter)]
    pub fn at_infinity(&self) -> bool {
        self.at_infinity
    }

    pub fn set_model_vec(&self, v: &mut WasmVec3f64) {
        v.set_array(self.model.as_ref());
    }

    pub fn model_as_array(&self) -> Box<[f64]> {
        Box::new(*self.model.as_ref())
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> f64 {
        self.uncertainty
    }

    pub(crate) fn set_np(&self, np: &NamedPoint) {
        crate::console_log!("set np Color {}", self.color);
        if let Ok(color) = self.color.as_str().try_into() {
            crate::console_log!("set np Color {}", color);
            np.set_color(color);
        }
        np.set_model(Some((self.at_infinity, self.model, self.uncertainty)));
    }
}
