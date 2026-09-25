//a Documentation
/*!

From multiple rays-with-errors can generate mode positions, errors and
confidences

A ray with error is a starting Point3D and a unit direction vector
Point3D and a error tan ratio E

The target area of the ray is a distance D from the starting point
such that the error circle around the point of the target area has
radius R such that E = R / D

If the ray is generated from a picture without a model position then
the error angle is perhaps more obvious

A line can be described as P = A + k*D, for a starting position a with direction d

Hence P^D = A^D + k*(D^D) = A^D

k*D = P-A => k = k*(D.D) = (P-A).D

Given a ray with error has a starting point A and directon D, and an
error at a destination point (A + k*D), and the error is a circle of
radius R at this distance, i.e. R = k*E. i.e. error bar at distance k
is k*E

Now imagine two rays, and finding the best intersection point and its
error, to provide a point-with-error-and-confidence.

(From a point-with-error-and-confidence one can presumably generate more rays-with-error given starting points.)

If we have two rays then they will pass (in 3D) with a line between
them at their closest that is perpendicular to both lines.

The meeting line is then P0 to P1, with:

```ignore
  P0 = A0 + k0*D0

  P1 = A1 + k1*D1

  Call Dn = (D0 ^ D1) / |D0 ^ D1|

  (P1 - P0) = (A1 - A0) + (k1*D1 - k0*D0)

  (P1 - P0).D0 = 0 = (A1 - A0).D0 + (k1*D1 - k0*D0).D0

    Hence (A0 - A1).D0 = k1*(D1.D0) - k0 and
          (A0 - A1).D1 = k1         - k0*(D1.D0)

    Ad.D0 = k1*Dd - k0 => Dd*(Ad.D0) = k1*Dd*Dd - k0*Dd
    Ad.D1 = k1 - k0*Dd

    Subtracting these => Ad.D1 - Dd*(Ad.D0) = k1*(1-Dd*Dd)

    k1 = (Ad.D1 - Dd*(Ad.D0)) / (1-Dd*Dd)
    k0 = (Ad.D0 - Dd*(Ad.D1)) / (1-Dd*Dd)

  Also note that (P1-P0) = l * Dn

   l = l * Dn.Dn = (P1-P0).Dn = (A1 - A0).Dn (As D0.Dn=0 etc)

  The error at these distance is R0 = k0*E0, R1 = k1*E1

  The desired target point is at (R1*P0 + R0*P1) / (R0 + R1)

  The error is kinda the overlap; hence Rn = min(R1-l, R0-l)

  If Rn is less than 0

  The rays with error start at their original destinations

```

!*/

use serde::{Deserialize, Serialize};

use geo_nd::{Vector, matrix};

use crate::utils;
use crate::{JsonParsable, Point3D, Result};

/// A NamedRayList is a list of (name, ray);
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NamedRayList {
    named_rays: Vec<(String, Ray)>,
}

impl std::convert::From<Vec<(String, Ray)>> for NamedRayList {
    fn from(named_rays: Vec<(String, Ray)>) -> Self {
        Self { named_rays }
    }
}

impl NamedRayList {
    /// Return true if the list of rays is empty
    pub fn is_empty(&self) -> bool {
        self.named_rays.is_empty()
    }

    /// Get the length of the list of rays
    pub fn len(&self) -> usize {
        self.named_rays.len()
    }

    /// Get an iterator over the (name, ray) pairs
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &(String, Ray)> {
        self.named_rays.iter()
    }

    /// Add a named ray to the list
    pub fn add_ray<S: Into<String>>(&mut self, name: S, ray: Ray) {
        self.named_rays.push((name.into(), ray))
    }

    /// Append another list of named rays to this list
    pub fn append(&mut self, mut other: Self) {
        self.named_rays.append(&mut other.named_rays)
    }

    /// Generate the JSON of this named ray list
    pub fn to_json(&self, pretty: bool) -> Result<String> {
        if pretty {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            Ok(serde_json::to_string(self)?)
        }
    }
}

impl JsonParsable for NamedRayList {
    type PostParseArg = ();
    type PostParseResult = NamedRayList;
    fn reason() -> &'static str {
        "named ray list"
    }
    fn post_parse(self, _args: &()) -> Result<Self> {
        Ok(self)
    }
}

/// Simply, a Ray is a vector direction in space (model or camera); however, it
/// also has a 'start' point and an error value, recorded as the tan of the
/// angle that the ray is 'known' to be within (its error bar, effectively)
///
/// A ray is then effectively a cone from the starting point in the direction of
/// the vector with an angle given by tan_error
///
/// For a ray corresponding to a point-mapping that 'travels' from the sensor of
/// the camera, through the focus, through the lens mapping, to the world, the
/// direction is from the centre of the lens (the camera position) as given by
/// the lens mapping of the Roll/Yaw of the sensor position mapped through to
/// world space (accounting for the camera orientation). The error is such that
/// the error in the point mapping (a radius in pixels) yields other rays
/// (mapped in the same manner as above, from different sensor pixels); the
/// resultant ray with the largest divergence from the central ray subtends an
/// angle, and the tan of that angle is the tan_error.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Ray {
    /// Starting point
    start: Point3D,
    /// Direction (unit vector)
    direction: Point3D,
    /// Tan of error such that actual error radius = distance*tan_error
    tan_error: f64,
}

