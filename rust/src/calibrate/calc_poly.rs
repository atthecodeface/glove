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
pub trait CalcPoly {
    fn calc(&self, x: f64) -> f64;
}

//ip CalcPoly for &[f64]
impl CalcPoly for &[f64] {
    fn calc(&self, mut x: f64) -> f64 {
        let mut r = 0.;
        let mut xn = 1.0;
        for p in self.iter() {
            r += p * xn;
            xn *= x;
        }
        r
    }
}

//ip MinSquares
use geo_nd::matrix;
pub fn min_squares<
    const P: usize,
    const P2: usize,
    const N: usize,
    const N2: usize,
    const NP: usize,
>(
    xs: &[f64; N],
    ys: &[f64; N],
) -> [f64; P] {
    assert_eq!(N2, N * N);
    assert_eq!(NP, N * P);
    assert_eq!(P2, P * P);
    let mut xi_m = [0.; NP]; // N rows of P columns
    let mut xi_m_t = [0.; NP]; // P rows of N columns
    for (i, x) in xs.iter().enumerate() {
        let mut xn = 1.;
        for j in 0..P {
            xi_m[i * P + j] = xn;
            xi_m_t[j * N + i] = xn;
            xn *= x;
        }
    }
    let x_xt = matrix::multiply::<f64, NP, NP, P2, P, N, P>(&xi_m_t, &xi_m); // P by P matrix
    dbg!(&x_xt);
    let mut dm = nalgebra::base::DMatrix::from_element(P, P, 2.0);
    dm.copy_from_slice(&x_xt);
    dbg!(&dm);
    if !dm.try_inverse_mut() {
        panic!("Not invertible");
    }
    let xt_y = matrix::multiply::<f64, NP, N, P, P, N, 1>(&xi_m_t, ys); // P row vector
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
    let r = min_squares::<3, 9, 4, 16, 12>(&xi, &yi);
    dbg!(r);
    assert!(false);
}

