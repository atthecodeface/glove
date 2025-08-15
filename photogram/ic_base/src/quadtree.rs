//a Imports
use std::marker::PhantomData;

use geo_nd::Vector;

use crate::Point2D;

//a QtNode, QtPathEntry, QtPath
//ci NodesPerQt
/// A representation of an rectangular area of a plane to permit searching
const NodesPerQt: usize = 8; // max of 12

//tt QtNode
pub trait QtNode: std::fmt::Debug + Clone + Sized + Default {}

//it QtNode for T
impl<T> QtNode for T where T: std::fmt::Debug + Clone + Sized + Default {}

//tp QtPathEntry
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum QtPathEntry {
    ChildLL,
    ChildGL,
    ChildLG,
    ChildGG,
    Node0,
    Node1,
    Node2,
    Node3,
    Node4,
    Node5,
    Node6,
    Node7,
}

//tp QtPath
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QtPath {
    value: u64,
}

//ip QtPath
impl QtPath {
    //ci new
    fn new(value: u64) -> Self {
        Self { value }
    }

    //ci of_node
    fn of_node(node: u8) -> Self {
        Self::new((node + 4) as u64)
    }

    //ci of_qtpe
    fn of_qtpe(qtpe: QtPathEntry) -> Self {
        Self::new((qtpe as u8) as u64)
    }

    //fp child_mask
    pub fn child_mask(
        allow_left: bool,
        allow_right: bool,
        allow_below: bool,
        allow_above: bool,
    ) -> u8 {
        let mut mask = 0;
        if allow_left && allow_below {
            mask |= 1;
        }
        if allow_right && allow_below {
            mask |= 2;
        }
        if allow_left && allow_above {
            mask |= 4;
        }
        if allow_right && allow_above {
            mask |= 8;
        }
        mask
    }

    //ci of_lx_by
    fn of_lx_by(lx: bool, by: bool) -> Self {
        match (lx, by) {
            (true, true) => Self::of_qtpe(QtPathEntry::ChildLL),
            (false, true) => Self::of_qtpe(QtPathEntry::ChildGL),
            (true, false) => Self::of_qtpe(QtPathEntry::ChildLG),
            (false, false) => Self::of_qtpe(QtPathEntry::ChildGG),
        }
    }

    //ci of_child
    fn of_child(idx: u8) -> Self {
        Self::new(idx as u64)
    }

    //ap is_none
    /// Returns true if self is 'None'
    pub fn is_none(self) -> bool {
        self.value == 0
    }

    //ap is_local
    /// Returns true if self refers to a local node
    pub fn is_local(self) -> bool {
        self.value >= 4 && ((self.value - 4) < (NodesPerQt as u64))
    }

    //ap node
    /// If this is a node, return Some(n); if it refers to a
    /// hierarchy, return None
    pub fn node(self) -> Option<u8> {
        if self.is_local() {
            Some((self.value as u8) - 4)
        } else {
            None
        }
    }

    //ap branch
    /// If this is a node, returns None; if a child, return which quad
    /// it belongs to and the subpath within that quad
    pub fn branch(self) -> Option<(u8, Self)> {
        if self.is_local() {
            None
        } else {
            let branch = (self.value & 3) as u8;
            let value = self.value >> 4;
            Some((branch, Self::new(value)))
        }
    }

    //cp with_subpath
    /// Return this (which must be a single-layer branch) with the new subpath for that branch
    pub fn with_subpath(mut self, sub_qtp: Self) -> Self {
        assert!(!self.is_local());
        assert_eq!(
            self.value & !0xf,
            0,
            "Adding subpath is only permitted to a single-layer branch"
        );
        self.value |= sub_qtp.value << 4;
        self
    }

