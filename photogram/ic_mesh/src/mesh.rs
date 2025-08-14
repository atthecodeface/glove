//a Imports
use std::cmp::Ordering;
use std::collections::HashMap;
use std::default::Default;

use geo_nd::Vector;
use serde::{Deserialize, Serialize};

use ic_base::Point2D;

use crate::{IndexLine, IndexTriangle, LineIndex, PointIndex, TriangleIndex};

//a Mesh
//tp Mesh
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Mesh {
    pxy: Vec<Point2D>,
    lines: Vec<IndexLine>,
    triangles: Vec<IndexTriangle>,
    line_set: HashMap<(PointIndex, PointIndex), LineIndex>,
}

//ip Index<PointIndex>
impl std::ops::Index<PointIndex> for Mesh {
    type Output = Point2D;
    fn index(&self, index: PointIndex) -> &Self::Output {
        &self.pxy[index.as_usize()]
    }
}

//ip IndexMut<PointIndex>
impl std::ops::IndexMut<PointIndex> for Mesh {
    fn index_mut(&mut self, index: PointIndex) -> &mut Self::Output {
        &mut self.pxy[index.as_usize()]
    }
}

//ip Index<LineIndex>
impl std::ops::Index<LineIndex> for Mesh {
    type Output = IndexLine;
    fn index(&self, index: LineIndex) -> &Self::Output {
        &self.lines[index.as_usize()]
    }
}

//ip IndexMut<LineIndex>
impl std::ops::IndexMut<LineIndex> for Mesh {
    fn index_mut(&mut self, index: LineIndex) -> &mut Self::Output {
        &mut self.lines[index.as_usize()]
    }
}

//ip Index<TriangleIndex>
impl std::ops::Index<TriangleIndex> for Mesh {
    type Output = IndexTriangle;
    fn index(&self, index: TriangleIndex) -> &Self::Output {
        &self.triangles[index.as_usize()]
    }
}

//ip IndexMut<TriangleIndex>
impl std::ops::IndexMut<TriangleIndex> for Mesh {
    fn index_mut(&mut self, index: TriangleIndex) -> &mut Self::Output {
        &mut self.triangles[index.as_usize()]
    }
}

//ip Mesh
impl Mesh {
    //cp new
    pub fn new<I: Iterator<Item = Point2D>>(pts: I) -> Self {
        let pxy = pts.collect();
        let lines = vec![];
        let triangles = vec![];
        let line_set = HashMap::new();
        Self {
            pxy,
            lines,
            triangles,
            line_set,
        }
    }

    //cp optimized
    pub fn optimized<I: Iterator<Item = Point2D>>(pts: I, min_dist: f64) -> Self {
        let mut mesh = Self::new(pts);
        while mesh.remove_duplicates(&mesh.find_duplicates(min_dist)) {}
        mesh.create_mesh_triangles();
        while mesh.optimize_mesh_quads() {}
        mesh
    }

    //mp find_duplicates
    pub fn find_duplicates(&self, min_dist: f64) -> Vec<(PointIndex, PointIndex)> {
        let mut result = vec![];
        let min_dist2 = min_dist * min_dist;
        for i in 1..self.pxy.len() {
            let pt_i = &self.pxy[i];
            for j in 0..i {
                if pt_i.distance_sq(&self.pxy[j]) < min_dist2 {
                    result.push((i.into(), j.into()));
                    break;
                }
            }
        }
        result
    }

    //mp remove_duplicates
    pub fn remove_duplicates(&mut self, dups: &[(PointIndex, PointIndex)]) -> bool {
        if dups.is_empty() {
            false
        } else {
            for (n, _) in dups.iter().rev() {
                self.pxy.remove(n.as_usize());
            }
            true
        }
    }

    //mp add_pt
    pub fn add_pt(&mut self, p: Point2D) -> PointIndex {
        let n = self.pxy.len();
        self.pxy.push(p);
        n.into()
    }

    //mp clear
    pub fn clear(&mut self) {
        self.lines.clear();
        self.triangles.clear();
    }

