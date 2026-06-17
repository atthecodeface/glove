use geo_nd::Vector;

use crate::Point3D;

/// A trianglular spherical patch defined by three great circles intersections (at P0, P1, P2)
///
/// A great circle is defined by the normal to that great circle; if two sphere
/// points are on it (and not opposite or the same) then the normal is the unit
/// cross product of the two points
///
/// n01 = p0 x p1; etc
///
/// n01 x n20 = (p0 x p1) x (p2 x p0) = -(p0 x p1) x (p0 x p2) = -(p0.(p1 x p2))p0
///
/// i.e. p0 = -k (n01 x n20) = k (n20 x n01)
///
///  Where k = p0.(p1 x p2) = p1.(p2 x p0) = p2.(p0 x p1)
///
/// Hence P0 is (normal_20 cross_product normal_01) *normalized*, etc
///
/// Note, though that if the normals are *normalized* then the points derived
/// from them are *NOT* unit points
#[derive(Debug, Clone)]
pub struct GCTriangle {
    /// The (nonunit) normal to the great circle joining the first two points on the sphere
    pub normal_01: Point3D,
    /// The (nonunit) normal to the great circle joining the second two points on the sphere
    pub normal_12: Point3D,
    /// The (nonunit) normal to the great circle joining the third point on the sphere to the first point on the sphere
    pub normal_20: Point3D,
}
impl GCTriangle {
    pub fn of_normals(
        normal_01: &Point3D,
        normal_12: &Point3D,
        normal_20: &Point3D,
    ) -> Self {
        Self {
            normal_01: *normal_01,
            normal_12: *normal_12,
            normal_20: *normal_20,
        }
    }
    pub fn contains_pt(&self, p: &Point3D) -> bool {
        self.normal_01.dot(p) >= 0.0 && self.normal_12.dot(p) >= 0.0 && self.normal_20.dot(p) >= 0.0
    }
    pub fn of_points(p0: &Point3D, p1: &Point3D, p2: &Point3D) -> Self {
        Self::of_normals(
            &p0.cross_product(&p1), &p1.cross_product(&p2), &p2.cross_product(&p0))
    }
    pub fn nonunit_points(&self) -> [Point3D; 3] {
        return [
            self.normal_20.cross_product(&self.normal_01),
            self.normal_01.cross_product(&self.normal_12),
            self.normal_12.cross_product(&self.normal_20),
        ];
    }

    /// Find the normal to the *flat* triangle consisting of the *unit* points
    ///
    /// If the normals are derived from *unit* points then this could be the sum of the normals; however, if they are not then the proper calculation is required.
    ///
    /// Normal is (p1-p0) x (p2-p0) = (p1xp2) - (p0xp2) - (p1xp0) - (p0xp0)
    ///          = (p1 x p2) + (p2 x p0) + (p0 x p1)
    ///
    ///  Where k = p0.(p1 x p2) = p1.(p2 x p0) = p2.(p0 x p1)
    ///
    ///  p0 = k (n20 x n01), p1 = k (n01 x n12), p2 = k (n12 x n20)
    ///     p1 x p2 = k ((n01 x n12) x (n12 x n20))
    ///             = -k ((n12 x n01) x (n12 x n20))
    ///             = -k (n12.(n01 x n20)) n12
    ///             = -k . c . n12
    ///
    ///   Where c = n12.(n01 x n20) =
    pub fn normal_nonunit(&self) -> Point3D {
        // Assume the normals are not derived from unit points
        let [p0, p1, p2] = self.nonunit_points();
        let p0 = p0.normalize();
        let p1 = p1.normalize();
        let p2 = p2.normalize();
        (p1-p0).cross_product(p2-p0)
    }
}

/// A triangle embedded in 3D space, derived from 3 points (P0, P1, P2)
///
/// The triangle has a unit normal and a value; all points P on the plane of the
/// triangle have P.normal = value
///
/// Three tangent vectors (perpendicular to the normal, and to one edge of the
/// triangle, non-unit vectors) are kept; tangent_01 is perpendicular to the
/// line (P0-P1), and as such P0.tangent_01=P1.tangent_01 = value_01, and
/// tangent_01 is scaled such that P2.tangent_01=value_01+1
///
/// Consider a point P = p0*P0 + p1*P1 + p2*P2 such that p0+p1+p2=1 (i.e. it has
/// Barycentric coordinates of the triangle P012). Then:
///
///   P.tangent_01 - value_01 = p0.value_01 + p1.value_01 + p2.(value_01+1) - value_01
///                           = (p0+p1+p2).value_01 + p2 - value_01
///                           = value_01 - value_01 + p2
///                           = p2
///
/// Similarly P.tangent_20 - value_20 = p1, and P.tangent_12 - value_12 = p0
///
/// Hence the Barycentric coordinates of P can be simply calculated
#[derive(Debug, Clone)]
pub struct Triangle3D {
    /// Unit normal
    normal: Point3D,