/*a Old
width_20 = 6230
bars_20 = [112, 199.5, 284, 370, 456, 543, 629.5, 715.5, 804, 890.5, 979, 1064.5, 1151, 1230, 1317, 1406.5,
           1493.5, 1586, 1672, 1764.5, 1855.5, 1945.5, 2034.5, 2125.5, 2218.5, 2305.5, 2401.5, 2493, 2585, 2675.5, 2768, 2857.5, 2955, 3044.5,
           3139.5, 3229.5, 3324.5, 3413.5, 3504.5, 3598, 3689.5,
           3784.5, 3874, 3967, 4056.5, 4148, 4238.5, 4330, 4418, 4508, 4598, 4686.5, 4776.5,
           None, None, None, None, 5213, 5298, 5384.5, 5471, 5556, 5639.5, 5726.7, 5812.5, 5894, 5979.5, 6063, 6148.5]
camera_distance_mm = (42+1/16.0) * 25.4
bar_width_mm = 31.5*25.4/40 # 20mm
x_scale_20 = 5184.0/width_20 * 20/22.3 * 2
x_scale_20 = 1/x_scale_20
(poly_20, inv_poly_20) = find_lens_polynomial(width_20, bars_20, bar_width_mm, camera_distance_mm, poly_degree=5, do_plot=True, x_scale=x_scale_20)

#f find_lens_polynomial
def find_lens_polynomial(pixel_width, equispaced_data, data_width_mm, camera_distance_mm, poly_degree=6, weight_of_zero=10, x_scale=1, do_plot=False):
    """
    A projection provides a function theta(r), where r is the distance from the center of the image
    So create a (pixel distance, physical distance) map
    Hence for each data entry we need a theta
    """
    cr_index = None
    cx_pix = pixel_width / 2.0
    for i in range(len(equispaced_data)):
        b = equispaced_data[i]
        if b>cx_pix:
            cr_index=i
            break
        pass
    cl_index=cr_index-1

    cl_pix = equispaced_data[cl_index]
    cr_pix = equispaced_data[cr_index]

    ofs_pix_closest_halfway_bars_to_center = (cr_pix+cl_pix)/2.0 - cx_pix
    data_width_pix_at_center = float(cr_pix-cl_pix)
    ofs_mm_center = ofs_pix_closest_halfway_bars_to_center / data_width_pix_at_center * data_width_mm

    sample_data = []
    for i in range(len(equispaced_data)):
        b_pix = equispaced_data[i]
        if b_pix is None: continue
        b_mm  = (i-cl_index-0.5)*data_width_mm + ofs_mm_center
        b_r_pix = abs(b_pix-cx_pix)
        b_r_mm  = abs(b_mm)
        b_theta = math.atan2(b_r_mm,camera_distance_mm) # for viewing only
        sample_data.append( (i, b_pix, b_mm, b_r_pix, b_r_mm, b_theta) )
        pass
    sample_data.sort(cmp=lambda x,y:cmp(x[3],y[3])) # Sort by pixels, in case a plots is needed

    # y = x.B where x is the row vector (x, x^2, x^3, x^4, x^5, x^6), and y is a scalar, hence B is a 6x1 matrix
    # We find best B through
    # B = (Xt.X)-1.Xt.y
    # Where y is an n x 1 (column) vector, and X is the n rows each of 6 columns (n x 6) matrix of n samples with their 6 values x, x^2, ...
    # Xt is then 6 x n; Xt.X is then 6 x 6, and its inverse is also 6 x 6; (Xt.X)-1.Xt is 6 x n, and hence B is 6 x 1.
    #
    # We can use numpy though
    import numpy

    xs = []
    ys = []
    nxs = []
    nys = []
    for i in range(weight_of_zero):
        xs.append(0)
        ys.append(0)
        nxs.append(0)
        nys.append(0)
        pass
    for d in sample_data:
        if d[0]<cl_index:
            nxs.append(d[3]/float(pixel_width/2.0) * x_scale)
            nys.append(d[5])
            pass
        else:
            xs.append(d[3]/float(pixel_width/2.0) * x_scale)
            ys.append(d[5])
            pass
        pass
    pass
    poly = numpy.polyfit(nxs, nys, poly_degree)
    inv_poly = numpy.polyfit(nys, nxs, poly_degree)

    #print xs
    #print nxs
    if do_plot:
        dx = 1.0/float(pixel_width/2.0) * x_scale
        import matplotlib.pyplot as plt
        yrs = []
        nyrs = []
        for x in xs:
            yrs.append(poly_calc(poly,x))
            pass
        for x in nxs:
            nyrs.append(poly_calc(poly,x))
            pass
        plt.plot(xs,ys)
        plt.plot(xs,yrs)
        plt.plot(nxs,nys)
        plt.plot(nxs,nyrs)
        plt.show()
        pass
    return (poly, inv_poly)

#a Generate poly for 20mm lens
width_20 = 6230
bars_20 = [112, 199.5, 284, 370, 456, 543, 629.5, 715.5, 804, 890.5, 979, 1064.5, 1151, 1230, 1317, 1406.5,
           1493.5, 1586, 1672, 1764.5, 1855.5, 1945.5, 2034.5, 2125.5, 2218.5, 2305.5, 2401.5, 2493, 2585, 2675.5, 2768, 2857.5, 2955, 3044.5,
           3139.5, 3229.5, 3324.5, 3413.5, 3504.5, 3598, 3689.5,
           3784.5, 3874, 3967, 4056.5, 4148, 4238.5, 4330, 4418, 4508, 4598, 4686.5, 4776.5,
           None, None, None, None, 5213, 5298, 5384.5, 5471, 5556, 5639.5, 5726.7, 5812.5, 5894, 5979.5, 6063, 6148.5]
camera_distance_mm = (42+1/16.0) * 25.4
bar_width_mm = 31.5*25.4/40 # 20mm

bar_diff = []
last = None
for b in bars_20:
    if b is not None and last is not None:
        bar_diff.append(b-last)
        pass
    last=b
    pass
#print bar_diff

x_scale_20 = 5184.0/width_20 * 20/22.3 * 2
x_scale_20 = 1/x_scale_20
for pd in range(3,8):
    (poly, inv_poly) = find_lens_polynomial(width_20, bars_20, bar_width_mm, camera_distance_mm, poly_degree=pd, x_scale=x_scale_20)
    p =list(poly)
    ip =list(inv_poly)
    p.reverse()
    ip.reverse()
    print pd, p, ip
    pass
# looking at this, the top coefficient keeps dropping into degrees=5, when it goes up - implying that things are starting to get a bit less stable
(poly_20, inv_poly_20) = find_lens_polynomial(width_20, bars_20, bar_width_mm, camera_distance_mm, poly_degree=5, do_plot=True, x_scale=x_scale_20)
for px in (555,900,1876,2194,2592):
    angle = poly_calc(poly_20,px/(width_20/2.0))
    print px, angle, math.degrees(angle)
    pass
 */
