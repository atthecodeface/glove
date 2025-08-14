use ic_mesh::IndexTriangle;
use ic_mesh::Mesh;
use ic_mesh::TriangleIndex;

//a Tests
//fi assert_triangle
#[cfg(test)]
#[track_caller]
fn assert_triangle(t: &IndexTriangle, (p0, p1, p2): (usize, usize, usize)) {
    eprintln!("Check triangle {t} includes {p0} {p1} {p2}");
    assert!(t.contains_pt(p0));
    assert!(t.contains_pt(p1));
    assert!(t.contains_pt(p2));
}

//fi assert_mesh_triangle
#[cfg(test)]
#[track_caller]
fn assert_mesh_triangle(mesh: &Mesh, t: usize, p012: (usize, usize, usize)) {
    let t: TriangleIndex = t.into();
    assert_triangle(&mesh[t], p012);
}

//fi assert_mesh_has_triangle
/// Assert that the mesh has a triangle with the points in anticlockwise order
#[cfg(test)]
#[track_caller]
fn assert_mesh_has_triangle(mesh: &Mesh, (p0, p1, p2): (usize, usize, usize)) {
    for (tp0, tp1, tp2) in mesh.triangle_pts() {
        if tp0 == p0 {
            if (tp1 == p1 && tp2 == p2) {
                return;
            }
        } else if tp0 == p1 {
            if (tp1 == p2 && tp2 == p0) {
                return;
            }
        } else {
            if (tp0 == p2 && tp1 == p0 && tp2 == p1) {
                return;
            }
        }
    }
    assert!(
        false,
        "Failed to find triangle {p0} {p1} {p2} in mesh\n{mesh:?}"
    );
}

//ft test_sweep
#[test]
fn test_sweep() -> Result<(), String> {
    let mut mesh = Mesh::default();
    mesh.add_pt([1., 0.].into());
    mesh.add_pt([0., 0.].into());
    mesh.add_pt([0., 1.].into());
    mesh.add_pt([1., 1.].into());
    assert_eq!(mesh.find_lbx_pt(), 1);
    let sweep = mesh.find_sweep();
    assert_eq!(&sweep.1, &[1, 0, 3, 2]);
    Ok(())
}

//ft test_sweep2
/// create a mesh with 0, 0 as the origin
///
/// Then in anticlockwise order 5, 0, 4, 3, 7, 2, 6
#[test]
fn test_sweep2() -> Result<(), String> {
    let mut mesh = Mesh::default();
    mesh.add_pt([1., 0.].into());
    mesh.add_pt([0., 0.].into());
    mesh.add_pt([0., 1.].into());
    mesh.add_pt([1., 1.].into());
    mesh.add_pt([2., 0.].into());
    mesh.add_pt([2., -1.].into());
    mesh.add_pt([0., 3.].into());
    mesh.add_pt([1., 4.].into());

    assert_eq!(mesh.find_lbx_pt(), 1);

    let sweep = mesh.find_sweep();

    assert_eq!(&sweep.1, &[1, 5, 0, 4, 3, 7, 2, 6]);

    let lbx = mesh[sweep.1[0]];
    for i in 1..(sweep.1.len() - 1) {
        let p0 = mesh[sweep.1[i]];
        let p1 = mesh[sweep.1[i + 1]];
        let p0 = p0 - lbx;
        let p1 = p1 - lbx;
        assert!(p0[0].atan2(p0[1]) >= p1[0].atan2(p1[1]));
    }
    Ok(())
}

//ft test_hull
/// Just do a square - since the diagonals are equal, cannot say which
/// diagonal is chosen
///
/// This just does test that it runs
#[test]
fn test_hull() -> Result<(), String> {
    let mut mesh = Mesh::default();
    mesh.add_pt([1., 0.].into());
    mesh.add_pt([0., 0.].into());
    mesh.add_pt([0., 1.].into());
    mesh.add_pt([1., 1.].into());
    mesh.create_mesh_triangles();
    mesh.optimize_mesh_quads();
    Ok(())
}

