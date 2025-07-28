// Make this crate-only?
pub mod polynomial;
pub mod utils;

mod camera_body;
mod camera_lens;
pub use camera_body::{serialize_body_name, CameraBody};
pub use camera_lens::{serialize_lens_name, CameraLens, LensPolys};

mod camera_database;
pub use camera_database::CameraDatabase;

mod camera_calibrate;
mod camera_instance;
mod camera_instance_desc;
pub use camera_calibrate::CalibrationMapping;
pub use camera_instance::CameraInstance;
pub use camera_instance_desc::CameraInstanceDesc;

mod traits;
pub use traits::{CameraProjection, CameraSensor};
