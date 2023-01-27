//a Imports
use geo_nd::vector;

//a Type aliases
pub type Point2D = geo_nd::FArray<f64, 2>;
pub type Point3D = geo_nd::FArray<f64, 3>;
pub type Point4D = geo_nd::FArray<f64, 4>;
pub type Quat = geo_nd::QArray<f64, Point3D, Point4D>;

//a TanXTanY
//tp TanXTanY
/// A view-space effectively encoding a direction vector [x,y,-1]
///
/// This is held as a Point2D vector (X, Y)
#[derive(Debug, Clone, Copy)]
pub struct TanXTanY {
    /// X and Y coordinates
    data: geo_nd::FArray<f64, 2>,
}

//ip TanXTanY
impl TanXTanY {
    //fp to_ry
    /// Convert to a Roll/Yaww
    #[inline]
    pub fn to_ry(self) -> RollYaw {
        RollYaw::from_txty(self)
    }
}
impl std::convert::AsRef<[f64; 2]> for TanXTanY {
    fn as_ref(&self) -> &[f64; 2] {
        self.data.as_ref()
    }
}
impl std::convert::AsMut<[f64; 2]> for TanXTanY {
    fn as_mut(&mut self) -> &mut [f64; 2] {
        self.data.as_mut()
    }
}
impl std::ops::Index<usize> for TanXTanY {
    type Output = f64;
    fn index(&self, index: usize) -> &f64 {
        let slice: &[_] = self.as_ref();
        &slice[index]
    }
}

//ip From<&[f64; 3]> for TanXTanY
impl From<&[f64; 3]> for TanXTanY {
    #[inline]
    fn from(p: &[f64; 3]) -> TanXTanY {
        let mut z = p[2];
        if z.abs() < 1.0E-8 {
            z = z.signum() * 1.0E-8;
        }
        TanXTanY {
            data: [p[0] / z, p[1] / z].into(),
        }
    }
}

//ip From<[f64; 3]> for TanXTanY
// impl From<[f64; 3]> for TanXTanY {
//     #[inline]
//     fn from(p: [f64; 3]) -> TanXTanY {
//         TanXTanY::from(&p)
//     }
// }

//ip From<&Point3D> for TanXTanY
impl From<&Point3D> for TanXTanY {
    #[inline]
    fn from(p: &Point3D) -> TanXTanY {
        let p: &[f64; 3] = p.as_ref();
        TanXTanY::from(p)
    }
}

//ip From<Point3D> for TanXTanY
impl From<Point3D> for TanXTanY {
    #[inline]
    fn from(p: Point3D) -> TanXTanY {
        let p: &[f64; 3] = p.as_ref();
        TanXTanY::from(p)
    }
}

//ip From<[f64; 2]> for TanXTanY
impl From<[f64; 2]> for TanXTanY {
    #[inline]
    fn from(p: [f64; 2]) -> Self {
        TanXTanY { data: p.into() }
    }
}

//ip From<&[f64; 2]> for TanXTanY
impl From<&[f64; 2]> for TanXTanY {
    #[inline]
    fn from(p: &[f64; 2]) -> TanXTanY {
        TanXTanY::from(*p)
    }
}

//ip From<&Point2D> for TanXTanY
impl From<&Point2D> for TanXTanY {
    #[inline]
    fn from(p: &Point2D) -> TanXTanY {
        let p: &[f64; 2] = p.as_ref();
        TanXTanY::from(p)
    }
}

//ip From<TanXTanY> for Point2D
impl From<TanXTanY> for Point2D {
    #[inline]
    fn from(p: TanXTanY) -> Point2D {
        p.data
    }
}

//ip Display for TanXTanY
impl std::fmt::Display for TanXTanY {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "[{:0.4},{:0.4},-1]", self[0], self[1],)
    }
}