//ft test_hull1
/// A diamond, and the lines should be swapped.
///
/// There must end up being a triangle (0,1,3) and another (1,2,3)
#[test]
fn test_hull1() -> Result<(), String> {
    let mut mesh = Mesh::default();
    mesh.add_pt([0., 0.].into());
    mesh.add_pt([1., 1.].into());
    mesh.add_pt([1., 2.].into());
    mesh.add_pt([0., 1.].into());
    mesh.create_mesh_triangles();

    while mesh.optimize_mesh_quads() {}

    assert_mesh_triangle(&mesh, 0, (0, 1, 3));
    assert_mesh_triangle(&mesh, 1, (1, 2, 3));

    Ok(())
}

//ft test_hull2
#[test]
fn test_hull2() -> Result<(), String> {
    let mut mesh = Mesh::default();
    mesh.add_pt([0., 0.].into());
    mesh.add_pt([1., 0.].into());
    mesh.add_pt([1., 1.].into());
    mesh.add_pt([0.3, 0.4].into());
    mesh.add_pt([0., 1.].into());
    mesh.create_mesh_triangles();

    while mesh.optimize_mesh_quads() {}

    assert_mesh_has_triangle(&mesh, (0, 1, 3));
    assert_mesh_has_triangle(&mesh, (1, 2, 3));
    assert_mesh_has_triangle(&mesh, (0, 3, 4));
    assert_mesh_has_triangle(&mesh, (2, 4, 3));

    Ok(())
}

//ft test_hull3
#[test]
fn test_hull3() -> Result<(), String> {
    let mut mesh = Mesh::default();
    mesh.add_pt([0., 0.].into());
    mesh.add_pt([10., 0.].into());
    mesh.add_pt([0., 10.].into());
    mesh.add_pt([5.03, 2.].into()); // to make it consistent
    mesh.add_pt([2., 5.].into());
    mesh.add_pt([5., 8.].into());
    mesh.add_pt([8.01, 5.].into()); // to make it consistent
    mesh.add_pt([10., 10.].into());
    mesh.create_mesh_triangles();

    assert_mesh_has_triangle(&mesh, (0, 1, 3));
    assert_mesh_has_triangle(&mesh, (0, 3, 6));
    assert_mesh_has_triangle(&mesh, (0, 6, 7));
    assert_mesh_has_triangle(&mesh, (0, 7, 5));
    assert_mesh_has_triangle(&mesh, (0, 5, 4));
    assert_mesh_has_triangle(&mesh, (0, 4, 2));
    assert_mesh_has_triangle(&mesh, (1, 6, 3));
    assert_mesh_has_triangle(&mesh, (1, 7, 6));
    assert_mesh_has_triangle(&mesh, (5, 2, 4));
    assert_mesh_has_triangle(&mesh, (7, 2, 5));

    while mesh.optimize_mesh_quads() {}

    for (i, t) in mesh.triangle_pts().enumerate() {
        eprintln!("Triangle {i}, {t:?}");
    }

    assert_mesh_has_triangle(&mesh, (0, 1, 3));
    assert_mesh_has_triangle(&mesh, (0, 3, 4));
    assert_mesh_has_triangle(&mesh, (3, 6, 5));
    assert_mesh_has_triangle(&mesh, (6, 7, 5));
    assert_mesh_has_triangle(&mesh, (3, 5, 4));
    assert_mesh_has_triangle(&mesh, (0, 4, 2));
    assert_mesh_has_triangle(&mesh, (1, 6, 3));
    assert_mesh_has_triangle(&mesh, (1, 7, 6));
    assert_mesh_has_triangle(&mesh, (5, 2, 4));
    assert_mesh_has_triangle(&mesh, (7, 2, 5));

    Ok(())
}

