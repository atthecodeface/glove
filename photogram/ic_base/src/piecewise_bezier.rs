use bezier_nd::{BezierBuilder, BezierConstruct, BezierElevate, BezierEval};
use serde::{Deserialize, Serialize};

use crate::{Error, JsonParsable, Result};

/// A node in a Piecewise Bezier curve
///
/// As a leaf this is just a cubic Bezier; if not a leaf then it is instead a 'pivot' and a pair of deltas
///
/// If element[0] is not NAN then it is a leaf; otherwise all parameters < element[1] use the node of +element[2], > element[1] the node of +element[3]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct PiecewiseBezierNode {
    data: [[f64; 1]; 4],
}

impl PiecewiseBezierNode {
    /// Create a pivot node - i.e. one with element[0] as a NAN, element[1]
    /// being the pivot, element[2] and [3] being the delta indices for the
    /// lt/gt directions
    fn new_pivot_node(t: f64, lt: usize, gt: usize) -> Self {
        Self {
            data: [[f64::NAN; 1], [t; 1], [lt as f64; 1], [gt as f64; 1]],
        }
    }

    /// Set the link for a pivot node; this assumes element[0] is NAN
    fn set_link(&mut self, for_gt: bool, delta: usize) {
        if for_gt {
            self.data[3] = [delta as f64; 1];
        } else {
            self.data[3] = [delta as f64; 1];
        }
    }

    /// Create a constant Bezier node (i.e. a Bezier that has a constant value for all its argument values)
    fn constant(t: f64) -> Self {
        Self { data: [[t; 1]; 4] }
    }

    /// Create a linear Bezier node (i.e. a Bezier that is linear between xys[0] and xys[1])
    fn linear(xys: &[(f64, f64); 2]) -> Self {
        let dx = xys[1].0 - xys[0].0;
        let c0 = (xys[0].1 * xys[1].0 - xys[1].1 * xys[0].0) / dx;
        let c1 = (xys[0].1 * (xys[1].0 - 1.0) - xys[1].1 * (xys[0].0 - 1.0)) / dx;
        let b = [[c0], [c1]];
        let bq = b.elevate_by_one().unwrap();
        let bc = bq.elevate_by_one().unwrap();
        Self { data: bc }
    }

    /// Create a quadratic Bezier node
    fn quad(builder: &mut BezierBuilder<f64, 1>, xys: &[(f64, f64); 3]) -> Result<Self> {
        builder.clear();
        builder.add_point_at(xys[0].0, [xys[0].1; 1]);
        builder.add_point_at(xys[1].0, [xys[1].1; 1]);
        builder.add_point_at(xys[2].0, [xys[2].1; 1]);
        let bq = <[[f64; 1]; 3]>::of_builder(builder).map_err(|e| format!("{e:?}"))?;
        let bc = bq.elevate_by_one().unwrap();
        Ok(Self { data: bc })
    }

    /// Create a cubic Bezier node
    fn cubic(builder: &mut BezierBuilder<f64, 1>, xys: &[(f64, f64); 4]) -> Result<Self> {
        builder.clear();
        builder.add_point_at(xys[0].0, [xys[0].1; 1]);
        builder.add_point_at(xys[1].0, [xys[1].1; 1]);
        builder.add_point_at(xys[2].0, [xys[2].1; 1]);
        builder.add_point_at(xys[3].0, [xys[3].1; 1]);
        let bc = <[[f64; 1]; 4]>::of_builder(builder).map_err(|e| format!("{e:?}"))?;
        Ok(Self { data: bc })
    }

