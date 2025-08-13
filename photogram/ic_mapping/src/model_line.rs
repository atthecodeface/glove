//a Imports
use std::default::Default;

use geo_nd::Vector;
use ic_base::Point3D;
use serde::{Deserialize, Serialize};


//a ModelLine
//tp ModelLine
/// A line in model space
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ModelLine {
    pts: [Point3D; 2],
}

//ip ModelLine
impl ModelLine {
    #[track_caller]
    pub fn new(p0: Point3D, p1: Point3D) -> Self {
        assert!((p0 - p1).length() > 1E-10);
        Self { pts: [p0, p1] }
    }
    pub fn mid_point(&self) -> Point3D {
        (self.pts[0] + self.pts[1]) / 2.0
    }
    pub fn direction(&self) -> Point3D {
        self.pts[1] - self.pts[0]
    }
    pub fn length(&self) -> f64 {
        (self.pts[1] - self.pts[0]).length()
    }
    pub fn unit_perpendicular(&self) -> Point3D {
        let direction = self.direction();
        let direction = direction / direction.length();
        let k = self.pts[0].cross_product(&direction);
        let l = k.length();
        if l > 0.001 {
            return k / l;
        }
        for v in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] {
            let perp = direction.cross_product(&v);
            let l = perp.length();
            if l > 0.001 {
                return perp / l;
            }
        }
        unreachable!();
    }

    //cp of_vector_eq
    /// From a vector equation p x direction = k
    #[allow(dead_code)]
    pub fn of_vector_eq(direction: &Point3D, k: &Point3D) -> Self {
        let l = direction.length();
        let k = (*k) / l;
        let direction = (*direction) / l;
        let p0 = direction.cross_product(&k);
        let p1 = p0 + direction;
        Self::new(p0, p1)
    }

    //mp closest_pt_to_pt
    #[allow(dead_code)]
    pub fn closest_pt_to_pt(&self, p: &Point3D) -> Point3D {
        let p = *p - self.pts[0];
        let d = self.pts[1] - self.pts[0];
        let len_d2 = d.length_sq();
        let l = p.dot(&d);
        self.pts[0] + (d * (l / len_d2))
    }

    //mp cos_angle_subtended
    /// Get cos(angle) for the angle subtended by the line when viewed from p
    #[allow(dead_code)]
    pub fn cos_angle_subtended(&self, p: &Point3D) -> f64 {
        let p_p0 = *p - self.pts[0];
        let p_p1 = *p - self.pts[1];
        p_p0.dot(&p_p1) / (p_p0.length() * p_p1.length())
    }

    //mp radius_of_circumcircle
    /// Given a model space point p, find the radius of the circle that
    /// includes both the model points of the line and the point p
    ///
    /// The circle will be in the plane of the three points
    ///
    /// The center of the circumcircle will be at the intersection of
    /// the three spheres of *this* radius centred on the three points
    ///
    /// The equation really is |a||b||c| / (2|a x b|)
    ///
    /// which is |c| / 2sin(ACB) = |b| / 2sin(CBA) = |a| / 2sin(BAC)
    #[allow(dead_code)]
    pub fn radius_of_circumcircle(&self, p: &Point3D) -> f64 {
        let p0p = *p - self.pts[0];
        let p1p = *p - self.pts[1];
        let p0p_x_p1p = p0p.cross_product(&p1p);
        self.length() * p0p.length() * p1p.length() / (2.0 * p0p_x_p1p.length())
    }
}

//ip From<(Point3D, Point3D)> for ModelLine
impl From<(Point3D, Point3D)> for ModelLine {
    fn from((p0, p1): (Point3D, Point3D)) -> Self {
        Self::new(p0, p1)
    }
}