//ft test_hull4
#[test]
fn test_hull4() -> Result<(), String> {
    let mut mesh = Mesh::default();
    mesh.add_pt([0., 0.].into());
    mesh.add_pt([10., 0.].into());
    mesh.add_pt([10., 10.].into());
    mesh.add_pt([0., 10.].into());
    mesh.add_pt([2., 2.].into()); // to make it consistent
    mesh.add_pt([5., 5.].into());
    mesh.create_mesh_triangles();

    eprintln!("{:?}", mesh);
    for (i, t) in mesh.triangle_pts().enumerate() {
        eprintln!("Triangle {i}, {t:?}");
    }
    for (i, p) in mesh.points().enumerate() {
        eprintln!("Point {i}, {:?}", mesh[p]);
    }
    for (i, l) in mesh.lines().enumerate() {
        eprintln!("Line {i}, {:?}", mesh[l]);
    }

    assert_mesh_has_triangle(&mesh, (0, 1, 4));
    assert_mesh_has_triangle(&mesh, (0, 4, 5));
    assert_mesh_has_triangle(&mesh, (0, 5, 2));
    assert_mesh_has_triangle(&mesh, (0, 2, 3));
    assert_mesh_has_triangle(&mesh, (1, 5, 4));
    assert_mesh_has_triangle(&mesh, (1, 2, 5));

    eprintln!("Remove zero area triangles");
    mesh.remove_zero_area_triangles();

    eprintln!("{:?}", mesh);
    for (i, t) in mesh.triangle_pts().enumerate() {
        eprintln!("Triangle {i}, {t:?}");
    }
    for (i, p) in mesh.points().enumerate() {
        eprintln!("Point {i}, {:?}", mesh[p]);
    }
    for (i, l) in mesh.lines().enumerate() {
        eprintln!("Line {i}, {:?}", mesh[l]);
    }

    assert_mesh_has_triangle(&mesh, (0, 1, 4));
    assert_mesh_has_triangle(&mesh, (0, 4, 5));
    assert_mesh_has_triangle(&mesh, (0, 5, 3));
    assert_mesh_has_triangle(&mesh, (5, 2, 3));
    assert_mesh_has_triangle(&mesh, (1, 5, 4));
    assert_mesh_has_triangle(&mesh, (1, 2, 5));

    eprintln!("Optimize");
    while mesh.optimize_mesh_quads() {}

    eprintln!("{:?}", mesh);
    for (i, t) in mesh.triangle_pts().enumerate() {
        eprintln!("Triangle {i}, {t:?}");
    }
    for (i, p) in mesh.points().enumerate() {
        eprintln!("Point {i}, {:?}", mesh[p]);
    }
    for (i, l) in mesh.lines().enumerate() {
        eprintln!("Line {i}, {:?}", mesh[l]);
    }

    assert_mesh_has_triangle(&mesh, (0, 1, 4));
    assert_mesh_has_triangle(&mesh, (0, 4, 3));
    assert_mesh_has_triangle(&mesh, (4, 5, 3));
    assert_mesh_has_triangle(&mesh, (5, 2, 3));
    assert_mesh_has_triangle(&mesh, (1, 5, 4));
    assert_mesh_has_triangle(&mesh, (1, 2, 5));

    Ok(())
}

//ft test_hull5
/// Triangle around origin with one point (pt3) *way* left, others just to right (pt1 and pt2)
///
/// This would want to optimize the line origin pt0->pt3 for triangles
/// pt0,pt1,pt3 and pt0,pt2,pt3.
///
/// But the angle at the origin would make that illegal (it would want
/// to create a negative triangle)
#[test]
fn test_hull5() -> Result<(), String> {
    let mut mesh = Mesh::default();
    mesh.add_pt([0., 0.].into());
    mesh.add_pt([1., 1.].into());
    mesh.add_pt([1., -1.].into());
    mesh.add_pt([-20., 0.].into());
    mesh.create_mesh_triangles();

    eprintln!("{:?}", mesh);
    for (i, t) in mesh.triangle_pts().enumerate() {
        eprintln!("Triangle {i}, {t:?}");
    }
    for (i, p) in mesh.points().enumerate() {
        eprintln!("Point {i}, {:?}", mesh[p]);
    }
    for (i, l) in mesh.lines().enumerate() {
        eprintln!("Line {i}, {:?}", mesh[l]);
    }

    assert_mesh_has_triangle(&mesh, (0, 2, 1));
    assert_mesh_has_triangle(&mesh, (0, 1, 3));
    assert_mesh_has_triangle(&mesh, (0, 3, 2));

    eprintln!("Optimize");
    while mesh.optimize_mesh_quads() {}

    eprintln!("{:?}", mesh);
    for (i, t) in mesh.triangle_pts().enumerate() {
        eprintln!("Triangle {i}, {t:?}");
    }
    for (i, p) in mesh.points().enumerate() {
        eprintln!("Point {i}, {:?}", mesh[p]);
    }
    for (i, l) in mesh.lines().enumerate() {
        eprintln!("Line {i}, {:?}", mesh[l]);
    }

    assert_mesh_has_triangle(&mesh, (0, 2, 1));
    assert_mesh_has_triangle(&mesh, (0, 1, 3));
    assert_mesh_has_triangle(&mesh, (0, 3, 2));

    Ok(())
}
