//a Imports
use std::cmp::Ordering;
use std::collections::HashMap;
use std::default::Default;

use geo_nd::Vector;
use serde::{Deserialize, Serialize};

use crate::{Point2D, PointMappingSet};

//a PointIndex
//tp PointIndex
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct PointIndex {
    #[serde(flatten)]
    p: usize,
}

//ip From<usize> for PointIndex
impl From<usize> for PointIndex {
    fn from(p: usize) -> PointIndex {
        PointIndex { p }
    }
}

//ip Display for PointIndex
impl std::fmt::Display for PointIndex {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "Pt({})", self.p)
    }
}

//ip PartialEq<usize> for PointIndex
impl std::cmp::PartialEq<usize> for PointIndex {
    fn eq(&self, p: &usize) -> bool {
        self.p == *p
    }
}

//ip PartialEq<&usize> for PointIndex
impl std::cmp::PartialEq<&usize> for PointIndex {
    fn eq(&self, p: &&usize) -> bool {
        self.p == **p
    }
}

//a LineIndex
//tp LineIndex
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct LineIndex {
    #[serde(flatten)]
    l: usize,
}

//ip From<usize> for LineIndex {
impl From<usize> for LineIndex {
    fn from(l: usize) -> LineIndex {
        LineIndex { l }
    }
}

//ip std::fmt::Display for LineIndex {
impl std::fmt::Display for LineIndex {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "Ln({})", self.l)
    }
}

//ip PartialEq<usize> for LineIndex
impl std::cmp::PartialEq<usize> for LineIndex {
    fn eq(&self, l: &usize) -> bool {
        self.l == *l
    }
}

//ip PartialEq<&usize> for LineIndex
impl std::cmp::PartialEq<&usize> for LineIndex {
    fn eq(&self, l: &&usize) -> bool {
        self.l == **l
    }
}

//a TriangleIndex
//tp TriangleIndex
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct TriangleIndex {
    #[serde(flatten)]
    t: usize,
}

//ip From<usize> for TriangleIndex
impl From<usize> for TriangleIndex {
    fn from(t: usize) -> TriangleIndex {
        TriangleIndex { t }
    }
}

//ip Display for TriangleIndex
impl std::fmt::Display for TriangleIndex {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "Tr({})", self.t)
    }
}

//ip PartialEq<usize> for TriangleIndex
impl std::cmp::PartialEq<usize> for TriangleIndex {
    fn eq(&self, t: &usize) -> bool {
        self.t == *t
    }
}

//ip PartialEq<&usize> for TriangleIndex
impl std::cmp::PartialEq<&usize> for TriangleIndex {
    fn eq(&self, t: &&usize) -> bool {
        self.t == **t
    }
}

//a IndexLine and IndexTriangle
//tp IndexLine
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IndexLine {
    /// Two points such that t0 is on the left of the line
    ///
    /// t1 MAY not exist, but it would be on the right of the line
    p0: PointIndex,
    p1: PointIndex,
    /// Triangle that this line is definitely part of
    t0: TriangleIndex,
    /// Second triangle this is part of (to the right of the line); if
    /// this equals t0 then there is no second triangle
    t1: TriangleIndex,
}

//ip Display for IndexLine
impl std::fmt::Display for IndexLine {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "[{},{}]:L({}):R({})",
            self.p0, self.p1, self.t0, self.t1
        )
    }
}

//ip IndexLine
impl IndexLine {
    //mi opposite_diagonal
    fn opposite_diagonal(&self, triangles: &[IndexTriangle]) -> Option<(PointIndex, PointIndex)> {
        if self.t0 == self.t1 {
            return None;
        }
        let p0 = triangles[self.t0.t].other_pt(self.p0, self.p1);
        let p1 = triangles[self.t1.t].other_pt(self.p0, self.p1);
        Some((p0, p1))
    }

    //mi change_pt
    /// Change one point to be a different one
    fn change_pt(&mut self, old_p: PointIndex, new_p: PointIndex) {
        if self.p0 == old_p {
            self.p0 = new_p;
        } else {
            assert_eq!(self.p1, old_p);
            self.p1 = new_p;
        }
    }

