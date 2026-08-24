use bezier_nd::{BezierBuilder, BezierConstruct, BezierElevate, BezierEval};
use serde::{Deserialize, Serialize};

use crate::Result;

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
    /// For creating from constants use only, really
    fn of_f64s(f64s: &[f64; 4]) -> Self {
        let d0 = [f64s[0]];
        let d1 = [f64s[1]];
        let d2 = [f64s[2]];
        let d3 = [f64s[3]];
        Self {
            data: [d0, d1, d2, d3],
        }
    }

    /// For deserializing use only, really
    fn of_opt_f64s(f64s: &[Option<f64>; 4]) -> Self {
        let mk_data = |f: Option<f64>| [f.unwrap_or(f64::NAN)];
        let d0 = mk_data(f64s[0]);
        let d1 = mk_data(f64s[1]);
        let d2 = mk_data(f64s[2]);
        let d3 = mk_data(f64s[3]);
        Self {
            data: [d0, d1, d2, d3],
        }
    }

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
            self.data[2] = [delta as f64; 1];
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
    fn of_fn<F>(
        builder: &mut BezierBuilder<f64, 1>,
        t0: f64,
        t3: f64,
        f: &F,
        steps_per_bezier: usize,
    ) -> Result<(Self, f64)>
    where
        F: Fn(f64) -> f64,
    {
        let t1 = (t0 * 2.0 + t3) / 3.0;
        let t2 = (t0 + t3 * 2.0) / 3.0;
        let pts = [(t0, f(t0)), (t1, f(t1)), (t2, f(t2)), (t3, f(t3))];
        let s = Self::cubic(builder, &pts)?;
        let mut max_error_sq = 0.0_f64;
        for i in 0..=steps_per_bezier {
            let t = (i as f64) / (steps_per_bezier as f64) * (t3 - t0) + t0;
            let delta = f(t) - s.data.point_at(t)[0];
            max_error_sq = max_error_sq.max(delta * delta);
        }
        Ok((s, max_error_sq))
    }

    /// Return tru if this is a pivot node - i.e. [1] is a pivot value for the parameter
    fn is_pivot_node(&self) -> bool {
        self.data[0][0].is_nan()
    }

    /// Get the pivot_value for a pivot node
    fn pivot_value(&self) -> Option<f64> {
        if self.is_pivot_node() {
            Some(self.data[1][0])
        } else {
            None
        }
    }

    /// Get a link for a pivot node
    fn link(&self, for_gt: bool) -> Option<usize> {
        if self.is_pivot_node() {
            if for_gt {
                Some(self.data[3][0] as usize)
            } else {
                Some(self.data[2][0] as usize)
            }
        } else {
            None
        }
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

#[derive(Debug, Clone)]
struct PBIter<'a> {
    pb: &'a PiecewiseBezier,
    /// True if the iteration has been started
    started: bool,
    /// Minimum value
    min_t: f64,
    /// Maximum value
    max_t: f64,
    /// Stack of node ids (which should only be pivoy nodes), with the one to handle next at the top.
    /// If this is empty and !completed then the *first* node should be handled (assuming there is one...)
    ///
    /// If this is a pivot node, then follow its gt branch; the lt branch will already have been handled
    stack: Vec<(usize, f64)>,
}

impl<'a> std::iter::Iterator for PBIter<'a> {
    type Item = (f64, f64, PiecewiseBezierNode);
    fn next(&mut self) -> Option<Self::Item> {
        let mut node = 0;
        let mut min_t = self.min_t;
        let mut max_t = self.max_t;
        if !self.started {
            self.started = true;
            if self.pb.tree.is_empty() {
                return None;
            }
        } else {
            let Some((n, m)) = self.stack.pop() else {
                return None;
            };
            let x = &self.pb.tree[n];
            node = n + x.link(true).unwrap();
            min_t = x.pivot_value().unwrap();
            max_t = m;
        }
        // While node is a pivot node then push it and follow its lt branch
        while self.pb.tree[node].is_pivot_node() {
            let pivot = self.pb.tree[node].pivot_value().unwrap();
            self.stack.push((node, max_t));
            node += self.pb.tree[node].link(false).unwrap();
            max_t = pivot;
        }
        // node must be a leaf node; return it!
        Some((min_t, max_t, self.pb.tree[node].clone()))
    }
}

