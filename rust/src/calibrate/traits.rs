//a Imports
use super::{Point2D, TanXTanY};

//a Traits
//tt CameraSensor
/// A trait for a sensor in a digital camera, that maps
///
/// The concept is that there are absolute pixel positions within a sensor,
/// which can be converted to relative, which can be converted to a RollDist, which is a
pub trait CameraSensor: std::fmt::Debug {
    /// Map from absolute to centre-relative pixel
    fn px_abs_xy_to_px_rel_xy(&self, xy: Point2D) -> Point2D;

    /// Map from centre-relative to absolute pixel
    fn px_rel_xy_to_px_abs_xy(&self, xy: Point2D) -> Point2D;
}

//tt LensProjection
/// The concept is that there are absolute pixel positions within a sensor,
/// which can be converted to relative, which can be converted to an RollDist, RollYaw,
/// which can be converted to tan(x)/tan(y) - in model space X/Z and Y/Z, which can be
/// mapped from (but not really to) xyz
///
/// The lens projection is between RollYaw and
/// tan(x)/tan(y). Essentially RollYaw is kinda internal
pub trait LensProjection: std::fmt::Debug {
    fn sensor_to_world(&self, tan: f64) -> f64;
    fn world_to_sensor(&self, tan: f64) -> f64;
}

//tt CameraProjection
/// A camera projection is a combination of a camera sensor and a lens
// pub trait Camera: LensProjection + CameraSensor {
pub trait CameraProjection: std::fmt::Debug {
    /// Map from absolute to centre-relative pixel
    fn px_abs_xy_to_px_rel_xy(&self, xy: Point2D) -> Point2D;

    /// Map from centre-relative to absolute pixel
    fn px_rel_xy_to_px_abs_xy(&self, xy: Point2D) -> Point2D;

    /// Map an actual centre-relative XY pixel in the frame of the
    /// camera to a tan(x), tan(y)
    ///
    /// This must apply the lens projection
    fn px_rel_xy_to_txty(&self, xy: Point2D) -> TanXTanY;

    /// Map a tan(x), tan(y) (i.e. x/z, y/z) to a centre-relative XY
    /// pixel in the frame of the camera
    ///
    /// This must apply the lens projection
    fn txty_to_px_rel_xy(&self, txty: TanXTanY) -> Point2D;
}