    //mi change_triangle
    /// Change one triangle to be a different one
    fn change_triangle(&mut self, old_t: TriangleIndex, new_t: TriangleIndex) {
        if self.t0 == old_t {
            self.t0 = new_t;
            if self.t1 == old_t {
                self.t1 = self.t0;
            }
        } else {
            assert_eq!(self.t1, old_t);
            self.t1 = new_t;
        }
    }

    //ap pts
    pub fn pts(&self) -> (PointIndex, PointIndex) {
        (self.p0, self.p1)
    }

    //ap contains_pt
    pub fn contains_pt<I: Into<PointIndex>>(&self, p: I) -> bool {
        let p: PointIndex = p.into();
        self.p0 == p || self.p1 == p
    }

    //zz All done
}

//tp IndexTriangle
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IndexTriangle {
    /// Three points A,B,C in anticlockwise order
    p0: PointIndex,
    p1: PointIndex,
    p2: PointIndex,
    /// Three lines, AB, BC, CA
    l0: LineIndex,
    l1: LineIndex,
    l2: LineIndex,
}

//ip Display for IndexTriangle
impl std::fmt::Display for IndexTriangle {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            fmt,
            "[{},{},{}]:({}->{}->{}->)",
            self.p0, self.p1, self.p2, self.l0, self.l1, self.l2,
        )
    }
}

//ip IndexTriangle
impl IndexTriangle {
    fn other_pt(&self, p0: PointIndex, p1: PointIndex) -> PointIndex {
        if p0 != self.p0 && p1 != self.p0 {
            self.p0
        } else if p0 != self.p1 && p1 != self.p1 {
            self.p1
        } else {
            assert!(p0 != self.p2 && p1 != self.p2);
            self.p2
        }
    }
    fn line_from_pt(&self, p: PointIndex) -> LineIndex {
        if p == self.p0 {
            self.l0
        } else if p == self.p1 {
            self.l1
        } else {
            assert!(p == self.p2);
            self.l2
        }
    }
    fn change_pt(&mut self, old_p: PointIndex, new_p: PointIndex) {
        if self.p0 == old_p {
            self.p0 = new_p;
        } else if self.p1 == old_p {
            self.p1 = new_p;
        } else {
            assert_eq!(self.p2, old_p);
            self.p2 = new_p;
        }
    }
    fn change_ln(&mut self, old_l: LineIndex, new_l: LineIndex) {
        if self.l0 == old_l {
            self.l0 = new_l;
        } else if self.l1 == old_l {
            self.l1 = new_l;
        } else {
            assert_eq!(self.l2, old_l);
            self.l2 = new_l;
        }
    }

    //ap pts
    pub fn pts(&self) -> (PointIndex, PointIndex, PointIndex) {
        (self.p0, self.p1, self.p2)
    }

    //ap lines
    pub fn lines(&self) -> (LineIndex, LineIndex, LineIndex) {
        (self.l0, self.l1, self.l2)
    }

    //ap contains_pt
    pub fn contains_pt<I: Into<PointIndex>>(&self, p: I) -> bool {
        let p: PointIndex = p.into();
        self.p0 == p || self.p1 == p || self.p2 == p
    }
}

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
        &self.pxy[index.p]
    }
}

//ip IndexMut<PointIndex>
impl std::ops::IndexMut<PointIndex> for Mesh {
    fn index_mut(&mut self, index: PointIndex) -> &mut Self::Output {
        &mut self.pxy[index.p]
    }
}

//ip Index<LineIndex>
impl std::ops::Index<LineIndex> for Mesh {
    type Output = IndexLine;
    fn index(&self, index: LineIndex) -> &Self::Output {
        &self.lines[index.l]
    }
}

//ip IndexMut<LineIndex>
impl std::ops::IndexMut<LineIndex> for Mesh {
    fn index_mut(&mut self, index: LineIndex) -> &mut Self::Output {
        &mut self.lines[index.l]
    }
}

//ip Index<TriangleIndex>
impl std::ops::Index<TriangleIndex> for Mesh {
    type Output = IndexTriangle;
    fn index(&self, index: TriangleIndex) -> &Self::Output {
        &self.triangles[index.t]
    }
}

