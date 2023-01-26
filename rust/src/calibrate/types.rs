pub type Point2D = geo_nd::FArray<f64, 2>;
pub type Point3D = geo_nd::FArray<f64, 3>;
pub type Point4D = geo_nd::FArray<f64, 4>;
pub type Quat = geo_nd::QArray<f64, Point3D, Point4D>;
//tt LensProjection
pub trait LensProjection: std::fmt::Debug {
    /// Map from centre-relative to absolute pixel
    fn px_rel_xy_to_px_abs_xy(&self, xy: Point2D) -> Point2D;

    /// Map from absolute to centre-relative pixel
    fn px_abs_xy_to_px_rel_xy(&self, xy: Point2D) -> Point2D;

    /// Map an actual centre-relative XY pixel in the frame of the
    /// camera to a Roll/Yaw
    fn px_rel_xy_to_ry(&self, xy: Point2D) -> RollYaw;

    /// Map a Roll/Yaw to a centre-relative XY pixel in the frame of
    /// the camera
    fn ry_to_px_rel_xy(&self, ry: RollYaw) -> Point2D;

    /// Map an actual centre-relative XY pixel in the frame of the
    /// camera to a Roll/Yaw
    fn px_rel_xy_to_txty(&self, xy: Point2D) -> Point2D {
        let ry = self.px_rel_xy_to_ry(xy);
        let r = ry.yaw.tan();
        let c = ry.roll.cos();
        let s = ry.roll.sin();
        [r * c, r * s].into()
    }

    /// Map a tan(x), tan(y) (i.e. x/z, y/z) to a centre-relative XY
    /// pixel in the frame of the camera
    ///
    /// This must apply the lens projection
    fn txty_to_px_rel_xy(&self, txty: Point2D) -> Point2D {
        let yaw = (txty[0] * txty[0] + txty[1] * txty[1]).sqrt().atan();
        let roll = txty[1].atan2(txty[0]);
        self.ry_to_px_rel_xy(RollYaw { roll, yaw })
    }

    /// Map an (X,Y,Z) to tan(x), tan(y)
    fn rel_xyz_to_txty(&self, xyz: Point3D) -> Point2D {
        [xyz[0] / xyz[2], xyz[1] / xyz[2]].into()
    }
}

//tp RollYaw
/// To 'look at' a point in a framed image, roll around the -ve 'z'
/// axis (in/out of the frame), putting the point on the +ve X axis,
/// then yaw (rotate about +ve Y)
#[derive(Debug, Clone, Copy)]
pub struct RollYaw {
    pub roll: f64,
    pub yaw: f64,
}