    //cp append_child_path
    /// Return this (which is a potentially deep path) with a single-level subpath
    pub fn append_child_path(mut self, sub_qtp: Self) -> Self {
        assert!(!self.is_local());
        assert_eq!((self.value >> 60) & 0xf, 0, "Overflow in quadtree depth");
        let mut mask = 0xf;
        let mut subvalue = sub_qtp.value;
        while (self.value & mask != 0) {
            mask <<= 4;
            subvalue <<= 4;
        }
        self.value |= subvalue;
        self
    }

    //zz All done
}

//a QtNodePtPath
#[derive(Clone, Copy)]
pub struct QtNodePtPath<'a, N: QtNode> {
    node: &'a N,
    pt: &'a Point2D,
    qtp: QtPath,
}

//ip Debug for QtNodePtPath<'a, N>
impl<'a, N> std::fmt::Debug for QtNodePtPath<'a, N>
where
    N: QtNode,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "QtNodePtPath {{@{}:{:08x}:{:?}}}",
            self.pt, self.qtp.value, self.node
        )
    }
}

//ip QtNodePtPath<'a, N>
impl<'a, N> QtNodePtPath<'a, N>
where
    N: QtNode,
{
    fn new(node: &'a N, pt: &'a Point2D, qtp: QtPath) -> Self {
        Self { node, pt, qtp }
    }
    pub fn node(&self) -> &N {
        &self.node
    }
    pub fn pt(&self) -> &Point2D {
        &self.pt
    }
    pub fn qtp(&self) -> QtPath {
        self.qtp
    }
}

//a Quadtree
//tp Quadtree
#[derive(Debug, Default)]
pub struct Quadtree<N: QtNode> {
    num_valid: u8,
    nodes: [N; NodesPerQt],
    node_pts: [Point2D; NodesPerQt],
    pivot: Point2D,
    children: [Option<Box<Quadtree<N>>>; 4],
}