/// A Piecewise Bezier curve, which consists of a Vec of nodes, with the root at [0]
#[derive(Debug, Clone)]
pub struct PiecewiseBezier {
    tree: Vec<PiecewiseBezierNode>,
}

impl serde::Serialize for PiecewiseBezier {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v = self.as_opt_f64s();
        v.serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for PiecewiseBezier {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = Vec::<Option<f64>>::deserialize(deserializer)?;
        let s = PiecewiseBezier::of_opt_f64s(&v)
            .map_err(|e| serde::de::Error::custom(format!("invalid PiecewiseBezier: {e}")))?;
        Ok(s)
    }
}

/// Default is a pure linear Bezier
impl std::default::Default for PiecewiseBezier {
    fn default() -> Self {
        Self {
            tree: vec![PiecewiseBezierNode::of_f64s(&[
                0.0,
                0.3333333333333334,
                0.6666666666666665,
                1.0,
            ])],
        }
    }
}

/// Display for humans
impl std::fmt::Display for PiecewiseBezier {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "PiecewiseBezier{{")?;
        for (min_t, max_t, pbn) in self.iter() {
            write!(
                fmt,
                "({min_t:0.4}->{max_t:0.4}:[{:0.4},{:0.4},{:0.4},{:0.4}], ",
                pbn.data[0][0], pbn.data[1][0], pbn.data[2][0], pbn.data[3][0],
            )?;
        }
        write!(fmt, "}}")
    }
}

