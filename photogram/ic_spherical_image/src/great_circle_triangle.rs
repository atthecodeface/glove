use geo_nd::Vector;

use ic_base::Point3D;

use crate::{GreatCircleLineIndex, PtIndex, SphericalData};

/// A portion of a sphere bounded by three great circle segments
///
///
/// The midpoint may also be known; if so, then this GCLine is in essence the
/// definition of that midpoint's position.
#[derive(Debug)]
pub struct GcTriangle {
    /// The great circle segment containing points P0 and P1
    ///
    /// The segment has two points: if swapped bit 0 is clear then the segment's
    /// two points are P0 and P1, in that order; if swapped bit 0 is set the the
    /// segment's two points are in the reverse order
    gc0: GreatCircleLineIndex,
    gc1: GreatCircleLineIndex,
    gc2: GreatCircleLineIndex,
    swapped: u8,
    subdivision: u8,
}

impl GcTriangle {
    /// Create a new GcTriangle
    pub(crate) fn new(
        sd: &SphericalData,
        gc0: GreatCircleLineIndex,
        gc1: GreatCircleLineIndex,
        gc2: GreatCircleLineIndex,
        p0: PtIndex,
        p1: PtIndex,
        p2: PtIndex,
        subdivision: u8,
    ) -> Self {
        let mut swapped = 0;
        if sd[gc0].p0().index() != p0 {
            swapped |= 1;
        }
        if sd[gc1].p0().index() != p1 {
            swapped |= 2;
        }
        if sd[gc2].p0().index() != p2 {
            swapped |= 4;
        }
        Self {
            gc0,
            gc1,
            gc2,
            swapped,
            subdivision,
        }
    }

    /// Get the degree of subdivision for this triangle
    pub fn subdivision(&self) -> u8 {
        self.subdivision
    }

    /// Retrieve a line segment for the triangle, and whether it is included in
    /// the triangle (as a counter-clockwise triangle as viewed from outside) in
    /// its regular order, or whether it is included in the triangle 'backwards'
    pub fn gc_line(&self, n: usize) -> (bool, GreatCircleLineIndex) {
        match n {
            0 => ((self.swapped & 1) != 0, self.gc0),
            1 => ((self.swapped & 2) != 0, self.gc1),
            2 => ((self.swapped & 4) != 0, self.gc2),
            _ => {
                panic!("Cannot access more than 3 lines in a triangle");
            }
        }
    }

    /// Retrieve the three points in the triangle in counter-clockwise order as
    /// viewed from the outside (i.e. the order in which they were presented
    /// upon triangle creation)
    pub fn get_normals(&self, sd: &SphericalData) -> [Point3D; 3] {
        let sn0 = if (self.swapped & 1) != 0 { -1.0 } else { 1.0 };
        let sn1 = if (self.swapped & 2) != 0 { -1.0 } else { 1.0 };
        let sn2 = if (self.swapped & 4) != 0 { -1.0 } else { 1.0 };
        [
            *sd[self.gc0].normal().vector() * sn0,
            *sd[self.gc1].normal().vector() * sn1,
            *sd[self.gc2].normal().vector() * sn2,
        ]
    }

    /// Retrieve the three points in the triangle in counter-clockwise order as
    /// viewed from the outside (i.e. the order in which they were presented
    /// upon triangle creation)
    pub fn get_points(&self, sd: &SphericalData) -> (PtIndex, PtIndex, PtIndex) {
        let swapped = (self.swapped & 1) != 0;
        let p0 = sd[self.gc0].p0().index();
        let p1 = sd[self.gc0].p1().index();
        let (p0, p1) = if swapped { (p1, p0) } else { (p0, p1) };
        let p2a = sd[self.gc1].p0().index();
        let p2b = sd[self.gc1].p1().index();
        if p2a != p0 && p2a != p1 {
            (p0, p1, p2a)
        } else {
            (p0, p1, p2b)
        }
    }

    /// Determine if a point is within the triangle
    ///
    /// This returns zero if the triangle contains the point (projected onto the unit sphere)
    pub fn point_outside_lines(&self, sd: &SphericalData, p: &Point3D) -> u8 {
        let gc0 = sd[self.gc0].normal().vector().dot(p);
        let gc1 = sd[self.gc1].normal().vector().dot(p);
        let gc2 = sd[self.gc2].normal().vector().dot(p);
        let inside_gc0 = if (self.swapped & 1) != 0 {
            gc0 <= 0.0
        } else {
            gc0 >= 0.0
        };
        let inside_gc1 = if (self.swapped & 2) != 0 {
            gc1 <= 0.0
        } else {
            gc1 >= 0.0
        };
        let inside_gc2 = if (self.swapped & 4) != 0 {
            gc2 <= 0.0
        } else {
            gc2 >= 0.0
        };
        let mut result = 0;
        if !inside_gc0 {
            result |= 1;
        }
        if !inside_gc1 {
            result |= 2;
        }
        if !inside_gc2 {
            result |= 4;
        }
        result
    }

