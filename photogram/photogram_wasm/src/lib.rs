//a To do
//
pub(crate) mod wasm_import;

pub use bezier_wasm::{WasmBezier3f32, WasmBezierBuilder3f32};
pub use geo_nd_wasm::{Quatf32, Vec2f32, Vec3f32, Vec4f32};
pub use geo_nd_wasm::{Quatf64, Vec2f64, Vec3f64, Vec4f64};
pub use geo_nd_wasm::{WasmMat3f32, WasmMat3f64};
pub use geo_nd_wasm::{WasmMat4f32, WasmMat4f64};
pub use geo_nd_wasm::{WasmVec2f32, WasmVec3f32, WasmVec4f32};
pub use geo_nd_wasm::{WasmVec2f64, WasmVec3f64, WasmVec4f64};
pub use star_catalog_wasm::{WasmCatalog, WasmStar};

use wasm_import::{ToFromWasmArr, err_to_string};

mod wasm_base;
pub use wasm_base::WasmRay;

mod wasm_camera;
pub use wasm_camera::{WasmCameraDatabase, WasmCameraInstance};

mod wasm_mapping;
pub use wasm_mapping::{WasmNamedPoint, WasmNamedPointSet, WasmPointMappingSet};

mod wasm_cip;
pub use wasm_cip::WasmCip;

mod wasm_project;
pub use wasm_project::WasmProject;

//a Useful macros
#[macro_export]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    // ($($t:tt)*) => ( unsafe { crate::log(&format_args!($($t)*).to_string())} )
    ($($t:tt)*) => ( { $crate :: wasm_log(&format_args!($($t)*).to_string())} )
}