impl PiecewiseBezier {
    /// Get an iterator over all the nodes in order
    fn iter<'a>(&'a self) -> PBIter<'a> {
        PBIter {
            pb: self,
            started: false,
            stack: vec![],
            min_t: f64::NAN,
            max_t: f64::NAN,
        }
    }

    /// For serialization use only, really; export the nodes
    pub fn as_opt_f64s(&self) -> Vec<Option<f64>> {
        let mut result = vec![];
        for n in self.tree.iter() {
            for d in n.data.iter() {
                if d[0].is_nan() {
                    result.push(None)
                } else {
                    result.push(Some(d[0]))
                }
            }
        }
        result
    }

    /// For deserialization use only, really
    pub fn of_opt_f64s(f64s: &[Option<f64>]) -> Result<Self> {
        let mut tree = vec![];
        for n in f64s.as_chunks::<4>().0 {
            tree.push(PiecewiseBezierNode::of_opt_f64s(n));
        }
        let s = Self { tree };
        s.validate()?;
        Ok(s)
    }

    /// For constants use only, really; export the nodes
    pub fn as_f64s(&self) -> Vec<f64> {
        let mut result = vec![];
        for n in self.tree.iter() {
            result.extend_from_slice(&[n.data[0][0], n.data[1][0], n.data[2][0], n.data[3][0]]);
        }
        result
    }

    /// For creating from constants use only, really
    pub fn of_f64s(f64s: &[f64]) -> Result<Self> {
        let mut tree = vec![];
        for n in f64s.as_chunks::<4>().0 {
            tree.push(PiecewiseBezierNode::of_f64s(n));
        }
        let s = Self { tree };
        s.validate()?;
        Ok(s)
    }

    /// Build on to a tree (of Vec PiecewiseBezierNode), from a slice of (min_t,
    /// max_t, BezierForBetween)
    ///
    /// This is used recursively to create trees, and can be invoked with
    /// (vec![], nodes) to create a complete PiecewiseBezier
    fn tree_of_node_ranges(
        mut tree: Vec<PiecewiseBezierNode>,
        node_ranges: &[(f64, PiecewiseBezierNode)],
    ) -> Vec<PiecewiseBezierNode> {
        let this_node = tree.len();
        match node_ranges.len() {
            0 => {
                panic!("Should not have 0 xys to build a PiecewiseLinearPoly");
            }
            1 => {
                tree.push(node_ranges[0].1);
                tree
            }
            n => {
                let middle = n / 2;
                let split_x = node_ranges[middle].0;
                tree.push(PiecewiseBezierNode::new_pivot_node(split_x, 1, 0));
                let mut tree = Self::tree_of_node_ranges(tree, &node_ranges[0..middle]);
                let skip_to_upper = tree.len() - this_node;
                tree[this_node].set_link(true, skip_to_upper);
                Self::tree_of_node_ranges(tree, &node_ranges[middle..n])
            }
        }
    }

    /// Create a Piecewise Bezier of node ranges
    fn of_node_ranges(node_ranges: Vec<(f64, PiecewiseBezierNode)>) -> Result<Self> {
        let tree = Self::tree_of_node_ranges(vec![], &node_ranges);
        Ok(Self { tree })
    }

    /// Iterate over the tree as Beziers as (min, max, 1D bezier)
    pub fn iter_beziers<'a>(&'a self) -> impl Iterator<Item = (f64, f64, [[f64; 1]; 4])> + 'a {
        self.iter().map(|(min, max, pb)| (min, max, pb.data))
    }

    /// Build a PiecewiseBezier from a list of (min, max, 1D bezier)
    ///
    /// In theory min of n+1 should be max of n; however, only the min are used,
    /// and the max need not be supplied.
    pub fn of_beziers<I: Iterator<Item = (f64, f64, [[f64; 1]; 4])>>(iter: I) -> Result<Self> {
        let mut node_ranges = vec![];
        for (i, (min, _max, b)) in iter.enumerate() {
            node_ranges.push((
                min,
                PiecewiseBezierNode::of_f64s(&[b[0][0], b[1][0], b[2][0], b[3][0]]),
            ));
        }
        Self::of_node_ranges(node_ranges)
    }

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
        if self.tree.is_empty() {
            return Err(format!("PiecewiseBezier has no nodes!").into());
        }
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
    pub fn of_xy_pairs_for_test(xys: &[(f64, f64)]) -> Result<Self> {
        let mut builder = BezierBuilder::default();
        let tree = Self::build(&mut builder, vec![], xys)?;
        Ok(Self { tree })
    }

    /// Build a PiecewiseBezier that between min_t and max_t matches a function with a maximum error
    ///
    /// Create a Vec of (min_t, BezierBetweenThem), where the Beziers are
    /// within the maximum error; then construct the tree from this array
    pub fn of_fn<F>(
        mut min_t: f64,
        max_t: f64,
        f: &F,
        max_err: f64,
        steps_per_bezier: usize,
    ) -> Result<Self>
    where
        F: Fn(f64) -> f64,
    {
        let mut builder = BezierBuilder::default();
        let mut node_ranges = vec![];
        while min_t < max_t {
            let mut last_t = max_t;
            loop {
                let (node, error_sq) =
                    PiecewiseBezierNode::of_fn(&mut builder, min_t, last_t, f, steps_per_bezier)?;
                if error_sq < max_err * max_err {
                    node_ranges.push((min_t, node));
                    min_t = last_t;
                    break;
                }
                last_t = (min_t + last_t) / 2.0;
            }
        }
        Self::of_node_ranges(node_ranges)
    }

    /// Find the 'best' value of t given a value v, and a set of (t,v) pairs that are monotonic in v
    fn find_y_of_x(x_y_pairs: &[(f64, f64)], x: f64) -> f64 {
        assert!(x_y_pairs.len() >= 2, "Only works with 2 or more points");
        match x_y_pairs.binary_search_by(|(x_test, _y_test)| x_test.partial_cmp(&x).unwrap()) {
            Ok(idx) => x_y_pairs[idx].1,
            Err(mut idx) => {
                // Linear interpolation between the two closest values
                if idx > 0 {
                    idx -= 1;
                }
                if idx + 1 >= x_y_pairs.len() {
                    idx = x_y_pairs.len() - 2;
                }
                let (x0, y0) = x_y_pairs[idx];
                let (x1, y1) = x_y_pairs[idx + 1];
                let dx = x1 - x0;
                let dy = y1 - y0;
                let y = y0 + (x - x0) * dy / dx;
                y
            }
        }
    }

    /// Build a PiecewiseBezier of the inverse of a function between min_t and max_t
    ///
    /// Create a Vec of (min_t, max_t, BezierBetweenThem), where the Beziers are
    /// within the maximum error; then construct the tree from this array
    pub fn inv(
        &self,
        min_t: f64,
        max_t: f64,
        max_err: f64,
        num_steps: usize,
        steps_per_bezier: usize,
    ) -> Result<Self> {
        debug_assert!(
            num_steps >= 2,
            "Number of steps for inv PiecewiseBezier must be >=2"
        );

        let dt = max_t - min_t;
        let t_of_v: Vec<_> = (0..=num_steps)
            .map(|i| min_t + dt * (i as f64) / ((num_steps - 1) as f64))
            .map(|t| (self.evaluate(t), t))
            .collect();

        let min_v = t_of_v.first().unwrap().0;
        let max_v = t_of_v.last().unwrap().0;
        let fn_t_of_v = |v| Self::find_y_of_x(&t_of_v, v);
        Self::of_fn(min_v, max_v, &fn_t_of_v, max_err, steps_per_bezier)
    }

    /// Build a PiecewiseBezier of a set of (x,y) pairs such that it has a
    /// minimal specified error from the *linear* interpolation between the data
    /// points
    ///
    pub fn of_x_y_pairs(
        x_y_pairs: &[(f64, f64)],
        min_t: f64,
        max_t: f64,
        max_err: f64,
        steps_per_bezier: usize,
    ) -> Result<Self> {
        debug_assert!(x_y_pairs.len() >= 2, "Number of points must be >=2");

        let fn_y_of_x = |v| Self::find_y_of_x(x_y_pairs, v);
        Self::of_fn(min_t, max_t, &fn_y_of_x, max_err, steps_per_bezier)
    }
}

