pub type Point2D = geo_nd::FArray<f64, 2>;
pub type Point3D = geo_nd::FArray<f64, 3>;
pub type Point4D = geo_nd::FArray<f64, 4>;
pub type Quat = geo_nd::QArray<f64, Point3D, Point4D>;
//tt LensProjection
pub trait LensProjection {
    /// Map a Roll/Yaw to a centre-relative XY pixel in the frame of
    /// the camera
    fn ry_to_xy(&self, ry: RollYaw) -> [f64; 2];

    /// Map an actual centre-relative XY pixel in the frame of the
    /// camera to a Roll/Yaw
    fn xy_to_ry(&self, xy: [f64; 2]) -> RollYaw;
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
