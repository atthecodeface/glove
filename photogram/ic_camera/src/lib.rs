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

* Sensor/image absolute (top-left is 0,0, units are pixels) (sensor_px_abs_xy)
* Sensor/image relative (center is 0,0, units are pixels) (sensor_px_rel_xy, generally internal use only)
* Optical center relative (optical axis is 0,0, units are pixels, represented by a Point2D) (optical_xy generally internal use only)
* Optical direction vector ([X, Y, -1], represented by a TanXTanY) (optical_txty, generally internal use only)
* Camera direction vector ([X, Y, -1], represented by a TanXTanY) (camera_txty)
* World direction vectors as [X,Y,Z] (world_dir)
* World position vectors as [X,Y,Z] (world_xyz)

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

// mod camera_calibrate;
mod camera_instance;
mod camera_instance_desc;
// pub use camera_calibrate::CalibrationMapping;
pub use camera_instance::CameraInstance;
pub use camera_instance_desc::CameraInstanceDesc;

mod cylindrical_lens;
pub use cylindrical_lens::CylindricalLens;

mod traits;
pub use traits::{
    AdjustableCameraProjection, CameraLensProjection, CameraProjection, CameraSensor,
    CylindricalProjection, LensProjection,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct SimpleSensorLensCamera<
    L: LensProjection + std::default::Default + Clone + std::fmt::Debug,
> {
    pub sensor: SizedSensor,
    pub lens: L,
    pub camera: BaseCamera,
    pub mm_focal_length: f64,
    pub mm_focus_distance: f64,
}

impl<L> SimpleSensorLensCamera<L>
where
    L: LensProjection + std::default::Default + Clone + std::fmt::Debug,
{
    /// Create a new 'camera' at the origin, looking along -Z with X to the right, focussed at (near) infinity with given sensor and lens details
    pub fn new(width: u32, height: u32, mm_per_pixel: f64, lens: L, mm_focal_length: f64) -> Self {
        let sensor = SizedSensor {
            width,
            height,
            mm_height: (height as f64 * mm_per_pixel),
            mm_width: (width as f64 * mm_per_pixel),
        };
        let camera = BaseCamera::default();
        Self {
            sensor,
            lens,
            camera,
            mm_focal_length,
            mm_focus_distance: 1.0E7,
        }
    }
}

impl<L> AdjustableCameraProjection for SimpleSensorLensCamera<L>
where
    L: LensProjection + std::default::Default + Clone + std::fmt::Debug,
{
    fn set_position(&mut self, position: &Point3D) {
        self.camera.set_position(position);
    }
    fn set_orientation(&mut self, orientation: &Quat) {
        self.camera.set_orientation(orientation);
    }
    fn set_optical_axis_offset(&mut self, _p: &Point2D) {}
    fn set_focus_distance(&mut self, mm_focus_distance: f64) {
        self.mm_focus_distance = mm_focus_distance;
    }
}

impl<L> CameraSensor for SimpleSensorLensCamera<L>
where
    L: LensProjection + std::default::Default + Clone + std::fmt::Debug,
{
    fn sensor_px_size(&self) -> (f64, f64) {
        self.sensor.sensor_px_size()
    }

    fn sensor_px_center(&self) -> Point2D {
        self.sensor.sensor_px_center()
    }

    fn sensor_mm_single_pixel_height(&self) -> f64 {
        self.sensor.sensor_mm_single_pixel_height()
    }

    fn sensor_mm_single_pixel_width(&self) -> f64 {
        self.sensor.sensor_mm_single_pixel_width()
    }
}
impl<L> LensProjection for SimpleSensorLensCamera<L>
where
    L: LensProjection + std::default::Default + Clone + std::fmt::Debug,
{
    fn optical_txty_to_camera_txty(&self, optical_txty: TanXTanY) -> TanXTanY {
        self.lens.optical_txty_to_camera_txty(optical_txty)
    }
    fn camera_txty_to_optical_txty(&self, camera_txty: TanXTanY) -> TanXTanY {
        self.lens.camera_txty_to_optical_txty(camera_txty)
    }
}
impl<L> CameraProjection for SimpleSensorLensCamera<L>
where
    L: LensProjection + std::default::Default + Clone + std::fmt::Debug,
{
    fn position(&self) -> Point3D {
        self.camera.position()
    }
    fn orientation(&self) -> Quat {
        self.camera.orientation()
    }
}
impl<L> CameraLensProjection for SimpleSensorLensCamera<L>
where
    L: LensProjection + std::default::Default + Clone + std::fmt::Debug,
{
    fn optical_axis_offset(&self) -> Point2D {
        Point2D::default()
    }
    fn lens_sensor_distance(&self) -> f64 {
        1.0 / (1.0 / self.mm_focal_length - 1.0 / self.mm_focus_distance)
    }
}
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

impl AdjustableCameraProjection for BaseCamera {
    fn set_position(&mut self, position: &Point3D) {
        self.position = *position;
    }
    fn set_orientation(&mut self, orientation: &Quat) {
        self.orientation = *orientation;
    }
    fn set_optical_axis_offset(&mut self, _p: &Point2D) {}
    fn set_focus_distance(&mut self, _mm_focus_distance: f64) {}
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RectilinearLens();

impl LensProjection for RectilinearLens {
    fn camera_txty_to_optical_txty(&self, camera_txty: TanXTanY) -> TanXTanY {
        camera_txty
    }
    fn optical_txty_to_camera_txty(&self, sensor_txty: TanXTanY) -> TanXTanY {
        sensor_txty
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SizedSensor {
    pub width: u32,
    pub height: u32,
    pub mm_width: f64,
    pub mm_height: f64,
}

impl CameraSensor for SizedSensor {
    /// Get the size of the sensor in pixels
    fn sensor_px_size(&self) -> (f64, f64) {
        (self.width as f64, self.height as f64)
    }

    /// Get the center pixel (the pixel that aligns with the optical axis of the lens)
    fn sensor_px_center(&self) -> Point2D {
        [self.width as f64 / 2.0, self.height as f64 / 2.0].into()
    }

    fn sensor_mm_single_pixel_height(&self) -> f64 {
        self.mm_height / (self.height as f64)
    }

    fn sensor_mm_single_pixel_width(&self) -> f64 {
        self.mm_width / (self.width as f64)
    }
}