    /// Validate the triangle etc within the SphericalData, for debugging purposes
    ///
    /// This is not required in release builds
    ///
    /// If the triangle is sufficiently close to the sphere then its volume (1/3
    /// base * height) will have a height of 1 and hence a base area of three
    /// times the volume.
    pub fn validate(
        &self,
        sd: &SphericalData,
        min_volume: f64,
        max_volume: f64,
    ) -> Result<f64, String> {
        let mut p0 = sd[self.gc0].p0();
        let mut p1 = sd[self.gc0].p1();
        if (self.swapped & 1) != 0 {
            (p0, p1) = (p1, p0);
        }
        let mut p2 = sd[self.gc1].p0();
        let mut p3 = sd[self.gc1].p1();
        if (self.swapped & 2) != 0 {
            (p2, p3) = (p3, p2);
        }

        let mut p4 = sd[self.gc2].p0();
        let mut p5 = sd[self.gc2].p1();
        if (self.swapped & 4) != 0 {
            (p4, p5) = (p5, p4);
        }

        if p0.index() == p1.index() {
            return Err("GC line 0 must have different endpoints".into());
        }
        if p2.index() == p3.index() {
            return Err("GC line 1 must have different endpoints".into());
        }
        if p4.index() == p5.index() {
            return Err("GC line 2 must have different endpoints".into());
        }

        if p1.index() != p2.index() {
            return Err("GC line 0 ep must be 1 sp".into());
        }
        if p3.index() != p4.index() {
            return Err("GC line 1 ep must be 2 sp".into());
        }
        if p5.index() != p0.index() {
            return Err("GC line 2 ep must be 0 sp".into());
        }

        // Get the triangle vectors
        let tp0_vec = p0.vector();
        let tp1_vec = p1.vector();
        let tp2_vec = p3.vector();
        let volume = tp0_vec.cross_product(tp1_vec).dot(tp2_vec) / 6.0;
        assert!(
            volume >= min_volume && volume <= max_volume,
            "Exepected volume (effectively a measure of surface area) of triangles to be between {min_volume} and {max_volume} but had {volume}"
        );

        let t01_angle = tp0_vec.dot(tp1_vec).acos().to_degrees();
        let t12_angle = tp1_vec.dot(tp2_vec).acos().to_degrees();
        let t20_angle = tp2_vec.dot(tp0_vec).acos().to_degrees();
        eprintln!(
            "Angles between points: {t01_angle:.4}, {t12_angle:.4}, {t20_angle:.4} : {tp0_vec:.4} {tp1_vec:.4} {tp2_vec:.4}"
        );

        let should_be_p0_pm = sd[self.gc0]
            .normal()
            .vector()
            .cross_product(sd[self.gc2].normal().vector())
            .normalize();
        assert!(
            should_be_p0_pm.dot(tp0_vec).abs() > 0.9999,
            "P0 should be the cross product of the two normals to GC0 (p01) and GC2 (p20) {:?} {:?}",
            sd[self.gc0],
            sd[self.gc2],
        );

        let should_be_p1_pm = sd[self.gc0]
            .normal()
            .vector()
            .cross_product(sd[self.gc1].normal().vector())
            .normalize();
        assert!(
            should_be_p1_pm.dot(tp1_vec).abs() > 0.9999,
            "P1 should be the cross product of the two normals to GC0 (p01) and GC1 (p12) {:?} {:?}",
            sd[self.gc0],
            sd[self.gc1],
        );

        let should_be_p2_pm = sd[self.gc1]
            .normal()
            .vector()
            .cross_product(sd[self.gc2].normal().vector())
            .normalize();
        assert!(
            should_be_p2_pm.dot(tp2_vec).abs() > 0.9999,
            "P2 should be the cross product of the two normals to GC1 (p12) and GC2 (p20) {:?} {:?}",
            sd[self.gc1],
            sd[self.gc2],
        );

        // Check the normals for all the great circle segments match the gc normals
        for gc in [self.gc0, self.gc1, self.gc2] {
            let gcl = &sd[gc];
            let p0 = gcl.p0();
            let p1 = gcl.p1();
            let normal = p0.vector().cross_product(p1.vector()).normalize();
            let gcl_normal = gcl.normal().vector();
            if normal.distance_sq(gcl_normal) > 1E-6 {
                return Err(format!(
                    "Distance between normal {normal} and {gcl_normal} for {gcl:?} and its two points {p0:?}, {p1:?} is nonzero"
                ));
            }
            if p0.index() > p1.index() {
                return Err(format!(
                    "Points for {gcl:?} are not in the correct order {p0:?}, {p1:?}"
                ));
            }
        }

        Ok(volume)
    }
}