    //mi find_lbx_pt
    /// Find the 'origin' of the points
    ///
    /// The 'origin' of a set of points is the one with the smallest
    /// X, or for sets with many with the same smallest X, the one
    /// with smallest X and smallest Y
    ///
    /// Effectively left-most, with tiebreaker as bottom-most
    // pub for test purposes
    pub fn find_lbx_pt(&self) -> PointIndex {
        let (n, _lbx) = self
            .pxy
            .iter()
            .enumerate()
            .fold((0, self.pxy[0]), |(n, lbx), (i, pt)| {
                if (pt[0] < lbx[0]) || (pt[0] == lbx[0] && pt[1] < lbx[1]) {
                    (i, *pt)
                } else {
                    (n, lbx)
                }
            });
        n.into()
    }

    //fi polar_cmp
    /// Compare two points for the sweep from the 'origin', based on
    /// the gradient of the line from the origin
    ///
    /// The origin has the smallest X, with tiebreaker of smallest Y
    ///
    /// So smaller of the y/x of two points. This has issues if X=0, or if the two y/x are equal.
    ///
    /// If both have X=0, then order based on Y; indeed, this is true for non-zero X...
    ///
    /// If only one has X=0, then order based on X
    ///
    /// If neither have X=0 (i.e. X both +ve) then compare y0/x0 wih y1/x1: or y0*x1 with x0*y1 (note x is +ve)
    ///
    /// If they are equal then order based on X
    ///
    fn polar_cmp((r2_0, theta_0): &(f64, f64), (r2_1, theta_1): &(f64, f64)) -> Ordering {
        if *r2_0 == 0.0 {
            Ordering::Less
        } else if *r2_1 == 0.0 {
            Ordering::Greater
        } else {
            // Note theta increases as the points move anticlockwise
            match theta_0.partial_cmp(&theta_1).unwrap() {
                Ordering::Equal => r2_0.partial_cmp(&r2_1).unwrap(),
                x => x,
            }
        }
    }

    //mp find_sweep
    /// Sweep from an 'origin' round anti-clockwise
    ///
    /// The origin has the smallest X, with tiebreaker of smallest Y;
    /// all points thus have a non-negative relative X.
    ///
    /// To generate a stable sort each point is converted to polar coordinates (r2, theta) relative to the origin
    ///
    /// The distance is held as x*x+y*y; the angle as y/x for x>1E-6;
    /// if x is close enough to 0 then assume x=1E-6
    pub fn find_sweep(&self) -> (PointIndex, Vec<PointIndex>) {
        let lbx_n = self.find_lbx_pt();
        let lbx = self[lbx_n];
        fn compare_pts(a: &(PointIndex, (f64, f64)), b: &(PointIndex, (f64, f64))) -> Ordering {
            Mesh::polar_cmp(&a.1, &b.1)
        }
        fn to_polar(p: Point2D) -> (f64, f64) {
            let r2 = p.length_sq();
            // let result = (r2, p[1] / (p[0].max(1E-6)));
            // eprintln!("{p} {r2}, {},  {result:?}", r2 == 0.);
            (r2, p[1] / (p[0].max(1E-6)))
        }
        let mut n_dxys: Vec<(PointIndex, (f64, f64))> = self
            .pxy
            .iter()
            .enumerate()
            .map(|(n, p)| (n.into(), to_polar((*p) - lbx)))
            .collect();

        n_dxys.sort_by(compare_pts);
        let ns = n_dxys.into_iter().map(|a| a.0).collect();
        (lbx_n, ns)
    }

    //mp add_line
    /// Add a line from a to be with triangle t on the left of the line
    fn add_line(&mut self, p0: PointIndex, p1: PointIndex, t0: TriangleIndex) -> LineIndex {
        let line = IndexLine { p0, p1, t0, t1: t0 };
        let n = self.lines.len().into();
        self.lines.push(line);
        self.line_set.insert((p0, p1), n);
        n
    }

