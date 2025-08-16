//a Imports

use geo_nd::{Quaternion, Vector};

use crate::{Point2D, Point3D};

//a Plane of best fit
//tp Plane
/// Described by point . normal = value
///
/// normal is a unit vector here
#[derive(Default, Debug, Clone)]
pub struct Plane {
    /// Unit normal
    normal: Point3D,

    /// Closest distance of plane to origin
    value: f64,

    /// One tangent - a unit vector
    tangent_0: Point3D,

    /// The other tangent - tangent_0 X normal
    tangent_1: Point3D,
}

//ip From<(Point3D, f64)> for Plane {
impl From<(Point3D, f64)> for Plane {
    fn from((normal, value): (Point3D, f64)) -> Self {
        Self::of_normal_value(&normal, value)
    }
}

//ip Plane
impl Plane {
    //cp of_normal_value
    pub fn of_normal_value(normal: &Point3D, value: f64) -> Self {
        let l = normal.length();
        let normal = *normal / l;
        let value = value * l;
        let mut s = Self {
            normal,
            value,
            tangent_0: Point3D::default(),
            tangent_1: Point3D::default(),
        };
        if !s.set_tangent(&[1.0_f64, 0., 0.].into()) {
            let okay = s.set_tangent(&[0.0_f64, 1., 0.].into());
            assert!(okay);
        }
        s
    }

    //ap normal
    pub fn normal(&self) -> &Point3D {
        &self.normal
    }

    //ap value
    pub fn value(&self) -> f64 {
        self.value
    }

    //mp set_tangent
    pub fn set_tangent(&mut self, tangent: &Point3D) -> bool {
        let tangent = tangent.normalize();
        let other_tangent = self.normal.cross_product(&tangent);
        if other_tangent.length_sq() < 0.1 {
            false
        } else {
            self.tangent_1 = other_tangent.normalize();
            self.tangent_0 = self.tangent_1.cross_product(&self.normal).normalize();
            true
        }
    }

    //mp distance_sq
    /// Return the total distance_sq of an iterator of points
    pub fn distance_sq<'a, I: Iterator<Item = &'a Point3D>>(&self, pts: I) -> f64 {
        pts.fold(0.0, |acc, p| {
            acc + (self.normal.dot(p) - self.value).powi(2)
        })
    }

    //mp point_projected_onto
    /// Return the point in 3D where it is projected directly onto the
    /// plane by moving along the normal
    pub fn point_projected_onto(&self, p: &Point3D) -> (Point3D, f64) {
        let p_value = self.normal.dot(p);
        let result = *p + (self.normal * (self.value - p_value));
        (result, p_value - self.value)
    }

    //mp within_plane
    /// Find the coords (tangent_0, tangent_1) for the point
    pub fn within_plane(&self, p: &Point3D) -> Point2D {
        [p.dot(&self.tangent_0), p.dot(&self.tangent_1)].into()
    }

    //mp point_in_space
    /// Given a 2D point, find the coordinates in space
    pub fn point_in_space(&self, p: &Point2D) -> Point3D {
        self.normal * self.value + (self.tangent_0 * p[0]) + (self.tangent_1 * p[1])
    }

    //mp origin_in_space
    /// Get the origin in space
    pub fn origin_in_space(&self) -> Point3D {
        self.normal * self.value
    }

    //mp from_triangle
    pub fn from_triangle(p0: &Point3D, p1: &Point3D, p2: &Point3D) -> Option<Self> {
        let c = (*p0 + *p1 + *p2) / 3.0;
        let dp0 = *p0 - c;
        let dp1 = *p1 - c;
        let normal = dp0.cross_product(&dp1);
        if normal.length_sq() < 1E-10 {
            None
        } else {
            let normal = normal.normalize();
            let value = p0.dot(&normal);
            Some((normal, value).into())
        }
    }

