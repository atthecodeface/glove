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

//a Imports
use serde::{Deserialize, Serialize};

use crate::Point3D;

use geo_nd::matrix;
use geo_nd::vector;

//a Ray
//tp Ray
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Ray {
    /// Starting point
    pub start: Point3D,
    /// Direction (unit vector)
    pub direction: Point3D,
    /// Tan of error such that actual error radius = distance*tan_error
    pub tan_error: f64,
}

//ip Display for Ray
impl std::fmt::Display for Ray {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "[Ray {}+k*{} @ {}]",
            self.start, self.direction, self.tan_error
        )
    }
}

//ip Ray
impl Ray {
    //cp set_start
    #[inline]
    pub fn set_start(mut self, start: Point3D) -> Self {
        self.start = start;
        self
    }

    //cp set_direction
    #[inline]
    pub fn set_direction(mut self, direction: Point3D) -> Self {
        self.direction = vector::normalize(*direction.as_ref()).into();
        self
    }

    //cp set_tan_error
    #[inline]
    pub fn set_tan_error(mut self, tan_error: f64) -> Self {
        self.tan_error = tan_error;
        self
    }

    //ap start
    #[inline]
    pub fn start(&self) -> Point3D {
        self.start
    }

    //ap direction
    #[inline]
    pub fn direction(&self) -> Point3D {
        self.direction
    }

    //ap tan_error
    #[inline]
    pub fn tan_error(&self) -> f64 {
        self.tan_error
    }

    //fp closest_point
    /// Find the point whose minimum square distance from all the rays is minimized
    ///
    /// This uses the fact that the distance D a point p is from a line a + k*b can be found
    /// with p = a + k*b + D*n for some unit vector n perpendicular to b hence n.b=0, |n^b|=1;
    ///
    /// D*n = p-a-k*b; vector product with b yields
    /// D*n^b = (p-a)^b
    ///
    /// Taking the modulus of both sides
    /// D*|n^b|= D = |(p-a)^b)|
    ///
    /// Hence D^2 = (p-a)^b . (p-a)^b = (p^b.p^b) - 2(a^b.p^b) + (a^b.a^b)
    ///
    /// Summing this for a single point p and multiple rays (a and b)
    /// yields a total square error; differentiating with respect to
    /// the coordinates of p yields
    ///
    /// d(Esq)/dpx = (by^2 + bz^2). 2px + 2bz.(ay.bz-az.by) - 2by(ay.bx - bx.ay)
    /// etc
    ///
    /// and we can find M such that M . (px py pz) = V, invert M, and find (px py pz)
    ///
    /// This does not take into account the 'error' of each ray; this
    /// could be used if an estimate for the *distance* error of each
    /// ray at an approximate solution can be found, and to weight
    /// each ray by some inversely proportional function of this
    /// *distance* error (such as 1/(base + distance^2)).
    pub fn closest_point<F: Fn(&Self) -> f64>(rays: &[Self], weight_fn: &F) -> Option<Point3D> {
        let mut m = [0.; 9];
        let mut v = [0.; 3];
        for r in rays {
            let w = weight_fn(r);
            let ax = r.start[0];
            let ay = r.start[1];
            let az = r.start[2];
            let bx = r.direction[0];
            let by = r.direction[1];
            let bz = r.direction[2];
            m[0] += w * (by * by + bz * bz);
            m[1] += w * (-bx * by);
            m[2] += w * (-bx * bz);
            v[0] += w * (bz * (ax * bz - az * bx) - by * (ay * bx - ax * by));

            m[3] += w * (-by * bx);
            m[4] += w * (bx * bx + bz * bz);
            m[5] += w * (-by * bz);
            v[1] += w * (bx * (ay * bx - ax * by) - bz * (az * by - ay * bz));

            m[6] += w * (-bz * bx);
            m[7] += w * (-bz * by);
            m[8] += w * (by * by + bx * bx);
            v[2] += w * (by * (az * by - ay * bz) - bx * (ax * bz - az * bx));
        }

        let mut dm = nalgebra::base::DMatrix::from_element(3, 3, 2.0);
        dm.copy_from_slice(&m);
        if !dm.try_inverse_mut() {
            return None;
        }
        // dbg!(&dm);
        let mut dm_2 = Vec::with_capacity(9); // P row vector
        for i in 0..9 {
            dm_2.push(dm[i]);
        }
        let mut p = [0.; 3];
        matrix::multiply_dyn(3, 3, 1, &dm_2, &v, &mut p);
        Some(p.into())
    }

