use std::collections::HashMap;
use std::rc::Rc;

use geo_nd::{Vector, vector::sub};

use ic_base::Point3D;
use indexed::{Idx, IndexedVec};

use crate::{
    GcLine, GcNormal, GcTriangle, ImagePt, SdSubtriangle, SphericalImageError, SubdivisionPath,
};

indexed::make_index!(
    /// An index into the 'points' vector of `Rc<(PtIndex, Point3D)>`
    PtIndex, usize, false);

indexed::make_index!(
    /// An index into the 'normals' vector of `Rc<(NormalIndex, Point3D)>`
    ///
    /// In theory these can be deduplicated (i.e. a specific value of Point3D is
    /// only stored once); however, this may no be the case initially
    NormalIndex, usize, false);

indexed::make_index!(
    /// An index into the 'great circles' vector of `image_gc`
    ///
    /// Each represents a portion of a great circle between two points, possibly
    /// with a defined mid-point, and it refers to the normal to the GC
    GreatCircleLineIndex, usize, false);

indexed::make_index!(
    /// An index into the 'great circle triangles' vector of `image_gc`
    ///
    /// This can be 'None'
    ///
    /// Each represents a portion of a great circle between two points, possibly
    /// with a defined mid-point, and it refers to the normal to the GC
    GreatCircleTriangleIndex, usize, true);

/// A set of points, normals, great circles, etc that make up the informatioqn for a spherical image
///
/// This does not include any pixel data, and is a purely internal structure
#[derive(Default, Debug)]
pub struct SphericalData {
    points: IndexedVec<PtIndex, Rc<ImagePt>, false>,
    gc_lines: IndexedVec<GreatCircleLineIndex, GcLine, true>,
    normals: IndexedVec<NormalIndex, Rc<GcNormal>, false>,
    gc_triangles: IndexedVec<GreatCircleTriangleIndex, GcTriangle, false>,
    gc_line_map: HashMap<(PtIndex, PtIndex), GreatCircleLineIndex>,
    gc_triangle_map: HashMap<(PtIndex, PtIndex, PtIndex), GreatCircleTriangleIndex>,
    gc_triangle_index: HashMap<(GreatCircleTriangleIndex, u64), GreatCircleTriangleIndex>,
}

impl std::ops::Index<PtIndex> for SphericalData {
    type Output = Rc<ImagePt>;
    fn index(&self, index: PtIndex) -> &Self::Output {
        &self.points[index]
    }
}

impl std::ops::Index<GreatCircleLineIndex> for SphericalData {
    type Output = GcLine;
    fn index(&self, index: GreatCircleLineIndex) -> &Self::Output {
        &self.gc_lines[index]
    }
}

impl std::ops::Index<NormalIndex> for SphericalData {
    type Output = Rc<GcNormal>;
    fn index(&self, index: NormalIndex) -> &Self::Output {
        &self.normals[index]
    }
}

impl std::ops::Index<GreatCircleTriangleIndex> for SphericalData {
    type Output = GcTriangle;
    fn index(&self, index: GreatCircleTriangleIndex) -> &Self::Output {
        &self.gc_triangles[index]
    }
}

impl SphericalData {
    /// Create from shape data - points and triangle indices
    pub fn of_shape(pts: &[[f64; 3]], triangle_indices: &[(usize, usize, usize)]) -> Self {
        let mut sd = Self::default();
        let pts: Vec<_> = pts.iter().map(|p| sd.add_initial_point(p.into())).collect();
        for (p0, p1, p2) in triangle_indices {
            sd.add_initial_gc_triangle(pts[*p0], pts[*p1], pts[*p2]);
        }
        sd
    }

