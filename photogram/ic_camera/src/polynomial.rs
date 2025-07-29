//a Documentation
/*!

Polynomial of best fit

given data xi, yi, we want E = Sum((yi-Sum(aj.xi^j))^2) to be a minimum

E = Sum((yi-Sum(aj.xi^j))^2)

Now d/dx f(g(x)) = g'(x) . f'(g(x))

  g(x) = yi-Sum(aj.xi^j)
  f(z) = z^2
  f'(z) = 2z

dE/daj = Sum( d/daj((yi-Sum(aj.xi^j))) . 2(yi-Sum(ak.xi^k)) )
       = Sum( 2(yi-Sum(ak.xi^k)) . (-xi^j) )

e.g.
Sum((yi - a - b.xi)^2) = Sum(yi*2 +a^2 +b^2.xi^2 - 2a.yi - 2.b.xi.yi + 2a.b.xi)
d/da(E) = Sum(2a - 2yi +2b.xi)
        = Sum(-2(yi -a - b.xi))
d/db(E) = Sum(2b.xi^2 - 2xi.yi +2a.xi)
        = Sum(-2xi.(yi - a - b.xi ))

dE/daj = Sum( 2(yi-Sum(aj.xi^j)) . (-xi^j) )
       = 0 for all aj at the minimum square error
Hence
Sum( 2(yi-Sum(ak.xi^k)) . (-xi^j) ) = 0 for all j
Sum( xi^j.yi ) = Sum(xi^j.Sum(ak.xi^k)) for all j

i.e. Xt.y = Xt.(X.a) (where Xt is X transpose)
or  (Xt.X).a = Xt.y
or         a = (Xt.X)' . Xt.y (where M' = inverse of M)

!*/
//a Imports
use geo_nd::matrix;

//a CalcPoly
//tt CalcPoly
/// A simple trait for a polynomial calculation
pub trait CalcPoly {
    /// Calculate the value of a polynomial at a certain value
    fn calc(&self, x: f64) -> f64;
}

//ip CalcPoly for &[f64]
impl CalcPoly for &[f64] {
    fn calc(&self, x: f64) -> f64 {
        let mut r = 0.;
        let mut xn = 1.0;
        for p in self.iter() {
            r += p * xn;
            xn *= x;
        }
        r
    }
}

//ip CalcPoly for &[f64; N]
impl<const N: usize> CalcPoly for &[f64; N] {
    fn calc(&self, x: f64) -> f64 {
        self.as_slice().calc(x)
    }
}
//ip CalcPoly for Vec<f64>
impl CalcPoly for Vec<f64> {
    fn calc(&self, x: f64) -> f64 {
        self.as_slice().calc(x)
    }
}

//a Find polynomial with minimum square error for data
//fp min_squares
pub fn min_squares<const P: usize, const P2: usize>(xs: &[f64], ys: &[f64]) -> [f64; P] {
    assert_eq!(P2, P * P);
    let n = xs.len();
    assert_eq!(ys.len(), xs.len());
    let mut xi_m = vec![0.; n * P]; // N rows of P columns
    let mut xi_m_t = vec![0.; n * P]; // P rows of N columns
    for (i, x) in xs.iter().enumerate() {
        let mut xn = 1.;
        for j in 0..P {
            xi_m[i * P + j] = xn;
            xi_m_t[j * n + i] = xn;
            xn *= x;
        }
    }
    let mut x_xt = [0.; P2]; // P by P matrix
    matrix::multiply_dyn(P, n, P, &xi_m_t, &xi_m, &mut x_xt);
    // dbg!(&x_xt);
    let mut dm = nalgebra::base::DMatrix::from_element(P, P, 2.0);
    dm.copy_from_slice(&x_xt);
    // dbg!(&dm);
    if !dm.try_inverse_mut() {
        panic!("Not invertible");
    }
    let mut xt_y = [0.; P]; // P row vector
    matrix::multiply_dyn(P, n, 1, &xi_m_t, ys, &mut xt_y);
    let mut dm_2 = [0.; P2];
    for i in 0..P2 {
        dm_2[i] = dm[i];
    }
    matrix::multiply::<f64, P2, P, P, P, P, 1>(&dm_2, &xt_y) // P row vector
}

//fp min_squares_dyn
pub fn min_squares_dyn(p: usize, xs: &[f64], ys: &[f64]) -> Vec<f64> {
    let n = xs.len();
    assert_eq!(ys.len(), xs.len());
    let mut xi_m = vec![0.; n * p]; // N rows of P columns
    let mut xi_m_t = vec![0.; n * p]; // P rows of N columns
    for (i, x) in xs.iter().enumerate() {
        let mut xn = 1.;
        for j in 0..p {
            xi_m[i * p + j] = xn;
            xi_m_t[j * n + i] = xn;
            xn *= x;
        }
    }
    let mut x_xt = vec![0.; p * p]; // P by P matrix
    matrix::multiply_dyn(p, n, p, &xi_m_t, &xi_m, &mut x_xt);
    let mut dm = nalgebra::base::DMatrix::from_element(p, p, 2.0);
    dm.copy_from_slice(&x_xt);
    if !dm.try_inverse_mut() {
        panic!("Not invertible");
    }
    let mut xt_y = vec![0.; p]; // P row vector
    matrix::multiply_dyn(p, n, 1, &xi_m_t, ys, &mut xt_y);
    let mut dm_2 = Vec::with_capacity(p * p); // P row vector
    for i in 0..p * p {
        dm_2.push(dm[i]);
    }
    let mut res = vec![0.; p]; // P row vector
    matrix::multiply_dyn(p, p, 1, &dm_2, &xt_y, &mut res);
    res
}

//fp error_in_y_stats
/// Run through a set of XY data, and compare it with the polynomial
///
/// Calculate the maximum sq_error in the y; the point which has that
/// maximum error, the total square error, the  error, and the
/// variance
pub fn error_in_y_stats(poly: &[f64], xs: &[f64], ys: &[f64]) -> (f64, usize, f64, f64, f64) {
    assert_eq!(ys.len(), xs.len());
    let mut total_sq_err = 0.0;
    let mut total_err = 0.0;
    let mut max_sq_err: f64 = 0.0;
    let mut max_n = 0;
    for (n, (x, y)) in xs.iter().zip(ys.iter()).enumerate() {
        let dy = poly.calc(*x) - y;
        let dy2 = dy * dy;
        if dy2 > max_sq_err {
            max_n = n;
        }
        total_err += dy;
        max_sq_err = max_sq_err.max(dy * dy);
        total_sq_err += dy * dy;
    }
    let n = xs.len() as f64;
    let mean_err = total_err / n;
    let mean_sq_err = total_sq_err / n;
    let variance_err = mean_sq_err - mean_sq_err.powi(2);
    (max_sq_err, max_n, mean_err, mean_sq_err, variance_err)
}

//fp find_outliers
/// Find points that are outside a range
pub fn find_outliers(poly: &[f64], xs: &[f64], ys: &[f64], dmin: f64, dmax: f64) -> Vec<usize> {
    assert_eq!(ys.len(), xs.len());
    let mut outliers = vec![];
    for (n, (x, y)) in xs.iter().zip(ys.iter()).enumerate() {
        let py = poly.calc(*x);
        if *y < py - dmin || *y > py + dmax {
            outliers.push(n)
        };
    }
    outliers
}
