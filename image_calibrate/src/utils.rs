use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

use geo_nd::Vector;
use serde::{Deserialize, Serialize};

//a run_to_completion
pub mod rtc {
    use std::future::Future;
    use std::task::Poll::Ready;
    static RTC_VTABLE: &std::task::RawWakerVTable = &std::task::RawWakerVTable::new(
        |x| std::task::RawWaker::new(x, RTC_VTABLE), // clone: unsafe fn(_: *const ()) -> RawWaker,
        |_| (),                                      // wake: unsafe fn(_: *const ()),
        |_| (),                                      // wake_by_ref: unsafe fn(_: *const ()),
        |_| (),                                      // drop: unsafe fn(_: *const ())
    );

    //fp run_to_completion
    pub fn run_to_completion<T, F: Future<Output = T>>(thing: F) -> T {
        let data = &() as *const ();
        let raw_waker = std::task::RawWaker::new(data, RTC_VTABLE);
        let waker = unsafe { std::task::Waker::from_raw(raw_waker) };
        let mut context = std::task::Context::from_waker(&waker);

        let mut adapter = Box::pin(thing);
        loop {
            if let Ready(val) = adapter.as_mut().poll(&mut context) {
                return val;
            }
        }
    }
}

//a Rrc
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
    pub fn borrow(&self) -> Ref<T> {
        self.0.borrow()
    }
    pub fn borrow_mut(&self) -> RefMut<T> {
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
