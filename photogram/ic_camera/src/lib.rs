//a Documentation
/*!

# Spherical Lens Projection

The concept is that there are absolute pixel positions within a
sensor, which can be converted to relative, which can be converted to
a tan(x)/tan(y), which can be mapped to a roll (around Z to the X axis) + yaw (around Y axis)

With a spherical lens yaw inside ('sensor yaw') is mapped to a lens
yaw outside ('camera yaw') by a bijective function - specifically that
the *roll* can be ignored (as it is a spherical lens)

This library uses polynomials to describe the mappings
(sensor-to-camera, and the inverse camera-to-sensor).

The polynmoial choice maps angle to angle, as the required mappings
are quite expressible for angles of up to 80 degree for most lens
types. An alternative that would be faster to process would be to map
tan(angle) to tan(angle); however, such mappings are not well
supported as small radix polynomials.

A particular lens may be focused on infinity, or closer; the
closer the focus, the larger the image on the sensor (as the lens
is further from the sensor). To allow for this a client requires
the knowledge of the focal length of the lens; the projection
mapping is not impacted by moving the lens, of course.

*/

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
