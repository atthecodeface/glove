use crate::{GcTriangle, SdSubtriangle, SphericalData, SphericalImageError};
use geo_nd::Vector;
use ic_base::{GCTriangle, Point3D, Triangle3D};

/// An internal type that manages mapping over a single square patch
pub struct MapXYToVec {
    /// Triangle3D from SdSubtriangle that is the subdivided triangle of size
    /// img_size scaled down by 2^subdivision - bottom left of the square patch
    t012: Triangle3D,

    /// Triangle3D from SdSubtriangle that is the subdivided triangle of size
    /// img_size scaled down by 2^subdivision - top right of the square patch
    t230: Triangle3D,
    patch_size: u32,
}
impl MapXYToVec {
    // fn new( sx, sy)
    fn map_xy(&mut self, x: u32, y: u32) -> Option<Point3D> {
        let dbl_size = 2 * self.patch_size;
        let f_sc = dbl_size as f64;

        if x >= self.patch_size || y >= self.patch_size {
            None
        } else if x + y <= self.patch_size {
            let c0 = 2 * x + 1;
            let c2 = 2 * y + 1;
            let c1 = dbl_size - c0 - c2;
            Some(self.t012.of_barycentric_coordinates(&[
                (c0 as f64) / f_sc,
                (c1 as f64) / f_sc,
                (c2 as f64) / f_sc,
            ]))
        } else {
            let c0 = dbl_size - (2 * x + 1);
            let c2 = dbl_size - (2 * y + 1);
            let c1 = dbl_size - c0 - c2;
            Some(self.t230.of_barycentric_coordinates(&[
                (c0 as f64) / f_sc,
                (c1 as f64) / f_sc,
                (c2 as f64) / f_sc,
            ]))
        }
    }
}

/// A pair of Spherical triangles that share an edge, that are stored as a rectangle in an image
///
/// This has points on the sphere of P0, P1, P2 and P3
///
/// It is in reality defined by *five* great circles; N01, N12, N20 (=-N02), N23, N30
///
/// This is stored as two great circle triangles with normals such that their
/// crossproducts produce the points P0, P1, P2 and P3
///
/// P0 is defined to be N20 x N01 == (-N20) x N30
/// P1 is defined to be N01 x N12
/// P2 is defined to be N01 x N12 == N23 x (-N20)
/// P3 is defined to be N23 x N30
#[derive(Debug)]
pub struct ImagePatch {
    t012: GCTriangle,
    t230: GCTriangle,
    img_xy: (u32, u32),
    img_sz: u32,
}

impl ImagePatch {
    pub fn of_gc_triangles(sd: &SphericalData, gc0: &GcTriangle, gc1: &GcTriangle) -> Option<Self> {
        let l0 = [gc0.gc_line(0), gc0.gc_line(1), gc0.gc_line(2)];
        let l1 = [gc1.gc_line(0), gc1.gc_line(1), gc1.gc_line(2)];

        let mut line_match = None;
        for (i, l0) in l0.iter().enumerate() {
            for (j, l1) in l1.iter().enumerate() {
                if l0.1 == l1.1 {
                    if line_match.is_some() {
                        return None;
                    }
                    line_match = Some((i, j));
                }
            }
        }
        let Some((l0_match, l1_match)) = line_match else {
            return None;
        };

        let (l01, l12, l20) = {
            match l0_match {
                0 => (l0[1], l0[2], l0[0]),
                1 => (l0[2], l0[0], l0[1]),
                _ => (l0[0], l0[1], l0[2]),
            }
        };
        let (l23, l30) = {
            match l1_match {
                0 => (l1[1], l1[2]),
                1 => (l1[2], l1[0]),
                _ => (l1[0], l1[1]),
            }
        };

        let mut n01 = *sd[l01.1].normal().vector();
        let mut n12 = *sd[l12.1].normal().vector();
        let mut n20 = *sd[l20.1].normal().vector();
        let mut n23 = *sd[l23.1].normal().vector();
        let mut n30 = *sd[l30.1].normal().vector();
        if l01.0 {
            n01 = -n01
        };
        if l12.0 {
            n12 = -n12
        };
        if l20.0 {
            n20 = -n20
        };
        if l23.0 {
            n23 = -n23
        };
        if l30.0 {
            n30 = -n30
        };
        eprintln!("{n01:?} {n12:?} {n20:?} {n23:?} {n30:?}");

        let t012 = GCTriangle::of_normals_on_sphere(&n01, &n12, &n20);
        let t230 = GCTriangle::of_normals_on_sphere(&n23, &n30, &-n20);
        Some(Self {
            t012,
            t230,
            img_sz: 0,
            img_xy: (0, 0),
        })
    }
    pub fn set_img_xy(&mut self, img_xy: (u32, u32)) -> &mut Self {
        self.img_xy = img_xy;
        self
    }
    pub fn set_img_sz(&mut self, img_sz: u32) -> &mut Self {
        self.img_sz = img_sz;
        self
    }
    /// Map the subsquare of size self.img_sz >> subdivision
    ///
    ///  0 <= sx < (1<<subdivision)
    ///  0 <= sy < (1<<subdivision)
    ///
    /// Only works with subdivision == 0 for now
    pub fn map_subsquare(&self, subdivision: u8, _sx: u32, _sy: u32) -> MapXYToVec {
        assert_eq!(subdivision, 0);
        let t012 = SdSubtriangle::new(
            &self.t012.nonunit_normal_01,
            &self.t012.nonunit_normal_12,
            &self.t012.nonunit_normal_20,
        );
        let t230 = SdSubtriangle::new(
            &self.t230.nonunit_normal_01,
            &self.t230.nonunit_normal_12,
            &self.t230.nonunit_normal_20,
        );
        let t012 = t012.to_triangle3d_on_sphere();
        let t230 = t230.to_triangle3d_on_sphere();
        let patch_size = self.img_sz;
        MapXYToVec {
            t012,
            t230,
            patch_size,
        }
    }
}
