use std::collections::{HashMap, HashSet};

use geo_nd::Vector;

use ic_base::Point3D;
use indexed::Idx;

use crate::{GreatCircleTriangleIndex, SphericalData};

/// An SdIndex maps from an array of GC normals to GC Triangles within a [SphericalData]
///
/// Any point in any region on a sphere (bounded by great circles, where the
/// point does not lie on any of the circle) will have an array of signs of the
/// dot product of it with the array of GC normals; this converts to a bitmask
/// (1 being sign positive, 0 being sign negative) with bit N corresponding to
/// the Nth vector of the array.
///
/// Indeed, every distinct region has a different bitmask; crossing a line from
/// one region to another toggles a single bit. Furthermore, for a GC triangle,
/// every region inside the triangle must have a *one* in its bitmask for the
/// three GC normals that form the edges of the triangle. Any other great circle
/// that does not cross through the trianglw will have a the same bit value for
/// all regions in the triangle.
///
/// This each region in the triangle has a set of *one* bits, another set of
/// *constant* bits, and a set of bits that may be different.
///
/// A bit in the bitmask that is not constant within the triangle indicates that
/// the corresponding great circle passes through the triangle; in doing so it
/// must cut two of the triangle's edges, or one edge and it passes through a
/// vertex of the triangle. In the first case, two of the triangle vertices will
/// be on one side of the GC (and have bit value X) and the other will be on the
/// other side of the great circle (with bit value !X); in the latter case the
/// vertices that the GC does *not* pass through will have different bit values.
///
/// From this one can deduce that the bitmasks for (just within the triangle
/// from) each of the vertices of the triangle will have a set of bits Km that have the
/// same Kv for all three points and a bitmask M of bits that can change between the points.
///
/// Note that *every* region that has the set of bits Km with the value Kv
/// *must* either be empty regions or they must correspond to regions within the
/// GC triangle, as they *must* all lie within the three great circles that
/// define the triangle.
///
/// An index can thus be created that maps bitmask values to GC Triangles; the
/// index can map every GC triangle through its Km, Kv, M such that that every
/// bitmaask that has bits Km set to Kv with any possible value for the bits M
/// maps to the GC triangle. This mapping may include bitmasks that actually
/// cannot be generated (a triangle can be cut by two GC 'parallel' to one edge
/// such that only three, not four, regions are formed, but this methods would
/// have four bitmask values), but the index cannot map a point through its
/// bitmask to a triangle that the point does *not* lie within.
///
/// This can be used to find the region associated with points for:
///
/// * a tetrahedron, with 6 great circles and 32 regions (for 4 triangles)
///
/// * a tetrahedron subdivided once, with 9 great circles and 56 regions (for 16 triangles)
///
/// * a tetrahedron subdivided twice, with 37 great circles and 9728 regions (for 64 triangles)
///
/// * an octahedron, with 3 great circles and 8 regions (all whole triangles)
///
/// * an octahedron subdivided once, with 7 great circles and 32 regions (all whole triangles)
///
/// * an octahedron subdivided twice, with 55 great circles and 20480 regions (for 128 triangles)
///
/// * an icosahedron, with 15 great circles and 160 regions (for 20 triangles)
///
/// * an icosahedron subdivided once, with 21 great circles and 280 regions (for 80 triangles)
#[derive(Debug)]
pub struct SdIndex {
    normals: Vec<Point3D>,
    map: HashMap<u64, GreatCircleTriangleIndex>,
}

impl SdIndex {
    fn mask_of_vec(vec: &Point3D, normals: &[Point3D]) -> (u64, u64) {
        let mut mask: u64 = 0;
        let mut zero_mask: u64 = 0;
        for (i, n) in normals.iter().enumerate() {
            let d = vec.dot(n);
            if d >= 0.0 {
                mask |= 1 << i;
            }
            if d.abs() < 1E-6 {
                zero_mask |= 1 << i;
            }
        }
        (mask, zero_mask)
    }

    pub fn new<I: Iterator<Item = GreatCircleTriangleIndex>>(
        sd: &SphericalData,
        triangle_indices: I,
    ) -> Self {
        let mut normals_required = HashSet::new();
        let mut triangles = vec![];
        for t in triangle_indices {
            normals_required.insert(sd[sd[t].gc_line(0).1].normal().lower_index());
            normals_required.insert(sd[sd[t].gc_line(1).1].normal().lower_index());
            normals_required.insert(sd[sd[t].gc_line(2).1].normal().lower_index());
            triangles.push(t);
        }
        let mut normals = vec![];
        for gcn in normals_required {
            normals.push(*sd[gcn].vector());
        }

        let n = normals.len();
        assert!(n < 64, "Too many normals {n} for a complete index");
        let mut map = HashMap::new();

        for t in &triangles {
            let (p0, p1, p2) = sd[*t].get_points(sd);
            let p0_vec = sd[p0].vector();
            let p1_vec = sd[p1].vector();
            let p2_vec = sd[p2].vector();
            let p = p0_vec + p1_vec + p2_vec;
            // let m = Self::mask_of_vec(&p, &normals).0;
            let m0 = Self::mask_of_vec(&(p * 1.0E-6 + p0_vec), &normals).0;
            let m1 = Self::mask_of_vec(&(p * 1.0E-6 + p1_vec), &normals).0;
            let m2 = Self::mask_of_vec(&(p * 1.0E-6 + p2_vec), &normals).0;

            // map.insert(m, t);

            // let m_can_change = (m ^ m0) | (m ^ m1) | (m ^ m2) | (m0 ^ m1) | (m0 ^ m2) | (m1 ^ m2);
            let m_can_change = (m0 ^ m1) | (m0 ^ m2) | (m1 ^ m2);

            // Note that 'm' MUST be nonchanging for the GC for the triangle, as all three points MUST have >=0 for these.
            //
            // Furthermore, if these three bits *are* set for *any* vector then the point *MUST* be in this triangle
            //
            // Hence we can add every bit change indicated
            //
            // Create a vector of those bits; then run through all 2^n possiblities (even if they are not achievable in reality!) and add them to the hash map
            let mut dm = vec![];
            for i in 0..n {
                let b = 1 << i;
                if m_can_change & b == 0 {
                    continue;
                }
                dm.push(b);
            }
            for i in 0..(1 << dm.len()) {
                let mut delta = 0;
                for (j, b) in dm.iter().enumerate() {
                    if i & (1 << j) != 0 {
                        delta |= *b;
                    }
                }
                map.insert(m0 ^ delta, *t);
            }
        }
        eprintln!(
            "Found {} regions, {} triangles, from {} bits",
            map.len(),
            triangles.len(),
            n
        );
        //        eprintln!("{:#0x?}", map.keys());
        Self { normals, map }
    }

    pub fn map_vector(&self, v: &Point3D) -> Option<GreatCircleTriangleIndex> {
        self.map
            .get(&Self::mask_of_vec(v, &self.normals).0)
            .copied()
    }
}
