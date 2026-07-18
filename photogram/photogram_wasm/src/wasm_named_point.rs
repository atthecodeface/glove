use wasm_bindgen::prelude::*;

use ic_base::Point3D;
use ic_mapping::NamedPoint;

use crate::WasmVec3f64;

/*
 * A WasmNamedPoint is a transient structure containing the data that is in the
 * NamedPointSet database. It is read-only.
 *
 */
#[wasm_bindgen]
#[derive(Debug, Clone, Default)]
pub struct WasmNamedPoint {
    /// Name of the NamedPoint
    pub(crate) name: String,
    /// Color of the Named Point
    pub(crate) color: String,
    /// True if the Named Point is mapped
    pub(crate) mapped: bool,
    /// Set if the model is at infinity (i.e. is a direction). Invalid if mapped is false
    pub(crate) at_infinity: bool,
    /// Direction/position of the named point in World space. Invalid if mapped is false
    pub(crate) model: Point3D,
    /// Uncertainty of the model direction/position. Invalid if mapped is false
    pub(crate) uncertainty: f64,
}

impl std::convert::From<&NamedPoint> for WasmNamedPoint {
    fn from(np: &NamedPoint) -> Self {
        let mut s = Self::default();
        s.update_from_np(np);
        s
    }
}

impl WasmNamedPoint {
    /// Set a NamedPoint to the values from this WasmNp
    pub fn set_np(&self, np: &NamedPoint) {
        if let Ok(color) = self.color.as_str().try_into() {
            np.set_color(color);
        }
        if self.is_mapped() {
            np.set_model(Some((self.at_infinity, self.model, self.uncertainty)));
        } else {
            np.set_model(None);
        }
    }

    /// Update data from a NamedPoint
    pub fn update_from_np(&mut self, np: &NamedPoint) {
        self.name = np.ref_tag().as_str().into();
        self.color = np.color().as_string();
        self.mapped = np.is_mapped();
        self.at_infinity = np.model_is_direction();
        self.model = np.model_pt().into();
        self.uncertainty = np.model_uncertainty();
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
        let mapped = false;
        Self {
            name,
            color,
            mapped,
            at_infinity,
            model,
            uncertainty,
        }
    }

    /// The name of the NamedPoint
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        (&self.name).into()
    }

    /// The color associated with the NamedPoint
    #[wasm_bindgen(getter)]
    pub fn color(&self) -> String {
        (&self.color).into()
    }

    /// True if the NamedPoint is mapped
    #[wasm_bindgen(getter)]
    pub fn is_mapped(&self) -> bool {
        self.mapped
    }

    /// True if the NamedPoint maps to a direction, not a position in world space
    #[wasm_bindgen(getter)]
    pub fn at_infinity(&self) -> bool {
        self.at_infinity
    }

    /// Set a WasmVec3f64 to the model direction/position
    pub fn model_set_vec(&self, v: &mut WasmVec3f64) {
        v.set_array(self.model.as_ref());
    }

    /// Allocate and set a new Float64Array of the model position
    pub fn model_as_array(&self) -> Box<[f64]> {
        Box::new(*self.model.as_ref())
    }

    /// The uncertainty of the model position/direction
    #[wasm_bindgen(getter)]
    pub fn error(&self) -> f64 {
        self.uncertainty
    }
}