//ip Quadtree
impl<N> Quadtree<N>
where
    N: QtNode,
{
    //mi idx_qtp_of_pt
    fn idx_qtp_of_pt(&self, pt: &Point2D) -> (usize, QtPath) {
        let lx = pt[0] < self.pivot[0];
        let by = pt[1] < self.pivot[1];
        let qtp = QtPath::of_lx_by(lx, by);
        let (idx, _) = qtp.branch().unwrap();
        let idx: usize = idx as usize;
        (idx, qtp)
    }

    //ap is_empty
    pub fn is_empty(&self) -> bool {
        self.num_valid == 0
    }

    //ap qt_blk_count
    pub fn qt_blk_count(&self) -> usize {
        let mut blk_count = 1;
        for c in &self.children {
            if let Some(child) = c.as_ref() {
                blk_count += child.qt_blk_count();
            }
        }
        blk_count
    }

    //ap node_count
    pub fn node_count(&self) -> usize {
        let mut node_count = self.num_valid as usize;
        for c in &self.children {
            if let Some(child) = c.as_ref() {
                node_count += child.node_count();
            }
        }
        node_count
    }

    //mp add_node
    pub fn add_node(&mut self, node: N, pt: Point2D) -> QtPath {
        let n = self.num_valid as usize;
        if n < NodesPerQt {
            self.nodes[n] = node;
            self.node_pts[n] = pt;
            self.num_valid += 1;
            if n + 1 == NodesPerQt {
                self.pivot = self.node_pts.iter().fold(Point2D::default(), |a, p| a + p)
                    / (NodesPerQt as f64);
            }
            QtPath::of_node(n as u8)
        } else {
            let (idx, qtp) = self.idx_qtp_of_pt(&pt);
            if self.children[idx].is_none() {
                self.children[idx] = Some(Box::new(Self::default()));
            }
            let sub_qtp = self.children[idx].as_mut().unwrap().add_node(node, pt);
            qtp.with_subpath(sub_qtp)
        }
    }

    //mp get
    pub fn get(&self, qtp: QtPath) -> Option<(&N, &Point2D)> {
        if qtp.is_none() {
            None
        } else if let Some((branch, sub_qtp)) = qtp.branch() {
            let idx = branch as usize;
            assert!(
                self.children[idx].is_some(),
                "Invalid QtPath {qtp:?}, specifies empty child"
            );
            self.children[idx].as_ref().unwrap().get(sub_qtp)
        } else if let Some(node) = qtp.node() {
            assert!(
                self.num_valid > node,
                "Invalid QtPath {qtp:?} - node {node} out of range {}",
                self.num_valid
            );
            Some((&self.nodes[node as usize], &self.node_pts[node as usize]))
        } else {
            panic!("Invalid QtPath {qtp:?}");
        }
    }

    //mp map_point
    /// Find pt in the Quadtree with an equality match, and invoke f
    /// on it
    pub fn map_point<F, T>(&self, pt: &Point2D, f: F) -> Option<T>
    where
        F: FnOnce(&N, &Point2D, QtPath) -> T,
    {
        self.map_point_from_qtp(pt, f, QtPath::default())
    }

    //mi map_point_from_qtp
    fn map_point_from_qtp<F, T>(&self, pt: &Point2D, f: F, qtp: QtPath) -> Option<T>
    where
        F: FnOnce(&N, &Point2D, QtPath) -> T,
    {
        for i in 0..(self.num_valid as usize) {
            if &self.node_pts[i] == pt {
                let qtp = qtp.append_child_path(QtPath::of_node(i as u8));
                return Some(f(&self.nodes[i], pt, qtp));
            }
        }
        if self.num_valid > (NodesPerQt as u8) {
            let (idx, sub_qtp) = self.idx_qtp_of_pt(pt);
            if let Some(child) = self.children[idx].as_ref() {
                let qtp = qtp.append_child_path(sub_qtp);
                return child.map_point_from_qtp(pt, f, qtp);
            }
        }
        None
    }

    //mp iter
    /// Returns an iterator of QtNodePtPath<'a, N>
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = QtNodePtPath<'a, N>> + 'a {
        self.iter_with_pivot(|_| 0xf)
    }

    //mp iter_within_radius_of
    /// Returns an iterator of QtNodePtPath<'a, N>
    pub fn iter_within_radius_of<'a>(
        &'a self,
        center: Point2D,
        radius: f64,
    ) -> impl Iterator<Item = QtNodePtPath<'a, N>> + 'a {
        self.iter_with_pivot(move |pivot| {
            let allow_left = center[0] <= pivot[0] + radius;
            let allow_right = center[0] >= pivot[0] - radius;
            let allow_below = center[1] <= pivot[1] + radius;
            let allow_above = center[1] >= pivot[1] - radius;
            // eprintln!(
            // "mask : {}",
            // QtPath::child_mask(allow_left, allow_right, allow_below, allow_above)
            // );
            QtPath::child_mask(allow_left, allow_right, allow_below, allow_above)
        })
        .filter(move |qtpp| qtpp.pt().distance(&center) <= radius)
    }

    //mp iter_with_pivot
    /// Returns an iterator of QtNodePtPath<'a, N>
    pub fn iter_with_pivot<'a, F>(
        &'a self,
        pivot_filter: F,
    ) -> impl Iterator<Item = QtNodePtPath<'a, N>> + 'a
    where
        F: Fn(&Point2D) -> u8 + 'a,
    {
        let mut stack = vec![];
        stack.push((self, 0, QtPath::default(), 0));
        QuadtreeIter {
            pivot_filter,
            stack,
        }
    }

    //zz All done
}

//a QuadtreeIter
//tp QuadtreeIter
pub struct QuadtreeIter<'a, N: QtNode, F: Fn(&Point2D) -> u8> {
    pivot_filter: F,
    stack: Vec<(&'a Quadtree<N>, u8, QtPath, u8)>,
}

