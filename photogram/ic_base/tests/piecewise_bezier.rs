use ic_base::{PiecewiseBezier, Result};

#[test]
fn test_piecewise() -> Result<()> {
    let p = PiecewiseBezier::of_xy_pairs_for_test(&[(0., 0.), (1., 2.0)])?;
    for i in 0..10 {
        let t = i as f64;
        eprintln!("{i} {}", p.evaluate(t));
    }

    let p = PiecewiseBezier::of_xy_pairs_for_test(&[(0., 0.), (1., 2.0), (2., 8.)])?;
    for i in 0..10 {
        let t = i as f64;
        eprintln!("{i} {}", p.evaluate(t));
    }

    let mut d = vec![];
    for i in 0..50 {
        let t = i as f64;
        let v = t.to_radians().tan();
        d.push((t, v));
    }
    let p = PiecewiseBezier::of_xy_pairs_for_test(&d)?;
    for i in 0..50 {
        let t = i as f64;
        eprintln!("{i} {} {}", p.evaluate(t), t.to_radians().tan());
    }
    eprintln!("{p:?}");
    // assert!(false, "Force fail");
    Ok(())
}

#[test]
fn test_piecewise_fn() -> Result<()> {
    let p = PiecewiseBezier::of_fn(-0.1, 1.4, &f64::tan, 1E-4, 100, 1E-3)?;
    let mut errors = 0;
    for i in 0..400 {
        let t = (i as f64).to_radians() / 5.0;
        let delta = p.evaluate(t) - t.tan();
        eprintln!("{i} {} {} {}", p.evaluate(t), t.tan(), delta);
        if delta.abs() > 1E-4 {
            errors += 1;
        }
    }
    eprintln!("{p:?}");
    eprintln!("Total errors {errors}");
    assert!(errors == 0, "Errors in PiecewiseBezier of_fn");
    Ok(())
}

#[test]
fn test_piecewise_inv_fn() -> Result<()> {
    let p = PiecewiseBezier::of_fn(-0.1, 1.4, &f64::tan, 1E-4, 100, 1E-3)?;

    let p_i = p.inv(-0.1_f64, 1.4_f64, 1E-4, 1000, 100)?;
    let mut errors = 0;
    for i in 0..400 {
        let t = (i as f64).to_radians() / 5.0;
        let delta = p_i.evaluate(t.tan()) - t;
        eprintln!("{i} p_i(t.tan()):{} {t} {delta}", p_i.evaluate(t.tan()));
        if delta.abs() > 1E-4 {
            errors += 1;
        }
    }
    eprintln!("{p:?}");
    eprintln!("Total errors {errors}");
    assert!(errors == 0, "Errors in PiecewiseBezier inv of_fn");
    Ok(())
}