    /// Closest distance of plane to origin
    value: f64,

    /// Points P0, P1, P2
    points: [Point3D; 3],

    /// Tangent on plane to the line P0-P1, with length such that P2.tangent_01 = value_01+1
    tangent_01: Point3D,

    /// The value associated with tangent_01 such that P0.tangent_01=P1.tangent_01=value_01
    value_01: f64,

    /// Tangent on plane to the line P1-P2, with length such that P0.tangent_12 = value_12+1
    tangent_12: Point3D,

    /// The value associated with tangent_12 such that P1.tangent_12=P2.tangent_12=value_12
    value_12: f64,

    /// Tangent on plane to the line P2-P0, with length such that P1.tangent_20 = value_20+1
    tangent_20: Point3D,
    /// The value associated with tangent_20 such that P2.tangent_20=P0.tangent_20=value_20
    value_20: f64,
}

impl Triangle3D {
    #[track_caller]
    pub fn validate(&self) {
        assert!((self.normal.length_sq()-1.0).abs()<1E-6, "Normal must be a unit vector");
        assert!(
            self.normal.dot(self.points[1] - self.points[0]).abs() < 1E-6,
            "Invalid normal p1-p0 {} in Triangle3D {:0.4} {:0.4}  {:0.4} {:0.4} {:0.4}",
            self.normal.dot(self.points[2] - self.points[0]),
            self.normal,
            self.points[1] - self.points[0],
            self.points[0],
            self.points[1],
            self.points[2]
        );
        assert!(
            self.normal.dot(self.points[2] - self.points[0]).abs() < 1E-6,
            "Invalid normal p2-p0 {} in Triangle3D {:0.4} {:0.4}  {:0.4} {:0.4} {:0.4}",
            self.normal.dot(self.points[2] - self.points[0]),
            self.normal,
            self.points[2] - self.points[0],
            self.points[0],
            self.points[1],
            self.points[2]
        );
        assert!(
            self.normal.dot(self.points[2] - self.points[1]).abs() < 1E-6,
            "Invalid normal p2-p1 {} in Triangle3D {:0.4} {:0.4}  {:0.4} {:0.4} {:0.4}",
            self.normal.dot(self.points[2] - self.points[0]),
            self.normal,
            self.points[2] - self.points[1],
            self.points[0],
            self.points[1],
            self.points[2]
        );

        assert!((self.tangent_01.dot(self.points[0]) - self.value_01).abs() < 1E-6);
        assert!((self.tangent_01.dot(self.points[1]) - self.value_01).abs() < 1E-6);
        assert!((self.tangent_01.dot(self.points[2]) - self.value_01 - 1.0).abs() < 1E-6);

        assert!((self.tangent_12.dot(self.points[0]) - self.value_12 - 1.0).abs() < 1E-6);
        assert!((self.tangent_12.dot(self.points[1]) - self.value_12).abs() < 1E-6);
        assert!((self.tangent_12.dot(self.points[2]) - self.value_12).abs() < 1E-6);

        assert!((self.tangent_20.dot(self.points[0]) - self.value_20).abs() < 1E-6);
        assert!((self.tangent_20.dot(self.points[1]) - self.value_20 - 1.0).abs() < 1E-6);
        assert!((self.tangent_20.dot(self.points[2]) - self.value_20).abs() < 1E-6);
    }
    fn make(
        normal: Point3D,
        tangent_01: Point3D,
        tangent_12: Point3D,
        tangent_20: Point3D,
        p0: Point3D,
        p1: Point3D,
        p2: Point3D,
    ) -> Self {
        let value = p0.dot(&normal);

        // Note tangent_01.dot(&p0) == tangent_01.dot(&p1);
        let value_01_p01 = tangent_01.dot(&p0);
        let value_01_p2 = tangent_01.dot(&p2);
        let value_01_diff = value_01_p2 - value_01_p01;

        let value_12_p12 = tangent_12.dot(&p1);
        let value_12_p0 = tangent_12.dot(&p0);
        let value_12_diff = value_12_p0 - value_12_p12;

        let value_20_p20 = tangent_20.dot(&p2);
        let value_20_p1 = tangent_20.dot(&p1);
        let value_20_diff = value_20_p1 - value_20_p20;

        Self {
            normal,
            value,
            points: [p0, p1, p2],
            tangent_01: tangent_01 / value_01_diff,
            value_01: value_01_p01 / value_01_diff,
            tangent_12: tangent_12 / value_12_diff,
            value_12: value_12_p12 / value_12_diff,
            tangent_20: tangent_20 / value_20_diff,
            value_20: value_20_p20 / value_20_diff,
        }
    }