impl JsonParsable for Ray {
    type PostParseArg = ();
    type PostParseResult = Ray;
    fn reason() -> &'static str {
        "ray"
    }
    fn post_parse(self, _args: &()) -> Result<Self> {
        Ok(self)
    }
}

impl std::fmt::Display for Ray {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "[Ray {}+k*{} @ {}]",
            self.start, self.direction, self.tan_error
        )
    }
}

impl Ray {
    /// Constructor: set the start point of the ray
    #[inline]
    #[must_use]
    pub fn with_start(mut self, start: Point3D) -> Self {
        self.start = start;
        self
    }

    /// Constructor: set the direction of the ray
    #[inline]
    #[must_use]
    pub fn with_direction(mut self, direction: Point3D) -> Self {
        self.direction = direction.normalize();
        self
    }

    /// Constructor: set the tan(angle error) of the ray
    #[inline]
    #[must_use]
    pub fn with_tan_error(mut self, tan_error: f64) -> Self {
        self.tan_error = tan_error;
        self
    }

    /// Get the start position of the ray
    #[inline]
    pub fn start(&self) -> Point3D {
        self.start
    }

    /// Get the direction of the ray
    #[inline]
    pub fn direction(&self) -> Point3D {
        self.direction
    }

    /// Get the tan(angle error) of the ray
    #[inline]
    pub fn tan_error(&self) -> f64 {
        self.tan_error
    }

