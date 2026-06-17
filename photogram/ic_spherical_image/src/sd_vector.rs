use geo_nd::Vector;
use ic_base::{GCTriangle, Point3D, Triangle3D};

/// This type describes a specific subdivision of an initial GCTriangle
///
/// This type operates on GC normals, and in essence keeps the points on the
/// triangle more as *direction* vectors away from the origin that the
/// triangle's points are actually at; this means there is much less normalization occurring, and
#[derive(Debug, Clone)]
pub struct SdSubtriangle {
    /// The *relative* degree of subdivision that the triangle stack contains with respect to tri_toplevel
    ///
    /// If this is 0, then the gcn_vectors are those of the toplevel triangle, and tri_lowest == tri_toplevel
    subdivision: u8,
    /// Pairs of bits indicating which subtriangles have been followed to reach this subdivision
    ///
    /// If subdivision is 0, then this is zero
    triangle_stack: u64,
    /// The (nonunit) great circle that is normal to the plane containing the origin and points 0 and 1
    gcn_01: Point3D,
    /// The (nonunit) great circle that is normal to the plane containing the origin and points 1 and 2
    gcn_12: Point3D,
    /// The (nonunit) great circle that is normal to the plane containing the origin and points 2 and 0
    gcn_20: Point3D,
    /// (nonunit) The point P0 of the triangle this defines
    ///
    /// This is defined to be *some scaling* of gcn_01.cross_product(gcn_20)
    p0: Point3D,
    /// (nonunit) The point P1 of the triangle this defines
    ///
    /// This is defined to be *some scaling* of gcn_12.cross_product(gcn_01)
    p1: Point3D,
    /// (nonunit) The point P2 of the triangle this defines
    ///
    /// This is defined to be *some scaling* of gcn_20.cross_product(gcn_12)
    p2: Point3D,
}

impl SdSubtriangle {
    pub fn new(gcn_01: &Point3D, gcn_12: &Point3D, gcn_20: &Point3D) -> Self {
        Self {
            subdivision: 0,
            triangle_stack: 0,
            gcn_01: *gcn_01,
            gcn_12: *gcn_12,
            gcn_20: *gcn_20,
            p0: gcn_20.cross_product(gcn_01),
            p1: gcn_01.cross_product(gcn_12),
            p2: gcn_12.cross_product(gcn_20),
        }
    }
    pub fn to_triangle3d_on_sphere(&self) -> Triangle3D {
        Triangle3D::of_normals_on_sphere(&self.gcn_01, &self.gcn_12, &self.gcn_20)
    }
    pub fn find_subtriangle_of_point(&mut self, p: &Point3D) {
        // Create midpoints of the lines (no need to normalize)
        let p01_mp = self.p0 + self.p1;
        let p12_mp = self.p1 + self.p2;
        let p20_mp = self.p2 + self.p0;

        // Create four triangles with counter-clockwise points as viewed from the outside
        let gcn_01mp_12mp = p01_mp.cross_product(&p12_mp);
        let gcn_12mp_20mp = p12_mp.cross_product(&p20_mp);
        let gcn_20mp_01mp = p20_mp.cross_product(&p01_mp);

        let subtriangle = if gcn_20mp_01mp.dot(p) <= 0.0 {
            0
        } else if gcn_01mp_12mp.dot(p) <= 0.0 {
            1
        } else if gcn_12mp_20mp.dot(p) <= 0.0 {
            2
        } else {
            3
        };
        self.triangle_stack |= subtriangle << (2 * (self.subdivision as u64));
        self.subdivision += 1;
        match subtriangle {
            0 => {
                self.gcn_12 = -gcn_20mp_01mp;
                self.p1 = p01_mp;
                self.p2 = p20_mp;
            }
            1 => {
                self.gcn_20 = -gcn_01mp_12mp;
                self.p0 = p01_mp;
                self.p2 = p12_mp;
            }
            2 => {
                self.gcn_01 = -gcn_12mp_20mp;
                self.p0 = p20_mp;
                self.p1 = p12_mp;
            }
            _ => {
                self.gcn_01 = gcn_12mp_20mp;
                self.gcn_12 = gcn_20mp_01mp;
                self.gcn_20 = gcn_01mp_12mp;
                self.p0 = p12_mp;
                self.p1 = p20_mp;
                self.p2 = p01_mp;
            }
        }
    }
}
