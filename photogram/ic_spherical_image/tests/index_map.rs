use geo_nd::vector;
use ic_base::Point3D;
use ic_spherical_image::shapes;
use ic_spherical_image::{SdIndex, SphericalData};

// Create an icosahedron and test point indexing
#[test]
fn test_icos() -> Result<(), Box<dyn std::error::Error>> {
    let mut sd = SphericalData::default();
    let pts: Vec<_> = shapes::ICOS_POINTS
        .iter()
        .map(|p| sd.add_initial_point(p.into()))
        .collect();

    let _tris: Vec<_> = shapes::ICOSAHEDRON
        .iter()
        .map(|(p0, p1, p2)| sd.add_initial_gc_triangle(pts[*p0], pts[*p1], pts[*p2]))
        .collect();

    let index = SdIndex::new(&sd, sd.iter_triangle_indices());

    let p = [0.1, 0.2, 0.3].into();
    let t = index
        .map_vector(&p)
        .expect("All points should be in the index");
    assert_eq!(
        sd[t].point_outside_lines(&sd, &p),
        0,
        "Point must not be at all outside the triangle"
    );

    for x in 0..1000 {
        let x = (x as f64) / 1000.0;
        for y in 0..1000 {
            let y = (y as f64) / 1000.0;
            let p: Point3D = vector::uniform_dist_sphere3([x, y], true).into();
            sd.find_gc_triangle_of_vector(&p, 0).unwrap();
        }
    }

    let t_vec: Vec<_> = sd
        .iter_triangle_indices()
        .filter(|t| sd[*t].subdivision_path().subdivision_matches(0))
        .collect();
    for t in t_vec {
        sd.subdivide_triangle(t);
    }

    let index = SdIndex::new(
        &sd,
        sd.iter_triangle_indices()
            .filter(|t| sd[*t].subdivision_path().subdivision_matches(1)),
    );

    //    let index = SdIndex::new(&sd, sd.iter_triangle_indicess());

    for x in 0..1000 {
        let x = (x as f64) / 1000.0;
        for y in 0..1000 {
            let y = (y as f64) / 1000.0;
            let p: Point3D = vector::uniform_dist_sphere3([x, y], true).into();
            let t = index
                .map_vector(&p)
                .expect("All points should be in the index");
            assert_eq!(
                sd[t].point_outside_lines(&sd, &p),
                0,
                "Point must not be at all outside the triangle"
            );
            let tf = sd.find_gc_triangle_of_vector(&p, 1).unwrap();
            assert_eq!(tf, t, "Index should return same triangle as the find");
        }
    }

    // assert!(false, "Force fail");
    Ok(())
}

#[test]
fn test_octa() -> Result<(), Box<dyn std::error::Error>> {
    let mut sd = SphericalData::default();
    let pts: Vec<_> = shapes::OCTA_POINTS
        .iter()
        .map(|p| sd.add_initial_point(p.into()))
        .collect();

    let _tris: Vec<_> = shapes::OCTAHEDRON
        .iter()
        .map(|(p0, p1, p2)| sd.add_initial_gc_triangle(pts[*p0], pts[*p1], pts[*p2]))
        .collect();

    let index = SdIndex::new(&sd, sd.iter_triangle_indices());

    let p = [0.1, 0.2, 0.3].into();
    let t = index
        .map_vector(&p)
        .expect("All points should be in the index");
    assert_eq!(
        sd[t].point_outside_lines(&sd, &p),
        0,
        "Point must not be at all outside the triangle"
    );

    for x in 0..1000 {
        let x = (x as f64) / 1000.0;
        for y in 0..1000 {
            let y = (y as f64) / 1000.0;
            let p: Point3D = vector::uniform_dist_sphere3([x, y], true).into();
            sd.find_gc_triangle_of_vector(&p, 0).unwrap();
        }
    }

    let t_vec: Vec<_> = sd
        .iter_triangle_indices()
        .filter(|t| sd[*t].subdivision_path().subdivision_matches(0))
        .collect();
    for t in t_vec {
        sd.subdivide_triangle(t);
    }

    let index = SdIndex::new(
        &sd,
        sd.iter_triangle_indices()
            .filter(|t| sd[*t].subdivision_path().subdivision_matches(1)),
    );

    for x in 0..1000 {
        let x = (x as f64) / 1000.0;
        for y in 0..1000 {
            let y = (y as f64) / 1000.0;
            let p: Point3D = vector::uniform_dist_sphere3([x, y], true).into();
            let t = index
                .map_vector(&p)
                .expect("All points should be in the index");
            assert_eq!(
                sd[t].point_outside_lines(&sd, &p),
                0,
                "Point must not be at all outside the triangle"
            );
            let tf = sd.find_gc_triangle_of_vector(&p, 1).unwrap();
            assert_eq!(tf, t, "Index should return same triangle as the find");
        }
    }

    let t_vec: Vec<_> = sd
        .iter_triangle_indices()
        .filter(|t| sd[*t].subdivision_path().subdivision_matches(1))
        .collect();
    for t in t_vec {
        sd.subdivide_triangle(t);
    }

    let index = SdIndex::new(
        &sd,
        sd.iter_triangle_indices()
            .filter(|t| sd[*t].subdivision_path().subdivision_matches(2)),
    );
    // let index = SdIndex::new(&sd);/

    for x in 0..1000 {
        let x = (x as f64) / 1000.0;
        for y in 0..1000 {
            let y = (y as f64) / 1000.0;
            let p: Point3D = vector::uniform_dist_sphere3([x, y], true).into();
            let t = index
                .map_vector(&p)
                .expect("All points should be in the index");
            assert_eq!(
                sd[t].point_outside_lines(&sd, &p),
                0,
                "Point must not be at all outside the triangle"
            );
            let tf = sd.find_gc_triangle_of_vector(&p, 2).unwrap();
            assert_eq!(tf, t, "Index should return same triangle as the find");
        }
    }

    // assert!(false, "Force fail");
    Ok(())
}

