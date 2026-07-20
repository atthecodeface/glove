//a Imports
use ic_base::Result;
use ic_camera::LensPolys;
use ic_camera::polynomial;
use ic_camera::polynomial::CalcPoly;

/// Test the mapping by generating LensPolys for sensor yaw in 1000 steps from wmin to wmax mapped to world yaw through the mapping
fn test_mapping<F>(
    name: &str,
    wts_fn: F,
    degree: usize,
    wmin: f64,
    wmax: f64,
    test_lens_poly: &LensPolys,
    ignore_tab: bool,
) -> Result<()>
where
    F: Fn(f64) -> f64,
{
    let mut num_out_of_range = 0;

    let wrange = wmax - wmin;
    let yaws = (0..1000).map(|i| (i as f64) / 1000.0 * wrange + wmin);

    let lens_poly = LensPolys::of_wts_fn(&wts_fn, wmin, wmax).unwrap();

    // First test world-to-swnsoe mapping
    let mut num_errors = 0;
    for world in yaws.clone() {
        let lens_sensor = lens_poly.map_world_to_sensor(world);
        let sensor = wts_fn(world);
        if (sensor - lens_sensor).abs() < 0.01 {
            continue;
        }
        eprintln!(
            "world:{world} sensor:{sensor} poly(world):{lens_sensor} delta:{}",
            sensor - lens_sensor
        );
        num_errors += 1;
    }

    if num_errors > 0 {
        return Err(format!(
            "Mismatch in *calibration* constructor in camera_lens {num_errors} errors"
        )
        .into());
    }

    // Now test sensor-to-world mapping of wts_fn(world) so round-tripping is ok
    for world in yaws.clone() {
        let sensor = wts_fn(world);
        let lens_world = lens_poly.map_sensor_to_world(sensor);
        if (world - lens_world).abs() > 0.001 {
            eprintln!(
                "world {world:0.4} lens there and back {lens_world:0.4} error {}",
                (world - lens_world)
            );
            num_out_of_range += 1;
        }
    }

    if num_out_of_range > 0 {
        return Err(format!("Failed with {num_out_of_range} out of range").into());
    }

    // Now test the provided poly matches
    for world in yaws.clone() {
        let sensor = wts_fn(world);

        let test_lens_sensor = test_lens_poly.map_world_to_sensor(world);
        let test_lens_world = test_lens_poly.map_sensor_to_world(sensor);

        if (test_lens_world - world).abs() > 0.001 {
            eprintln!(
                "world {world:0.4} test_lens_world {test_lens_world:0.4} error {}",
                (test_lens_world - world)
            );
            num_out_of_range += 1;
        }

        if (test_lens_sensor - sensor).abs() > 0.001 {
            eprintln!(
                "sensor {sensor:0.4} test_lens_sensor {test_lens_sensor:0.4} error {}",
                (test_lens_sensor - sensor)
            );
            num_out_of_range += 1;
        }
    }

    eprintln!(
        "pub const LP_{name}_WTS: &'static [f64] = &{:?};",
        lens_poly.wts_poly_as_f64s()
    );
    eprintln!(
        "pub const LP_{name}_STW: &'static [f64] = &{:?};",
        lens_poly.stw_poly_as_f64s()
    );

    if num_out_of_range > 0 {
        return Err(format!("Failed with {num_out_of_range} out of range").into());
    }
    Ok(())
}

#[test]
fn test_stereographic() -> Result<()> {
    let lens = LensPolys::stereographic();
    // tan(sensor) = 2 tan(world/2)
    let wts_fn = |world: f64| ((world / 2.0).tan() * 2.0).atan();
    test_mapping(
        "STEREOGRAPHIC",
        wts_fn,
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
    // tan(sensor) = 2 tan(world/2)
    let wts_fn = |world: f64| ((world / 2.0).tan() * 2.0).atan();
    test_mapping(
        "EQUIANGULAR",
        wts_fn,
        9,
        0.0,
        std::f64::consts::PI / 2.0 * 1.5,
        &lens,
        false,
    )
}

#[test]
fn test_rectilinear() -> Result<()> {
    let lens = LensPolys::rectilinear();
    // tan(sensor) = tan(world)
    let wts_fn = |world: f64| world;
    test_mapping(
        "RECTILINEAR",
        wts_fn,
        8,
        0.0,
        std::f64::consts::PI / 2.0,
        &lens,
        false,
    )?;
    // assert!(false, "Force fail");
    Ok(())
}

#[test]
fn test_equisolid() -> Result<()> {
    let lens = LensPolys::equisolid();
    // tan(sensor) = 2 sin(world/2)
    let wts_fn = |world: f64| ((world / 2.0).sin() * 2.0).atan();
    test_mapping(
        "EQUISOLID",
        wts_fn,
        8,
        0.0,
        0.97 * std::f64::consts::PI / 2.0,
        &lens,
        false,
    )?;
    // assert!(false, "Force fail");
    Ok(())
}

#[test]
fn test_orthographic() -> Result<()> {
    let lens = LensPolys::orthographic();
    // tan(sensor) = sin(world)
    let wts_fn = |world: f64| world.sin().atan();
    test_mapping(
        "ORTHOGRAPHIC",
        wts_fn,
        9,
        0.0,
        0.93 * std::f64::consts::PI / 2.0,
        &lens,
        true,
    )
}

#[test]
fn test_equidistant() -> Result<()> {
    let lens = LensPolys::equidistant();

    // tan(sensor) = world
    let wts_fn = |world: f64| world.atan();
    test_mapping(
        "EQUIDISTANT",
        wts_fn,
        8,
        0.0,
        0.97 * std::f64::consts::PI / 2.0,
        &lens,
        false,
    )?;
    // assert!(false, "Force fail");
    Ok(())
}
