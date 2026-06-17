use std::collections::HashMap;

use geo_nd::Vector;

use ic_base::Point3D;

use crate::{GreatCircleTriangleIndex, SphericalData};

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

    pub fn new(sd: &SphericalData) -> Self {
        let mut normals = vec![];
        for (_, n) in sd.iter_normals().enumerate().filter(|(i, _)| (i & 1) == 0) {
            normals.push(*n.vector());
        }

        let n = normals.len();
        assert!(n < 64, "Too many normals {n} for a complete index");
        let mut map = HashMap::new();

        for t in sd.iter_triangle_indicess() {
            let (p0, p1, p2) = sd[t].get_points(sd);
            let p0_vec = sd[p0].vector();
            let p1_vec = sd[p1].vector();
            let p2_vec = sd[p2].vector();
            let p = p0_vec + p1_vec + p2_vec;
            let m = Self::mask_of_vec(&p, &normals).0;
            let m0 = Self::mask_of_vec(&(p * 1.0E-6 + p0_vec), &normals).0;
            let m1 = Self::mask_of_vec(&(p * 1.0E-6 + p1_vec), &normals).0;
            let m2 = Self::mask_of_vec(&(p * 1.0E-6 + p2_vec), &normals).0;

            map.insert(m, t);

            let m_can_change = (m ^ m0) | (m ^ m1) | (m ^ m2) | (m0 ^ m1) | (m0 ^ m2) | (m1 ^ m2);

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
                map.insert(m ^ delta, t);
            }
        }
        // eprintln!("Found {} regions", map.len());
        // eprintln!("{:#0x?}", map.keys());
        Self { normals, map }
    }

    pub fn map_vector(&self, v: &Point3D) -> Option<GreatCircleTriangleIndex> {
        self.map
            .get(&Self::mask_of_vec(v, &self.normals).0)
            .copied()
    }
}
