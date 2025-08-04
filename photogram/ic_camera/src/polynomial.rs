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
use std::collections::VecDeque;

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
pub fn filter_ws_yaws(ws_yaws: &[(f64, f64)]) -> Vec<(f64, f64)> {
    // world, sensor
    let mut mean_median_wc_yaws = vec![];

    // sensor, grad at sensor
    let mut filter: VecDeque<_> = vec![(0.0, 1.0); 8].into();
    let n = (filter.len() + 1) as f64;
    let mid = (filter.len() + 1) / 2;
    for (i, (w, c)) in ws_yaws.iter().enumerate() {
        filter.push_back((*c, w / c));
        let mut total = 0.0;
        let mut smallest = filter[0].1;
        let mut largest = filter[0].1;
        for (_w, v) in &filter {
            total += *v;
            smallest = smallest.min(*v);
            largest = largest.max(*v);
        }
        let mean = (total - smallest - largest) / (n - 2.0);
        if i >= mid * 2 {
            mean_median_wc_yaws.push((mean * filter[mid].0, filter[mid].0));
            if false {
                eprintln!(
                "Orig s,w {:0.4},{:0.4} : Filter mid s,w {:0.4},{:0.4}, pushed s,w {:0.4},{:0.4}",
                c.to_degrees(),
                w.to_degrees(),
                filter[mid].0.to_degrees(),
                (filter[mid].1 * filter[mid].0).to_degrees(),
                mean_median_wc_yaws.last().unwrap().1.to_degrees(),
                mean_median_wc_yaws.last().unwrap().0.to_degrees(),
            );
            }
        }
        filter.pop_front();
    }
    mean_median_wc_yaws
}

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
pub fn min_squares_dyn<I: ExactSizeIterator<Item = (f64, f64)>>(p: usize, iter: I) -> Vec<f64> {
    let n = iter.len();
    let mut xi_m = vec![0.; n * p]; // N rows of P columns
    let mut xi_m_t = vec![0.; n * p]; // P rows of N columns
    let mut ys = vec![0.; n];
    for (i, (x, y)) in iter.enumerate() {
        ys[i] = y;
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
    matrix::multiply_dyn(p, n, 1, &xi_m_t, &ys, &mut xt_y);
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
pub fn error_in_y_stats<I: Iterator<Item = (f64, f64)>>(
    poly: &[f64],
    iter: I,
) -> (f64, usize, f64, f64, f64) {
    let mut total_sq_err = 0.0;
    let mut total_err = 0.0;
    let mut max_sq_err: f64 = 0.0;
    let mut max_n = 0;
    let mut number_xy = 0;
    for (n, (x, y)) in iter.enumerate() {
        let dy = poly.calc(x) - y;
        let dy2 = dy * dy;
        if dy2 > max_sq_err {
            max_n = n;
        }
        total_err += dy;
        max_sq_err = max_sq_err.max(dy * dy);
        total_sq_err += dy * dy;
        number_xy = n;
    }
    let n = number_xy as f64;
    let mean_err = total_err / n;
    let mean_sq_err = total_sq_err / n;
    let variance_err = mean_sq_err - mean_sq_err.powi(2);
    (max_sq_err, max_n, mean_err, mean_sq_err, variance_err)
}

//fp find_outliers
/// Find points that are outside a range
pub fn find_outliers<I: Iterator<Item = (f64, f64)>>(
    poly: &[f64],
    iter: I,
    dmin: f64,
    dmax: f64,
) -> Vec<usize> {
    let mut outliers = vec![];
    for (n, (x, y)) in iter.enumerate() {
        let py = poly.calc(x);
        if y < py - dmin || y > py + dmax {
            outliers.push(n)
        };
    }
    outliers
}

//a Tests
#[test]
fn test_poly() -> Result<(), String> {
    let f = |x: f64| (x / 10.).sin().atan();
    let err = |x0: f64, x1: f64| {
        (if x0.abs() < 0.000001 {
            x1 - x0
        } else {
            (x1 / x0 - 1.0)
        })
    };

    // x in 1.41
    let xys = (0..100).map(|x| (x as f64) / 70.0).map(|x| (x, f(x)));
    let yxs = xys.clone().map(|(x, y)| (y, x));

    let poly = min_squares_dyn(7, xys.clone());
    let rev_poly = min_squares_dyn(7, yxs.clone());

    eprintln!("{poly:?}");
    let mut num_errors = 0;
    for (x, y) in xys.clone() {
        eprintln!(
            "{x} {y} {:.4e} {:.4e}     {:.4e} {:.4e}",
            poly.calc(x),
            rev_poly.calc(y),
            err(y, poly.calc(x)),
            err(x, rev_poly.calc(y)),
        );
        if err(y, poly.calc(x)).abs() > 0.001 || err(x, rev_poly.calc(y)).abs() > 0.001 {
            num_errors += 1;
        }
    }
    if num_errors > 0 {
        Err(format!("Number of errors {num_errors}"))
    } else {
        Ok(())
    }
}
