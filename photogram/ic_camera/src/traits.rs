//a Imports
use geo_nd::quat;

use ic_base::{Point2D, Point3D, Quat, RollYaw, TanXTanY};

//a Traits
//tt CameraSensor
/// A trait for a sensor in a digital camera, that maps absolute to
/// centre-of-lens-pixel relative, still in units of pixels
///
/// The concept is that there are absolute pixel positions within a sensor,
/// which can be converted to relative, which can be converted to a RollDist, which is a
pub trait CameraSensor: std::fmt::Debug {
    /// Name of the sensor (camera), for recording in files
    fn name(&self) -> &str;

    //mp sensor_size
    fn sensor_size(&self) -> (f64, f64);

    //mp sensor_center
    fn sensor_center(&self) -> Point2D;

    /// Map from absolute to centre-relative pixel
    ///
    /// The units are pixels in both coordinates
    fn px_abs_xy_to_px_rel_xy(&self, px_xy: Point2D) -> Point2D;

    /// Map from centre-relative to absolute pixel
    ///
    /// The units are pixels in both coordinates
    fn px_rel_xy_to_px_abs_xy(&self, px_xy: Point2D) -> Point2D;
}

//tt CameraProjection
/// A camera projection is a combination of a camera sensor and a lens
///
/// It provides methods that map XY points on an image taken by the
/// camera to [TanXTanY] 'vectors' in world space relative to the
/// camera, which will depend on the lens in the camera and the
/// focusing distance
pub trait CameraProjection: std::fmt::Debug + Clone {
    //ap camera_name
    /// Name of the camera, for recording in files
    fn camera_name(&self) -> String;

    //ap lens_name
    /// Name of the lens, for recording in files
    fn lens_name(&self) -> String;

    //ap focus_distance
    /// Get the distance from the sensor that the projection is focused on
    fn focus_distance(&self) -> f64;

    //mp position
    /// Get a Point3D indicating the placement of the camera in world space
    fn position(&self) -> Point3D;

    //mp orientation
    /// Get a quaternion indicating the orientation of the camera
    fn orientation(&self) -> Quat;

    //mp set_position
    /// Get a Point3D indicating the placement of the camera in world space
    fn set_position(&mut self, position: Point3D);

    //mp set_orientation
    /// Set a quaternion indicating the orientation of the camera
    fn set_orientation(&mut self, orientation: Quat);

    //mp set_focus_distance
    /// Set the distance from the sensor that the projection is focused on
    fn set_focus_distance(&mut self, mm_focus_distance: f64);

    //mp sensor_size
    fn sensor_size(&self) -> (f64, f64);

    //mp sensor_center
    fn sensor_center(&self) -> Point2D;

    //mp sensor_ry_to_camera_ry
    /// Apply the lens projection
    #[must_use]
    fn sensor_ry_to_camera_ry(&self, ry: RollYaw) -> RollYaw;

    //mp camera_ry_to_sensor_ry
    /// Apply the lens projection
    #[must_use]
    fn camera_ry_to_sensor_ry(&self, ry: RollYaw) -> RollYaw;

    //mp sensor_txty_to_px_abs_xy
    /// Map a sensor (unprojected) tan(x)/tan(y) to sensor Point2D coordinate
    fn sensor_txty_to_px_abs_xy(&self, txty: TanXTanY) -> Point2D;

    //mp px_abs_xy_to_sensor_txty
    /// Map a sensor Point2D coordinate to sensor (unprojected) tan(x)/tan(y)
    fn px_abs_xy_to_sensor_txty(&self, px_abs_xy: Point2D) -> TanXTanY;

    //mp px_abs_xy_to_camera_txty (derived)
    /// Map a sensor Point2D coordinate to camera (projected) tan(x)/tan(y)
    fn px_abs_xy_to_camera_txty(&self, px_abs_xy: Point2D) -> TanXTanY {
        let sensor_txty = self.px_abs_xy_to_sensor_txty(px_abs_xy);
        let sensor_ry = sensor_txty.into();
        let camera_ry = self.sensor_ry_to_camera_ry(sensor_ry);
        camera_ry.into()
    }

    //mp camera_txty_to_px_abs_xy (derived)
    /// Map a camera (projected) tan(x)/tan(y) to a sensor Point2D coordinate
    fn camera_txty_to_px_abs_xy(&self, camera_txty: TanXTanY) -> Point2D {
        let camera_ry = camera_txty.into();
        let sensor_ry = self.camera_ry_to_sensor_ry(camera_ry);
        let sensor_txty = sensor_ry.into();
        self.sensor_txty_to_px_abs_xy(sensor_txty)
    }

    //md camera_txty_to_world_dir (derived)
    /// Convert a TanXTanY in camera space to a direction from the camera in world space
    ///
    /// This applies the orientation of the camera
    fn camera_txty_to_world_dir(&self, txty: &TanXTanY) -> Point3D {
        let camera_xyz = txty.to_unit_vector();
        quat::apply3(
            &quat::conjugate(self.orientation().as_ref()),
            camera_xyz.as_ref(),
        )
        .into()
    }

    //md world_xyz_to_camera_xyz (derived)
    /// Convert a Point3D in world space (XYZ) to camera-space
    /// coordinates (XYZ)
    #[inline]
    fn world_xyz_to_camera_xyz(&self, world_xyz: Point3D) -> Point3D {
        let camera_relative_xyz = world_xyz - self.position();
        quat::apply3(self.orientation().as_ref(), camera_relative_xyz.as_ref()).into()
    }

    //md camera_xyz_to_world_xyz (derived)
    /// Convert a Point3D in camera space (XYZ) to world space
    /// coordinates (XYZ)
    fn camera_xyz_to_world_xyz(&self, camera_xyz: Point3D) -> Point3D {
        let camera_relative_xyz: Point3D = quat::apply3(
            &quat::conjugate(self.orientation().as_ref()),
            camera_xyz.as_ref(),
        )
        .into();
        camera_relative_xyz + self.position()
    }

    //md camera_xyz_to_world_dir (derived)
    /// Convert a Point3D in camera space (XYZ) to world space
    /// coordinates (XYZ)
    fn camera_xyz_to_world_dir(&self, camera_xyz: Point3D) -> Point3D {
        quat::apply3(
            &quat::conjugate(self.orientation().as_ref()),
            camera_xyz.as_ref(),
        )
        .into()
    }

    //md world_xyz_to_camera_txty (derived)
    /// Convert a Point3D in world space (XYZ) to camera-space
    /// TanX/TanY coordinates (XY)
    #[inline]
    fn world_xyz_to_camera_txty(&self, world_xyz: Point3D) -> TanXTanY {
        self.world_xyz_to_camera_xyz(world_xyz).into()
    }

    //md world_xyz_to_px_abs_xy (derived)
    /// Map a world Point3D coordinate to camera-space coordinates,
    /// and then to tan(x)/tan(y), then to camera sensor pixel X-Y coordinates
    #[inline]
    fn world_xyz_to_px_abs_xy(&self, world_xyz: Point3D) -> Point2D {
        let camera_txty = self.world_xyz_to_camera_txty(world_xyz);
        self.camera_txty_to_px_abs_xy(camera_txty)
    }
}
