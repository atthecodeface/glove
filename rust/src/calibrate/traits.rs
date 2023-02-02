//a Imports
use geo_nd::quat;

use super::{Point2D, Point3D, Quat, TanXTanY};

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

    /// Map from absolute to centre-relative pixel
    ///
    /// The units are pixels in both coordinates
    fn px_abs_xy_to_px_rel_xy(&self, px_xy: Point2D) -> Point2D;

    /// Map from centre-relative to absolute pixel
    ///
    /// The units are pixels in both coordinates
    fn px_rel_xy_to_px_abs_xy(&self, px_xy: Point2D) -> Point2D;
}

//tt SphericalLensProjection
/// The concept is that there are absolute pixel positions within a
/// sensor, which can be converted to relative, which can be converted
/// to a tan(x)/tan(y) - in world space X/Z and Y/Z, which can be
/// mapped from (but not really to) xyz
///
/// The projection of the lens maps a world-angle to a sensor-angle,
/// and vice-versa
///
/// A particular lens may be focused on infinity, or closer; the
/// closer the focus, the larger the image on the sensor (as the lens
/// is further from the sensor). To allow for this a client requires
/// the knowledge of the focal length of the lens; the projection
/// mapping is not impacted by moving the lens, of course.
pub trait SphericalLensProjection: std::fmt::Debug {
    fn mm_focal_length(&self) -> f64;
    fn sensor_to_world(&self, tan: f64) -> f64;
    fn world_to_sensor(&self, tan: f64) -> f64;
}

//tt CameraProjection
/// A camera projection is a combination of a camera sensor and a lens
///
/// It provides methods that map XY points on an image taken by the
/// camera to [TanXTanY] 'vectors' in world space relative to the
/// camera, which will depend on the lens in the camera and the
/// focusing distance
pub trait CameraProjection: std::fmt::Debug {
    /// Name of the camera, for recording in files
    fn camera_name(&self) -> &str;

    /// Name of the lens, for recording in files
    fn lens_name(&self) -> &str;

    /// Set the distance from the sensor that the projection is focused on
    fn set_focus_distance(&mut self, mm_focus_distance: f64);

    /// Get the distance from the sensor that the projection is focused on
    fn focus_distance(&self) -> f64;

    /// Map from absolute to centre-relative pixel
    ///
    /// The units are pixels in both coordinates
    fn px_abs_xy_to_px_rel_xy(&self, px_xy: Point2D) -> Point2D;

    /// Map from centre-relative to absolute pixel
    ///
    /// The units are pixels in both coordinates
    fn px_rel_xy_to_px_abs_xy(&self, px_xy: Point2D) -> Point2D;

    /// Map an actual centre-relative XY pixel in the frame of the
    /// camera to a world-space tan(x), tan(y)
    ///
    /// This must apply the lens projection
    fn px_rel_xy_to_txty(&self, px_xy: Point2D) -> TanXTanY;

    /// Map a world-space tan(x), tan(y) (i.e. x/z, y/z) to a
    /// centre-relative XY pixel in the frame of the camera
    ///
    /// This must apply the lens projection
    fn txty_to_px_rel_xy(&self, txty: TanXTanY) -> Point2D;
}

//tt CameraView
/// A camera view is a camera projection placed at a particular location in world space in a particular orientation
pub trait CameraView: std::fmt::Debug {
    //mp location
    /// Get a Point3D indicating the placement of the camera in world space
    fn location(&self) -> Point3D;

    //mp direction
    /// Get a quaternion indicating the direction of the camera
    fn direction(&self) -> Quat;

    //fp px_abs_xy_to_camera_txty
    /// Map a screen Point2D coordinate to tan(x)/tan(y)
    fn px_abs_xy_to_camera_txty(&self, px_abs_xy: &Point2D) -> TanXTanY;

    //fp camera_txty_to_px_abs_xy
    /// Map a tan(x)/tan(y) to screen Point2D coordinate
    fn camera_txty_to_px_abs_xy(&self, txty: &TanXTanY) -> Point2D;

    //fp world_xyz_to_camera_xyz (derived)
    /// Convert a Point3D in world space (XYZ) to camera-space
    /// coordinates (XYZ)
    #[inline]
    fn world_xyz_to_camera_xyz(&self, world_xyz: &Point3D) -> Point3D {
        let camera_relative_xyz = *world_xyz - self.location();
        quat::apply3(self.direction().as_ref(), camera_relative_xyz.as_ref()).into()
    }

    //fp camera_xyz_to_world_xyz (derived)
    /// Convert a Point3D in camera space (XYZ) to world space
    /// coordinates (XYZ)
    fn camera_xyz_to_world_xyz(&self, camera_xyz: &Point3D) -> Point3D {
        let camera_relative_xyz: Point3D = quat::apply3(
            &quat::conjugate(self.direction().as_ref()),
            camera_xyz.as_ref(),
        )
        .into();
        camera_relative_xyz + self.location()
    }

    //fp world_xyz_to_camera_txty (derived)
    /// Convert a Point3D in world space (XYZ) to camera-space
    /// TanX/TanY coordinates (XY)
    #[inline]
    fn world_xyz_to_camera_txty(&self, world_xyz: &Point3D) -> TanXTanY {
        self.world_xyz_to_camera_xyz(world_xyz).into()
    }

    //fp world_xyz_to_px_abs_xy (derived)
    /// Map a world Point3D coordinate to camera-space coordinates,
    /// and then to tan(x)/tan(y), then to camera sensor pixel X-Y coordinates
    #[inline]
    fn world_xyz_to_px_abs_xy(&self, world_xyz: &Point3D) -> Point2D {
        self.camera_txty_to_px_abs_xy(&self.world_xyz_to_camera_txty(world_xyz))
    }
}
