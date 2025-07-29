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
) -> Result<()>
where
    F: Fn(f64) -> f64,
{
    //    let mut linear_pts = vec![];
    //    let mut stereographic_pts = vec![];
    //    let mut equiangular_pts = vec![];
    //    let mut equisolid_pts = vec![];
    //    let mut orthographic_pts = vec![];
    //    for i in 0..=100 {
    //        let world_yaw = (i as f64) / 100.0 * (yaw_range_max - yaw_range_min) + yaw_range_min;
    //        linear_pts.push(plot_f(world_yaw, world_yaw));
    //        stereographic_pts.push(plot_f(world_yaw, ((world_yaw / 2.0).tan() * 2.0).atan()));
    //        equiangular_pts.push(plot_f(world_yaw, world_yaw.atan()));
    //        equisolid_pts.push(plot_f(world_yaw, (world_yaw / 2.0).sin().atan()));
    //        orthographic_pts.push(plot_f(world_yaw, world_yaw.sin().atan()));
    //    }

    let mut num_out_of_range = 0;

    let wrange = wmax - wmin;
    let mut yaws = vec![];
    let mut mapped = vec![];
    for i in 0..=1000 {
        let world = (i as f64) / 1000.0 * wrange + wmin;
        yaws.push(world);
        mapped.push(fwd_fn(world));
    }

    let mut wts = polynomial::min_squares_dyn(degree, &yaws, &mapped);
    let mut stw = polynomial::min_squares_dyn(degree, &mapped, &yaws);
    wts[0] = 0.0;
    stw[0] = 0.0;
    for world in yaws.iter().copied() {
        let sensor = wts.calc(world);
        let tab = stw.calc(sensor);
        if (world - tab).abs() > 0.001 {
            eprintln!("world {world:0.4} there and back {tab:0.4}");
            num_out_of_range += 1;
        }
        if (sensor - fwd_fn(world)).abs() > 0.001 {
            eprintln!("sensor {sensor:0.4} fwd {:0.4}", fwd_fn(world));
            num_out_of_range += 1;
        }

        let lens_sensor = lens.wts_poly().calc(world);
        let lens_tab = lens.stw_poly().calc(sensor);
        if (world - lens_tab).abs() > 0.001 {
            eprintln!("world {world:0.4} lens there and back {lens_tab:0.4}");
            num_out_of_range += 1;
        }
        if (lens_sensor - fwd_fn(world)).abs() > 0.001 {
            eprintln!("lens_sensor {lens_sensor:0.4} fwd {:0.4}", fwd_fn(world));
            num_out_of_range += 1;
        }
    }

    eprintln!("pub const LP_{name}: ([f64; {degree}], [f64; {degree}]) = (");
    eprintln!("  {wts:?},");
    eprintln!("  {stw:?},");
    eprintln!(");");

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
    )
}

#[test]
fn test_equisolid() -> Result<()> {
    let lens = LensPolys::equisolid();
    let fwd_fn = |x: f64| (x / 2.0).sin().atan();
    test_mapping(
        "EQUISOLID",
        fwd_fn,
        7,
        0.0,
        std::f64::consts::PI / 2.0,
        &lens,
    )
}

// Orthographic does not have a valid polynomial of any reasonable degree
// #[test]
// fn test_orthographic() -> Result<()> {
//    let fwd_fn = |x: f64| x.sin().atan();
// }