    //mp find_or_add_line
    /// Find or add a line from p0 to p1
    ///
    /// If p1 to p0 is in the line_set, then return that with this as
    /// t1; otherwise return a new line
    fn find_or_add_line(&mut self, p0: PointIndex, p1: PointIndex, t0: TriangleIndex) -> LineIndex {
        if let Some(ln) = self.line_set.get_mut(&(p1, p0)) {
            // Cannot use IndexMut trait as self.line_set is mutably borrowed
            self.lines[ln.as_usize()].t1 = t0;
            *ln
        } else {
            self.add_line(p0, p1, t0)
        }
    }

    //mp add_triangle
    pub fn add_triangle(
        &mut self,
        p0: PointIndex,
        p1: PointIndex,
        p2: PointIndex,
    ) -> TriangleIndex {
        assert!(
            p0 != p1 && p1 != p2 && p2 != p0,
            "Cannot create triangle with duplicate points {p0} {p1} {p2}"
        );
        let tn = self.triangles.len().into();
        let l0 = self.find_or_add_line(p0, p1, tn);
        let l1 = self.find_or_add_line(p1, p2, tn);
        let l2 = self.find_or_add_line(p2, p0, tn);
        self.triangles
            .push(IndexTriangle::new(p0, p1, p2, l0, l1, l2));
        tn
    }

    //mp create_mesh_triangles
    pub fn create_mesh_triangles(&mut self) {
        self.clear();
        if self.pxy.is_empty() {
            return;
        }
        let (lbx, sweep) = self.find_sweep();
        let num_triangles = sweep.len() - 1;
        for n in 1..num_triangles {
            // eprintln!(
            //    "sweep {:4}: add {lbx}, {}, {} : {:.4}",
            //    self.triangles.len(),
            //    sweep[n],
            //    sweep[n + 1],
            //    self[sweep[n + 1]],
            // );
            self.add_triangle(lbx, sweep[n], sweep[n + 1]);
        }
        let mut hull = sweep;
        let mut p0 = 0;
        let mut p1 = 1;
        let mut p2 = 2;
        let hull_len = hull.len();
        while p2 <= hull_len {
            let ph0 = hull[p0];
            let ph1 = hull[p1];
            let ph2 = hull[p2 % hull_len];
            // eprintln!("Check {:4} {ph0}, {ph1}, {ph2} ({p0}, {p1}, {p2})", self.triangles.len());
            let pt0 = self[ph0];
            let pt1 = self[ph1];
            let pt2 = self[ph2];
            let d01 = pt0 - pt1;
            let d21 = pt2 - pt1;
            if d01[0] * d21[1] > d01[1] * d21[0] {
                // eprintln!(
                // "convex_hull {:4}: add {ph0}, {ph1}, {ph2}",
                // self.triangles.len()
                // );
                self.add_triangle(ph0, ph2, ph1);
                hull[p1] = hull[p0];
                while hull[p0] == hull[p1] {
                    if p0 == 0 {
                        p1 += 1;
                    } else {
                        p0 -= 1;
                    }
                }
                if p1 >= p2 {
                    p2 = p1 + 1;
                }
            } else {
                p0 = p1;
                p1 = p2;
                p2 = p1 + 1;
            }
            if p1 == hull.len() {
                break;
            }
            loop {
                if p2 == hull.len() {
                    break;
                }
                if hull[p2] != hull[p1] {
                    break;
                }
                p2 += 1;
            }
        }
    }

    //mp remove_zero_area_triangles
    pub fn remove_zero_area_triangles(&mut self) -> bool {
        let mut changed = false;
        let mut zero_area_triangles = self.find_zero_area_triangles();
        zero_area_triangles.sort_by(|(_, a_l2), (_, b_l2)| b_l2.partial_cmp(&a_l2).unwrap());
        for (t, _) in zero_area_triangles {
            if self.triangle_area(t) < 1E-12 {
                let (p0, p1, p2) = self[t].pts();
                let (l01, l12, l20) = self[t].lines();
                let p01 = self[p0] - self[p1];
                let p02 = self[p0] - self[p2];
                let p12 = self[p1] - self[p2];
                let p01_l2 = p01.length_sq();
                let p02_l2 = p02.length_sq();
                let p12_l2 = p12.length_sq();
                // eprintln!("Fix triangle! {t} {p01_l2} {p12_l2} {p02_l2} : {l01} {l12} {l20}");
                if p01_l2 >= p02_l2 && p01_l2 >= p12_l2 {
                    changed |= self.swap_or_remove_triangles_for_line(l01);
                } else if p02_l2 >= p12_l2 {
                    changed |= self.swap_or_remove_triangles_for_line(l20);
                } else {
                    changed |= self.swap_or_remove_triangles_for_line(l12);
                }
                break;
            }
        }
        changed
    }

