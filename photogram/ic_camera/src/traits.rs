use geo_nd::{Quaternion, quat};

use ic_base::{
    Point2D, Point3D, Quat, Ray, RollYaw, TanXTanY, utils::orientation_mapping_triangle,
};

/// A trait for a sensor in a digital camera, that maps absolute to
/// centre-of-lens-pixel relative, still in units of pixels
///
/// The concept is that there are absolute pixel positions within a sensor,
/// which can be converted to relative coordinates
pub trait CameraSensor: std::fmt::Debug {
    /// Get the size of the sensor in pixels, width and height
    fn sensor_px_size(&self) -> (f64, f64);

    /// Get the width of a single sensor pixel in mm
    fn sensor_mm_single_pixel_width(&self) -> f64;

    /// Get the width of a single sensor pixel in mm
    fn sensor_mm_single_pixel_height(&self) -> f64;

    /// Get the width and height of the sensor in mm, derived from the pixel
    /// (w,h) and width/height of each pixel
    ///
    /// This is a convenience method, for Display trait implementations
    fn sensor_mm_size(&self) -> (f64, f64) {
        let (w, h) = self.sensor_px_size();
        (
            w * self.sensor_mm_single_pixel_width(),
            h * self.sensor_mm_single_pixel_height(),
        )
    }

    /// Get the center of the sensor in pixels
    ///
    /// This is expected to account for any *sensor* based optical axis offset; for removable lenses, the individual
    /// lens may have its own offset, which must be accounted for in conversion from sensor XY to optical XY
    fn sensor_px_center(&self) -> Point2D;

    /// Map from absolute to centre-relative pixel
    ///
    /// The units are pixels in both coordinates (sensor relative up and right being positive Y and X)
    ///
    /// This does *not* take into account any *lens* optical axis offset; it converts y=0 at top to y=0 at bottom
    fn sensor_px_abs_to_px_rel(&self, px_xy: Point2D) -> Point2D {
        let cxy_inverted = px_xy - self.sensor_px_center();
        [cxy_inverted[0], -cxy_inverted[1]].into()
    }

    /// Map from centre-relative to absolute pixel
    ///
    /// The units are pixels in both coordinates
    ///
    /// This does *not* take into account any *lens* optical axis offset; it converts y=0 at top to y=0 at bottom
    fn sensor_px_rel_to_px_abs(&self, px_xy: Point2D) -> Point2D {
        let pxy_inverted: Point2D = [px_xy[0], -px_xy[1]].into();
        pxy_inverted + self.sensor_px_center()
    }

    /// Map from a sensor optical XY to a direction vector (as TanXTanY) from the sensor to the lens
    ///
    /// The optical XY should be the sensor XY plus any accounting for the lens optical axis offset
    fn optical_xy_to_optical_txty(
        &self,
        optical_xy: Point2D,
        lens_sensor_distance: f64,
    ) -> TanXTanY {
        TanXTanY::of_tx_ty(
            optical_xy[0] * self.sensor_mm_single_pixel_width() / lens_sensor_distance,
            optical_xy[1] * self.sensor_mm_single_pixel_height() / lens_sensor_distance,
        )
    }
    /// Map from a sensor direction vector (from sensor to lens, as TanXTanY) to optical (pixel) XY
    ///
    /// The optical XY will be the sensor XY plus any accounting for the lens optical axis offset
    fn optical_txty_to_optical_xy(&self, txty: TanXTanY, lens_sensor_distance: f64) -> Point2D {
        [
            txty.tanx() * lens_sensor_distance / self.sensor_mm_single_pixel_width(),
            txty.tany() * lens_sensor_distance / self.sensor_mm_single_pixel_height(),
        ]
        .into()
    }
}

/// A mapping through a lens, be it spherical or cylindrical; it maps a vector in the direction 'to the lens' from the sensor to a direction out of the lens;
/// it also supports the reverse mapping
pub trait LensProjection: std::fmt::Debug + Clone {
    /// Map a camera (projected) tan(x), tan(y) to a sensor to lens direction vector as tan(x),tan(y)
    ///
    /// *Camera* [TanXTanY] map through the lens mapping to/from *Sensor* [Point2D]/[TanXTanY]
    fn camera_txty_to_optical_txty(&self, camera_txty: TanXTanY) -> TanXTanY;

