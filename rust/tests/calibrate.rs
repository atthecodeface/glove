//a Modules
use glove::calibrate::PointMapping;
// use glove::calibrate::Projection;
use glove::calibrate::*;
use glove::calibrate::{LCamera, Rotations};
use glove::calibrate::{Point2D, Point3D}; // , Point4D, Quat};

use geo_nd::Vector;
use geo_nd::{matrix, quat};

//a Tests
// CAM 0
// WE 28 4.12 20.86 Camera @[-197.71,-200.37,435.25] yaw -20.09 pitch -16.35 + [-0.33,-0.28,0.90]

// CAM 1
// WE 27 8.94 51.26 Camera @[-1.92,  -4.18,782.75] yaw -1.14 pitch -4.12 + [-0.02,-0.07,1.00]
// WE 99 3.58 22.17 Camera @[-21.98,112.49,768.56] yaw -2.56 pitch 4.43 + [-0.04,0.08,1.00]
// WE 92 5.81 31.90 Camera @[-29.29, 76.42,776.45] yaw -3.12 pitch 1.75 + [-0.05,0.03,1.00]
// WE 62 4.26 26.16 Camera @[-26.37, 85.59,773.16] yaw -2.89 pitch 2.45 + [-0.05,0.04,1.00]
// WE 17 1.09  7.37 Camera @[-75.64,172.25,745.83] yaw -6.61 pitch 9.02 + [-0.11,0.16,0.98]
// WE    4.85       Camera @[-83.37,151.51,743.16] yaw -7.26 pitch 7.57 + [-0.13,0.13,0.98]
// WE 63 7.25 20.16 Camera @[-95.45,156.38,737.22] yaw -8.19 pitch 7.99 + [-0.14,0.14,0.98]
// #[test]
fn test_find_coarse_position() {
    let camera = LCamera::new(
        [0., 0., 0.].into(),
        quat::look_at(&[-220., -310., -630.], &[0.10, -1., -0.1]).into(),
    );
    let mappings: Vec<PointMapping> = C1_DATA_ALL
        .iter()
        .map(|(model, screen)| {
            PointMapping::new(&Point3D::from_array(*model), &Point2D::from_array(*screen))
        })
        .collect();
    let cam = camera.find_coarse_position(&mappings, &[1000., 1000., 2000.], 51);
    let cam = cam.find_coarse_position(&mappings, &[300., 300., 650.], 31);
    let cam = cam.find_coarse_position(&mappings, &[100., 100., 200.], 31);
    let cam = cam.find_coarse_position(&mappings, &[30., 30., 65.], 31);
    let cam = cam.find_coarse_position(&mappings, &[10., 10., 20.], 31);
    let te = cam.total_error(&mappings);
    let we = cam.worst_error(&mappings);
    eprintln!("Final WE {:.2} {:.2} Camera {}", we, te, cam);

    let mut cam = cam;
    let num = mappings.len();
    // let coarse_rotations = Rotations::new(1.0_f64.to_radians());
    let fine_rotations = Rotations::new(0.1_f64.to_radians());
    for _ in 0..100 {
        cam = cam
            .get_best_direction(10000, &fine_rotations, &mappings[0])
            .0;
    }
    let mut worst_data = (1_000_000.0, 0, cam, 0.);
    for i in 0..100 {
        let mut last_n = cam.find_worst_error(&mappings).0;
        for i in 0..30 {
            let n = cam.find_worst_error(&mappings).0;
            // dbg!(i, n, last_n);
            if n == last_n {
                last_n = (last_n + 1 + (i % (num - 1))) % num;
            }
            cam = cam
                .adjust_direction_while_keeping_one_okay(
                    100_000,
                    0.02_f64.to_radians(),
                    &fine_rotations,
                    &|c, m, n| m[n].get_sq_error(c),
                    &mappings,
                    last_n,
                    n,
                )
                .0;
            last_n = n;
        }
        for pm in mappings.iter() {
            pm.show_error(&cam);
        }
        if true {
            cam = cam
                .adjust_position_in_out(&mappings, &|c, m| c.worst_error(m))
                .0;
            cam = cam.adjust_position(&mappings, &|c, m| c.worst_error(m)).0;
        }
        eprintln!("Loop {} completed", i);
        let we = cam.worst_error(&mappings);
        for pm in mappings.iter() {
            pm.show_error(&cam);
        }
        eprintln!("WE {} {:.2}", i, we);
        if we < worst_data.0 {
            eprintln!("WE {} {:.2} Camera {}", i, we, cam);
            worst_data = (we, i, cam, cam.total_error(&mappings));
        }
    }
    eprintln!(
        "Lowest WE {} {:.2} {:.2} Camera {}",
        worst_data.1, worst_data.0, worst_data.3, worst_data.2
    );
    assert!(false);
}

