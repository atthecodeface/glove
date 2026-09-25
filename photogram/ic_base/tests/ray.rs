use ic_base::{JsonParsable, Ray, Result};

#[test]
fn test_ray() -> Result<()> {
    let r0 = Ray::default()
        .with_start([1., 0., 0.].into())
        .with_direction([-1., 0., 0.].into())
        .with_tan_error(0.1);
    let r1 = Ray::default()
        .with_start([0., 1., 0.].into())
        .with_direction([0., -1., 0.01].into())
        .with_tan_error(0.1);
    r0.intersect(&r1);
    eprintln!("{}", serde_json::to_string_pretty(&[r0, r1]).unwrap());
    Ok(())
}

//ft test_ray2
#[test]
fn test_ray2() -> Result<()> {
    let ray_4060 = Ray::load_json(
        r#"
{
      "start": [
        -257.61000000000007,
        -292.0,
        186.81
      ],
      "direction": [
        0.72802906255846,
        0.641401039594509,
        -0.2420297305649314
      ],
      "tan_error": 0.1
    }"#,
        &(),
    )?;

    let ray_4062 = Ray::load_json(
        r#"
{
      "start": [
        -272.47666666666686,
        -98.69999999999999,
        261.94333333333316
      ],
      "direction": [
        0.8558988940122954,
        0.2414215789446973,
        -0.4573321598667418
      ],
      "tan_error": 0.1
    }"#,
        &(),
    )?;
    ray_4060.intersect(&ray_4062);
    //    assert!(false);
    Ok(())
}

//ft test_ray3
#[test]
fn test_ray3() -> Result<()> {
    let ray_4060 = Ray::load_json(
        r#"
{
      "start": [
        -257.61000000000007,
        -292.0,
        186.81
      ],
      "direction": [
        0.72802906255846,
        0.641401039594509,
        -0.2420297305649314
      ],
      "tan_error": 0.1
    }"#,
        &(),
    )?;

    let ray_4062 = Ray::load_json(
        r#"
{
      "start": [
        -272.47666666666686,
        -98.69999999999999,
        261.94333333333316
      ],
      "direction": [
        0.8558988940122954,
        0.2414215789446973,
        -0.4573321598667418
      ],
      "tan_error": 0.2
    }"#,
        &(),
    )?;

    let p = Ray::closest_point([ray_4060, ray_4062].iter(), &|_, _| 1.0).unwrap();
    dbg!(p);
    let (_k0, d0_sq) = ray_4060.distances(&p);
    let (_k1, d1_sq) = ray_4062.distances(&p);
    assert!(
        (d0_sq - d1_sq).abs() < 1E-6,
        "Distance between the closest point and each of the rays should be about the same"
    );

    let p = Ray::closest_point([ray_4060, ray_4062].iter(), &|r, _| 1.0 / r.tan_error()).unwrap();
    dbg!(p);
    let (_k0, d0_sq) = ray_4060.distances(&p);
    let (_k1, d1_sq) = ray_4062.distances(&p);
    dbg!(d0_sq.sqrt(), d1_sq.sqrt());
    assert!(
        (d0_sq.sqrt() * 2.0 - d1_sq.sqrt()) < 1E-4,
        "Point should be half the distance from ray 0 compared to ray 0"
    );

    Ok(())
}