    //mp find_zero_area_triangles
    pub fn find_zero_area_triangles(&self) -> Vec<(TriangleIndex, f64)> {
        let mut result = vec![];
        for t in (0..self.triangles.len()).map(|t| t.into()) {
            if self.triangle_area(t) < 1E-12 {
                let (p0, p1, p2) = self[t].pts();
                let (l01, l12, l20) = self[t].lines();
                let p01 = self[p0] - self[p1];
                let p02 = self[p0] - self[p2];
                let p12 = self[p1] - self[p2];
                let p01_l2 = p01.length_sq();
                let p02_l2 = p02.length_sq();
                let p12_l2 = p12.length_sq();
                let max_l2 = p01_l2.max(p02_l2).max(p12_l2);
                // eprintln!("Zero-area triangle {t} {max_l2}");
                result.push((t, max_l2));
            }
        }
        result
    }

    //mi swap_or_remove_triangles_for_line
    /// If the line has two triangles (a quad) then swap the diagonals
    ///
    /// Else just leave it alone - it is that one triangle is an
    /// exterior triangle which cannot be optimized, but we cannot
    /// remove the line.
    fn swap_or_remove_triangles_for_line(&mut self, l: LineIndex) -> bool {
        let line = &self[l];
        // eprintln!("swap_or_remove_triangles_for_line {l} {line}");
        let Some((o_p0, o_p1)) = line.opposite_diagonal(&self.triangles) else {
            return false;
        };
        let (c_p0, c_p1) = line.pts();
        self.quad_swap_diagonals_unless_it_makes_zero_area(l, c_p0, c_p1, o_p0, o_p1)
    }

    //mp triangle_area
    pub fn triangle_area(&self, t: TriangleIndex) -> f64 {
        let (p0, p1, p2) = self[t].pts();
        self.triangle_area_of_pts(p0, p1, p2)
    }

    //mp triangle_area_of_pts
    pub fn triangle_area_of_pts(&self, p0: PointIndex, p1: PointIndex, p2: PointIndex) -> f64 {
        let p01 = self[p0] - self[p1];
        let p02 = self[p0] - self[p2];
        ((p01[0] * p02[1] - p01[1] * p02[0]) / 2.0).abs()
    }