    //mp best_fit
    pub fn best_fit<'a, I: Clone + ExactSizeIterator<Item = &'a Point3D>>(pts: I) -> Option<Self> {
        let sum_x2 = pts.clone().fold(0., |acc, p| acc + p[0].powi(2));
        let sum_y2 = pts.clone().fold(0., |acc, p| acc + p[1].powi(2));
        let sum_z2 = pts.clone().fold(0., |acc, p| acc + p[2].powi(2));
        let sum_x = pts.clone().fold(0., |acc, p| acc + p[0]);
        let sum_y = pts.clone().fold(0., |acc, p| acc + p[1]);
        let sum_z = pts.clone().fold(0., |acc, p| acc + p[2]);
        let sum_xy = pts.clone().fold(0., |acc, p| acc + p[0] * p[1]);
        let sum_yz = pts.clone().fold(0., |acc, p| acc + p[1] * p[2]);
        let sum_zx = pts.clone().fold(0., |acc, p| acc + p[2] * p[0]);
        use geo_nd::matrix;
        let mut dm = nalgebra::base::DMatrix::from_element(3, 3, 2.0);
        let n = pts.len() as f64;
        let n2 = n * n;
        dm.copy_from_slice(&[
            sum_x2 / n2,
            sum_xy / n2,
            sum_zx / n2,
            sum_xy / n2,
            sum_y2 / n2,
            sum_yz / n2,
            sum_zx / n2,
            sum_yz / n2,
            sum_z2 / n2,
        ]);
        let midpoint: Point3D = [sum_x / n, sum_y / n, sum_z / n].into();
        // eprintln!("{dm:?}");
        if !dm.try_inverse_mut() {
            // Plane goes nearly through the origin - d must close to zero
            //
            // Could try adding (1,1,1) to all the points - then d
            // will be about sqrt(3), dm should be invertible, and we will have
            //
            //   p . n' = d' - where d' is presumably sqrt(3)
            //
            // Adding (1,1,1) maps (x,y,z) to (x+1,y+1,z+1)
            //
            //  x^2 => x^+2x+1 ; xy => xy+x+y+1
            //
            // sum_x2' = sum_x2 + 2*sum_x + n ; sum_xy' = sum_xy + sum_x + sum_y + n; etc
            return None;
        }
        // eprintln!("{dm:?}");
        let mut dm_2 = [0.; 9];
        for i in 0..9 {
            dm_2[i] = dm[i];
        }
        let r = matrix::multiply::<f64, 9, 3, 3, 3, 3, 1>(&dm_2, midpoint.as_ref());

        let r: Point3D = r.into();
        let rl = r.length();
        let r = r.normalize();

        let a = r[0];
        let b = r[1];
        let c = r[2];
        let d = n / rl;

        Some((r, d).into())
    }
}

//a Tests
#[test]
fn test_plane() -> Result<(), String> {
    // The test plane here is the plane X=3
    //
    // The tangents are *known* (white-box) to be 0,1,0 and 0,0,1
    let normal: Point3D = [1.0, 0.0, 0.0].into();
    let p: Plane = (normal, 3.0).into();
    assert_eq!(p.normal(), &normal);

    let x: Point3D = [5.0, 6.0, 4.0].into();
    eprintln!("x: {}", x);
    eprintln!("x in plane: {}", p.within_plane(&x));
    eprintln!("point in space: {}", p.point_in_space(&p.within_plane(&x)));
    eprintln!(
        "x projected onto: {} {}",
        p.point_projected_onto(&x).0,
        p.point_projected_onto(&x).1
    );
    assert_eq!(
        p.point_in_space(&p.within_plane(&x)),
        p.point_projected_onto(&x).0
    );
    assert_eq!(p.within_plane(&x)[0], 6.0);
    assert_eq!(p.within_plane(&x)[1], 4.0);

    let pts: &[Point3D] = &[
        [1., 2., 3.].into(),
        [3., 5., 6.].into(),
        [1., 2., 5.].into(),
    ];
    for x in pts.iter() {
        assert!(
            p.point_in_space(&p.within_plane(x))
                .distance(&p.point_projected_onto(x).0)
                < 1E-4
        );
    }
    Ok(())
}

#[test]
fn test_plane2() -> Result<(), String> {
    // Plane x=-z, y = anything, (3,_,3) is on the plane
    let normal: Point3D = [1.0, 0.0, 1.0].into();
    let p: Plane = (normal, 3.0).into();

    eprintln!("plane {p:?}");

    eprintln!("origin  {}", p.origin_in_space());

    // normal will be length 1/sqrt(2)

    let pts: &[Point3D] = &[
        [1., 2., 3.].into(), // .normal = 4/sqrt(2); d = 3*sqrt(2) = 6/sqrt(2); on is +2/sqrt(2) normal = (2,2,4)
        [3., 5., 6.].into(),
        [1., 2., 5.].into(),
    ];
    for x in pts.iter() {
        eprintln!("\n{}", x);
        eprintln!("  {}", p.within_plane(x));
        eprintln!("  {}", p.point_in_space(&p.within_plane(x)));
        eprintln!(
            "  =? {} {}",
            p.point_projected_onto(x).0,
            p.point_projected_onto(x).1
        );
        assert!(
            p.point_in_space(&p.within_plane(x))
                .distance(&p.point_projected_onto(x).0)
                < 1E-4
        );
    }
    Ok(())
}