//a RollYaw
//tp RollYaw
/// A view-space effectively encoding a vector [x,y,-1] as a kind
/// of spherical coordinates, with 'yaw' as the off-centre angle
/// (angle away from [0,0,-1]), and roll as the angle required to
/// rotate around [0,0,-1] to place the point on the Z=0 plane with +ve
/// X.
///
/// The Euclidean vector is (tan(yaw).cos(roll), tan(yaw).sin(roll), -1)
///
/// To 'look at' a point in a framed image, roll around the -ve 'z'
/// axis (in/out of the frame), putting the point on the +ve X axis,
/// then yaw (rotate about +ve Y)
#[derive(Debug, Clone, Copy)]
pub struct RollYaw {
    /// Angle in radians that the RollYaw direction is offset from [0,0,-1]
    pub yaw: f64,
    /// Angle in radians that yaw([0,0,-1]) must be rotated around the
    /// -ve Z axis to achieve the direction inndicated by this RollYaw
    pub roll: f64,
}

//ip RollYaw
impl RollYaw {
    //fp to_txty
    /// Apply RollYaw to the point [0,0,-1] and return the direction vector (X, Y, -1) it corresponds to:
    /// the coordinates will be (tanT.cosR, tanT.sinR) (T=yaw)
    ///
    /// Apply the Yaw (rotate about Y axis by T) yields (-sinT, 0, -cosT)
    ///
    /// Apply the Roll (rotate about -ve Z axis by R) yields (sinT.cosR, sinT.sinR, -cosT)
    ///
    /// Convert to tanx/tany (divide by Z) yieds (tanT.cosR, tanT.sinR)
    #[inline]
    pub fn to_txty(self) -> TanXTanY {
        let r = self.yaw.tan();
        let c = self.roll.cos();
        let s = self.roll.sin();
        [r * c, r * s].into()
    }

    //fp from_txty
    /// From a direction vector [tanX, tanY, -1] determine the Roll
    /// and Yaw that must be applied to [0,0,-1]
    ///
    /// The direction vector txty is effectively (tanT.cosR, tanT.sinR, -1) (T=yaw)
    ///
    /// Hence roll = arctan(ty / tx); yaw = arctan(|txty|)
    #[inline]
    pub fn from_txty(txty: TanXTanY) -> Self {
        let yaw = vector::length(txty.as_ref()).atan();
        let roll = txty[1].atan2(txty[0]);
        Self { roll, yaw }
    }
}

//ip From<RollYaw> for TanXTanY
impl From<RollYaw> for TanXTanY {
    fn from(ry: RollYaw) -> TanXTanY {
        ry.to_txty()
    }
}

//ip From<TanXTanY> for RollYaw
impl From<TanXTanY> for RollYaw {
    fn from(txty: TanXTanY) -> RollYaw {
        RollYaw::from_txty(txty)
    }
}

//ip Display for RollYaw
impl std::fmt::Display for RollYaw {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "[yaw {:0.4}:roll {:0.4}]",
            self.yaw.to_degrees(),
            self.roll.to_degrees()
        )
    }
}

//a RollDist
//tp RollDist
/// This is a real-world 2D polar coordinate most useful for spherical lenses
///
/// This represents a mm distance from the 'centre' of the frame (the
/// point on the frame that the direction the camera is facing in
/// i.e. [0,0,-1] hits the frame) and the 'roll' angle (rotation around [0,0,-1]).
///
/// An (x,y) (in mm) on the frame maps to (arctan(y/x), sqrt(x^2+y^2))
#[derive(Debug, Clone, Copy)]
pub struct RollDist {
    pub roll: f64,
    pub dist: f64,
}
//ip RollDist
impl RollDist {
    //fp to_mm_xy
    #[inline]
    pub fn to_mm_xy(self) -> Point2D {
        let c = self.roll.cos();
        let s = self.roll.sin();
        [self.dist * c, self.dist * s].into()
    }

    //fp from_mm_xy
    #[inline]
    pub fn from_mm_xy(mm_xy: Point2D) -> Self {
        let dist = vector::length(mm_xy.as_ref());
        let roll = mm_xy[1].atan2(mm_xy[0]);
        Self { roll, dist }
    }
}

//ip From<RollDist> for Point2D
impl From<RollDist> for Point2D {
    fn from(rd: RollDist) -> Point2D {
        rd.to_mm_xy()
    }
}

//ip From<Point2D> for RollDist
impl From<Point2D> for RollDist {
    fn from(mm_xy: Point2D) -> RollDist {
        RollDist::from_mm_xy(mm_xy)
    }
}

//ip Display for RollDist
impl std::fmt::Display for RollDist {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "[{:0.4}mm @ {:0.4}]",
            self.dist,
            self.roll.to_degrees()
        )
    }
}