    pub fn of_gc_triangle(tri: &GCTriangle) -> Self {
        let [p0, p1, p2] = tri.nonunit_points();
        let p0 = p0.normalize();
        let p1 = p1.normalize();
        let p2 = p2.normalize();

        // let normal = (p1-p0).cross_product(p2-p0).normalize();
        let normal = tri.normal_nonunit().normalize();
        let tangent_01 = tri.normal_01 - normal * tri.normal_01.dot(&normal);
        let tangent_12 = tri.normal_12 - normal * tri.normal_12.dot(&normal);
        let tangent_20 = tri.normal_20 - normal * tri.normal_20.dot(&normal);

        let p = Self::make(normal, tangent_01, tangent_12, tangent_20, p0, p1, p2);
        p.validate();
        p
    }

    pub fn of_points(p0: &Point3D, p1: &Point3D, p2: &Point3D) -> Option<Self> {
        let p01 = p1 - p0;
        let p12 = p2 - p1;
        let p20 = p0 - p2;
        let normal = p01.cross_product(&p12);
        if normal.length_sq() < 1E-10 {
            None
        } else {
            let normal = normal.normalize();
            // Note these will be scaled appropriately, so no need to normalize
            let tangent_01 = normal.cross_product(&p01);
            let tangent_12 = normal.cross_product(&p12);
            let tangent_20 = normal.cross_product(&p20);
            Some(Self::make(
                normal, tangent_01, tangent_12, tangent_20, *p0, *p1, *p2,
            ))
        }
    }

    /// Borrow the points
    pub fn points(&self) -> &[Point3D; 3] {
        &self.points
    }

    pub fn unit_normal(&self) -> &Point3D {
        &self.normal
    }

    /// Find the Barycentric coordinates of a point
    ///
    /// It need not be projected onto the triangle, as the tangent vectors are
    /// perpendicular to the normal; adding any amount of normal therefore has
    /// no impace on the dot products
    ///
    /// ARGH
    ///
    /// No, must project point by *scaling* on to plane.
    pub fn barycentric_coordinates(&self, p: &Point3D) -> [f64; 3] {
        let p = self.point_projected_onto_by_scaling(p);
        [
            self.tangent_12.dot(p) - self.value_12,
            self.tangent_20.dot(p) - self.value_20,
            self.tangent_01.dot(p) - self.value_01,
        ]
    }

    /// Find the Barycentric coordinates of a point projected onto the triangle
    pub fn of_barycentric_coordinates(&self, p: &[f64; 3]) -> Point3D {
        self.points[0] * p[0] + self.points[1] * p[1] + self.points[2] * p[2]
    }

    /// Returns true if the point (projected onto the plane) is within the triangle
    pub fn contains_point(&self, p: &Point3D) -> bool {
        let [b0, b1, b2] = self.barycentric_coordinates(p);
        b0 >= 0.0 && b1 >= 0.0 && b2 >= 0.0
    }

    /// Return the point in 3D where it is scaled onto the
    /// plane of the triangle
    pub fn point_projected_onto_by_scaling(&self, p: &Point3D) -> Point3D {
        let p_value = self.normal.dot(p);
        let r = *p * self.value / p_value;
        eprintln!("{p} {r} {}", self.normal.dot(r) - self.value);
        r
    }

    /// Return the point in 3D where it is projected directly onto the
    /// plane of the triangle by moving along the normal
    pub fn point_projected_onto_by_normal(&self, p: &Point3D) -> (Point3D, f64) {
        let p_value = self.normal.dot(p);
        let result = *p + (self.normal * (self.value - p_value));
        (result, p_value - self.value)
    }

    /// Get the origin of the plane in space
    pub fn origin_in_space(&self) -> Point3D {
        self.normal * self.value
    }
}