#[test]
fn test_tetra() -> Result<(), Box<dyn std::error::Error>> {
    let mut sd = SphericalData::default();
    let pts: Vec<_> = shapes::TETRA_POINTS
        .iter()
        .map(|p| sd.add_initial_point(p.into()))
        .collect();

    let _tris: Vec<_> = shapes::TETRAHEDRON
        .iter()
        .map(|(p0, p1, p2)| sd.add_initial_gc_triangle(pts[*p0], pts[*p1], pts[*p2]))
        .collect();

    let index = SdIndex::new(&sd, sd.iter_triangle_indices());

    let p = [0.1, 0.2, 0.3].into();
    let t = index
        .map_vector(&p)
        .expect("All points should be in the index");
    assert_eq!(
        sd[t].point_outside_lines(&sd, &p),
        0,
        "Point must not be at all outside the triangle"
    );

    for x in 0..1000 {
        let x = (x as f64) / 1000.0;
        for y in 0..1000 {
            let y = (y as f64) / 1000.0;
            let p: Point3D = vector::uniform_dist_sphere3([x, y], true).into();
            sd.find_gc_triangle_of_vector(&p, 0).unwrap();
        }
    }

    let t_vec: Vec<_> = sd
        .iter_triangle_indices()
        .filter(|t| sd[*t].subdivision_path().subdivision_matches(0))
        .collect();
    for t in t_vec {
        sd.subdivide_triangle(t);
    }

    let index = SdIndex::new(
        &sd,
        sd.iter_triangle_indices()
            .filter(|t| sd[*t].subdivision_path().subdivision_matches(1)),
    );

    for x in 0..1000 {
        let x = (x as f64) / 1000.0;
        for y in 0..1000 {
            let y = (y as f64) / 1000.0;
            let p: Point3D = vector::uniform_dist_sphere3([x, y], true).into();
            let t = index
                .map_vector(&p)
                .expect("All points should be in the index");
            assert_eq!(
                sd[t].point_outside_lines(&sd, &p),
                0,
                "Point must not be at all outside the triangle"
            );
            let tf = sd.find_gc_triangle_of_vector(&p, 1).unwrap();
            assert_eq!(tf, t, "Index should return same triangle as the find");
        }
    }

    let t_vec: Vec<_> = sd
        .iter_triangle_indices()
        .filter(|t| sd[*t].subdivision_path().subdivision_matches(1))
        .collect();
    for t in t_vec {
        sd.subdivide_triangle(t);
    }

    let index = SdIndex::new(
        &sd,
        sd.iter_triangle_indices()
            .filter(|t| sd[*t].subdivision_path().subdivision_matches(2)),
    );
    // let index = SdIndex::new(&sd);/

    for x in 0..1000 {
        let x = (x as f64) / 1000.0;
        for y in 0..1000 {
            let y = (y as f64) / 1000.0;
            let p: Point3D = vector::uniform_dist_sphere3([x, y], true).into();
            let t = index
                .map_vector(&p)
                .expect("All points should be in the index");
            assert_eq!(
                sd[t].point_outside_lines(&sd, &p),
                0,
                "Point must not be at all outside the triangle"
            );
            let tf = sd.find_gc_triangle_of_vector(&p, 2).unwrap();
            assert_eq!(tf, t, "Index should return same triangle as the find");
        }
    }

    // assert!(false, "Force fail");
    Ok(())
}