    /// Map a sensor to lens direction vector as tan(x),tan(y) to a camera (projected) tan(x),tan(y)
    fn optical_txty_to_camera_txty(&self, optical_txty: TanXTanY) -> TanXTanY;
}

/// A mapping of a placed camera, which contains a lens and sensor (with their own projection etc)
///
/// This allows for a mapping of orientation and position of the camera in world space
///
/// This trait uses 'camera_txty', 'camera_dir', 'world_dir', and 'world_xyz';
/// the two camera direction vector forms differ in that camera_dir is an arbitrary length
/// vector, and camera_txty is essentially a direction vector (tanX, tanY, -1), but only tanX and tanY are stored.
///
/// 'world_dir' is the world direction vector given the orientation of the
/// camera (self), and is always an (x,y,z) vector; this may or may not be a
/// unit vector (indeed, it might be the actual vector between the camera and a model point)
///
/// 'world_xyz' is always world direction vector plus the camera position
pub trait CameraProjection: std::fmt::Debug + Clone {
    /// Get a Point3D indicating the placement of the camera in world space
    ///
    /// World/Model XYZ  = Camera relative XYZ + camera position
    fn position(&self) -> Point3D;

    /// Get a quaternion indicating the orientation of the camera
    ///
    /// Orientation is the world-to-camera quaternion; its conjugate is camera-to-world
    fn orientation(&self) -> Quat;

    /// Camera direction [TanXTanY] map to a *unit* world direction, by appling the inverse of the camera orientaion
    ///
    /// Convert a *camera* [TanXTanY] to a direction from the camera in world
    /// space, by applying the camera orientation.
    #[inline]
    fn camera_txty_to_world_dir(&self, camera_txty: TanXTanY) -> Point3D {
        let camera_dir = camera_txty.to_unit_vector();
        self.orientation().conjugate().apply3(&camera_dir)
    }

    /// Camera direction [TanXTanY] map to a *unit* world direction, by appling the inverse of the camera orientaion
    ///
    /// Convert a *camera* [TanXTanY] to a direction from the camera in world
    /// space, by applying the camera orientation.
    #[inline]
    fn camera_dir_to_world_dir(&self, camera_dir: Point3D) -> Point3D {
        self.orientation().conjugate().apply3(&camera_dir)
    }

    /// World direction (x,y,z) to a camera direction (x,y,z) of arbitrary length by appling the camera orientaion
    ///
    /// This preserves 'z' so that 'behind the camera' can be determined
    #[inline]
    fn world_dir_to_camera_dir(&self, world_dir: Point3D) -> Point3D {
        self.orientation().apply3(&world_dir)
    }

    /// Camera direction [TanXTanY] map to a Ray (with 0 error), using the camera position and orientation
    #[inline]
    fn camera_dir_to_world_ray(&self, camera_dir: Point3D) -> Ray {
        Ray::default()
            .with_start(self.position())
            .with_direction(self.orientation().conjugate().apply3(&camera_dir))
    }

    /// World direction (x,y,z) to a camera direction [TanXTanY], by appling the camera orientaion
    ///
    /// Note that a TanXTanY is always valid; if the world direction is *behind* the camera then this returns the TanXTanY of the *opposite* direction
    #[inline]
    fn world_dir_to_camera_txty(&self, world_dir: Point3D) -> TanXTanY {
        self.world_dir_to_camera_dir(world_dir).into()
    }

    /// Convert a [Point3D] *position* vector in world space (XYZ) to the direction vector from the camera
    #[inline]
    fn world_xyz_to_world_dir(&self, world_xyz: Point3D) -> Point3D {
        world_xyz - self.position()
    }

    /// Convert a [Point3D] *position* vector in world space (XYZ) to the direction vector from the camera
    #[inline]
    fn world_dir_to_world_xyz(&self, world_dir: Point3D) -> Point3D {
        world_dir + self.position()
    }
}

