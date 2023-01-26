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
//ip MinSquares
use geo_nd::matrix;
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
    dbg!(&x_xt);
    let mut dm = nalgebra::base::DMatrix::from_element(P, P, 2.0);
    dm.copy_from_slice(&x_xt);
    dbg!(&dm);
    if !dm.try_inverse_mut() {
        panic!("Not invertible");
    }
    let mut xt_y = [0.; P]; // P row vector
    matrix::multiply_dyn(P, n, 1, &xi_m_t, ys, &mut xt_y);
    let mut dm_2 = [0.; P2];
    for i in 0..P2 {
        dm_2[i] = dm[i];
    }
    let r = matrix::multiply::<f64, P2, P, P, P, P, 1>(&dm_2, &xt_y); // P row vector
    r
}
