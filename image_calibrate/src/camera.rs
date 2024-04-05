// Make this crate-only?
pub mod polynomial;

mod camera_body;
pub use camera_body::{serialize_body_name, CameraBody};

mod camera_lens;
pub use camera_lens::{serialize_lens_name, CameraLens};

mod camera_database;
pub use camera_database::CameraDatabase;

mod camera_polynomial;
pub use camera_polynomial::{CameraPolynomial, CameraPolynomialDesc};

mod camera_polynomial_calibrate;
pub use camera_polynomial_calibrate::{CameraPolynomialCalibrate, CameraPolynomialCalibrateDesc};

// mod camera_instance;
// pub use camera_instance::{CameraInstance, CameraInstanceDesc};

mod camera_mapping;
pub use camera_mapping::{CameraAdjustMapping, CameraPtMapping, CameraShowMapping};

mod traits;
pub use traits::{CameraProjection, CameraSensor, CameraView};

mod best_mapping;
pub use best_mapping::BestMapping;
