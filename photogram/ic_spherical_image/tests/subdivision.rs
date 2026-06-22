use geo_nd::Vector;
use ic_spherical_image::{GreatCircleTriangleIndex, SphericalData, shapes};

fn subdivide_faces(
    sd: &mut SphericalData,
    tris: &[GreatCircleTriangleIndex],
) -> Vec<GreatCircleTriangleIndex> {
    let mut detailed_tris = vec![];
    for t in tris {
        let (new, sub_t) = sd.subdivide_triangle(*t);
        assert_eq!(
            new, 4,
            "Adding subdivide triangles should produce new triangles"
        );
        detailed_tris.push(sub_t[0]);
        detailed_tris.push(sub_t[1]);
        detailed_tris.push(sub_t[2]);
        detailed_tris.push(sub_t[3]);
    }
    detailed_tris
}

// Create an octahedron and subdivide all faces once, and validate
#[test]
fn test_gct1() -> Result<(), Box<dyn std::error::Error>> {
    let mut sd = SphericalData::default();
    let pts: Vec<_> = shapes::OCTA_POINTS
        .iter()
        .map(|p| sd.add_initial_point(p.into()))
        .collect();

    let tris: Vec<_> = shapes::OCTAHEDRON
        .iter()
        .map(|(p0, p1, p2)| sd.add_initial_gc_triangle(pts[*p0], pts[*p1], pts[*p2]))
        .collect();

    let detailed_tris = subdivide_faces(&mut sd, &tris);

    for d in &detailed_tris {
        sd[*d].validate(&sd, 0.083, 0.118)?;
    }

    for (i, p) in sd.iter_points().enumerate() {
        for (j, q) in sd.iter_points().enumerate() {
            if i == j {
                continue;
            }
            assert!(
                p.vector().dot(q.vector()) < 0.999,
                "Points {i} {j} have identical vectors {p:?} and {q:?}"
            );
        }
    }

    // assert!(false, "Force fail");
    Ok(())
}

// Create an octahedron and subdivide all faces twice, and validate, then four times, and validate
//
// The volume of a sphere radius 1 is 4/3pi = 4.18879, and the surface area is 12.5663706
// Since each subdivision leads to 4 times as many faces, this yields 8*4^4=2048 faces
//
// Each face, if equal, should have an area of 4pi/2048 = 6.136E-3, and the
// segments a volume of 2.05E-3
//
// The triangles have angle-at-centre-of-sphere of between 5.6250 and 8.7460
// degrees (the former is from the octahedron angle of 90 divided by 2^4)
#[test]
fn test_gct2() -> Result<(), Box<dyn std::error::Error>> {
    let mut sd = SphericalData::default();
    let pts: Vec<_> = shapes::OCTA_POINTS
        .iter()
        .map(|p| sd.add_initial_point(p.into()))
        .collect();

    let tris: Vec<_> = shapes::OCTAHEDRON
        .iter()
        .map(|(p0, p1, p2)| sd.add_initial_gc_triangle(pts[*p0], pts[*p1], pts[*p2]))
        .collect();

    let mut total_volume = 0.0;
    for d in &tris {
        total_volume += sd[*d].validate(&sd, 0.166, 0.167)?;
    }
    eprintln!("Total volume {total_volume}");

    let detailed_tris = subdivide_faces(&mut sd, &tris);
    let mut total_volume = 0.0;
    for d in &detailed_tris {
        total_volume += sd[*d].validate(&sd, 0.08, 0.12)?;
    }
    eprintln!("Total volume {total_volume}");

    let detailed_tris = subdivide_faces(&mut sd, &detailed_tris);
    let mut total_volume = 0.0;
    for d in &detailed_tris {
        total_volume += sd[*d].validate(&sd, 0.024, 0.0454)?;
    }
    eprintln!("Total volume {total_volume}");

    let detailed_tris = subdivide_faces(&mut sd, &detailed_tris);
    let mut total_volume = 0.0;
    for d in &detailed_tris {
        total_volume += sd[*d].validate(&sd, 6.34E-3, 13.0E-3)?;
    }
    eprintln!("Total volume {total_volume}");

    // Note the expectation of an average of 2.05E-3
    let detailed_tris = subdivide_faces(&mut sd, &detailed_tris);
    let mut total_volume = 0.0;
    for d in &detailed_tris {
        total_volume += sd[*d].validate(&sd, 1.6E-3, 3.35E-3)?;
    }
    eprintln!("Total volume {total_volume}");

    for (i, p) in sd.iter_points().enumerate() {
        for (j, q) in sd.iter_points().enumerate() {
            if i == j {
                continue;
            }
            assert!(
                p.vector().dot(q.vector()) < 0.999,
                "Points {i} {j} have identical vectors {p:?} and {q:?}"
            );
        }
    }

    // dbg!(&sd);
    // assert!(false, "Force fail");
    Ok(())
}

