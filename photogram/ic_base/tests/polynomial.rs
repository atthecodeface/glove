use ic_base::Result;
use ic_base::polynomial::{CalcPoly, min_squares_dyn};

#[test]
fn test_poly() -> Result<()> {
    let f = |x: f64| (x / 10.).sin().atan();
    let err = |x0: f64, x1: f64| {
        if x0.abs() < 0.000001 {
            x1 - x0
        } else {
            x1 / x0 - 1.0
        }
    };

    // x in 1.41
    let xys = (0..100).map(|x| (x as f64) / 70.0).map(|x| (x, f(x)));
    let yxs = xys.clone().map(|(x, y)| (y, x));

    let poly = min_squares_dyn(7, xys.clone())?;
    let rev_poly = min_squares_dyn(7, yxs.clone())?;

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
        Err(format!("Number of errors {num_errors}").into())
    } else {
        Ok(())
    }
}