    /// Iterate through the points
    pub fn iter_points(&self) -> impl ExactSizeIterator<Item = &'_ Rc<ImagePt>> {
        self.points.iter()
    }

    /// Get the number of triangles
    pub fn num_triangles(&self) -> usize {
        self.gc_triangles.len()
    }

    /// Iterate through the triangles
    pub fn iter_triangles(&self) -> impl ExactSizeIterator<Item = &'_ GcTriangle> {
        self.gc_triangles.iter()
    }

    /// Iterate through the triangle indices
    pub fn iter_triangle_indicess(
        &self,
    ) -> impl ExactSizeIterator<Item = GreatCircleTriangleIndex> {
        self.gc_triangles.indices()
    }

    /// Iterate through the triangles
    pub fn iter_normals(&self) -> impl ExactSizeIterator<Item = &'_ Rc<GcNormal>> {
        self.normals.iter()
    }

    /// Add an initial point, from one of the vertices of the `shape` triangles
    pub fn add_initial_point(&mut self, pt: Point3D) -> PtIndex {
        let idx = self.points.next_index();
        self.points.push(ImagePt::new(idx, pt).into());
        idx
    }

    /// Add a normal to the data, (if it is not already present)
    fn add_normal(&mut self, normal: Point3D) -> (NormalIndex, NormalIndex) {
        let normal = normal.normalize();
        for (i, n) in self
            .normals
            .iter()
            .enumerate()
            .filter(|(i, _)| (*i & 1) == 0)
        {
            let n_dot_n = normal.dot(n.vector());
            if n_dot_n.abs() > 0.999999 {
                if n_dot_n > 0.0 {
                    return (NormalIndex::from_usize(i), NormalIndex::from_usize(i + 1));
                } else {
                    return (NormalIndex::from_usize(i + 1), NormalIndex::from_usize(i));
                }
            }
        }
        let idx = self.normals.next_index();
        self.normals.push(GcNormal::new(idx, normal).into());
        let inv_idx = self.normals.next_index();
        self.normals.push(GcNormal::new(inv_idx, -normal).into());
        (idx, inv_idx)
    }

    /// Insert the specified GC line based on the two point indices
    fn add_gc_line_to_map(&mut self, p0: PtIndex, p1: PtIndex, gc: GreatCircleLineIndex) {
        let (p0, p1) = (p0.min(p1), p0.max(p1));
        self.gc_line_map.insert((p0, p1), gc);
    }

    /// Find the GC line between the two points, and return (false, gc) if the
    /// great circle line is between p0 and p1; (true, gc) if it is between p1
    /// and p0; none if it is not present
    fn find_gc_line(&self, p0: PtIndex, p1: PtIndex) -> Option<GreatCircleLineIndex> {
        let (p0, p1) = (p0.min(p1), p0.max(p1));
        self.gc_line_map.get(&(p0, p1)).copied()
    }

    /// Insert the specified GC line based on the two point indices
    fn add_gc_triangle(
        &mut self,
        p0: PtIndex,
        p1: PtIndex,
        p2: PtIndex,
        gc: GreatCircleTriangleIndex,
    ) {
        let (p0, p1) = (p0.min(p1), p0.max(p1));
        let (p0, p2) = (p0.min(p2), p0.max(p2));
        let (p1, p2) = (p1.min(p2), p1.max(p2));
        self.gc_triangle_map.insert((p0, p1, p2), gc);
        self.gc_triangle_index.insert(
            (self[gc].toplevel(gc), self[gc].subdivision_path().path()),
            gc,
        );
    }

    /// Find the GC triangle of toplevel and path
    pub fn find_gc_triangle_of_toplevel_and_path(
        &self,
        top_gc: GreatCircleTriangleIndex,
        subdivision_path: SubdivisionPath,
    ) -> Option<GreatCircleTriangleIndex> {
        self.gc_triangle_index
            .get(&(top_gc, subdivision_path.path()))
            .copied()
    }

    /// Find the GC triangle using the three points
    ///
    /// This is used in 'find or add' to stop duplicate triangles
    pub fn find_gc_triangle_of_points(
        &self,
        p0: PtIndex,
        p1: PtIndex,
        p2: PtIndex,
    ) -> Option<GreatCircleTriangleIndex> {
        let (p0, p1) = (p0.min(p1), p0.max(p1));
        let (p0, p2) = (p0.min(p2), p0.max(p2));
        let (p1, p2) = (p1.min(p2), p1.max(p2));
        self.gc_triangle_map.get(&(p0, p1, p2)).copied()
    }

    /// Add an great circle line segment between two points, as par of adding a triangle
    ///
    /// This may be invoked by initial triangle creation or triangle subdivision
    /// (both of which will do soe through find_or_add_gc_triangle)
    fn find_or_add_gc_line(&mut self, p0: PtIndex, p1: PtIndex) -> GreatCircleLineIndex {
        if let Some(swap_gc) = self.find_gc_line(p0, p1) {
            return swap_gc;
        }
        let swapped = p0 > p1;
        let p0_vec = self.points[p0].vector();
        let p1_vec = self.points[p1].vector();
        let normal = p0_vec.cross_product(p1_vec);
        let normal = if swapped { -normal } else { normal };
        let normal_idx = self.add_normal(normal).0;
        let (_swapped, gc_line) = GcLine::new(
            &self.normals[normal_idx],
            &self.points[p0],
            &self.points[p1],
        );
        let gc_index = self.gc_lines.push(gc_line);
        self.add_gc_line_to_map(p0, p1, gc_index);
        gc_index
    }

    /// Find, or add if not present, a GC triangle that uses P0, P1, P2
    /// counterclockwise when viewed from the outside
    pub fn find_or_add_subdivided_gc_triangle(
        &mut self,
        subdivides: GreatCircleTriangleIndex,
        p0: PtIndex,
        p1: PtIndex,
        p2: PtIndex,
        subdivision_path_step: u8,
    ) -> (bool, GreatCircleTriangleIndex) {
        if let Some(gc) = self.find_gc_triangle_of_points(p0, p1, p2) {
            return (false, gc);
        }

        let subdivision_path = self[subdivides]
            .subdivision_path()
            .subpath(subdivision_path_step as u64);
        let gc_p0_p1 = self.find_or_add_gc_line(p0, p1);
        let gc_p1_p2 = self.find_or_add_gc_line(p1, p2);
        let gc_p2_p0 = self.find_or_add_gc_line(p2, p0);
        let top_gc = self[subdivides].toplevel(subdivides);
        let gc_tri = GcTriangle::new(
            self,
            top_gc,
            gc_p0_p1,
            gc_p1_p2,
            gc_p2_p0,
            p0,
            p1,
            p2,
            subdivision_path,
        );
        let gc_idx = self.gc_triangles.push(gc_tri);
        self.add_gc_triangle(p0, p1, p2, gc_idx);
        (true, gc_idx)
    }

    /// Add initial GC triangle that uses P0, P1, P2
    /// counterclockwise when viewed from the outside
    pub fn add_initial_gc_triangle(
        &mut self,
        p0: PtIndex,
        p1: PtIndex,
        p2: PtIndex,
    ) -> GreatCircleTriangleIndex {
        if let Some(_gc) = self.find_gc_triangle_of_points(p0, p1, p2) {
            panic!("Triangle already exists");
        }
        let gc_p0_p1 = self.find_or_add_gc_line(p0, p1);
        let gc_p1_p2 = self.find_or_add_gc_line(p1, p2);
        let gc_p2_p0 = self.find_or_add_gc_line(p2, p0);
        let gc_tri = GcTriangle::new(
            self,
            GreatCircleTriangleIndex::none(),
            gc_p0_p1,
            gc_p1_p2,
            gc_p2_p0,
            p0,
            p1,
            p2,
            SubdivisionPath::default(),
        );
        let gc_idx = self.gc_triangles.push(gc_tri);
        self.add_gc_triangle(p0, p1, p2, gc_idx);
        gc_idx
    }

    /// Get the midpoint point index of a great circle line; if ths point does
    /// not exist, then create it by splitting the line
    pub fn get_or_add_midpoint_of_line(&mut self, gc_line: GreatCircleLineIndex) -> PtIndex {
        let mid_pt = {
            let gc_line = self
                .gc_lines
                .get(gc_line)
                .expect("Bad GC line index into spherical image data");
            gc_line.midpoint()
        };
        if let Some(mid_pt) = mid_pt {
            return mid_pt.index();
        }
        let p0 = self.gc_lines[gc_line].p0().clone();
        let p1 = self.gc_lines[gc_line].p1().clone();
        let mid_pt_vec = (p0.vector() + p1.vector()).normalize();
        let mid_pt_idx = self.points.next_index();
        self.points
            .push(ImagePt::new(mid_pt_idx, mid_pt_vec).into());
        let mid_pt = &self.points[mid_pt_idx];
        self.gc_lines[gc_line].set_midpoint(mid_pt.clone());

        let normal = self.gc_lines[gc_line].normal();
        let inv_normal_idx = self.gc_lines[gc_line].normal().inv_index();
        let inv_normal = &self.normals[inv_normal_idx];
        let (s0, gc_p0_mp) = GcLine::new(normal, &p0, mid_pt);
        assert!(!s0, "Ordering of points is fixed");
        let (s1, gc_p1_mp) = GcLine::new(inv_normal, &p1, mid_pt);
        assert!(!s1, "Ordering of points is fixed");
        let gc_p0_mp_index = self.gc_lines.push(gc_p0_mp);
        let gc_p1_mp_index = self.gc_lines.push(gc_p1_mp);
        self.add_gc_line_to_map(p0.index(), mid_pt_idx, gc_p0_mp_index);
        self.add_gc_line_to_map(p1.index(), mid_pt_idx, gc_p1_mp_index);
        mid_pt_idx
    }

    /// Find the GreatCircleTriangleIndex of the triangle at the required
    /// subdivision level that contains the point
    pub fn find_gc_triangle_of_vector(
        &self,
        v: &Point3D,
        subdivision: u8,
    ) -> Option<GreatCircleTriangleIndex> {
        for t in self.gc_triangles.indices() {
            if self[t].subdivision_path().subdivision() != subdivision {
                continue;
            }
            if self.gc_triangles[t].point_outside_lines(self, v) == 0 {
                return Some(t);
            }
        }
        None
    }

    /// Subdivide a triangle into 4 smaller triangles and return them
    ///
    /// The 4 subtriangles are T0, T1, T2, T3.
    ///
    /// Given the triangle is P0, P1, P2:
    ///
    /// ```text
    ///            P0
    ///           /  \
    ///          / T0 \
    ///        M01----M20
    ///        /  \T3/  \
    ///       / T1 \/ T2 \
    ///      P1----M12----P2
    /// ```
    pub fn subdivide_triangle(
        &mut self,
        gc_triangle: GreatCircleTriangleIndex,
    ) -> (u8, [GreatCircleTriangleIndex; 4]) {
        // Retrieve the points in counter-clockwise order as viewed from the outside
        let gc = &self[gc_triangle];
        let (p0, p1, p2) = gc.get_points(self);

        // Retrieve the lines to create midpoints
        let p01 = gc.gc_line(0).1;
        let p12 = gc.gc_line(1).1;
        let p02 = gc.gc_line(2).1;

        // Create midpoints of the lines
        let p01_mp = self.get_or_add_midpoint_of_line(p01);
        let p12_mp = self.get_or_add_midpoint_of_line(p12);
        let p02_mp = self.get_or_add_midpoint_of_line(p02);

        // Create four triangles with counter-clockwise points as viewed from the outside
        let (t0_new, t0) =
            self.find_or_add_subdivided_gc_triangle(gc_triangle, p0, p01_mp, p02_mp, 0);
        let (t1_new, t1) =
            self.find_or_add_subdivided_gc_triangle(gc_triangle, p1, p12_mp, p01_mp, 1);
        let (t2_new, t2) =
            self.find_or_add_subdivided_gc_triangle(gc_triangle, p2, p02_mp, p12_mp, 2);
        let (t3_new, t3) =
            self.find_or_add_subdivided_gc_triangle(gc_triangle, p02_mp, p01_mp, p12_mp, 3);

        // Count the total new triangles (should be 0 or 4)
        let mut total_new = 0;
        if t0_new {
            total_new += 1;
        }
        if t1_new {
            total_new += 1;
        }
        if t2_new {
            total_new += 1;
        }
        if t3_new {
            total_new += 1;
        }
        (total_new, [t0, t1, t2, t3])
    }

    /// Find which subtriangle a point is in
    ///
    /// The 4 subtriangles are T0, T1, T2, T3; return a value between 0 and 3
    ///
    /// Given the triangle is P0, P1, P2:
    ///
    /// ```text
    ///            P0
    ///           /  \
    ///          / T0 \
    ///        M01----M20
    ///        /  \T3/  \
    ///       / T1 \/ T2 \
    ///      P1----M12----P2
    /// ```
    ///
    pub fn find_subtriangle_of_point_in_triangle(
        &self,
        gc_triangle: GreatCircleTriangleIndex,
        p: &Point3D,
        subdivide: u8,
    ) -> SdSubtriangle {
        let normals = self[gc_triangle].get_normals(self);

        let mut sub = SdSubtriangle::new(&normals[0], &normals[1], &normals[2]);
        for _ in 0..subdivide {
            sub.find_subtriangle_of_point(p);
        }
        sub
    }

    pub fn find_triangle_or_parent(
        &self,
        gc_triangle: GreatCircleTriangleIndex,
        mut subdivision_path: SubdivisionPath,
    ) -> GreatCircleTriangleIndex {
        let top_gc = self[gc_triangle].toplevel(gc_triangle);
        while subdivision_path.subdivision() > 0 {
            if let Some(gct) = self.find_gc_triangle_of_toplevel_and_path(top_gc, subdivision_path)
            {
                return gct;
            }
            subdivision_path = subdivision_path.parent();
        }
        top_gc
    }

    /// Find the required SubdivisionPath of a GC Triangle in the SphereData -
    /// which may not be a toplevel triangle
    pub fn find_or_subdivide_to_gc_triangle(
        &mut self,
        gc_triangle: GreatCircleTriangleIndex,
        subdivision_path: SubdivisionPath,
    ) -> Result<GreatCircleTriangleIndex, SphericalImageError> {
        loop {
            let gct = self.find_triangle_or_parent(gc_triangle, subdivision_path);
            if self[gct].subdivision_path().subdivision() == subdivision_path.subdivision() {
                return Ok(gct);
            }
            self.subdivide_triangle(gct);
        }
    }
}
