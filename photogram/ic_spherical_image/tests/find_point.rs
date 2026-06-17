use geo_nd::{Vector, vector};
use ic_base::Point3D;
use ic_spherical_image::{SdIndex, SphericalData, shapes};

#[test]
fn test_find_icos_pt() -> Result<(), Box<dyn std::error::Error>> {
    let mut sd = SphericalData::default();
    let pts: Vec<_> = shapes::ICOS_POINTS
        .iter()
        .map(|p| sd.add_initial_point(p.into()))
        .collect();

    for (p0, p1, p2) in shapes::ICOSAHEDRON {
        sd.add_initial_gc_triangle(pts[*p0], pts[*p1], pts[*p2]);
    }

    let index = SdIndex::new(&sd);

    let p: Point3D = [0.1, 0.2, 0.3].into();
    let p = p.normalize();

    let t = index
        .map_vector(&p)
        .expect("All points should be in the index");
    assert_eq!(
        sd[t].point_outside_lines(&sd, &p),
        0,
        "Point must be inside the triangle the index indicated"
    );

    for subdivision in 0..10 {
        let st = sd.find_subtriangle_of_point_in_triangle(t, &p, subdivision);

        let t3 = st.to_triangle3d_on_sphere();
        t3.validate();

        let b: Point3D = t3.barycentric_coordinates(&p).into();
        let pt = t3.of_barycentric_coordinates(&[b[0], b[1], b[2]]); //.normalize();
        eprintln!(
            "Barycentric coords of {p:0.4} are {b:0.4} {pt:0.4} ==? p_sc:{:0.4}",
            t3.point_projected_onto_by_scaling(&p)
        );

        assert!(
            t3.contains_point(&p),
            "Expect the triangle that found the point contains the point"
        );
    }
    // assert!(false, "Force fail");
    Ok(())
}