#[test]
fn test_piecewise() -> Result<()> {
    let p = PiecewiseBezier::of_xy_pairs_for_test(&[(0., 0.), (1., 2.0)])?;
    for i in 0..10 {
        let t = (i as f64);
        eprintln!("{i} {}", p.evaluate(t));
    }

    let p = PiecewiseBezier::of_xy_pairs_for_test(&[(0., 0.), (1., 2.0), (2., 8.)])?;
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
    let p = PiecewiseBezier::of_xy_pairs_for_test(&d)?;
    for i in 0..50 {
        let t = i as f64;
        eprintln!("{i} {} {}", p.evaluate(t), t.to_radians().tan());
    }
    eprintln!("{p:?}");
    // assert!(false, "Force fail");
    Ok(())
}

#[test]
fn test_piecewise_fn() -> Result<()> {
    let p = PiecewiseBezier::of_fn(-0.1, 1.4, &f64::tan, 1E-4, 100)?;
    let mut errors = 0;
    for i in 0..400 {
        let t = (i as f64).to_radians() / 5.0;
        let delta = p.evaluate(t) - t.tan();
        eprintln!("{i} {} {} {}", p.evaluate(t), t.tan(), delta);
        if delta.abs() > 1E-4 {
            errors += 1;
        }
    }
    eprintln!("{p:?}");
    eprintln!("Total errors {errors}");
    assert!(errors == 0, "Errors in PiecewiseBezier of_fn");
    Ok(())
}

#[test]
fn test_piecewise_inv_fn() -> Result<()> {
    let p = PiecewiseBezier::of_fn(-0.1, 1.4, &f64::tan, 1E-4, 100)?;

    let p_i = p.inv(-0.1_f64, 1.4_f64, 1E-4, 1000, 100)?;
    let mut errors = 0;
    for i in 0..400 {
        let t = (i as f64).to_radians() / 5.0;
        let delta = p_i.evaluate(t.tan()) - t;
        eprintln!("{i} p_i(t.tan()):{} {t} {delta}", p_i.evaluate(t.tan()));
        if delta.abs() > 1E-4 {
            errors += 1;
        }
    }
    eprintln!("{p:?}");
    eprintln!("Total errors {errors}");
    assert!(errors == 0, "Errors in PiecewiseBezier inv of_fn");
    Ok(())
}