//ip IndexMut<TriangleIndex>
impl std::ops::IndexMut<TriangleIndex> for Mesh {
    fn index_mut(&mut self, index: TriangleIndex) -> &mut Self::Output {
        &mut self.triangles[index.t]
    }
}

//ip Mesh
impl Mesh {
    //cp new
    pub fn new(pms: &PointMappingSet) -> Self {
        let pxy = pms.mappings().iter().map(|p| p.screen()).collect();
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

    //mp find_lbx_pt
    pub fn find_lbx_pt(&self) -> PointIndex {
        let (n, _lbx) = self
            .pxy
            .iter()
            .enumerate()
            .fold((0, self.pxy[0]), |(n, lbx), (i, pt)| {
                if pt[0] < lbx[0] {
                    (i, *pt)
                } else if pt[0] == lbx[0] && pt[1] < lbx[1] {
                    (i, *pt)
                } else {
                    (n, lbx)
                }
            });
        n.into()
    }

    //fp concave_cmp
    pub fn concave_cmp(p0: Point2D, p1: Point2D) -> Ordering {
        if p0[0] == 0. && p0[1] == 0. {
            Ordering::Less
        } else if p1[0] == 0. && p1[1] == 0. {
            Ordering::Greater
        } else if p0[0] == p1[0] {
            p0[1].partial_cmp(&p1[1]).unwrap()
        } else {
            (p0[1] * p1[0]).partial_cmp(&(p0[0] * p1[1])).unwrap()
        }
    }

    //mp find_sweep
    pub fn find_sweep(&self) -> (PointIndex, Vec<PointIndex>) {
        let lbx_n = self.find_lbx_pt();
        let lbx = self[lbx_n];
        fn compare_pts(a: &(PointIndex, Point2D), b: &(PointIndex, Point2D)) -> Ordering {
            Mesh::concave_cmp(a.1, b.1)
        }
        let mut n_dxys: Vec<(PointIndex, Point2D)> = self
            .pxy
            .iter()
            .enumerate()
            .map(|(n, p)| (n.into(), (*p) - lbx))
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
            self.lines[ln.l].t1 = t0;
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
        self.triangles.push(IndexTriangle {
            p0,
            p1,
            p2,
            l0,
            l1,
            l2,
        });
        tn
    }

    //mp create_mesh_triangles
    pub fn create_mesh_triangles(&mut self) {
        self.clear();
        let (lbx, sweep) = self.find_sweep();
        let num_triangles = sweep.len() - 1;
        for n in 1..num_triangles {
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
            eprintln!("Check {ph0}, {ph1}, {ph2} ({p0}, {p1}, {p2})");
            let pt0 = self[ph0];
            let pt1 = self[ph1];
            let pt2 = self[ph2];
            let d01 = pt0 - pt1;
            let d21 = pt2 - pt1;
            if d01[0] * d21[1] > d01[1] * d21[0] {
                self.add_triangle(ph0, ph2, ph1);
                eprintln!("Add {ph0}, {ph1}, {ph2}");
                hull[p1] = hull[p0];
                while hull[p0] == hull[p1] {
                    if p0 == 0 {
                        p1 = p1 + 1;
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

    //mp quad_swap_diagonals
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
    fn quad_swap_diagonals(
        &mut self,
        ln_i: LineIndex,
        c_p0: PointIndex,
        c_p1: PointIndex,
        o_p0: PointIndex,
        o_p1: PointIndex,
    ) {
        eprintln!("Swap quad diagonals {ln_i} {c_p0} {c_p1} {o_p0} {o_p1}");
        let t0 = self[ln_i].t0;
        let t1 = self[ln_i].t1;
        let ln_b = self[t0].line_from_pt(o_p0);
        let ln_d = self[t1].line_from_pt(o_p1);
        eprintln!(" {t0} {t1} {ln_b} {ln_d}");

        eprintln!(" T0: {}", self[t0]);
        eprintln!(" T1: {}", self[t1]);
        eprintln!(" L_i: {}", self[ln_i]);
        eprintln!(" L_b: {}", self[ln_b]);
        eprintln!(" L_d: {}", self[ln_d]);
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

        eprintln!("Afterwards:");
        eprintln!(" T0: {}", self[t0]);
        eprintln!(" T1: {}", self[t1]);
        eprintln!(" L_i: {}", self[ln_i]);
        eprintln!(" L_b: {}", self[ln_b]);
        eprintln!(" L_d: {}", self[ln_d]);
    }

    //mp optimize_mesh_quads
    pub fn optimize_mesh_quads(&mut self) -> bool {
        let mut changed = false;
        for i in 0..self.lines.len() {
            let ln_i: LineIndex = i.into();
            if let Some((o_p0, o_p1)) = self[ln_i].opposite_diagonal(&self.triangles) {
                let (c_p0, c_p1) = (self[ln_i].p0, self[ln_i].p1);
                let cd = self[c_p1] - self[c_p0];
                let od = self[o_p1] - self[o_p0];
                let cn = [cd[1], -cd[0]].into();
                let on = [od[1], -od[0]].into();
                let c_p0_side_of_od = (self[c_p0] - self[o_p0]).dot(&on);
                let c_p1_side_of_od = (self[c_p1] - self[o_p0]).dot(&on);
                let o_p0_side_of_od = (self[o_p0] - self[c_p0]).dot(&cn);
                let o_p1_side_of_od = (self[o_p1] - self[c_p0]).dot(&cn);
                let c_l2 = cd.length();
                let o_l2 = od.length();
                // Note that > 0. should allow empty triangles with
                // c_p0 on the line o_p0/o_p1 to be optimized
                //
                // Except for floating point...
                if c_p0_side_of_od * c_p1_side_of_od > 0. {
                    continue;
                }
                if o_p0_side_of_od * o_p1_side_of_od > 0. {
                    continue;
                }
                if c_l2 > o_l2 {
                    self.quad_swap_diagonals(ln_i, c_p0, c_p1, o_p0, o_p1);
                    changed = true;
                }
            }
        }
        changed
    }

    //zz All done
}

//a Tests
//fi assert_triangle
#[cfg(test)]
fn assert_triangle(t: &IndexTriangle, (p0, p1, p2): (usize, usize, usize)) {
    eprintln!("Check triangle {t} includes {p0} {p1} {p2}");
    assert!(t.contains_pt(p0));
    assert!(t.contains_pt(p1));
    assert!(t.contains_pt(p2));
}

//fi assert_mesh_triangle
#[cfg(test)]
fn assert_mesh_triangle(mesh: &Mesh, t: usize, p012: (usize, usize, usize)) {
    let t: TriangleIndex = t.into();
    assert_triangle(&mesh[t], p012);
}

//ft test_sweep
#[test]
fn test_sweep() -> Result<(), String> {
    let mut mesh = Mesh::default();
    mesh.add_pt([1., 0.].into());
    mesh.add_pt([0., 0.].into());
    mesh.add_pt([0., 1.].into());
    mesh.add_pt([1., 1.].into());
    assert_eq!(mesh.find_lbx_pt(), 1);
    let sweep = mesh.find_sweep();
    assert_eq!(&sweep.1, &[1, 0, 3, 2]);
    Ok(())
}

//ft test_sweep2
#[test]
fn test_sweep2() -> Result<(), String> {
    let mut mesh = Mesh::default();
    mesh.add_pt([1., 0.].into());
    mesh.add_pt([0., 0.].into());
    mesh.add_pt([0., 1.].into());
    mesh.add_pt([1., 1.].into());
    mesh.add_pt([2., 0.].into());
    mesh.add_pt([2., -1.].into());
    mesh.add_pt([0., 3.].into());
    mesh.add_pt([1., 4.].into());
    assert_eq!(mesh.find_lbx_pt(), 1);
    let sweep = mesh.find_sweep();
    assert_eq!(&sweep.1, &[1, 5, 0, 4, 3, 7, 2, 6]);
    let lbx = mesh[sweep.1[0]];
    for i in 1..(sweep.1.len() - 1) {
        let p0 = mesh[sweep.1[i]];
        let p1 = mesh[sweep.1[i + 1]];
        let p0 = p0 - lbx;
        let p1 = p1 - lbx;
        assert!(p0[0].atan2(p0[1]) >= p1[0].atan2(p1[1]));
    }
    Ok(())
}

//ft test_hull
/// Just do a square - since the diagonals are equal, cannot say which
/// diagonal is chosen
///
/// This just does test that it runs
#[test]
fn test_hull() -> Result<(), String> {
    let mut mesh = Mesh::default();
    mesh.add_pt([1., 0.].into());
    mesh.add_pt([0., 0.].into());
    mesh.add_pt([0., 1.].into());
    mesh.add_pt([1., 1.].into());
    mesh.create_mesh_triangles();
    mesh.optimize_mesh_quads();
    Ok(())
}

//ft test_hull1
/// A diamond, and the lines should be swapped.
///
/// There must end up being a triangle (0,1,3) and another (1,2,3)
#[test]
fn test_hull1() -> Result<(), String> {
    let mut mesh = Mesh::default();
    mesh.add_pt([0., 0.].into());
    mesh.add_pt([1., 1.].into());
    mesh.add_pt([1., 2.].into());
    mesh.add_pt([0., 1.].into());
    mesh.create_mesh_triangles();

    while (mesh.optimize_mesh_quads()) {}

    assert_mesh_triangle(&mesh, 0, (0, 1, 3));
    assert_mesh_triangle(&mesh, 1, (1, 2, 3));

    Ok(())
}

//ft test_hull2
#[test]
fn test_hull2() -> Result<(), String> {
    let mut mesh = Mesh::default();
    mesh.add_pt([0., 0.].into());
    mesh.add_pt([1., 0.].into());
    mesh.add_pt([1., 1.].into());
    mesh.add_pt([0.3, 0.4].into());
    mesh.add_pt([0., 1.].into());
    mesh.create_mesh_triangles();

    while (mesh.optimize_mesh_quads()) {}

    assert_mesh_triangle(&mesh, 0, (0, 1, 3));
    assert_mesh_triangle(&mesh, 1, (1, 2, 3));
    assert_mesh_triangle(&mesh, 2, (0, 3, 4));
    assert_mesh_triangle(&mesh, 3, (2, 3, 4));

    Ok(())
}

//ft test_hull3
#[test]
fn test_hull3() -> Result<(), String> {
    let mut mesh = Mesh::default();
    mesh.add_pt([0., 0.].into());
    mesh.add_pt([10., 0.].into());
    mesh.add_pt([0., 10.].into());
    mesh.add_pt([5.03, 2.].into()); // to make it consistent
    mesh.add_pt([2., 5.].into());
    mesh.add_pt([5., 8.].into());
    mesh.add_pt([8.01, 5.].into()); // to make it consistent
    mesh.add_pt([10., 10.].into());
    mesh.create_mesh_triangles();

    assert_mesh_triangle(&mesh, 0, (0, 1, 3));
    assert_mesh_triangle(&mesh, 1, (0, 3, 6));
    assert_mesh_triangle(&mesh, 2, (0, 6, 7));
    assert_mesh_triangle(&mesh, 3, (0, 7, 5));
    assert_mesh_triangle(&mesh, 4, (0, 5, 4));
    assert_mesh_triangle(&mesh, 5, (0, 4, 2));
    assert_mesh_triangle(&mesh, 6, (1, 3, 6));
    assert_mesh_triangle(&mesh, 7, (1, 6, 7));
    assert_mesh_triangle(&mesh, 8, (5, 2, 4));
    assert_mesh_triangle(&mesh, 9, (7, 5, 2));

    while (mesh.optimize_mesh_quads()) {}

    assert_mesh_triangle(&mesh, 0, (0, 1, 3));
    assert_mesh_triangle(&mesh, 1, (0, 3, 4));
    assert_mesh_triangle(&mesh, 2, (3, 6, 5));
    assert_mesh_triangle(&mesh, 3, (6, 7, 5));
    assert_mesh_triangle(&mesh, 4, (3, 5, 4));
    assert_mesh_triangle(&mesh, 5, (0, 4, 2));
    assert_mesh_triangle(&mesh, 6, (1, 3, 6));
    assert_mesh_triangle(&mesh, 7, (1, 6, 7));
    assert_mesh_triangle(&mesh, 8, (5, 2, 4));
    assert_mesh_triangle(&mesh, 9, (7, 5, 2));

    Ok(())
}
