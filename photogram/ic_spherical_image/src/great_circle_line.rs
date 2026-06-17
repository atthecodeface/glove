use std::rc::Rc;

use crate::{GcNormal, ImagePt};

/// A portion of a great circle
///
/// The great circle is defined by its normal; this is a portion of the great
/// circle, which has two points that must be on the great circle (i.e. the dot
/// product of their coordinates with the normal must be 0)
///
/// The midpoint may also be known; if so, then this GCLine is in essence the
/// definition of that midpoint's position.
#[derive(Debug)]
pub struct GcLine {
    normal: Rc<GcNormal>,
    p0: Rc<ImagePt>,
    p1: Rc<ImagePt>,
    mid_point: Option<Rc<ImagePt>>,
}

impl GcLine {
    pub fn new(normal: &Rc<GcNormal>, p0: &Rc<ImagePt>, p1: &Rc<ImagePt>) -> (bool, Self) {
        let swapped = (*p0).index() > p1.index();
        if !swapped {
            (
                false,
                Self {
                    normal: normal.clone(),
                    p0: p0.clone(),
                    p1: p1.clone(),
                    mid_point: None,
                },
            )
        } else {
            (
                true,
                Self {
                    normal: normal.clone(),
                    p0: p1.clone(),
                    p1: p0.clone(),
                    mid_point: None,
                },
            )
        }
    }
    pub(crate) fn set_midpoint(&mut self, mid_point: Rc<ImagePt>) {
        self.mid_point = Some(mid_point);
    }
    pub fn p0(&self) -> &Rc<ImagePt> {
        &self.p0
    }
    pub fn p1(&self) -> &Rc<ImagePt> {
        &self.p1
    }
    pub fn normal(&self) -> &Rc<GcNormal> {
        &self.normal
    }
    pub fn midpoint(&self) -> Option<&Rc<ImagePt>> {
        self.mid_point.as_ref()
    }
}