// #[test]
fn test_optimize() {
    // let camera0 = LCamera::new(
    //     [-80., -120., 280.].into(), // 540 mm fromm model 280 for fov 35
    //     quat::look_at(&[-33., -30., -570.], &[0.10, -1., -0.1]).into(),
    // );
    let camera0 = LCamera::new(
        // for C0_DATA_ALL
        // [-196., -204., 435.].into(), // 540 mm fromm model origin?
        // for -201.77,-292.29,648.1
        // [54.10, -32.0, 781.].into(),
        // [-32.10, -7.0, 784.].into(),
        [-22., 32.0, 784.].into(),
        quat::look_at(&[-220., -310., -630.], &[0.10, -1., -0.1]).into(),
    );
    let mappings: Vec<PointMapping> = C1_DATA_ALL
        .iter()
        .map(|(model, screen)| {
            PointMapping::new(&Point3D::from_array(*model), &Point2D::from_array(*screen))
        })
        .collect();

    let da = 0.01_f64.to_radians();
    let rotations = Rotations::new(da);
    let mut cam = camera0;
    for _ in 0..100 {
        cam = cam.get_best_direction(10000, &rotations, &mappings[0]).0;
    }
    let num = mappings.len();
    let mut worst_data = (1_000_000.0, 0, cam, 0.);
    for i in 0..100 {
        let mut last_n = cam.find_worst_error(&mappings).0;
        for i in 0..30 {
            let n = cam.find_worst_error(&mappings).0;
            dbg!(i, n, last_n);
            if n == last_n {
                last_n = (last_n + 1 + (i % (num - 1))) % num;
            }
            cam = cam
                .adjust_direction_while_keeping_one_okay(
                    100_000,
                    0.02_f64.to_radians(),
                    &rotations,
                    &|c, m, n| m[n].get_sq_error(c),
                    &mappings,
                    last_n,
                    n,
                )
                .0;
            last_n = n;
        }
        /*
        for i in 0..100 {
            let best_n = cam.find_best_error(&mappings).0;
            let worst_n = cam.find_worst_error(&mappings).0;
            dbg!(i, best_n, worst_n);
            if best_n == worst_n {
                break;
            }
            cam = cam
                .adjust_direction_while_keeping_one_okay(
                           100_000,
                    0.02_f64.to_radians(),
                    &rotations,
                    // &|c, m, n| m[n].get_sq_error(c),
                    // &|c, m, n| c.total_error(m),
                    &|c, m, n| c.worst_error(m),
                    &mappings,
                    best_n,
                    worst_n,
                )
                .0;
        }
         */
        /*
        for i in 0..num {
            cam = cam
                .adjust_direction_rotating_around_one_point(
                   0.02_f64.to_radians(),
                    // &|c, m, n| m[n].get_sq_error(c),
                    // &|c, m, n| c.total_error(&mappings),
                    &|c, m, n| c.worst_error(&mappings),
                    &mappings,
                    i,
                    0,
                )
                .0;
        }
         */
        dbg!(
            "Total error pre move",
            cam.total_error(&mappings),
            cam.worst_error(&mappings)
        );
        dbg!(cam);
        for pm in mappings.iter() {
            pm.show_error(&cam);
        }
        if true {
            cam = cam
                .adjust_position_in_out(&mappings, &|c, m| c.worst_error(m))
                .0;
            cam = cam.adjust_position(&mappings, &|c, m| c.worst_error(m)).0;
        }
        eprintln!("Loop {} completed", i);
        dbg!(
            "Total error post move",
            cam.total_error(&mappings),
            cam.worst_error(&mappings)
        );
        let we = cam.worst_error(&mappings);
        eprintln!("WE {:.2} Camera {}", we, cam);
        for pm in mappings.iter() {
            pm.show_error(&cam);
        }
        if we < worst_data.0 {
            worst_data = (we, i, cam, cam.total_error(&mappings));
        }
    }
    eprintln!(
        "Lowest WE {} {:.2} {:.2} Camera {}",
        worst_data.1, worst_data.0, worst_data.3, worst_data.2
    );
    assert!(false);
}