//ip QuadtreeIter
impl<'a, N, F> std::iter::Iterator for QuadtreeIter<'a, N, F>
where
    N: QtNode,
    F: Fn(&Point2D) -> u8,
{
    type Item = QtNodePtPath<'a, N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.stack.is_empty() {
            return None;
        }
        let top = self.stack.last_mut().unwrap();
        if top.1 == 0 && top.0.num_valid >= NodesPerQt as u8 {
            let mut child_mask = (self.pivot_filter)(&top.0.pivot) & 0xf;
            if top.0.children[0].is_none() {
                child_mask &= 0xe;
            }
            if top.0.children[1].is_none() {
                child_mask &= 0xd;
            }
            if top.0.children[2].is_none() {
                child_mask &= 0xb;
            }
            if top.0.children[3].is_none() {
                child_mask &= 0x7;
            }
            top.3 = child_mask;
        }
        if top.1 < top.0.num_valid {
            let n = top.1 as usize;
            let qtp = top.2.append_child_path(QtPath::of_node(top.1));
            top.1 += 1;
            Some((QtNodePtPath::new(&top.0.nodes[n], &top.0.node_pts[n], qtp)))
        } else if top.3 == 0 {
            self.stack.pop();
            self.next()
        } else {
            let (child, mask) = {
                match top.3 {
                    8 => (3, 0),
                    4 | 12 => (2, 8),
                    2 | 6 | 10 | 14 => (1, 12),
                    _ => (0, 14),
                }
            };
            top.3 &= mask;
            let top = self.stack.pop().unwrap();
            let child_qt = top.0.children[child].as_ref().unwrap();
            let qtp = top.2.append_child_path(QtPath::of_child(child as u8));
            self.stack.push(top);
            self.stack.push((child_qt, 0, qtp, 0));
            self.next()
        }
    }
}

//a Tests
#[test]
fn test_quadtree_0() -> Result<(), String> {
    let mut qt = Quadtree::<usize>::default();

    assert!(qt.is_empty());

    let mut paths = vec![];
    paths.push(qt.add_node(0, [0., 0.].into()));

    assert!(!paths[0].is_none());
    assert!(paths[0].is_local());
    assert_eq!(paths[0].node(), Some(0));
    assert_eq!(paths[0].branch(), None);

    let v: Vec<_> = qt.iter().map(|qtnp| *qtnp.node()).collect();
    assert_eq!(&v, &[0]);

    for i in 1..8 {
        paths.push(qt.add_node(i, [0., 0.].into()));
    }

    let v: Vec<_> = qt.iter().map(|qtnp| *qtnp.node()).collect();
    assert_eq!(&v, &[0, 1, 2, 3, 4, 5, 6, 7]);

    Ok(())
}

#[test]
fn test_quadtree_1() -> Result<(), String> {
    let mut qt = Quadtree::<(usize, usize)>::default();

    assert!(qt.is_empty());

    // Add 21*21 = 441 points
    let mut paths = vec![];
    for x in 0..=20 {
        for y in 0..=20 {
            paths.push(qt.add_node((x, y), [x as f64, y as f64].into()));
        }
    }
    assert!(!paths[0].is_none());
    assert!(paths[0].is_local());
    assert_eq!(paths[0].node(), Some(0));
    assert_eq!(paths[0].branch(), None);

    assert_eq!(qt.node_count(), 441);
    eprintln!("{}", qt.qt_blk_count());

    assert!(!paths[440].is_none());
    assert!(!paths[440].is_local());

    let mut paths_in_circle = std::collections::HashSet::new();
    let center = [13., 14.].into();
    let radius = 5.1;
    for qtpp in qt.iter_within_radius_of(center, radius) {
        eprintln!("{qtpp:?}");
        let d = center.distance(qtpp.pt());
        assert!(d <= radius);
        paths_in_circle.insert(qtpp.qtp());
    }
    let mut count = 0;
    for qtpp in qt.iter() {
        eprintln!("{qtpp:?}");
        let d = center.distance(qtpp.pt());
        assert_eq!(paths_in_circle.contains(&qtpp.qtp()), d <= radius);
        count += 1;
    }
    assert_eq!(count, 441);

    Ok(())
}