/// The combination of a camera sensor, lens, and positioned/oriented camera
///
/// The combination implies a camera sensor and a lens, focussed at a particular distance, with the lens
/// not necessarily being centred on the sensor center.
///
/// Hence this trait requires accessors for the optical axis offset and
/// lens-to-sensor distance (which together with the CameraSensor trait
/// determine the mapping from sensor-relative pixel offsets to sensor-to-lens
/// direction vectors)
///
pub trait CameraLensProjection:
    std::fmt::Debug + Clone + CameraProjection + LensProjection + CameraSensor
{
    /// Get the optical axis offset for a lens on this body in this instance
    ///
    /// This is in pixels; it is added to the sensor center
    fn optical_axis_offset(&self) -> Point2D;

    /// Get the distance of the lens from the sensor in mm
    fn lens_sensor_distance(&self) -> f64;

    /// Map a sensor Point2D coordinate to the sensor TanX/TanY for calibration purposes *ONLY*
    ///
    /// The internal sensor-to-lens direction should only really be required for calibration
    #[inline]
    fn sensor_px_abs_xy_to_optical_txty(&self, px_abs_xy: Point2D) -> TanXTanY {
        let optical_xy = self.sensor_px_abs_to_px_rel(px_abs_xy) - self.optical_axis_offset();
        self.optical_xy_to_optical_txty(optical_xy, self.lens_sensor_distance())
    }

    /// Map a sensor Point2D coordinate to *Camera* (projected) tan(x)/tan(y)
    ///
    /// *Camera* [TanXTanY] map through the lens mapping to/from *Sensor* [Point2D]/[TanXTanY]
    fn sensor_px_abs_xy_to_camera_txty(&self, px_abs_xy: Point2D) -> TanXTanY {
        let optical_xy = self.sensor_px_abs_to_px_rel(px_abs_xy) - self.optical_axis_offset();
        let sensor_txty = self.optical_xy_to_optical_txty(optical_xy, self.lens_sensor_distance());
        let camera_txty = self.optical_txty_to_camera_txty(sensor_txty);
        camera_txty
    }

    /// Convert an absolute sensor position to a unit [Point3D] *direction* vector in camera space (XYZ) by applying the
    /// lens mapping.
    fn sensor_px_abs_xy_to_camera_dir(&self, px_abs_xy: Point2D) -> Point3D {
        let camera_txty = self.sensor_px_abs_xy_to_camera_txty(px_abs_xy);
        camera_txty.to_unit_vector()
    }

    /// Map a camera (projected) tan(x)/tan(y) to a sensor Point2D coordinate
    ///
    /// *Camera* [TanXTanY] map through the lens mapping to/from *Sensor* [Point2D]/[TanXTanY]
    fn camera_txty_to_sensor_px_abs_xy(&self, camera_txty: TanXTanY) -> Point2D {
        let sensor_txty = self.camera_txty_to_optical_txty(camera_txty);
        let optical_xy = self.optical_txty_to_optical_xy(sensor_txty, self.lens_sensor_distance());
        let px_abs_xy = self.sensor_px_rel_to_px_abs(optical_xy + self.optical_axis_offset());
        px_abs_xy
    }

    /// Convert a [Point3D] camera-relative *direction* vector to sensor
    /// absolute positions [Point2D] by applying the lens mapping and converting
    /// to the sensor position.
    ///
    /// If the direction is *behind* the camera then return None
    #[inline]
    fn camera_dir_to_opt_sensor_px_abs_xy(&self, camera_dir: Point3D) -> Option<Point2D> {
        (camera_dir[2] < 1E-6).then(|| self.camera_txty_to_sensor_px_abs_xy(camera_dir.into()))
    }

    /// Get the tan of half of the field-of-view for horizontal and vertical (to a reasonable approximation)
    ///
    /// This does not take into account the optical axis offsets
    ///
    /// The diagonal tan-half-fov is the sqrt(sum(squares)) of these two values
    fn tan_hfov(&self) -> (f64, f64) {
        let wh = <Self as CameraSensor>::sensor_px_size(self);
        let txty0 = self.sensor_px_abs_xy_to_camera_txty([0., 0.].into());
        let txty1 = self.sensor_px_abs_xy_to_camera_txty([wh.0, wh.1].into());
        (txty0[0].max(txty1[0]), txty0[1].max(txty1[1]))
    }

    /// Convenience method to convert *world_dir* all the way to a sensor XY
    ///
    /// If the direction is *behind* the camera then return None
    #[inline]
    fn world_dir_to_opt_sensor_px_abs_xy(&self, world_dir: Point3D) -> Option<Point2D> {
        self.camera_dir_to_opt_sensor_px_abs_xy(self.world_dir_to_camera_dir(world_dir))
    }

    /// Convenience method to convert sensor XY all the way to *unit* world direction
    #[inline]
    fn sensor_px_abs_xy_to_world_dir(&self, px_abs_xy: Point2D) -> Point3D {
        self.camera_txty_to_world_dir(self.sensor_px_abs_xy_to_camera_txty(px_abs_xy))
    }
}