    //fp distances
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
        let k = vector::dot(p_minus_a.as_ref(), self.direction.as_ref());
        let cross = vector::cross_product3(p_minus_a.as_ref(), self.direction.as_ref());
        let d_sq = vector::length_sq(&cross);
        (k, d_sq)
    }

    //mp intersect
    pub fn intersect(&self, other: &Self) -> () {
        let d_n = vector::cross_product3(self.direction.as_ref(), other.direction.as_ref());
        let l_d_n_sq = vector::length_sq(&d_n);

        // dbg!(d_n, l_d_n_sq);
        if l_d_n_sq < 1.0E-8 {}
        let a_diff = self.start - other.start;
        let dot_ds = vector::dot(self.direction.as_ref(), other.direction.as_ref());
        let a_diff_dot_d0 = vector::dot(self.direction.as_ref(), a_diff.as_ref());
        let a_diff_dot_d1 = vector::dot(other.direction.as_ref(), a_diff.as_ref());

        // dbg!(a_diff, dot_ds, a_diff_dot_d0, a_diff_dot_d1);

        let k0 = -(a_diff_dot_d0 - dot_ds * a_diff_dot_d1) / (1.0 - dot_ds * dot_ds);
        let k1 = (a_diff_dot_d1 - dot_ds * a_diff_dot_d0) / (1.0 - dot_ds * dot_ds);

        let r0 = (k0 * self.tan_error).abs();
        let r1 = (k1 * other.tan_error).abs();

        // dbg!(k0, k1, r0, r1);

        let p0 = self.start + self.direction * k0;
        let p1 = other.start + other.direction * k1;

        let l = vector::dot(a_diff.as_ref(), &d_n) / l_d_n_sq.sqrt();
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

    //zz All done
}

//a Tests
//ft test_ray
#[test]
fn test_ray() -> Result<(), String> {
    let r0 = Ray::default()
        .set_start([1., 0., 0.].into())
        .set_direction([-1., 0., 0.].into())
        .set_tan_error(0.1);
    let r1 = Ray::default()
        .set_start([0., 1., 0.].into())
        .set_direction([0., -1., 0.01].into())
        .set_tan_error(0.1);
    r0.intersect(&r1);
    eprintln!("{}", serde_json::to_string_pretty(&[r0, r1]).unwrap());
    Ok(())
}

//ft test_ray2
#[test]
fn test_ray2() -> Result<(), String> {
    use crate::json;
    let ray_4060: Ray = json::from_json(
        "Ray 1",
        r#"
{
      "start": [
        -257.61000000000007,
        -292.0,
        186.81
      ],
      "direction": [
        0.72802906255846,
        0.641401039594509,
        -0.2420297305649314
      ],
      "tan_error": 0.1
    }"#,
    )?;

    let ray_4062: Ray = json::from_json(
        "Ray 2",
        r#"
{
      "start": [
        -272.47666666666686,
        -98.69999999999999,
        261.94333333333316
      ],
      "direction": [
        0.8558988940122954,
        0.2414215789446973,
        -0.4573321598667418
      ],
      "tan_error": 0.1
    }"#,
    )?;
    ray_4060.intersect(&ray_4062);
    //    assert!(false);
    Ok(())
}

//ft test_ray3
#[test]
fn test_ray3() -> Result<(), String> {
    use crate::json;
    let ray_4060: Ray = json::from_json(
        "Ray 1",
        r#"
{
      "start": [
        -257.61000000000007,
        -292.0,
        186.81
      ],
      "direction": [
        0.72802906255846,
        0.641401039594509,
        -0.2420297305649314
      ],
      "tan_error": 0.1
    }"#,
    )?;

    let ray_4062: Ray = json::from_json(
        "Ray 2",
        r#"
{
      "start": [
        -272.47666666666686,
        -98.69999999999999,
        261.94333333333316
      ],
      "direction": [
        0.8558988940122954,
        0.2414215789446973,
        -0.4573321598667418
      ],
      "tan_error": 0.2
    }"#,
    )?;

    let p = Ray::closest_point(&[ray_4060, ray_4062], &|_| 1.0).unwrap();
    dbg!(p);
    let (_k0, d0_sq) = ray_4060.distances(&p);
    let (_k1, d1_sq) = ray_4062.distances(&p);
    assert!(
        (d0_sq - d1_sq).abs() < 1E-6,
        "Distance between the closest point and each of the rays should be about the same"
    );

    let p = Ray::closest_point(&[ray_4060, ray_4062], &|r| 1.0 / r.tan_error()).unwrap();
    dbg!(p);
    let (_k0, d0_sq) = ray_4060.distances(&p);
    let (_k1, d1_sq) = ray_4062.distances(&p);
    dbg!(d0_sq.sqrt(), d1_sq.sqrt());
    assert!(
        (d0_sq.sqrt() * 2.0 - d1_sq.sqrt()) < 1E-4,
        "Point should be half the distance from ray 0 compared to ray 0"
    );

    Ok(())
}
