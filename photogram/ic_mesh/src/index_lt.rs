//a Imports
use std::default::Default;

use serde::{Deserialize, Serialize};

use crate::{LineIndex, PointIndex, TriangleIndex};

//a IndexLine
//tp IndexLine
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IndexLine {
    /// Two points such that t0 is on the left of the line
    ///
    /// t1 MAY not exist, but it would be on the right of the line
    pub p0: PointIndex,
    pub p1: PointIndex,
    /// Triangle that this line is definitely part of
    pub t0: TriangleIndex,
    /// Second triangle this is part of (to the right of the line); if
    /// this equals t0 then there is no second triangle
    pub t1: TriangleIndex,
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
    //ap contains_pt
    pub fn contains_pt<I: Into<PointIndex>>(&self, p: I) -> bool {
        let p: PointIndex = p.into();
        self.p0 == p || self.p1 == p
    }

    //ap has_one_triangle
    pub fn has_one_triangle(&self) -> bool {
        self.t0 == self.t1
    }

    //ap t0_is_on_left
    pub fn t0_is_on_left(&self, triangles: &[IndexTriangle]) -> bool {
        let t0 = &triangles[self.t0.as_usize()];
        let (tp0, tp1, tp2) = t0.pts();
        (tp0 == self.p0 && tp1 == self.p1)
            || (tp1 == self.p0 && tp2 == self.p1)
            || (tp2 == self.p0 && tp0 == self.p1)
    }

    //mi opposite_diagonal
    pub fn opposite_diagonal(
        &self,
        triangles: &[IndexTriangle],
    ) -> Option<(PointIndex, PointIndex)> {
        if self.t0 == self.t1 {
            return None;
        }
        let p0 = triangles[self.t0.as_usize()].other_pt(self.p0, self.p1);
        let p1 = triangles[self.t1.as_usize()].other_pt(self.p0, self.p1);
        Some((p0, p1))
    }

    //mi change_pt
    /// Change one point to be a different one
    pub fn change_pt(&mut self, old_p: PointIndex, new_p: PointIndex) {
        if self.p0 == old_p {
            self.p0 = new_p;
        } else {
            assert_eq!(self.p1, old_p);
            self.p1 = new_p;
        }
    }

    //mp change_triangle
    /// Change one triangle to be a different one
    pub fn change_triangle(&mut self, old_t: TriangleIndex, new_t: TriangleIndex) {
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

    //ap triangles
    pub fn triangles(&self) -> (TriangleIndex, TriangleIndex) {
        (self.t0, self.t1)
    }

    //zz All done
}

//a IndexTriangle
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
    //cp new
    pub fn new(
        p0: PointIndex,
        p1: PointIndex,
        p2: PointIndex,
        l0: LineIndex,
        l1: LineIndex,
        l2: LineIndex,
    ) -> Self {
        assert!(
            !(p0 == p1 || p0 == p2 || p1 == p2),
            "Should not make a triangle with two identical points - was given {p0}, {p1}, {p2}"
        );

        Self {
            p0,
            p1,
            p2,
            l0,
            l1,
            l2,
        }
    }

    //mp other_pt
    pub fn other_pt(&self, p0: PointIndex, p1: PointIndex) -> PointIndex {
        // If neither point is self.p0 then the third point is that
        //
        // else if neither point is self.p1 then the third point is that
        //
        // else neither point must equal self.p2
        //
        // assert if p0 /p1 are the same as all the points - which
        // implies that two of the points on this triangle are the
        // same
        if p0 != self.p0 && p1 != self.p0 {
            self.p0
        } else if p0 != self.p1 && p1 != self.p1 {
            self.p1
        } else {
            assert!(p0 != self.p2 && p1 != self.p2, "Attempt to find other point in {self:?} from points {p0} and {p1}, so the triangle must have two identical points which is a bug" );
            self.p2
        }
    }

    //mp line_from_pt
    pub fn line_from_pt(&self, p: PointIndex) -> LineIndex {
        if p == self.p0 {
            self.l0
        } else if p == self.p1 {
            self.l1
        } else {
            assert!(p == self.p2);
            self.l2
        }
    }

    //mp change_pt
    #[track_caller]
    pub fn change_pt(&mut self, old_p: PointIndex, new_p: PointIndex) {
        assert!(
            new_p != self.p0 && new_p != self.p1 && new_p != self.p2,
            "Attempt to change the point of a triangle to one of its current points"
        );
        if self.p0 == old_p {
            self.p0 = new_p;
        } else if self.p1 == old_p {
            self.p1 = new_p;
        } else {
            assert_eq!(self.p2, old_p);
            self.p2 = new_p;
        }
    }
    pub fn change_ln(&mut self, old_l: LineIndex, new_l: LineIndex) {
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
    /// Return the points A, B, C of the triangle (in that order)
    pub fn pts(&self) -> (PointIndex, PointIndex, PointIndex) {
        (self.p0, self.p1, self.p2)
    }

    //ap lines
    /// Return the lines AB, BC, CA of the triangle (in that order)
    pub fn lines(&self) -> (LineIndex, LineIndex, LineIndex) {
        (self.l0, self.l1, self.l2)
    }

    //ap contains_pt
    pub fn contains_pt<I: Into<PointIndex>>(&self, p: I) -> bool {
        let p: PointIndex = p.into();
        self.p0 == p || self.p1 == p || self.p2 == p
    }
}
