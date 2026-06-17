use geo_nd::Vector;
use ic_base::{Plane, Point3D};

#[test]
fn test_plane() -> Result<(), String> {
    // The test plane here is the plane X=3
    //
    // The tangents are *known* (white-box) to be 0,1,0 and 0,0,1
    let normal: Point3D = [1.0, 0.0, 0.0].into();
    let p: Plane = (normal, 3.0).into();
    assert_eq!(p.normal(), &normal);

    let x: Point3D = [5.0, 6.0, 4.0].into();
    eprintln!("x: {x}");
    eprintln!("x in plane: {}", p.within_plane(&x));
    eprintln!("point in space: {}", p.point_in_space(&p.within_plane(&x)));
    eprintln!(
        "x projected onto: {} {}",
        p.point_projected_onto(&x).0,
        p.point_projected_onto(&x).1
    );
    assert_eq!(
        p.point_in_space(&p.within_plane(&x)),
        p.point_projected_onto(&x).0
    );
    assert_eq!(p.within_plane(&x)[0], 6.0);
    assert_eq!(p.within_plane(&x)[1], 4.0);

    let pts: &[Point3D] = &[
        [1., 2., 3.].into(),
        [3., 5., 6.].into(),
        [1., 2., 5.].into(),
    ];
    for x in pts.iter() {
        assert!(
            p.point_in_space(&p.within_plane(x))
                .distance(&p.point_projected_onto(x).0)
                < 1E-4
        );
    }
    Ok(())
}

#[test]
fn test_plane2() -> Result<(), String> {
    // Plane x=-z, y = anything, (3,_,3) is on the plane
    let normal: Point3D = [1.0, 0.0, 1.0].into();
    let p: Plane = (normal, 3.0).into();

    eprintln!("plane {p:?}");

    eprintln!("origin  {}", p.origin_in_space());

    // normal will be length 1/sqrt(2)

    let pts: &[Point3D] = &[
        [1., 2., 3.].into(), // .normal = 4/sqrt(2); d = 3*sqrt(2) = 6/sqrt(2); on is +2/sqrt(2) normal = (2,2,4)
        [3., 5., 6.].into(),
        [1., 2., 5.].into(),
    ];
    for x in pts.iter() {
        eprintln!("\n{x}");
        eprintln!("  {}", p.within_plane(x));
        eprintln!("  {}", p.point_in_space(&p.within_plane(x)));
        eprintln!(
            "  =? {} {}",
            p.point_projected_onto(x).0,
            p.point_projected_onto(x).1
        );
        assert!(
            p.point_in_space(&p.within_plane(x))
                .distance(&p.point_projected_onto(x).0)
                < 1E-4
        );
    }
    Ok(())
}
