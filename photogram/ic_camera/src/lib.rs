// Make this crate-only?
pub mod polynomial;
pub mod utils;

mod camera_body;
mod camera_instance;
mod camera_lens;
pub use camera_body::{serialize_body_name, CameraBody};
pub use camera_instance::CameraInstanceDesc;
pub use camera_lens::{serialize_lens_name, CameraLens};

mod camera_database;
pub use camera_database::CameraDatabase;

mod camera_polynomial;
pub use camera_polynomial::CameraPolynomial as CameraInstance;
pub use camera_polynomial::{CameraPolynomial, CameraPolynomialDesc};

mod camera_polynomial_calibrate;
pub use camera_polynomial_calibrate::CameraPolynomialCalibrate;

mod traits;
pub use traits::{CameraProjection, CameraSensor};
