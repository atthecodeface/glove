//a To do
//
pub(crate) mod wasm_import;

use wasm_import::{err_to_string, ToFromWasmArr};

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
