use geo_nd::Vector;
use ic_spherical_image::shapes::*;
use ic_spherical_image::{GreatCircleTriangleIndex, SphericalData};

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

fn validate_sd(sd: &SphericalData) -> Result<(), Box<dyn std::error::Error>> {
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
    for (_i, t) in sd.iter_triangles().enumerate() {
        t.validate(sd, 0.0, 2.0)?;
    }
    Ok(())
}

// Create an tetrahedron and subdivide all faces three times
//
// The triangles have angle-at-centre-of-sphere of between 13.6839 and 33.5573
// degrees (the former is from the tetrahedron angle of 109.5 divided by 2^3)
//
// This is clearly quite irregular in scale factor (as expected)
#[test]
fn test_tetra() -> Result<(), Box<dyn std::error::Error>> {
    let mut sd = SphericalData::default();
    let pts: Vec<_> = TETRA_POINTS
        .iter()
        .map(|p| sd.add_initial_point(p.into()))
        .collect();

    let tris: Vec<_> = TETRAHEDRON
        .iter()
        .map(|(p0, p1, p2)| sd.add_initial_gc_triangle(pts[*p0], pts[*p1], pts[*p2]))
        .collect();

    validate_sd(&sd)?;
    let tris = subdivide_faces(&mut sd, &tris);

    let mut total_volume = 0.0;
    for d in &tris {
        total_volume += sd[*d].validate(&sd, 0.096, 0.167)?;
    }
    eprintln!("Total volume {total_volume}");

    let tris = subdivide_faces(&mut sd, &tris);
    let mut total_volume = 0.0;
    for d in &tris {
        total_volume += sd[*d].validate(&sd, 0.03, 0.118)?;
    }
    eprintln!("Total volume {total_volume}");

    let tris = subdivide_faces(&mut sd, &tris);
    let mut total_volume = 0.0;
    for d in &tris {
        total_volume += sd[*d].validate(&sd, 8E-3, 45.4E-3)?;
    }
    eprintln!("Total volume {total_volume}");

    validate_sd(&sd)?;
    // assert!(false, "Force fail");
    Ok(())
}

// Create an icosahedron and subdivide all faces four times
//
// This produces 20*4^4 = 5120 faces
//
// The triangles have angle-at-centre-of-sphere of between 3.9647 and 4.7342
// degrees (the former is from the tetrahedron angle of 63.4349 divided by 2^4)
#[test]
fn test_icos() -> Result<(), Box<dyn std::error::Error>> {
    let mut sd = SphericalData::default();
    let pts: Vec<_> = ICOS_POINTS
        .iter()
        .map(|p| sd.add_initial_point(p.into()))
        .collect();

    let tris: Vec<_> = ICOSAHEDRON
        .iter()
        .map(|(p0, p1, p2)| sd.add_initial_gc_triangle(pts[*p0], pts[*p1], pts[*p2]))
        .collect();
    // dbg!(&sd);

    let mut total_volume = 0.0;
    for d in &tris {
        total_volume += sd[*d].validate(&sd, 0.12680, 0.12681)?;
    }
    eprintln!("Total volume {total_volume}");

    validate_sd(&sd)?;
    let tris = subdivide_faces(&mut sd, &tris);

    let mut total_volume = 0.0;
    for d in &tris {
        total_volume += sd[*d].validate(&sd, 43.8E-3, 51.6E-3)?;
    }
    eprintln!("Total volume {total_volume}");

    let tris = subdivide_faces(&mut sd, &tris);
    let mut total_volume = 0.0;
    for d in &tris {
        total_volume += sd[*d].validate(&sd, 11.8E-3, 15.0E-3)?;
    }
    eprintln!("Total volume {total_volume}");

    let tris = subdivide_faces(&mut sd, &tris);
    let mut total_volume = 0.0;
    for d in &tris {
        total_volume += sd[*d].validate(&sd, 3.01E-3, 3.90E-3)?;
    }
    eprintln!("Total volume {total_volume}");

    let tris = subdivide_faces(&mut sd, &tris);
    let mut total_volume = 0.0;
    for d in &tris {
        total_volume += sd[*d].validate(&sd, 7.57E-4, 9.84E-4)?;
    }
    eprintln!("Total volume {total_volume}");

    validate_sd(&sd)?;
    //    assert!(false, "Force fail");
    Ok(())
}
