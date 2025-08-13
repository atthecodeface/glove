//a Imports
use geo_nd::Vector;

//a Type aliases
pub type Point2D = geo_nd::FArray<f64, 2>;
pub type Point3D = geo_nd::FArray<f64, 3>;
pub type Mat3x3 = geo_nd::FArray2<f64, 3, 9>;
pub type Point4D = geo_nd::FArray<f64, 4>;
pub type Quat = geo_nd::QArray<f64>;

//a TanXTanY
//tp TanXTanY
/// A view-space effectively encoding a direction vector [x,y,-1]
///
/// This is held as a Point2D vector (X, Y)
///
/// This might be used to represent a 'normalized' XY position of a
/// ray through a focal point - be it the model side or the camera
/// side of a lens.
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

    //fp to_unit_vector
    /// Convert to a unit vector
    #[inline]
    pub fn to_unit_vector(self) -> Point3D {
        let p: Point3D = [self.data[0], self.data[1], 1.0].into();
        p.normalize()
    }
}

//ip AsRef<[f64; 2]> for TanXTanY
impl std::convert::AsRef<[f64; 2]> for TanXTanY {
    fn as_ref(&self) -> &[f64; 2] {
        self.data.as_ref()
    }
}

//ip std::convert::AsMut<[f64; 2]> for TanXTanY
impl std::convert::AsMut<[f64; 2]> for TanXTanY {
    fn as_mut(&mut self) -> &mut [f64; 2] {
        self.data.as_mut()
    }
}

//ip std::ops::Index<usize> for TanXTanY
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
/// The Euclidean vector is (tan(yaw).cos(roll), tan(yaw).sin(roll), -1) normalized.
///
/// (len is sqrt(1+tan^2(yaw)) = sec(yaw)); 1/len = cos(yaw)
///
/// Hence the unit vector is (sin(yaw).cos(roll), sin(yaw).sin(roll), -cos(yaw))
///
/// To 'look at' a point in a framed image, roll around the -ve 'z'
/// axis (in/out of the frame), putting the point on the +ve X axis,
/// then yaw (rotate about +ve Y)
#[derive(Debug, Clone, Copy)]
pub struct RollYaw {
    /// Angle that the RollYaw direction is offset from [0,0,-1]
    ///
    /// Held as a tan
    tan_yaw: f64,

    /// Angle that yaw([0,0,-1]) must be rotated around the
    /// -ve Z axis to achieve the direction inndicated by this RollYaw
    ///
    /// Held as sin and cos, with sin^2 + cos^2 = 1
    roll: (f64, f64),
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
        let r = self.tan_yaw;
        let c = self.roll.1;
        let s = self.roll.0;
        [r * c, r * s].into()
    }

    //fp from_txty
    /// From a direction vector [tanX, tanY, -1] determine the Roll
    /// and Yaw that must be applied to [0,0,-1]
    ///
    /// The direction vector txty is effectively (tanT.cosR, tanT.sinR, -1) (T=yaw)
    ///
    /// Hence roll = arctan(ty / tx); yaw = arctan(|txty|)
    ///
    /// sin(roll) = ty / |txty|
    /// cos(roll) = tx / |txty|
    #[inline]
    pub fn from_txty(txty: TanXTanY) -> Self {
        let tan_yaw = txty.data.length();
        if tan_yaw < 1E-8 {
            Self {
                roll: (1., 0.),
                tan_yaw: 0.,
            }
        } else {
            let roll = (txty[1] / tan_yaw, txty[0] / tan_yaw);
            Self { roll, tan_yaw }
        }
    }

    //cp of_yaw
    pub fn of_yaw(yaw: f64) -> Self {
        let tan_yaw = yaw.tan();
        let roll = (0.0, 1.0);
        Self { roll, tan_yaw }
    }

    //cp from_roll_yaw
    pub fn from_roll_yaw(sin_roll: f64, cos_roll: f64, yaw: f64) -> Self {
        let tan_yaw = yaw.tan();
        let roll = (sin_roll, cos_roll);
        Self { roll, tan_yaw }
    }

    //cp from_roll_tan_yaw
    pub fn from_roll_tan_yaw(sin_roll: f64, cos_roll: f64, tan_yaw: f64) -> Self {
        let roll = (sin_roll, cos_roll);
        Self { roll, tan_yaw }
    }

    //ap cos_roll
    pub fn cos_roll(&self) -> f64 {
        self.roll.1
    }

    //ap sin_roll
    pub fn sin_roll(&self) -> f64 {
        self.roll.0
    }

    //ap tan_yaw
    pub fn tan_yaw(&self) -> f64 {
        self.tan_yaw
    }

    //ap with_tan_yaw
    pub fn with_tan_yaw(mut self, tan_yaw: f64) -> Self {
        self.tan_yaw = tan_yaw;
        self
    }

    //ap yaw
    /// Not a high performance operation
    pub fn yaw(&self) -> f64 {
        self.tan_yaw.atan()
    }

    //ap roll
    /// Not a high performance operation
    pub fn roll(&self) -> f64 {
        self.roll.0.atan2(self.roll.1)
    }

    //zz All done
}

//ip From<RollYaw> for TanXTanY
impl From<RollYaw> for TanXTanY {
    fn from(ry: RollYaw) -> TanXTanY {
        ry.to_txty()
    }
}

//ip From<&RollYaw> for TanXTanY
impl<'a> From<&'a RollYaw> for TanXTanY {
    fn from(ry: &'a RollYaw) -> TanXTanY {
        ry.to_txty()
    }
}

//ip From<TanXTanY> for RollYaw
impl From<TanXTanY> for RollYaw {
    fn from(txty: TanXTanY) -> RollYaw {
        RollYaw::from_txty(txty)
    }
}

//ip From<&TanXTanY> for RollYaw
impl<'a> From<&'a TanXTanY> for RollYaw {
    fn from(txty: &'a TanXTanY) -> RollYaw {
        RollYaw::from_txty(*txty)
    }
}

//ip Display for RollYaw
impl std::fmt::Display for RollYaw {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "[yaw {:0.4}:roll {:0.4}]",
            self.tan_yaw.tan().to_degrees(),
            self.roll().to_degrees()
        )
    }
}
