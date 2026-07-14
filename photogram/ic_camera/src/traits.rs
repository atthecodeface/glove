//a Imports
use geo_nd::quat;

use ic_base::{Point2D, Point3D, Quat, RollYaw, TanXTanY};

/// A trait for a sensor in a digital camera, that maps absolute to
/// centre-of-lens-pixel relative, still in units of pixels
///
/// The concept is that there are absolute pixel positions within a sensor,
/// which can be converted to relative, which can be converted to a RollDist, which is a
pub trait CameraSensor: std::fmt::Debug {
    /// Name of the sensor (camera), for recording in files
    fn name(&self) -> &str;

    //mp sensor_size
    fn sensor_px_size(&self) -> (f64, f64);

    //mp sensor_center
    fn sensor_px_center(&self) -> Point2D;

    /// Map from absolute to centre-relative pixel
    ///
    /// The units are pixels in both coordinates
    fn px_abs_xy_to_px_rel_xy(&self, px_xy: &Point2D) -> Point2D;

    /// Map from centre-relative to absolute pixel
    ///
    /// The units are pixels in both coordinates
    fn px_rel_xy_to_px_abs_xy(&self, px_xy: &Point2D) -> Point2D;
}

/// A camera projection is a combination of a camera body and a lens
///
/// It provides methods that map XY points on an image taken by the
/// camera to [TanXTanY] 'vectors' in world space relative to the
/// camera, which will depend on the lens in the camera and the
/// focusing distance
///
/// It utilizes a world space XYZ coordinate system, which maps to a
/// camera-relative XYZ coordinate system where *(0,0,-1)* is on-axis, (1,0,0) is
/// to the right of the image, and (0,1,0) is up the image (so it forms a
/// right-handed-set)
pub trait CameraProjection: std::fmt::Debug + Clone {
    /// Get the name of the camera body
    fn camera_name(&self) -> String;

    /// Get the name of the lens
    fn lens_name(&self) -> String;

    /// Get the focal length of the lens
    fn focal_length(&self) -> f64;

    /// For the image(s) in the projection, return the distance of focus
    fn focus_distance(&self) -> f64;

    /// Get a Point3D indicating the placement of the camera in world space
    ///
    /// World/Model XYZ  = Camera relative XYZ + camera position
    fn position(&self) -> Point3D;

    /// Get a quaternion indicating the orientation of the camera
    ///
    /// Orientation is the world-to-camera quaternion; its conjugate is camera-to-world
    fn orientation(&self) -> Quat;

    /// Get a Point3D indicating the placement of the camera in world space
    fn set_position(&mut self, position: &Point3D);

    /// Set a quaternion indicating the orientation of the camera
    fn set_orientation(&mut self, orientation: &Quat);

    /// Set the distance from the sensor that the projection is focused on
    fn set_focus_distance(&mut self, mm_focus_distance: f64);

    /// Get the size of the sensor in pixels, width and height
    fn sensor_px_size(&self) -> (f64, f64);

    /// Get the center of the sensor in pixels
    fn sensor_px_center(&self) -> Point2D;

    /// Get the tan of half of the field-of-view for horizontal and vertical
    ///
    /// The diagonal tan-half-fov is the sqrt(sum(squares)) of these two values
    fn tan_hfov(&self) -> (f64, f64) {
        let wh = self.sensor_px_size();
        let txty0 = self.px_abs_xy_to_camera_txty(&[0., 0.].into());
        let txty1 = self.px_abs_xy_to_camera_txty(&[wh.0, wh.1].into());
        (txty0[0].max(txty1[0]), txty0[1].max(txty1[1]))
    }

    /// Apply the lens projection, to convert from *sensor* [RollYaw] to *camera* [RollYaw]
    #[must_use]
    fn sensor_ry_to_camera_ry(&self, ry: &RollYaw) -> RollYaw;

    /// Apply the lens projection, to convert from *camera* [RollYaw] to *sensor* [RollYaw]
    #[must_use]
    fn camera_ry_to_sensor_ry(&self, ry: &RollYaw) -> RollYaw;

    /// Map a sensor tan(x)/tan(y) to sensor Point2D coordinate
    ///
    /// Sensor [TanXTanY] and [Point2D] are in the same domain (i.e. this does not apply a projection)
    fn sensor_txty_to_px_abs_xy(&self, txty: &TanXTanY) -> Point2D;

    /// Map a sensor Point2D coordinate to sensor tan(x)/tan(y)
    ///
    /// Sensor [TanXTanY] and [Point2D] are in the same domain (i.e. this does not apply a projection)
    fn px_abs_xy_to_sensor_txty(&self, px_abs_xy: &Point2D) -> TanXTanY;

    /// Map a sensor Point2D coordinate to *Camera* (projected) tan(x)/tan(y)
    ///
    /// *Camera* [TanXTanY] map through the lens mapping to/from *Sensor* [Point2D]/[TanXTanY]
    fn px_abs_xy_to_camera_txty(&self, px_abs_xy: &Point2D) -> TanXTanY {
        let sensor_txty = self.px_abs_xy_to_sensor_txty(px_abs_xy);
        let sensor_ry = sensor_txty.into();
        let camera_ry = self.sensor_ry_to_camera_ry(&sensor_ry);
        camera_ry.into()
    }