/// A camera that can be moved, or with a lens that can be moved in/out
pub trait AdjustableCameraProjection: std::fmt::Debug + Clone {
    /// Get a Point3D indicating the placement of the camera in world space
    fn set_position(&mut self, position: &Point3D);

    /// Set a quaternion indicating the orientation of the camera
    fn set_orientation(&mut self, orientation: &Quat);

    /// Set the optical axis offset for a lens on this body in this instance
    fn set_optical_axis_offset(&mut self, p: &Point2D);

    /// Set the distance from the sensor that the projection is focused on
    fn set_focus_distance(&mut self, mm_focus_distance: f64);
}

/// A CylindricalProjection mapping maps (reversibly) a sphere with polar coordinates
/// (lambda, phi) with -pi<lambda<=pi and -pi/2 <= phi <= pi/2 to a cylinder
/// with coordinates (lambda, y). Hence it is really a mapping of phi to y.
///
/// In actuality the projection is confined to a vertical section of the
/// cylinder referred to with y=1 as the bottom y=0 as the top of the section,
/// mapping phi = v_ofs - vfov/2 to y=1, and y=0 maps to v_ofs + vfov/2
///
/// The mapping is x,y to λ (lambda), φ (phi)
///
/// The spherical coords λ and φ
/// map to a direction (relative to the camera orientation) of (sin(λ), tan(φ), cos(λ)) normalized,
/// i.e. a rotation of (0,0,1) about the X axis by φ (latitude) then about the Y axis by λ (longitude).
///
/// x_relative is in the range -1 to +1 for left to right of the image; y_relative +1 to -1 for bottom to top
///
/// All cylindrical projections use x = λ, or rather λ = x; this is the 'main' axis
///
/// The minor axis (y) can map the actual y value in the range +-1 to +-hfov_v; this is the equirectangular projection with phi = y
///
/// The minor axis (y) can map the actual y value with phi = atan(y), with
/// phi in the range +-hfov_v; then y must map to a range of tan(-hfovh) to
/// tan(hfovh) (linearly)
///
///
/// "Equirectangular projection" uses x = λ, y = φ
/// "Central cylindrical projection" uses x = λ, y = tan(φ) [ hence φ = atan(y) ]
/// "Lambert cylindrical projection (equal area) " uses x = λ, y = sin(φ)  [ hence φ = asin(y) ]
/// "Gall stereographic projection " uses x = λ, y = tan(φ/2) [ hence φ = 2*atan(y) ]
pub trait CylindricalProjection: std::fmt::Debug {
    /// Get the name of the projection (such as "equirectangular")
    fn name(&self) -> &str;
    /// Set the vertical field of view, in radians, and the offset from 0
    ///
    /// A value of 0 in y should map to v_ofs - fov_v/2
    fn set_vfov(&mut self, fov_v: f64, v_ofs: f64);
    /// Map y in range 0 to 1 (max to min) to phi
    ///
    ///
    fn phi_of_y(&self, y: f64) -> f64;
    /// Map y in range 0 to 1 to phi in range (v_ofs-fov_v/2) to (v_ofs+fov_v/2) appropriately
    fn tan_phi_of_y(&self, y: f64) -> f64 {
        self.phi_of_y(y).tan()
    }
    /// Map phi to y, with (v_ofs+fov_v/2) mapping to y=0 and (v_ofs-fov_v/2 ) to y=1
    ///
    /// This must use the inverse mapping for phi(y)
    fn y_of_phi(&self, phi: f64) -> f64;

    /// Create a boxed clone to allow CylindricalLens to be Clone
    fn boxed_clone(&self) -> Box<dyn CylindricalProjection>;
}
