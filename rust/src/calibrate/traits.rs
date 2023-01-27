//a Imports
use super::{Point2D, Point3D, RollDist, RollYaw};

//a Traits
//tt LensProjection
/// The concept is that there are absolute pixel positions within a sensor,
/// which can be converted to relative, which can be converted to an RollDist, RollYaw,
/// which can be converted to tan(x)/tan(y) - in model space X/Z and Y/Z, which can be
/// mapped from (but not really to) xyz
///
/// The lens projection is between RollYaw and
/// tan(x)/tan(y). Essentially RollYaw is kinda internal
pub trait LensProjection: std::fmt::Debug {
    /// Map from absolute to centre-relative pixel
    fn px_abs_xy_to_px_rel_xy(&self, xy: Point2D) -> Point2D;

    /// Map from centre-relative to absolute pixel
    fn px_rel_xy_to_px_abs_xy(&self, xy: Point2D) -> Point2D;

    /// Map an actual centre-relative XY pixel in the frame of the
    /// camera to a Roll/Yaw
    ///
    /// This generally does not use a lens projection but does need
    /// frame sizes etc
    fn px_rel_xy_to_ry(&self, xy: Point2D) -> RollYaw;

    /// Map a Roll/Yaw to a centre-relative XY pixel in the frame of
    /// the camera
    ///
    /// This generally does not use a lens projection but does need
    /// frame sizes etc
    fn ry_to_px_rel_xy(&self, ry: RollYaw) -> Point2D;

    /// Map a roll/yaw to a tan(x), tan(y) (i.e. x/z, y/z)
    ///
    /// This must apply the lens projection
    ///
    /// The default function has a 'null' lens projection mapping,
    /// which probably does not make sense
    #[inline]
    fn ry_to_txty(&self, ry: RollYaw) -> Point2D {
        let r = ry.yaw.tan();
        let c = ry.roll.cos();
        let s = ry.roll.sin();
        [r * c, r * s].into()
    }

    /// Map a tan(x), tan(y) (i.e. x/z, y/z) to a roll/yaw
    ///
    /// This must apply the lens projection
    ///
    /// The default function has a 'null' lens projection mapping,
    /// which probably does not make sense
    #[inline]
    fn txty_to_ry(&self, txty: Point2D) -> RollYaw {
        let yaw = (txty[0] * txty[0] + txty[1] * txty[1]).sqrt().atan();
        let roll = txty[1].atan2(txty[0]);
        RollYaw { roll, yaw }
    }

    /// Map an actual centre-relative XY pixel in the frame of the
    /// camera to a tan(x), tan(y)
    ///
    /// This must apply the lens projection
    ///
    /// The default functions combines other mapping functions, so may
    /// not be fully optimized
    #[inline]
    fn px_rel_xy_to_txty(&self, xy: Point2D) -> Point2D {
        let ry = self.px_rel_xy_to_ry(xy);
        self.ry_to_txty(ry)
    }

    /// Map a tan(x), tan(y) (i.e. x/z, y/z) to a centre-relative XY
    /// pixel in the frame of the camera
    ///
    /// This must apply the lens projection
    ///
    /// An implementation can improve the performance for some lenses
    /// where this is a much simpler mapping than the two stages combined
    #[inline]
    fn txty_to_px_rel_xy(&self, txty: Point2D) -> Point2D {
        let ry = self.txty_to_ry(txty);
        self.ry_to_px_rel_xy(ry)
    }

    /// Map an (X,Y,Z) to tan(x), tan(y)
    #[inline]
    fn rel_xyz_to_txty(&self, xyz: Point3D) -> Point2D {
        [xyz[0] / xyz[2], xyz[1] / xyz[2]].into()
    }
}
