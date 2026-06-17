use geo_nd::Vector;
use ic_base::{GCTriangle, Point3D, Triangle3D};

use std::rc::Rc;

#[test]
fn test_gct0() -> Result<(), Box<dyn std::error::Error>> {
    let x: Point3D = [1.0, 0.0, 0.0].into();
    let y: Point3D = [0.0, 1.0, 0.0].into();
    let z: Point3D = [0.0, 0.0, 1.0].into();

    let t = GCTriangle::of_points(&x, &y, &z);
    assert_eq!(t.nonunit_points()[0], x);
    assert_eq!(t.nonunit_points()[1], y);
    assert_eq!(t.nonunit_points()[2], z);

    assert!(t.contains_pt_scaled(&x));
    assert!(t.contains_pt_scaled(&y));
    assert!(t.contains_pt_scaled(&z));

    assert!(!t.contains_pt_scaled(&-x));
    assert!(!t.contains_pt_scaled(&-y));
    assert!(!t.contains_pt_scaled(&-z));

    let mid_xy = (x + y) * 0.5;
    let mid_yz = (y + z) * 0.5;
    let mid_zx = (z + x) * 0.5;

    let mid_xy_n = mid_xy.normalize();
    let mid_yz_n = mid_yz.normalize();
    let mid_zx_n = mid_zx.normalize();

    let t0 = GCTriangle::of_points(&x, &mid_xy_n, &mid_zx_n);
    let t1 = GCTriangle::of_points(&y, &mid_yz_n, &mid_xy_n);
    let t2 = GCTriangle::of_points(&z, &mid_zx_n, &mid_yz_n);
    let t3 = GCTriangle::of_points(&mid_xy_n, &mid_zx_n, &mid_yz_n);

    let x = Some(Rc::new(t0.clone()));
    eprintln!("{}", std::mem::size_of_val(&x));
    let t = GCTriangle::of_points(
        &[0.8, 0.0, 0.5].into(),
        &[0.5, 0.8, 0.0].into(),
        &[0.0, 0.5, 0.8].into(),
    );

    // GC Normals of t are scaled versions of (-0.5, 0.3, 0.8); (0.8, -0.5, 0.3); (0.3, 0.8, -0.5)
    dbg!(&t);
    let ot = GCTriangle::of_normals(
        &t.nonunit_normal_01,
        &t.nonunit_normal_12,
        &t.nonunit_normal_20,
    );
    assert!(t.nonunit_normal_01.distance_sq(&ot.nonunit_normal_01) < 1E-4);
    assert!(t.nonunit_normal_12.distance_sq(&ot.nonunit_normal_12) < 1E-4);
    assert!(t.nonunit_normal_20.distance_sq(&ot.nonunit_normal_20) < 1E-4);

    let oot = GCTriangle::of_points(
        &ot.nonunit_points()[0],
        &ot.nonunit_points()[1],
        &ot.nonunit_points()[2],
    );

    assert!(
        t.nonunit_normal_01
            .normalize()
            .distance_sq(&oot.nonunit_normal_01.normalize())
            < 1E-4
    );
    assert!(
        t.nonunit_normal_12
            .normalize()
            .distance_sq(&oot.nonunit_normal_12.normalize())
            < 1E-4
    );
    assert!(
        t.nonunit_normal_20
            .normalize()
            .distance_sq(&oot.nonunit_normal_20.normalize())
            < 1E-4
    );

    let oot3 = Triangle3D::of_normals_on_sphere(
        oot.nonunit_normal(0),
        oot.nonunit_normal(1),
        oot.nonunit_normal(2),
    );

    let oot3b = Triangle3D::of_points(
        &ot.nonunit_points()[0].normalize(),
        &ot.nonunit_points()[1].normalize(),
        &ot.nonunit_points()[2].normalize(),
    );

    assert!(
        oot3.points()[0]
            .normalize()
            .distance_sq(&oot3b.points()[0].normalize())
            < 1E-4,
        "Point {:0.4} too far from {:0.4}",
        oot3.points()[0].normalize(),
        oot3b.points()[0].normalize()
    );
    assert!(
        oot3.points()[1]
            .normalize()
            .distance_sq(&oot3b.points()[1].normalize())
            < 1E-4
    );
    assert!(
        oot3.points()[2]
            .normalize()
            .distance_sq(&oot3b.points()[2].normalize())
            < 1E-4
    );

    // assert!(false, "Force fail");
    //
    Ok(())
}