fn test_calibrate() {
    let camera0 = LCamera::new(
        [-10., 20., 540.].into(), // 540 mm fromm model
        quat::look_at(&[-33., -130., -540.], &[0.10, -1., -0.1]).into(),
    );
    let mappings: Vec<PointMapping> = C0_DATA
        .iter()
        .map(|(model, screen)| {
            PointMapping::new(&Point3D::from_array(*model), &Point2D::from_array(*screen))
        })
        .collect();
    let da = 0.02_f64.to_radians();
    let rotations = Rotations::new(da);
    let camera0 = camera0
        .get_best_direction(10000, &rotations, mappings.last().unwrap())
        .0;
    for pm in mappings.iter() {
        pm.show_error(&camera0);
    }
    let mut cam = camera0.clone();
    cam = cam.adjust_position(&mappings, &|c, m| c.total_error(m)).0;
    for pm in mappings.iter() {
        pm.show_error(&cam);
    }
    assert!(false);
    // For the given direction and currrent estimate of position we can deduce the
    // view_xyz of each mapping model point [i]
    //
    // That gives an estimate of 'z' for view_xyz (z_est[i])
    //
    // Now, view_xyz[i] = direction * (model[i] - position),
    //
    // and scr_xy_est[i] = view_xyz.xy / z_est[i]
    //
    // Hence scr_xy_est[i] = direction/z_est[i] * model[i] - direction/z_est[i]*position
    //
    // Hence E_sq_x = 1/(z_est[i]^2) * (((d * m[i]) - s[i]*z_est[i]) - (d*p)).x ^ 2
    //
    //              = 1/(z_est[i]^2) * ((d[row 0] * m[i]) - s[i].x*z_est[i] - (d[row 0]*p)) ^ 2
    //
    // let blah = d[row 0] * m[i]) - s[i].x*z_est[i]
    //
    // Hence E_sq_x = 1/(z_est[i]^2) * (blah - d[row 0]*p) ^ 2
    //
    // Hence E_sq_x = 1/(z_est[i]^2) * (blah^2 - 2*blah*d[row 0]*p + (d[row 0]*p) ^ 2)
    //
    // d[row 0]*p   = d[0,0]*p.x + d[1,0]*p.y + d[2,0]*p.z
    // d[row 0]*p^2 = d[0,0]*p.x ^ 2 + 2*d[0,0]*p.x*(d[1,0]*p.y + d[2,0]*p.z) + (d[1,0]*p.y + d[2,0]*p.z)^2
    //
    // d/d(p.x)[ d[row 0]*p ] = d[0,0]
    // d/d(p.x)[ d[row 0]*p^2 ] = 2*d[0,0]*p.x + 2*d[0,0]*(d[1,0]*p.y + d[2,0]*p.z)
    // d/d(p.x)[ d[row 0]*p^2 ] = 2*d[0,0]*(p.x + d[1,0]*p.y + d[2,0]*p.z)
    //
    // d/d(p.y)[ d[row 0]*p ] = d[1,0]
    // d/d(p.y)[ d[row 0]*p^2 ] = 2*d[1,0]*p.y + 2*d[1,0]*(d[0,0]*p.x + d[2,0]*p.z)
    // d/d(p.y)[ d[row 0]*p^2 ] = 2*d[1,0]*(p.y + d[0,0]*p.x + d[2,0]*p.z)
    //
    // dE_sq_x / d(p.x) = 1/(z_est[i]^2) * (2 * blahx[i] * d[0,0] + 2*d[0,0]*(p.x + d[1,0]*p.y + d[2,0]*p.z)
    // dE_sq_x / d(p.x) = 2*d[0,0]/(z_est[i]^2) * (blahx[i] + d[0,0]*p.x + d[1,0]*p.y + d[2,0]*p.z)
    // dE_sq_x / d(p.y) = 2*d[1,0]/(z_est[i]^2) * (blahx[i] + d[0,0]*p.x + d[1,0]*p.y + d[2,0]*p.z)
    // dE_sq_x / d(p.z) = 2*d[2,0]/(z_est[i]^2) * (blahx[i] + d[0,0]*p.x + d[1,0]*p.y + d[2,0]*p.z)
    //
    // Similarly E_sq_y = 1/(z_est[i]^2) * (blahy^2 - 2*blahy*d[row 1]*p + (d[row 1]*p) ^ 2)
    // where blahy[i] =  d[row 1] * m[i]) - s[i].y*z_est[i]
    //
    // dE_sq_y / d(p.x) = 2*d[0,1]/(z_est[i]^2) * (blahy[i] + d[0,1]*p.x + d[1,1]*p.y + d[2,1]*p.z)
    // dE_sq_y / d(p.y) = 2*d[1,1]/(z_est[i]^2) * (blahy[i] + d[0,1]*p.x + d[1,1]*p.y + d[2,1]*p.z)
    // dE_sq_y / d(p.z) = 2*d[2,1]/(z_est[i]^2) * (blahy[i] + d[0,1]*p.x + d[1,1]*p.y + d[2,1]*p.z)
    //
    // dE_sq = Sum(dE_sq_x[i]) + Sum(dE_sq_y[i])
    //
    // We can write dE_sq = M * p + v for some M and v
    //
    // Then when dE_sq = 0, M * p = -v and hence p = - M(inv) * v
    //
    // Sadly M is singular, as each row of the matrix M is a
    //
    // So for a given p estimate (which yields z_est and blahx/y/z[i] and hence M_est, in a sense)
    //
    // we can generate a new 'minimum square error' p
    let mut m = [0.0_f64; 9];
    let mut v = [0.0_f64; 3];
    for pm in mappings.iter() {
        pm.add_sq_error_mat(&camera0, &mut m, &mut v);
    }
    for mv in m.iter() {
        dbg!(1000. * mv);
    }
    // The matrix will be singular
    //
    // (matrix) * px = c0
    let mi = matrix::inverse3(&m);
    let new_p = matrix::transform_vec3(&mi, &v);
    dbg!(camera0.position());
    dbg!(new_p);
    assert!(false);
}