    /// Create a Bezier node of a function between two argument values, and return the max sq error
    fn of_fn<F>(builder: &mut BezierBuilder<f64, 1>, t0: f64, t3: f64, f: &F) -> Result<(Self, f64)>
    where
        F: Fn(f64) -> f64,
    {
        let t1 = (t0 * 2.0 + t3) / 3.0;
        let t2 = (t0 + t3 * 2.0) / 3.0;
        let s = Self::cubic(
            builder,
            &[(t0, f(t0)), (t1, f(t1)), (t2, f(t2)), (t3, f(t3))],
        )?;
        let mut max_error_sq = 0.0_f64;
        for i in 0..=100 {
            let t = (i as f64) / 100.0 * (t3 - t0) + t0;
            let delta = f(t) - s.data.point_at(t)[0];
            max_error_sq = max_error_sq.max(delta * delta);
        }
        Ok((s, max_error_sq))
    }

    /// Return tru if this is a pivot node - i.e. [1] is a pivot value for the parameter
    fn is_pivot_node(&self) -> bool {
        self.data[0][0].is_nan()
    }

    /// Evaluate the code, returning Ok(v) if this is a Bezier node, Err(delta)
    /// if a node branch should be taken
    fn evaluate(&self, t: f64) -> std::result::Result<f64, usize> {
        if self.is_pivot_node() {
            if t < self.data[1][0] {
                Err(self.data[2][0] as usize)
            } else {
                Err(self.data[3][0] as usize)
            }
        } else {
            Ok(self.data.point_at(t)[0])
        }
    }
}

/// A Piecewise Bezier curve, which consists of a Vec of nodes, with the root at [0]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiecewiseBezier {
    tree: Vec<PiecewiseBezierNode>,
}

impl PiecewiseBezier {
    /// Evaluate at a parameter value of t
    ///
    /// This will evaluate each node - if it indicates a branch in the tree,
    /// then that is followed and evaluated, etc
    pub fn evaluate(&self, t: f64) -> f64 {
        let mut node = 0;
        loop {
            match self.tree[node].evaluate(t) {
                Ok(r) => {
                    return r;
                }
                Err(n) => {
                    node += n;
                }
            }
        }
    }

    /// Validate the tree - ensure that all of the deltas map within the [PiecewiseBezier]
    pub fn validate(&self) -> Result<()> {
        for (i, n) in self.tree.iter().enumerate() {
            if let Err(next) = n.evaluate(0.0) {
                if next == 0 || i + next >= self.tree.len() {
                    return Err(format!("PiecewiseBezier has invalid node {i}").into());
                }
            }
        }
        Ok(())
    }

    /// Build function for test
    fn build(
        builder: &mut BezierBuilder<f64, 1>,
        mut tree: Vec<PiecewiseBezierNode>,
        xys: &[(f64, f64)],
    ) -> Result<Vec<PiecewiseBezierNode>> {
        match xys.len() {
            0 => {
                panic!("Should not have 0 xys to build a PiecewiseLinearPoly");
            }
            1 => {
                tree.push(PiecewiseBezierNode::constant(xys[0].1));
                Ok(tree)
            }
            2 => {
                tree.push(PiecewiseBezierNode::linear(&[xys[0], xys[1]]));
                Ok(tree)
            }
            3 => {
                tree.push(PiecewiseBezierNode::quad(
                    builder,
                    &[xys[0], xys[1], xys[2]],
                )?);
                Ok(tree)
            }
            4 => {
                tree.push(PiecewiseBezierNode::cubic(
                    builder,
                    &[xys[0], xys[1], xys[2], xys[3]],
                )?);
                Ok(tree)
            }
            n => {
                // n odd (such as 5) we want n/2 (0..=2, 2..=4)
                // n even (such as 4) then n/2 will do (0..=2, 2..=3)
                let this_node = tree.len();
                let middle = n / 2;
                let split_x = xys[middle].0;
                tree.push(PiecewiseBezierNode::new_pivot_node(split_x, 1, 0));
                let mut tree = Self::build(builder, tree, &xys[0..=middle])?;
                let skip_to_upper = tree.len() - this_node;
                tree[this_node].set_link(true, skip_to_upper);
                Self::build(builder, tree, &xys[middle..n])
            }
        }
    }