    /// Map a camera (projected) tan(x)/tan(y) to a sensor Point2D coordinate
    ///
    /// *Camera* [TanXTanY] map through the lens mapping to/from *Sensor* [Point2D]/[TanXTanY]
    fn camera_txty_to_px_abs_xy(&self, camera_txty: &TanXTanY) -> Point2D {
        let camera_ry = camera_txty.into();
        let sensor_ry = self.camera_ry_to_sensor_ry(&camera_ry);
        let sensor_txty = sensor_ry.into();
        self.sensor_txty_to_px_abs_xy(&sensor_txty)
    }

    /// Map a camera (projected) tan(x)/tan(y) to a sensor tan(x)/tan(y)
    ///
    /// *Camera* [TanXTanY] map through the lens mapping to/from *Sensor* [Point2D]/[TanXTanY]
    fn camera_txty_to_sensor_txty(&self, camera_txty: &TanXTanY) -> TanXTanY {
        let camera_ry = camera_txty.into();
        let sensor_ry = self.camera_ry_to_sensor_ry(&camera_ry);
        sensor_ry.into()
    }

    /// Map a sensor tan(x)/tan(y) to a camera (projected) tan(x)/tan(y)
    fn sensor_txty_to_camera_txty(&self, sensor_txty: &TanXTanY) -> TanXTanY {
        let sensor_ry = sensor_txty.into();
        let camera_ry = self.sensor_ry_to_camera_ry(&sensor_ry);
        camera_ry.into()
    }

    /// *Camera* [TanXTanY] map through the lens mapping to/from *Sensor* [Point2D]/[TanXTanY]
    ///
    /// Convert a *camera* [TanXTanY] to a direction from the camera in world
    /// space, by applying the camera orientation. This does not apply the lens mapping.
    fn camera_txty_to_world_dir(&self, txty: &TanXTanY) -> Point3D {
        let camera_xyz = txty.to_unit_vector();
        quat::apply3(
            &quat::conjugate(self.orientation().as_ref()),
            camera_xyz.as_ref(),
        )
        .into()
    }

    /// Convert a [Point3D] *direction* vector in world space (XYZ) to camera-space
    /// coordinates (XYZ) by applying the orientation of the camera
    ///
    /// This does not apply the lens mapping.
    #[inline]
    fn world_dir_to_camera_xyz(&self, world_dir: &Point3D) -> Point3D {
        quat::apply3(self.orientation().as_ref(), world_dir).into()
    }

    /// Convert a [Point3D] *position* vector in world space (XYZ) to camera-space
    /// coordinates (XYZ) by translating and then applying the orientation of the camera
    ///
    /// This does not apply the lens mapping.
    #[inline]
    fn world_xyz_to_camera_xyz(&self, world_xyz: &Point3D) -> Point3D {
        let camera_relative_xyz = world_xyz - self.position();
        quat::apply3(self.orientation().as_ref(), camera_relative_xyz.as_ref()).into()
    }

    /// Convert a [Point3D] *position* vector in camera space (XYZ) to world
    /// space coordinates (XYZ) by appling the orientation of the camera and
    /// translating by the camera position.
    ///
    /// This does not apply the lens mapping.
    fn camera_xyz_to_world_xyz(&self, camera_xyz: &Point3D) -> Point3D {
        let camera_relative_xyz: Point3D = quat::apply3(
            &quat::conjugate(self.orientation().as_ref()),
            camera_xyz.as_ref(),
        )
        .into();
        camera_relative_xyz + self.position()
    }

    /// Convert a [Point3D] direction vector in camera space (XYZ) to world space
    /// direction (XYZ) by applying the orientation
    ///
    /// This does not apply the lens mapping.
    fn camera_xyz_to_world_dir(&self, camera_xyz: &Point3D) -> Point3D {
        quat::apply3(
            &quat::conjugate(self.orientation().as_ref()),
            camera_xyz.as_ref(),
        )
        .into()
    }

    /// Convert a [Point3D] *position* vector in world space (XYZ) to camera-space
    /// [TanXTanY] by translating and then applying the orientation of the camera
    ///
    /// This does not apply the lens mapping.
    #[inline]
    fn world_xyz_to_camera_txty(&self, world_xyz: &Point3D) -> TanXTanY {
        self.world_xyz_to_camera_xyz(world_xyz).into()
    }

    /// Convert a [Point3D] *position* vector in world space (XYZ) to sensor
    /// absolute positions [Point2D] by translating and then applying the
    /// orientation of the camera, then applying the lens mapping and converting
    /// to the sensor position
    ///
    /// This *DOES* apply the lens mapping.
    #[inline]
    fn world_xyz_to_px_abs_xy(&self, world_xyz: &Point3D) -> Point2D {
        let camera_txty = self.world_xyz_to_camera_txty(world_xyz);
        self.camera_txty_to_px_abs_xy(&camera_txty)
    }

    /// Convert a [Point3D] *direvtion* vector in world space (XYZ) to semnsor
    /// absolute positions [Point2D] by translating and then applying the
    /// orientation of the camera, then applying the lens mapping and converting
    /// to the sensor position.
    ///
    /// If the direction is *behind* the camera then return None
    ///
    /// This *DOES* apply the lens mapping.
    #[inline]
    fn world_dir_to_opt_px_abs_xy(&self, world_dir: &Point3D) -> Option<Point2D> {
        let camera_xyz = self.world_dir_to_camera_xyz(world_dir);
        if camera_xyz[2] < 1E-6 {
            None
        } else {
            let camera_txty = camera_xyz.into();
            Some(self.camera_txty_to_px_abs_xy(&camera_txty))
        }
    }
}