    //mi quad_swap_diagonals_unless_it_makes_zero_area
    //
    // Quad is (c_p0, o_p1, c_p1, o_p0)
    //
    // Currently the diagonal line 'i' is c_p0 to c_p1
    // T0 is c_p1, o_p0, c_p0 : L?A, L?B, i  *in some order* (T0 on left of line)
    // T1 is c_p0, o_p1, c_p1 : L?C, L?D, i  *in some order* (T1 on right of line)
    //
    // We make the line o_p0 to o_p1
    // T0 to c_p1, o_p0, o_p1 : L?A, i, L?D  *in this order* (T0 on left of line)
    // T1 to c_p0, o_p1, o_p0 : L?C, i, L?B, *in this order* (T1 on right of line)
    //
    // Note that L?D now has T0 instead of T1 on its side
    // Note that L?B now has T1 instead of T0 on its side
    fn quad_swap_diagonals_unless_it_makes_zero_area(
        &mut self,
        ln_i: LineIndex,
        c_p0: PointIndex,
        c_p1: PointIndex,
        o_p0: PointIndex,
        o_p1: PointIndex,
    ) -> bool {
        // eprintln!("Swap quad diagonals {ln_i} {c_p0} {c_p1} {o_p0} {o_p1}");

        // If triangle c_p1, o_p0, o_p1 has tiny area then do not do it
        if self.triangle_area_of_pts(c_p1, o_p0, o_p1) < 1E-6 {
            return false;
        }

        // If triangle c_p0, o_p0, o_p1 has tiny area then do not do it
        if self.triangle_area_of_pts(c_p0, o_p0, o_p1) < 1E-6 {
            return false;
        }

        let cd = self[c_p1] - self[c_p0];
        let od = self[o_p1] - self[o_p0];

        let on = [od[1], -od[0]];
        let c_p0_side_of_od = (self[c_p0] - self[o_p0]).dot(&on);
        let c_p1_side_of_od = (self[c_p1] - self[o_p0]).dot(&on);
        if c_p0_side_of_od * c_p1_side_of_od > 0. {
            return false;
        }

        let cn = [cd[1], -cd[0]];
        let o_p0_side_of_od = (self[o_p0] - self[c_p0]).dot(&cn);
        let o_p1_side_of_od = (self[o_p1] - self[c_p0]).dot(&cn);
        if o_p0_side_of_od * o_p1_side_of_od > 0. {
            return false;
        }

        let t0 = self[ln_i].t0;
        let t1 = self[ln_i].t1;
        let ln_b = self[t0].line_from_pt(o_p0);
        let ln_d = self[t1].line_from_pt(o_p1);
        // eprintln!(" {t0} {t1} {ln_b} {ln_d}");

        // eprintln!(" T0: {}", self[t0]);
        // eprintln!(" T1: {}", self[t1]);
        // eprintln!(" L_i: {}", self[ln_i]);
        // eprintln!(" L_b: {}", self[ln_b]);
        // eprintln!(" L_d: {}", self[ln_d]);

        self[t0].change_pt(c_p0, o_p1);
        self[t0].change_ln(ln_i, ln_d);
        self[t0].change_ln(ln_b, ln_i);
        self[t1].change_pt(c_p1, o_p0);
        self[t1].change_ln(ln_i, ln_b);
        self[t1].change_ln(ln_d, ln_i);
        self[ln_i].change_pt(c_p0, o_p0);
        self[ln_i].change_pt(c_p1, o_p1);
        self[ln_b].change_triangle(t0, t1);
        self[ln_d].change_triangle(t1, t0);

        // eprintln!("Afterwards:");
        // eprintln!(" T0: {}", self[t0]);
        // eprintln!(" T1: {}", self[t1]);
        // eprintln!(" L_i: {}", self[ln_i]);
        // eprintln!(" L_b: {}", self[ln_b]);
        // eprintln!(" L_d: {}", self[ln_d]);

        return true;
    }

    //mp optimize_mesh_quads
    pub fn optimize_mesh_quads(&mut self) -> bool {
        if self.remove_zero_area_triangles() {
            return true;
        }
        let mut changed = false;
        for i in 0..self.lines.len() {
            let ln_i: LineIndex = i.into();
            if let Some((o_p0, o_p1)) = self[ln_i].opposite_diagonal(&self.triangles) {
                // eprintln!("{:?}", self[ln_i]);

                // Get current diagonal c_p0, c_p1 and other diagonal o_p0, o_p1
                let (c_p0, c_p1) = (self[ln_i].p0, self[ln_i].p1);
                let cd = self[c_p1] - self[c_p0];
                let od = self[o_p1] - self[o_p0];

                // If current diagonal is shorter, then okay
                let c_l2 = cd.length();
                let o_l2 = od.length();
                if c_l2 <= o_l2 {
                    continue;
                }

                changed |= self
                    .quad_swap_diagonals_unless_it_makes_zero_area(ln_i, c_p0, c_p1, o_p0, o_p1);
            }
        }
        changed
    }

    //ap triangles
    pub fn triangles(
        &self,
    ) -> impl std::iter::Iterator<Item = (PointIndex, PointIndex, PointIndex)> + '_ {
        self.triangles.iter().map(|t| t.pts())
    }

    //ap points
    pub fn points(&self) -> impl std::iter::Iterator<Item = PointIndex> + '_ {
        (0..self.pxy.len()).map(|p| p.into())
    }

    //ap lines
    pub fn lines(&self) -> impl std::iter::Iterator<Item = LineIndex> + '_ {
        (0..self.lines.len()).map(|p| p.into())
    }

    //zz All done
}