    /// Build a tree from a list of (x,y) values - used in testing
    pub fn of_xys(xys: &[(f64, f64)]) -> Result<Self> {
        let mut builder = BezierBuilder::default();
        let tree = Self::build(&mut builder, vec![], xys)?;
        Ok(Self { tree })
    }

    /// Build on to a tree (of Vec PiecewiseBezierNode), from a slice of (min_t, max_t, BezierForBetween)
    fn of_node_ranges(
        mut tree: Vec<PiecewiseBezierNode>,
        node_ranges: &[(f64, f64, PiecewiseBezierNode)],
    ) -> Vec<PiecewiseBezierNode> {
        let this_node = tree.len();
        match node_ranges.len() {
            0 => {
                panic!("Should not have 0 xys to build a PiecewiseLinearPoly");
            }
            1 => {
                tree.push(node_ranges[0].2);
                tree
            }
            n => {
                let middle = n / 2;
                let split_x = node_ranges[middle].0;
                tree.push(PiecewiseBezierNode::new_pivot_node(split_x, 1, 0));
                let mut tree = Self::of_node_ranges(tree, &node_ranges[0..middle]);
                let skip_to_upper = tree.len() - this_node;
                tree[this_node].set_link(true, skip_to_upper);
                Self::of_node_ranges(tree, &node_ranges[middle..n])
            }
        }
    }

    /// Build a PiecewiseBezier that between min_t and max_t matches a function with a maximum error
    ///
    /// Create a Vec of (min_t, max_t, BezierBetweenThem), where the Beziers are
    /// within the maximum error; then construct the tree from this array
    pub fn of_fn<F>(mut min_t: f64, max_t: f64, f: &F, max_err: f64) -> Result<Self>
    where
        F: Fn(f64) -> f64,
    {
        let mut builder = BezierBuilder::default();
        let mut node_ranges = vec![];
        while min_t < max_t {
            let mut last_t = max_t;
            loop {
                let (node, error_sq) = PiecewiseBezierNode::of_fn(&mut builder, min_t, last_t, f)?;
                if error_sq < max_err * max_err {
                    node_ranges.push((min_t, last_t, node));
                    min_t = last_t;
                    break;
                }
                last_t = (min_t + last_t) / 2.0;
            }
        }
        let tree = Self::of_node_ranges(vec![], &node_ranges);
        Ok(Self { tree })
    }
}

#[test]
fn test_piecewise() -> Result<()> {
    let p = PiecewiseBezier::of_xys(&[(0., 0.), (1., 2.0)])?;
    for i in 0..10 {
        let t = (i as f64);
        eprintln!("{i} {}", p.evaluate(t));
    }

    let p = PiecewiseBezier::of_xys(&[(0., 0.), (1., 2.0), (2., 8.)])?;
    for i in 0..10 {
        let t = (i as f64);
        eprintln!("{i} {}", p.evaluate(t));
    }

    let mut d = vec![];
    for i in 0..50 {
        let t = (i as f64);
        let v = t.to_radians().tan();
        d.push((t, v));
    }
    let p = PiecewiseBezier::of_xys(&d)?;
    for i in 0..50 {
        let t = (i as f64);
        eprintln!("{i} {} {}", p.evaluate(t), t.to_radians().tan());
    }
    eprintln!("{p:?}");
    // assert!(false, "Force fail");
    Ok(())
}

#[test]
fn test_piecewise_fn() -> Result<()> {
    let p = PiecewiseBezier::of_fn(0.0, 1.4, &f64::tan, 1E-4)?;
    for i in 0..40 {
        let i = i * 2;
        let t = (i as f64).to_radians();
        eprintln!("{i} {} {}", p.evaluate(t), t.tan());
    }
    eprintln!("{p:?}");
    // assert!(false, "Force fail");
    Ok(())
}
