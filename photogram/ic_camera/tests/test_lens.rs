//a Imports
use ic_base::{Error, Result};
use ic_camera::polynomial;
use ic_camera::polynomial::CalcPoly;
use ic_camera::LensPolys;

fn test_mapping<F>(
    name: &str,
    fwd_fn: F,
    degree: usize,
    wmin: f64,
    wmax: f64,
    lens: &LensPolys,
    ignore_tab: bool,
) -> Result<()>
where
    F: Fn(f64) -> f64,
{
    let mut num_out_of_range = 0;

    let wrange = wmax - wmin;
    let yaws = (0..1000).map(|i| ((i as f64) / 1000.0 * wrange + wmin));

    let sensor_yaws: Vec<_> = yaws.clone().map(|s| fwd_fn(s)).collect();
    let world_yaws: Vec<_> = yaws.clone().collect();
    let lens_poly = LensPolys::calibration(degree, &sensor_yaws, &world_yaws, 0.0, 10000.0);
    let mut num_errors = 0;
    for world in yaws.clone() {
        let lens_sensor = lens_poly.wts(world);
        let sensor = fwd_fn(world);
        if (sensor - lens_sensor).abs() < 0.01 {
            continue;
        }
        eprintln!("{world} {sensor} {lens_sensor} {}", sensor - lens_sensor);
        num_errors += 1;
    }

    if num_errors > 0 {
        return Err(format!(
            "Mismatch in *calibration* constructor in camera_lens {num_errors} errors"
        )
        .into());
    }

    eprintln!("pub const LP_{name}: ([f64; {degree}], [f64; {degree}]) = (");
    eprintln!("{}", lens_poly.to_json()?);
    eprintln!(");");

    let ytm = yaws.clone().map(|y| (y, fwd_fn(y)));
    let mty = yaws.clone().map(|y| (fwd_fn(y), y));

    for (a, b) in ytm.clone().take(10) {
        eprintln!("{a} {b}");
    }
    let mut wts = polynomial::min_squares_dyn(degree, ytm);
    let mut stw = polynomial::min_squares_dyn(degree, mty);
    eprintln!("{wts:?}");
    wts[0] = 0.0;
    stw[0] = 0.0;
    for world in yaws.clone() {
        let sensor = wts.calc(world);
        let tab = stw.calc(sensor);
        if !ignore_tab {
            if (world - tab).abs() > 0.001 {
                eprintln!("world {world:0.4} there and back {tab:0.4}");
                num_out_of_range += 1;
            }
        }
        if (sensor - fwd_fn(world)).abs() > 0.001 {
            eprintln!("sensor {sensor:0.4} fwd {:0.4}", fwd_fn(world));
            num_out_of_range += 1;
        }

        let lens_sensor = lens.wts(world);
        let lens_tab = lens.stw(sensor);
        if !ignore_tab {
            if (world - lens_tab).abs() > 0.001 {
                eprintln!("world {world:0.4} lens there and back {lens_tab:0.4}");
                num_out_of_range += 1;
            }
        }
        if (lens_sensor - fwd_fn(world)).abs() > 0.001 {
            eprintln!(
                "lens_sensor {lens_sensor:0.4} fwd {:0.4} world {world:0.4}",
                fwd_fn(world)
            );
            num_out_of_range += 1;
        }
    }

    if num_out_of_range > 0 {
        return Err("Failed".into());
    }
    return Ok(());
}

#[test]
fn test_stereographic() -> Result<()> {
    let lens = LensPolys::stereographic();
    let fwd_fn = |x: f64| ((x / 2.0).tan() * 2.0).atan();
    test_mapping(
        "STEREOGRAPHIC",
        fwd_fn,
        7,
        0.0,
        std::f64::consts::PI / 2.0,
        &lens,
        false,
    )
}

#[test]
fn test_equiangular() -> Result<()> {
    let lens = LensPolys::equiangular();
    let fwd_fn = |x: f64| x.atan();
    test_mapping(
        "EQUIANGULAR",
        fwd_fn,
        9,
        0.0,
        std::f64::consts::PI / 2.0,
        &lens,
        false,
    )
}

#[test]
fn test_equisolid() -> Result<()> {
    let lens = LensPolys::equisolid();
    let fwd_fn = |x: f64| (x / 2.0).sin().atan();
    test_mapping(
        "EQUISOLID",
        fwd_fn,
        8,
        0.0,
        0.95 * std::f64::consts::PI / 2.0,
        &lens,
        false,
    )
}

#[test]
fn test_orthographic() -> Result<()> {
    let lens = LensPolys::orthographic();
    let fwd_fn = |x: f64| x.sin().atan();
    test_mapping(
        "ORTHOGRAPHIC",
        fwd_fn,
        9,
        0.0,
        0.9 * std::f64::consts::PI / 2.0,
        &lens,
        true,
    )
}
