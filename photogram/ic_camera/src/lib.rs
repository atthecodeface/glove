/*!

# Image spaces

Pixels on an image are mapped as follows:

* Absoluate pixel is converted to a sensor-center relative coordinate

* Sensor-center relative coordinate is mapped to an optical-center relative coordinate

* Optical center relative coordinate is mapped to an internal optical direction
  vector (used to be sensor-relative direction). For a real camera, this depends
  on the sensor size in mm (for X and Y) and the distance of the sensor to the
  lens.

* Optical direction vector to camera-relative direction vector, using the lens
  projection. For a spherical lens this maps the optical direction to a RollYaw,
  the Yaw is mapped, and then the new RollYaw is mapped back to a direction

* Camera-relative direction vector to world direction vector, applying the camera orientation

* World-relative direction vector to World Ray (starting at the camera position in the world relative direction)

Hence there are the following coordinate spaces:

* Sensor/image absolute (top-left is 0,0, units are pixels)
* Sensor/image relative (center is 0,0, units are pixels)
* Optical center relative (optical axis is 0,0, units are pixels, represented by a Point2D)
* Optical direction vector ([X, Y, -1], represented by a TanXTanY)
* Camera direction vector ([X, Y, -1], represented by a TanXTanY)
* World (direction vectors and rays, using Ray and Point3D; start position of ray is in mm)

To compare the pixels for regions of different images the regions need to be mapped into the same coordinate space;
for small areas around a ray this is usually a rectilinear spherical projection.
The (normally square) image patch has an orientation that maps a central world direction vector and world 'up' vector to (respectively) [0,0,-1] and [0,1,0].
Then any world vector maps through this orientation to [X,Y,Z] which is mapped (rectilinear) to image relative using [-sc*X/Z, -sc*Y/Z] provided Z is less than 0, and where sc is
the required scale factor to achieve the field-of-view of the patch (patch size/2)/tan(half-FOV-horizontal).

To compare *strips* of different images the regions need to be mapped to identical cylinder projection images, rather than rectilinear; for these,
Y maps in the same fashion, but X requires an atan mapping.
The Width-by-Height rectangulare image patch has an orientation that maps a central world direction vector and world 'up' vector to (respectively) [0,0,-1] and [0,1,0].
Then any world vector maps through this orientation to [X,Y,Z] which is mapped to image relative using [xsc*atan2(-Z, X), -ysc*Y/Z] provided Z is less than 0, and where ysc is
the required scale factor to achieve the vertical field-of-view of the patch (height/2)/tan(half-FOV-vertical), and xsc is the requiref scale factor to map the
horizontal field of view to the width (width/FOV-horizontal in radians).

For regular camera/(spherical) lens pairings, the camera body supplies the sensor relative coordinate mapping; the lens maps this through its optical center, then using
the focal length of the lens and the actual focus distance to an optical direction vector; then through the calibrated lens mapping to a camera direction vector, and then by
camera orientation/placement to the world direction vector and ray.

# Traits

A camera body, or simply an image, have sensor/image sizes and mappings from the absolute coordinates to refer to pixels to relative pixels.
Using the sensor size pixel size, and a sensor size in mm (if it has one), allows a description of the mapping from sensor/image absolute to optical direction vector,
provided a method is given that maps pixels to that direction vector given a distance-to-lens value.

The mapping from optical direction vector to camera direction vector is through a lens projection; a camera body does not perform this, and so it is not a trait of the body.

## CameraSensor

The camera sensor traits maps absolute pixels within an image to relative, to the nominal 'center of the image'. It has a further relative-to-direction vector mapping method, given a distance-to-lens-in-mm value.

This trait can be implemented by a camera body, or by a simple image as required for generating small image patches to compare different captured images at common camera directions.

## LensProjection

This provides optical direction vector (i.e. the direction vector from the sensor to the lens) to camera direction vector (i.e. the direction vector of that sensor pixel through the lens out of the cemar)

This trait has a simple rectilinear implementation (vector in equals vector out); both spherical (using a LensPoly) and cylindrical projections can be provided.

## CameraProjection

This provides a camera orientation and position, and methods to map camera direction vectors to/from world directions and rays.

This has the sensor coordinate space (pixels relative to the centre of the image) and maps
this to the optical coordinate space, which takes into account the optical axis offset
(if there is one); it also applies the

## SensorIterator

This is a trait that provides an iterator that runs over a region of pixels (as x,y) with the associated camera direction

A methods is supplied which (incrementally) maps world direction onto an optional sensor absolute position.

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

mod lens_polys;
use ic_base::{Point2D, Point3D, Quat, TanXTanY};
pub use lens_polys::LensPolys;

mod camera_body;
mod camera_lens;
pub use camera_body::{CameraBody, serialize_body_name};
pub use camera_lens::{CameraLens, serialize_lens_name};

mod camera_database;
pub use camera_database::CameraDatabase;

mod camera_calibrate;
mod camera_instance;
mod camera_instance_desc;
pub use camera_calibrate::CalibrationMapping;
pub use camera_instance::CameraInstance;
pub use camera_instance_desc::CameraInstanceDesc;

mod traits;
pub use traits::{
    CameraInstanceProjection, CameraLensProjection, CameraProjection, CameraSensor, LensProjection,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct BaseCamera {
    pub position: Point3D,
    pub orientation: Quat,
}

impl CameraProjection for BaseCamera {
    fn orientation(&self) -> Quat {
        self.orientation
    }
    fn position(&self) -> Point3D {
        self.position
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RectilinearLens();

impl LensProjection for RectilinearLens {
    fn camera_txty_to_sensor_txty(&self, camera_txty: TanXTanY) -> TanXTanY {
        camera_txty
    }
    fn sensor_txty_to_camera_txty(&self, sensor_txty: TanXTanY) -> TanXTanY {
        sensor_txty
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SizedSensor {
    width: u32,
    height: u32,
    mm_per_pixel: f64,
}

impl CameraSensor for SizedSensor {
    /// Get the (main) name of the camera body
    fn sensor_name(&self) -> &str {
        "SizedSensor"
    }

    /// Get the size of the sensor in pixels
    fn sensor_px_size(&self) -> (f64, f64) {
        (self.width as f64, self.height as f64)
    }

    /// Get the center pixel (the pixel that aligns with the optical axis of the lens)
    fn sensor_px_center(&self) -> Point2D {
        [self.width as f64 / 2.0, self.height as f64 / 2.0].into()
    }

    fn sensor_mm_single_pixel_height(&self) -> f64 {
        self.mm_per_pixel
    }

    fn sensor_mm_single_pixel_width(&self) -> f64 {
        self.mm_per_pixel
    }
}
