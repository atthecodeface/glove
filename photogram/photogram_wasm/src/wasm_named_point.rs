use wasm_bindgen::prelude::*;

use ic_photogram::Point3D;
use ic_photogram::{ModelData, NamedPoint};

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
    /// Model data
    pub(crate) model_data: ModelData,
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
        *np.model_mut() = self.model_data;
    }

    /// Update data from a NamedPoint
    pub fn update_from_np(&mut self, np: &NamedPoint) {
        self.name = np.ref_tag().as_str().into();
        self.color = np.color().as_string();
        self.model_data = *np.model();
    }

    pub fn model_pt(&self) -> Point3D {
        self.model_data.model_pt()
    }
}

#[wasm_bindgen]
impl WasmNamedPoint {
    /// Create a new WasmNamedPoint
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, color: &str) -> WasmNamedPoint {
        let name = name.into();
        let color = color.into();
        let model_data = ModelData::default();
        Self {
            name,
            color,
            model_data,
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
        self.model_data.is_mapped()
    }

    /// True if the NamedPoint maps to a direction, not a position in world space
    #[wasm_bindgen(getter)]
    pub fn at_infinity(&self) -> bool {
        self.model_data.model_is_direction()
    }

    /// Set a WasmVec3f64 to the model direction/position
    pub fn model_set_vec(&self, v: &mut WasmVec3f64) -> Result<(), String> {
        if self.model_data.is_unmapped() {
            Err("Point is not mapped".into())
        } else {
            v.set_array(&*self.model_data.model_pt());
            Ok(())
        }
    }

    /// Allocate and set a new Float64Array of the model position
    pub fn model_as_array(&self) -> Box<[f64]> {
        Box::new(*self.model_data.model_pt().as_ref())
    }

    /// The uncertainty of the model position/direction
    #[wasm_bindgen(getter)]
    pub fn uncertainty(&self) -> f64 {
        self.model_data.model_uncertainty()
    }
}
