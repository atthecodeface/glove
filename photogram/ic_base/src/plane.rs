//a Imports
use geo_nd::Vector;

use crate::{Point2D, Point3D};

/// A simple plane in 3D, described by point . normal = value
///
/// The plane contains three vectors: the normal, a tangent on the plane, and
/// the second tangent that is perpendicular to both the normal and the other
/// tangent
#[derive(Default, Debug, Clone)]
pub struct Plane {
    /// Unit normal
    normal: Point3D,

    /// Closest distance of plane to origin
    value: f64,

    /// One tangent - a unit vector
    tangent_0: Point3D,

    /// The other tangent - the cross product of tangent_0 and the normal
    tangent_1: Point3D,
}

impl From<(Point3D, f64)> for Plane {
    /// Create a plane from a normal and a value
    fn from((normal, value): (Point3D, f64)) -> Self {
        Self::of_normal_value(&normal, value)
    }
}

impl Plane {
    /// Create a [Plane] given a normal and a value
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
        if !s.set_tangents(&[1.0_f64, 0., 0.].into()) {
            let okay = s.set_tangents(&[0.0_f64, 1., 0.].into());
            assert!(okay);
        }
        s
    }

    /// Borrow the normal
    pub fn normal(&self) -> &Point3D {
        &self.normal
    }

    /// Get the value associated with the plane (points on the plane have p.normal = this)
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Set the tangents to the plane to be in the direction of the cross
    /// product of the normal and a given vector, and perpendicular to both of these
    ///
    /// Returns false if the provided vector does not permit tangents to be set
    /// (for example, if the given vector is close-to-parallel to the normal)
    ///
    /// This is used both initially (to create the tangents arbitrarily for a
    /// plane) and by clients that have specific needs for the tangent
    /// directions
    pub fn set_tangents(&mut self, tangent: &Point3D) -> bool {
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

    /// Return the total distance_sq from the plane of an iterator of 3D points
    pub fn distance_sq<'a, I: Iterator<Item = &'a Point3D>>(&self, pts: I) -> f64 {
        pts.fold(0.0, |acc, p| {
            acc + (self.normal.dot(p) - self.value).powi(2)
        })
    }

    /// Return the point in 3D where it is projected directly onto the
    /// plane by moving along the normal
    pub fn point_projected_onto(&self, p: &Point3D) -> (Point3D, f64) {
        let p_value = self.normal.dot(p);
        let result = *p + (self.normal * (self.value - p_value));
        (result, p_value - self.value)
    }

    /// Find the coords (tangent_0, tangent_1) for the point
    ///
    /// As the tangents are perpendicular to each other, these are linearly independent
    /// values
    pub fn within_plane(&self, p: &Point3D) -> Point2D {
        [p.dot(&self.tangent_0), p.dot(&self.tangent_1)].into()
    }

    /// Given a 2D point on the plane (tangent 0, tangent 1), find the
    /// coordinates in 3D space on the plane that it corresponds to
    pub fn point_in_space(&self, p: &Point2D) -> Point3D {
        self.normal * self.value + (self.tangent_0 * p[0]) + (self.tangent_1 * p[1])
    }

    /// Get the origin of the plane in space
    pub fn origin_in_space(&self) -> Point3D {
        self.normal * self.value
    }

    /// Derive a plane from a triangle of points in 3D space; if the points are
    /// collinear then return None
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

    /// Generate a plane of best fit given an iterator of 3D points in space
    ///
    /// This uses a minimum-squared-distance-from-the-plane calculation
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
        Some((r, n / rl).into())
    }
}