#[test]
fn test_gct0() -> Result<(), Box<dyn std::error::Error>> {
    let mut sd = SphericalData::default();
    let xp = sd.add_initial_point(shapes::OCTA_POINTS[0].into());
    let xm = sd.add_initial_point(shapes::OCTA_POINTS[1].into());
    let yp = sd.add_initial_point(shapes::OCTA_POINTS[2].into());
    let ym = sd.add_initial_point(shapes::OCTA_POINTS[3].into());
    let zp = sd.add_initial_point(shapes::OCTA_POINTS[4].into());
    let zm = sd.add_initial_point(shapes::OCTA_POINTS[5].into());

    let t_ppp = sd.add_initial_gc_triangle(xp, yp, zp);
    let t_mpp = sd.add_initial_gc_triangle(yp, xm, zp);
    let t_mmp = sd.add_initial_gc_triangle(xm, ym, zp);
    let t_pmp = sd.add_initial_gc_triangle(ym, xp, zp);

    let t_ppm = sd.add_initial_gc_triangle(xp, zm, yp);
    let t_mpm = sd.add_initial_gc_triangle(yp, zm, xm);
    let t_mmm = sd.add_initial_gc_triangle(xm, zm, ym);
    let t_pmm = sd.add_initial_gc_triangle(ym, zm, xp);

    sd[t_ppp].validate(&sd, 0.166, 0.167)?;
    sd[t_mpp].validate(&sd, 0.166, 0.167)?;
    sd[t_pmp].validate(&sd, 0.166, 0.167)?;
    sd[t_mmp].validate(&sd, 0.166, 0.167)?;

    sd[t_ppm].validate(&sd, 0.166, 0.167)?;
    sd[t_mpm].validate(&sd, 0.166, 0.167)?;
    sd[t_pmm].validate(&sd, 0.166, 0.167)?;
    sd[t_mmm].validate(&sd, 0.166, 0.167)?;

    eprintln!("Finding triangles");
    assert!(
        sd.find_gc_triangle_of_points(ym, zm, xp).is_some(),
        "Triangle {:?}, {:?}, {:?} should be present",
        ym,
        zm,
        xp,
    );
    assert!(
        sd.find_gc_triangle_of_points(ym, xp, zm).is_some(),
        "Triangle {:?}, {:?}, {:?} should be present",
        ym,
        xp,
        zm,
    );
    assert!(
        sd.find_gc_triangle_of_points(xp, ym, zm).is_some(),
        "Triangle {:?}, {:?}, {:?} should be present",
        xp,
        ym,
        zm,
    );
    assert!(
        sd.find_gc_triangle_of_points(xp, zm, ym).is_some(),
        "Triangle {:?}, {:?}, {:?} should be present",
        xp,
        zm,
        ym,
    );
    assert!(
        sd.find_gc_triangle_of_points(zm, ym, xp).is_some(),
        "Triangle {:?}, {:?}, {:?} should be present",
        zm,
        ym,
        xp,
    );
    assert!(
        sd.find_gc_triangle_of_points(zm, xp, ym).is_some(),
        "Triangle {:?}, {:?}, {:?} should be present",
        zm,
        xp,
        ym,
    );

    Ok(())
}