#[test]
fn test_gct1() -> Result<(), Box<dyn std::error::Error>> {
    let x: Point3D = [1.0, 0.0, 0.0].into();
    let y: Point3D = [0.0, 1.0, 0.0].into();
    let z: Point3D = [0.0, 0.0, 1.0].into();
    let m = x + y + z;

    let gct = GCTriangle::of_points(&x, &y, &z);

    let t = Triangle3D::of_normals_on_sphere(
        gct.nonunit_normal(0),
        gct.nonunit_normal(1),
        gct.nonunit_normal(2),
    );

    assert!(gct.nonunit_points()[0].distance_sq(&x) < 1E-4);
    assert!(gct.nonunit_points()[1].distance_sq(&y) < 1E-4);
    assert!(gct.nonunit_points()[2].distance_sq(&z) < 1E-4);

    assert!(t.points()[0].distance_sq(&x) < 1E-4);
    assert!(t.points()[1].distance_sq(&y) < 1E-4);
    assert!(t.points()[2].distance_sq(&z) < 1E-4);

    let b: Point3D = t.barycentric_coordinates(&x).into();
    assert!(
        b.distance_sq(&Point3D::from([1.0, 0.0, 0.0])) < 1E-4,
        "Bad X barycentric coordinate {b:0.6}"
    );

    let b: Point3D = t.barycentric_coordinates(&y).into();
    assert!(
        b.distance_sq(&Point3D::from([0.0, 1.0, 0.0])) < 1E-4,
        "Bad Y barycentric coordinate {b:0.6}"
    );

    let b: Point3D = t.barycentric_coordinates(&z).into();
    assert!(
        b.distance_sq(&Point3D::from([0.0, 0.0, 1.0])) < 1E-4,
        "Bad Z barycentric coordinate {b:0.6}"
    );

    let b: Point3D = t.barycentric_coordinates(&m).into();
    assert!(
        b.distance_sq(&Point3D::from([0.33333, 0.33333, 0.33333])) < 1E-4,
        "Bad M barycentric coordinate {b:0.6}"
    );

    let (b, d) = t.point_projected_onto_by_normal(&(x / 1.0));
    assert!(
        b.distance_sq(&x) < 1E-4,
        "Bad point X projected onto {b:0.6}"
    );
    assert!((d - 0.0).abs() < 1E-4, "Bad distance {d}");

    let (b, d) = t.point_projected_onto_by_normal(&(m / 3.0));
    assert!(
        b.distance_sq(&Point3D::from([0.33333, 0.33333, 0.33333])) < 1E-4,
        "Bad point projected onto {b:0.6}"
    );
    assert!((d - 0.0).abs() < 1E-4, "Bad distance {d}");

    for p in &[
        [1.0, 2.0, 3.0],
        [2.0, -5.0, 6.0],
        [3.0, -15.0, -4.0],
        [5.0, 25.0, 2.0],
        [1.0, 17.0, -3.0],
    ] {
        let p: Point3D = p.into();
        let b = t.barycentric_coordinates(&p);
        let pt = t.of_barycentric_coordinates(&b);

        let p_proj = t.point_projected_onto_by_scaling(&p);
        assert!(
            p_proj.distance_sq(&pt) < 1E-4,
            "Point {p:0.4} projected onto plane {p_proj:0.4} is too far from barycentric point {pt:0.4}"
        );
    }

    Ok(())
}

#[test]
fn test_gct2() -> Result<(), Box<dyn std::error::Error>> {
    let x: Point3D = [0.4, 0.3, 0.1].into();
    let y: Point3D = [0.1, 0.4, 0.3].into();
    let z: Point3D = [0.3, 0.1, 0.4].into();
    let w: Point3D = [-0.3, 0.1, 0.4].into();
    test_pts(x, y, z)?;
    test_pts(x, y, w)?;
    Ok(())
}

fn test_pts(x: Point3D, y: Point3D, z: Point3D) -> Result<(), Box<dyn std::error::Error>> {
    let m = x + y + z;

    let gct = GCTriangle::of_points(&x, &y, &z);
    let t = Triangle3D::of_normals_on_sphere(
        gct.nonunit_normal(0),
        gct.nonunit_normal(1),
        gct.nonunit_normal(2),
    );
    let t = Triangle3D::of_points(
        &gct.nonunit_points()[0],
        &gct.nonunit_points()[1],
        &gct.nonunit_points()[2],
    );

    t.validate();
    assert!(
        gct.nonunit_points()[0]
            .normalize()
            .distance_sq(&x.normalize())
            < 1E-4
    );
    assert!(
        gct.nonunit_points()[1]
            .normalize()
            .distance_sq(&y.normalize())
            < 1E-4
    );
    assert!(
        gct.nonunit_points()[2]
            .normalize()
            .distance_sq(&z.normalize())
            < 1E-4
    );

    assert!(t.points()[0].normalize().distance_sq(&x.normalize()) < 1E-4);
    assert!(t.points()[1].normalize().distance_sq(&y.normalize()) < 1E-4);
    assert!(t.points()[2].normalize().distance_sq(&z.normalize()) < 1E-4);

    let b: Point3D = t.barycentric_coordinates(&x).into();
    assert!(
        b.distance_sq(&Point3D::from([1.0, 0.0, 0.0])) < 1E-4,
        "Bad X barycentric coordinate {b:0.6}"
    );

    let b: Point3D = t.barycentric_coordinates(&y).into();
    assert!(
        b.distance_sq(&Point3D::from([0.0, 1.0, 0.0])) < 1E-4,
        "Bad Y barycentric coordinate {b:0.6}"
    );

    let b: Point3D = t.barycentric_coordinates(&z).into();
    assert!(
        b.distance_sq(&Point3D::from([0.0, 0.0, 1.0])) < 1E-4,
        "Bad Z barycentric coordinate {b:0.6}"
    );

    let b: Point3D = t.barycentric_coordinates(&m).into();
    assert!(
        b.distance_sq(&Point3D::from([0.33333, 0.33333, 0.33333])) < 1E-4,
        "Bad M barycentric coordinate {b:0.6}"
    );

    let b = t.point_projected_onto_by_scaling(&(x / 1.0));
    assert!(
        b.normalize().distance_sq(&x.normalize()) < 1E-4,
        "Bad point X projected onto {b:0.6}"
    );

    for p in &[
        [1.0, 2.0, 3.0],
        [2.0, -5.0, 6.0],
        [3.0, -15.0, -4.0],
        [5.0, 25.0, 2.0],
        [1.0, 17.0, -3.0],
    ] {
        let p: Point3D = p.into();
        let b = t.barycentric_coordinates(&p);
        let pt = t.of_barycentric_coordinates(&b);

        let p_proj = t.point_projected_onto_by_scaling(&p);
        assert!(
            p_proj.distance_sq(&pt) < 1E-4,
            "Point {p:0.4} projected onto plane {p_proj:0.4} is too far from barycentric point {pt:0.4}"
        );
    }

    Ok(())
}
