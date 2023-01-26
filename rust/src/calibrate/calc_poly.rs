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

#[test]
fn test_min_sq() {
    let xi = [1., 2., 3., 4.];
    let yi = [1., 2.0, 3., 4.];
    let r = min_squares::<3, 9>(&xi, &yi);
    dbg!(r);
    // assert!(false);
}

#[test]
fn find_poly_from_bars() {
    let width_20 = 6230.0;
    const BARS_20MM: &[f64] = &[
        112.0, 199.5, 284.0, 370.0, 456.0, 543.0, 629.5, 715.5, 804.0, 890.5, 979.0, 1064.5,
        1151.0, 1230.0, 1317.0, 1406.5, 1493.5, 1586.0, 1672.0, 1764.5, 1855.5, 1945.5, 2034.5,
        2125.5, 2218.5, 2305.5, 2401.5, 2493.0, 2585.0, 2675.5, 2768.0, 2857.5, 2955.0, 3044.5,
        3139.5, 3229.5, 3324.5, 3413.5, 3504.5, 3598.0, 3689.5, 3784.5, 3874.0, 3967.0, 4056.5,
        4148.0, 4238.5, 4330.0, 4418.0, 4508.0, 4598.0, 4686.5, 4776.5, -1., -1., -1., -1., 5213.0,
        5298.0, 5384.5, 5471.0, 5556.0, 5639.5, 5726.7, 5812.5, 5894.0, 5979.5, 6063.0, 6148.5,
    ];
    let camera_distance_mm = (42.0 + 10. / 16.0) * 25.4;
    let bar_width_mm = 31.5 * 25.4 / 40.0; //  # 20mm
                                           // let bar_width_mm = 20.0;
    let x_scale_20 = 5184.0 / width_20 * 20.0 / 22.3 * 2.0;
    let x_scale_20 = 1.0 / x_scale_20;

    let pixel_width = width_20;
    let equispaced_data = BARS_20MM;
    let poly_degree = 5;
    let x_scale = x_scale_20;

    let cx_pix = pixel_width / 2.0;

    // Find which entries of the equispaced_data are either side of the center pixel
    // This assumes they are sorted (but with -1.0 for those that are uncertain)
    let mut cr_index = 0;
    for (i, b) in equispaced_data.iter().enumerate() {
        if *b > cx_pix {
            cr_index = i;
            break;
        }
    }
    let cl_index = cr_index - 1;

    let cl_pix = equispaced_data[cl_index];
    let cr_pix = equispaced_data[cr_index];

    // Find the offset from cx_pix of the middle of the centre bar in pixels
    let ofs_pix_closest_halfway_bars_to_center = (cr_pix + cl_pix) / 2.0 - cx_pix;

    // Find the width of the centre bar
    let bar_width_pix_at_center = cr_pix - cl_pix;

    // Find the offset in mm assuming the bar width in mm
    //
    // The bar to left of center is at ofs_mm_center - bar_width_mm / 2
    // The bar to right of center is at ofs_mm_center + bar_width_mm / 2
    let ofs_mm_center =
        ofs_pix_closest_halfway_bars_to_center / bar_width_pix_at_center * bar_width_mm;
    dbg!(ofs_mm_center);

    let mut sample_data = vec![];
    let mut pixs = vec![];
    let mut xs = vec![];
    let mut ys = vec![];
    for (i, b_pix) in equispaced_data.iter().enumerate() {
        if *b_pix < 0. {
            continue;
        }
        // b_bar_num is 0 for the bar just to the left of center
        let b_bar_num = i as f64 - cl_index as f64; // Signed!
        let b_mm = (b_bar_num - 0.5) * bar_width_mm + ofs_mm_center;
        // Quite possible b_mm is perhaps a little tight on the left-hand side
        // and a bit wider on the right-hand side
        // Hence bar_width_mm is perhaps of the form a*bar_num+c
        // hence b_mm = (b_bar_num - 0.5) * (a * bar_num + c + bar_width_mm) + ofs_mm_center;
        let b_mm = (b_bar_num - 0.5) * (-0.00094 * b_bar_num - 0.21 + bar_width_mm) + ofs_mm_center;
        // let b_mm = (b_bar_num * 1.0001 - 0.5) * (0. + bar_width_mm) + ofs_mm_center;
        let b_r_pix = (b_pix - cx_pix).abs();
        let b_r_mm = b_mm.abs();
        let b_theta = b_r_mm.atan2(camera_distance_mm); // for viewing only
                                                        // sample_data.push((i, b_pix, b_mm, b_r_pix, b_r_mm, b_theta));
        sample_data.push((b_r_pix, b_r_mm));
        pixs.push(b_r_pix);
        xs.push(b_r_pix / pixel_width * 2.0 * x_scale);
        ys.push(b_theta);
    }
    dbg!(&xs);
    dbg!(&ys);
    let frame_mm_to_angle = min_squares::<3, 9>(&xs, &ys);
    let angle_to_frame_mm = min_squares::<3, 9>(&ys, &xs);
    // let frame_mm_to_angle = min_squares::<4, 16>(&xs, &ys);
    // let angle_to_frame_mm = min_squares::<4, 16>(&ys, &xs);
    // let frame_mm_to_angle = min_squares::<6, 36>(&xs, &ys);
    // let angle_to_frame_mm = min_squares::<6, 36>(&ys, &xs);
    // let frame_mm_to_angle = min_squares::<10, 100>(&xs, &ys);
    // let angle_to_frame_mm = min_squares::<10, 100>(&ys, &xs);
    dbg!(frame_mm_to_angle);
    use CalcPoly;
    let mut e_sq = 0.;
    for i in 0..xs.len() {
        let nx = (&angle_to_frame_mm).calc(ys[i]);
        eprintln!(
            "{} {} {} : {} : {} v {} : {} : {}",
            i,
            xs[i],
            ys[i],
            nx,
            pixs[i],
            nx / x_scale / 2.0 * pixel_width,
            ((nx - xs[i]) * (nx - xs[i])).sqrt(),
            ((nx - xs[i]) * (nx - xs[i])).sqrt() / x_scale / 2.0 * pixel_width,
        );
        let e = ((nx - xs[i]) * (nx - xs[i])).sqrt() / x_scale / 2.0 * pixel_width;
        if e > 1.5 {
            e_sq += e;
        }
    }
    dbg!(e_sq);
    assert!(false);
    // sample_data.sort(cmp=lambda x,y:cmp(x[3],y[3])) # Sort by pixels, in case a plots is needed
}
