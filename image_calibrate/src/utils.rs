use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use geo_nd::Vector;
use serde::{Deserialize, Serialize};

//tp Rrc
#[derive(Debug, Serialize, Deserialize)]
pub struct Rrc<T>(Rc<RefCell<T>>);
impl<T> From<T> for Rrc<T> {
    fn from(data: T) -> Self {
        Self(Rc::new(RefCell::new(data)))
    }
}
impl<T> Clone for Rrc<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Rrc<T> {
    pub fn borrow<'a>(&'a self) -> Ref<'a, T> {
        self.0.borrow()
    }
    pub fn borrow_mut<'a>(&'a self) -> RefMut<'a, T> {
        self.0.borrow_mut()
    }
}
impl<T> std::default::Default for Rrc<T>
where
    T: Default,
{
    fn default() -> Self {
        T::default().into()
    }
}
impl<T> std::ops::Deref for Rrc<T> {
    type Target = RefCell<T>;
    fn deref(&self) -> &RefCell<T> {
        &self.0
    }
}

//tp delta
/// Generate a dxyz in the direction of most increasing e_fn
/// for a point p in model space
pub fn delta_p<P, F, const D: usize>(p: P, f: &F, delta: f64) -> P
where
    P: Vector<f64, D>,
    F: Fn(P) -> f64,
{
    let e = f(p);
    let mut dpdx = P::default();
    for i in 0..D {
        let mut dp = p;
        dp[i] += delta;
        dpdx[i] = f(dp) - e;
    }
    dpdx.normalize() * delta
}

//a better_pt
//tp better_pt
/// Look at the direction required to get the error function reduce
///
/// Go by at most max_d distance.
///
/// Bisect the range of (p, p-by-max_distance), and choose the
/// left or right subrange depending on which gives the closest
/// match to the desired e_fn
pub fn better_pt<P, F, const D: usize>(
    p: &P,
    dp: &P,
    f: &F,
    steps: usize,
    step_scale: f64,
) -> (bool, f64, P)
where
    P: Vector<f64, D>,
    F: Fn(P) -> f64,
{
    let mut dp = *dp;
    let mut moved = false;
    let mut center = *p;
    let mut err = f(center);
    for _ in 0..steps {
        let pt_l = center - dp;
        let pt_r = center + dp;
        let err_l = f(pt_l);
        let err_r = f(pt_r);
        if err_l < err && err_l < err_r {
            err = err_l;
            center = pt_l;
            moved = true;
        } else if err_r < err_l && err_r < err {
            err = err_r;
            center = pt_r;
            moved = true;
        }
        dp *= step_scale;
    }
    (moved, err, center)
}