    /// Find the point whose (weighted) square distance from all the rays is the minimum
    ///
    /// The distance of a point p from a line (given by an origin A and unit direction B) can be determined as:
    ///
    ///         P = A + k.B + d.N (where B.N=0, |N|=1, for some N), hence
    ///       d.N = P - A - k.B
    ///   d.(NxB) = (PxB) - (AxB) - k.(BxB)
    ///   d.(NxB) = (P-A)xB
    /// |d.(NxB)| = |(P-A)xB|, but |NxB|=1 so
    ///       |d| = |(P-A)xB|
    ///       d^2 = (PxB.PxB) - 2(AxB.PxB) + (AxB.AxB)
    ///
    /// Summing this for a single point P and multiple rays (each with an A and B)
    /// yields a total square error; differentiating with respect to
    /// the coordinates of p yields a vector of 0s when p the error is minimized
    ///
    ///        Sum(E^2) = Sum((PxB.PxB) - 2(AxB.PxB) + (AxB.AxB))
    /// d/dPx(Sum(E^2)) = Sum(d/dPx((PxB.PxB) - 2(AxB.PxB) + (AxB.AxB)))
    /// d/dPx(Sum(E^2)) = Sum(d/dPx((PxB.PxB) - 2(AxB.PxB)))
    ///
    /// First, multiplying out etc:
    ///
    ///        PxB       = (Py.Bz - Pz.By, Pz.Bx - Px.Bz, Px.By - Bx.Py)
    ///        PxB . PxB = (Py.Bz - Pz.By)^2 + (Pz.Bx - Px.Bz)^2 + (Px.By - Bx.Py)^2
    /// d/dPx(PxB . PxB) = 2(Px.Bz.Bz - Pz.Bx.Bz + Px.By.By - Py.Bx.By)
    ///
    ///        PxB . AxB = (Py.Bz - Pz.By).(Ay.Bz - Az.By) + (Pz.Bx - Px.Bz).(Az.Bx - Ax.Bz) + (Px.By - Bx.Py).(Ax.By - Bx.Ay)
    /// d/dPx(PxB . AxB) = -Bz.(Az.Bx - Ax.Bz) + By.(Ax.By - Bx.Ay)
    ///
    /// Hence
    ///
    /// d/dPx(Sum(E^2)) = Sum( 2(Px.Bz.Bz - Pz.Bx.Bz + Px.By.By - Py.Bx.By) +2Bz.(Az.Bx - Ax.Bz) - 2By.(Ax.By - Bx.Ay) )
    ///                 = 2Sum( Px.Bz.Bz - Pz.Bx.Bz + Px.By.By - Py.Bx.By +Bz.Az.Bx -Bz.Ax.Bz -By.Ax.By +By.Bx.Ay)
    ///                 = 2Sum( Px.(By.By + Bz.Bz) + Py.(-Bx.By) + Pz.(-Bx.Bz) - Ax.(Bz.Bz + By.By) + Ay.Bx.By + Az.Bx.Bz )
    ///
    /// At a minimum (and for each coordinate) this is 0; at this we have
    ///   Px.(By.By + Bz.Bz) + Py.(-Bx.By)        + Pz.(-Bx.Bz)        - Ax.(Bz.Bz + By.By) + Ay.Bx.By + Az.Bx.Bz = 0
    ///   Px.(-By.Bx)        + Py.(Bx.Bx + Bz.Bz) + Pz.(-By.Bz)        - Ay.(Bx.Bx + Bz.Bz) + Az.By.Bz + Ax.By.Bx = 0
    ///   Px.(-Bz.Bx)        + Py.(-Bz.By)        + Pz.(Bx.Bx + By.By) - Az.(By.By + Bx.Bx) + Ax.Bz.Bx + Ay.Bz.By = 0
    ///
    /// and we can find M such that M . (px py pz) = V, invert M, and find (px py pz)
    ///
    /// This does not take into account the 'error' of each ray; this
    /// could be used if an estimate for the *distance* error of each
    /// ray at an approximate solution can be found, and to weight
    /// each ray by some inversely proportional function of this
    /// *distance* error (such as 1/(base + distance^2)).
    pub fn closest_point<'a, F: Fn(&Self, usize) -> f64>(
        rays: impl Iterator<Item = &'a Self> + 'a,
        weight_fn: &F,
    ) -> Option<Point3D> {
        let mut m = [0.; 9];
        let mut v = [0.; 3];
        for (n, r) in rays.enumerate() {
            let w = weight_fn(r, n);
            let ax = r.start[0];
            let ay = r.start[1];
            let az = r.start[2];
            let bx = r.direction[0];
            let by = r.direction[1];
            let bz = r.direction[2];
            m[0] += w * (by * by + bz * bz);
            m[1] += w * (-bx * by);
            m[2] += w * (-bx * bz);
            v[0] += w * (ax * (by * by + bz * bz) - ay * bx * by - az * bx * bz);

            m[3] += w * (-by * bx);
            m[4] += w * (bx * bx + bz * bz);
            m[5] += w * (-by * bz);
            v[1] += w * (ay * (bx * bx + bz * bz) - ax * bx * by - az * by * bz);

            m[6] += w * (-bz * bx);
            m[7] += w * (-bz * by);
            m[8] += w * (by * by + bx * bx);
            v[2] += w * (az * (by * by + bx * bx) - ax * bx * bz - ay * by * bz);
        }

        let Some(dm) = utils::matrix_invert_dyn(3, &m) else {
            return None;
        };
        // dbg!(&dm);
        let mut dm_2 = Vec::with_capacity(9); // P row vector
        for i in 0..9 {
            dm_2.push(dm[i]);
        }
        let mut p = [0.; 3];
        matrix::multiply_dyn(3, 3, 1, &dm_2, &v, &mut p);
        Some(p.into())
    }

    /// Find the distance along and square distance from the ray of a point
    ///
    /// p = a + k*b + D*n where n.b=0 and |n|=1, and |b|=1, |n^b|=1
    ///
    /// k*b + D*n = (p-a)
    ///
    /// k*b.b => k = (p-a).b
    ///
    /// D*n^b = (p-a)^b
    ///
    /// D^2 = |(p-a)^b| ^ 2
    pub fn distances(&self, pt: &Point3D) -> (f64, f64) {
        let p_minus_a = *pt - self.start;
        let k = p_minus_a.dot(self.direction);
        let cross = p_minus_a.cross_product(self.direction);
        let d_sq = cross.length_sq();
        (k, d_sq)
    }

    /// Intersect two rays
    ///
    /// Output data for debug
    pub fn intersect(&self, other: &Self) {
        let d_n = self.direction.cross_product(other.direction);
        let l_d_n_sq = d_n.length_sq();

        // dbg!(d_n, l_d_n_sq);
        // if l_d_n_sq < 1.0E-8 {}
        let a_diff = self.start - other.start;
        let dot_ds = self.direction.dot(other.direction);
        let a_diff_dot_d0 = self.direction.dot(a_diff);
        let a_diff_dot_d1 = other.direction.dot(a_diff);

        // dbg!(a_diff, dot_ds, a_diff_dot_d0, a_diff_dot_d1);

        let k0 = -(a_diff_dot_d0 - dot_ds * a_diff_dot_d1) / (1.0 - dot_ds * dot_ds);
        let k1 = (a_diff_dot_d1 - dot_ds * a_diff_dot_d0) / (1.0 - dot_ds * dot_ds);

        let r0 = (k0 * self.tan_error).abs();
        let r1 = (k1 * other.tan_error).abs();

        // dbg!(k0, k1, r0, r1);

        let p0 = self.start + self.direction * k0;
        let p1 = other.start + other.direction * k1;

        let l = a_diff.dot(d_n) / l_d_n_sq.sqrt();
        // dbg!(p0, p1, l);
        dbg!(k0, k1, r0, r1, l);

        let rp0 = p0 * r0;
        let rp1 = p1 * r1;
        let rp0_plus_rp1 = rp0 + rp1;
        let target = rp0_plus_rp1 / (r0 + r1);

        let rm = (r1 - l.abs()).min(r0 - l.abs());

        // confidence is probably proportional to overlap / min(error)

        // dbg!(r0, r1, l_d_n_sq.sqrt());

        dbg!(rm);
        dbg!(target);
    }
}
