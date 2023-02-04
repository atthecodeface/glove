//a Modules
use std::rc::Rc;

use image_calibrate::PointMapping;
use image_calibrate::*;
use image_calibrate::{CameraMapping, Rotations};
use image_calibrate::{Point2D, Point3D}; // , Point4D, Quat};

use geo_nd::quat;
use geo_nd::Vector;

//a Consts
//a Tests
//ft test_find_coarse_position_canon_50_v2
#[test]
fn test_find_coarse_position_canon_50_v2() {
    let named_point_set = NamedPointSet::from_json(NOUGHTS_AND_CROSSES_MODEL_JSON).unwrap();

    let cdb = CameraDatabase::from_json(CAMERA_DB_JSON).unwrap();
    // should be 450mm??
    let mut camera = CameraMapping::of_camera(
        CameraInstance::from_json(
            &cdb,
            r#"
{
 "camera": {
"body":"Canon EOS 5D mark IV",
"lens":"EF50mm f1.8",
"mm_focus_distance":453.0
},
 "position":[-250.0,-90.0,250.0],
 "direction":[0.17,0.20,0.95,0.10]
}
"#,
        )
        .unwrap(),
    );
    //    eprintln!("******************************************************************************************");
    //    eprintln!("{}", serde_json::to_string(&camera).unwrap());

    // let mut point_mapping_set = PointMappingSet::new();
    // point_mapping_set.add_mappings(&named_point_set, NAC_4V3A6040);
    // point_mapping_set.add_mappings(&named_point_set, NAC_4V3A6041);
    // point_mapping_set.add_mappings(&named_point_set, NAC_4V3A6042);
    let point_mapping_set =
        PointMappingSet::from_json(&named_point_set, NAC_4V3A6040_JSON).unwrap();

    let mappings = point_mapping_set.mappings();
    let cam = camera;
    let cam = cam.find_coarse_position(mappings, &[3000., 3000., 3000.], 31);
    let cam = cam.find_coarse_position(mappings, &[300., 300., 300.], 31);
    let cam = cam.find_coarse_position(mappings, &[30., 30., 30.], 31);
    let cam = cam.find_coarse_position(mappings, &[3., 3., 3.], 31);

    let mut cam = cam;
    let num = mappings.len();
    for _ in 0..100 {
        for i in 0..num {
            cam = cam
                .adjust_direction_rotating_around_one_point(
                    &|c, m, _n| c.total_error(m),
                    // &|c, m, _n| c.worst_error(m),
                    0.1_f64.to_radians(),
                    mappings,
                    i,
                    0,
                )
                .0;
        }
    }
    let te = cam.total_error(mappings);
    let we = cam.worst_error(mappings);
    cam.show_mappings(mappings);
    cam.show_point_set(&named_point_set);

    eprintln!("Final WE {:.2} {:.2} Camera {}", we, te, cam);
    assert!(we < 300.0, "Worst error should be about 288 but was {}", we);
    assert!(
        te < 1500.0,
        "Total error should be about 1440 but was {}",
        te
    );
}

//ft test_find_coarse_position_canon_inf
// Need at least 4 points to get any sense
#[test]
fn test_find_coarse_position_canon_inf() {
    let sensor = CameraBody::new_35mm(6720, 4480);
    let lens = CameraLens::new("50mm", 50.)
        .set_stw_poly(C50MM_STI_POLY)
        .set_wts_poly(C50MM_ITS_POLY);
    let canon_50mm = CameraPolynomial::new(sensor.into(), lens.into(), 100_000_000.0);
    eprintln!("******************************************************************************************");
    eprintln!("{}", serde_json::to_string(&canon_50mm).unwrap());
    let camera = CameraMapping::new(
        Rc::new(canon_50mm),
        [0., 0., 0.].into(),
        quat::look_at(&[-220., -310., -630.], &[0.10, -1., -0.1]).into(),
    );
    let mut named_point_set = NamedPointSet::new();
    named_point_set.add_set(N_AND_X_TEST_INF);
    let mut point_mapping_set = PointMappingSet::new();
    point_mapping_set.add_mappings(&named_point_set, N_AND_X_TEST_INF_DATA);
    let mappings = point_mapping_set.mappings();
    let disp_mappings = point_mapping_set.mappings();
    let cam = camera;
    // -1500 to +1500 in steps of 100
    let cam = cam.find_coarse_position(&mappings, &[3000., 3000., 3000.], 31);
    let cam = cam.find_coarse_position(&mappings, &[300., 300., 300.], 31);
    let cam = cam.find_coarse_position(&mappings, &[30., 30., 30.], 31);
    let cam = cam.find_coarse_position(&mappings, &[3., 3., 3.], 31);

    let mut cam = cam;
    let num = mappings.len();
    for _ in 0..100 {
        for i in 0..num {
            cam = cam
                .adjust_direction_rotating_around_one_point(
                    &|c, m, _n| c.total_error(m),
                    // &|c, m, _n| c.worst_error(m),
                    0.1_f64.to_radians(),
                    &mappings,
                    i,
                    0,
                )
                .0;
        }
    }
    let te = cam.total_error(&mappings);
    let we = cam.worst_error(&mappings);
    for pm in disp_mappings.iter() {
        cam.show_pm_error(pm);
    }
    eprintln!("Final WE {:.2} {:.2} Camera {}", we, te, cam);
    assert!(we < 300.0, "Worst error should be about 250 but was {}", we);
    assert!(te < 900.0, "Total error should be about 800 but was {}", te);
}

//ft test_find_coarse_position_canon_50cm
const C50MM_50CM_DATA_ALL: &[([f64; 3], [f64; 2])] = &[
    ([0., 0., 0.], [2996.0, 2886.0]),
    ([109., 0., 0.], [5194.0, 1636.0]),
    ([-1., 105., 0.], [1580.0, 1157.0]),
    ([0., 0., 92.], [3002.0, 4023.0]),
    ([108., 0., 89.], [4886.0, 2881.0]),
    ([-1., 105., 90.], [1739.0, 2406.0]),
    // ([108., 109., 0.], [3667.0, 134.0]),
];
const C50MM_50CM_DATA_TEST: &[([f64; 3], [f64; 2])] = &[
    ([0., 0., 0.], [2996.0, 2886.0]),
    ([109., 0., 0.], [5194.0, 1636.0]),
    ([-1., 105., 0.], [1580.0, 1157.0]),
    ([0., 0., 92.], [3002.0, 4023.0]),
    ([108., 0., 89.], [4886.0, 2881.0]),
    ([-1., 105., 90.], [1739.0, 2406.0]),
    ([108., 109., 0.], [3667.0, 134.0]),
    ([107., 109., 0.], [3667.0, 134.0]),
    ([106., 109., 0.], [3667.0, 134.0]),
];
// Need at least 4 points to get any sense
// Distance of 250 mm
// Final WE 824.99 3061.18 Camera @[-137.00,-210.20,-363.40] yaw 24.44 pitch 29.52 + [0.36,0.49,0.79]
// Distance of 386.0 mm
// Final WE 824.49 3006.26 Camera @[-123.80,-189.20,-326.90] yaw 24.86 pitch 29.60 + [0.37,0.49,0.79]
// Distance of 400.0 mm
// Final WE 886.82 3099.49 Camera @[-122.80,-187.80,-324.70] yaw 24.87 pitch 29.60 + [0.37,0.49,0.79]
#[test]
fn test_find_coarse_position_canon_50cm() {
    let cdb = CameraDatabase::from_json(CAMERA_DB_JSON).unwrap();
    let canon_body = cdb.get_body("Canon EOS 5D mark IV").unwrap();
    let lens_50mm = cdb.get_lens("EF50mm f1.8").unwrap();
    let canon_50mm = CameraPolynomial::new(canon_body, lens_50mm, 400.0);
    // 310.0 yields the best
    let camera = CameraMapping::new(
        Rc::new(canon_50mm),
        [0., 0., 0.].into(),
        quat::look_at(&[-220., -310., -630.], &[0.10, -1., -0.1]).into(),
    );
    let mappings: Vec<PointMapping> = C50MM_50CM_DATA_ALL
        .iter()
        .map(|(model, screen)| {
            PointMapping::new(&Point3D::from_array(*model), &Point2D::from_array(*screen))
        })
        .collect();
    let disp_mappings: Vec<PointMapping> = C50MM_50CM_DATA_TEST
        .iter()
        .map(|(model, screen)| {
            PointMapping::new(&Point3D::from_array(*model), &Point2D::from_array(*screen))
        })
        .collect();
    let cam = camera;
    // -1500 to +1500 in steps of 100
    let cam = cam.find_coarse_position(&mappings, &[3000., 3000., 3000.], 31);
    let cam = cam.find_coarse_position(&mappings, &[300., 300., 300.], 31);
    let cam = cam.find_coarse_position(&mappings, &[30., 30., 30.], 31);
    let cam = cam.find_coarse_position(&mappings, &[3., 3., 3.], 31);

    let mut cam = cam;
    let num = mappings.len();
    for _ in 0..100 {
        for i in 0..num {
            cam = cam
                .adjust_direction_rotating_around_one_point(
                    &|c, m, _n| c.total_error(m),
                    // &|c, m, _n| c.worst_error(m),
                    0.1_f64.to_radians(),
                    &mappings,
                    i,
                    0,
                )
                .0;
        }
    }
    let te = cam.total_error(&mappings);
    let we = cam.worst_error(&mappings);
    for pm in disp_mappings.iter() {
        cam.show_pm_error(pm);
    }
    eprintln!("Final WE {:.2} {:.2} Camera {}", we, te, cam);
    assert!(we < 700.0, "Worst error should be about 635 but was {}", we);
    assert!(
        te < 2500.0,
        "Total error should be about 2200 but was {}",
        te
    );
}

//ft test_find_coarse_position
// CAM 0
// WE 28 4.12 20.86 Camera @[-197.71,-200.37,435.25] yaw -20.09 pitch -16.35 + [-0.33,-0.28,0.90]

// CAM 1
// WE 27 8.94 51.26 Camera @[-1.92,  -4.18,782.75] yaw -1.14 pitch -4.12 + [-0.02,-0.07,1.00]
// WE 99 3.58 22.17 Camera @[-21.98,112.49,768.56] yaw -2.56 pitch 4.43 + [-0.04,0.08,1.00]
// WE 92 5.81 31.90 Camera @[-29.29, 76.42,776.45] yaw -3.12 pitch 1.75 + [-0.05,0.03,1.00]
// WE 62 4.26 26.16 Camera @[-26.37, 85.59,773.16] yaw -2.89 pitch 2.45 + [-0.05,0.04,1.00]
// WE  6 1.30  7.18 Camera @[-74.46,180.50,741.43] yaw -6.54 pitch 9.71 + [-0.11,0.17,0.98]
// WE 17 1.09  7.37 Camera @[-75.64,172.25,745.83] yaw -6.61 pitch 9.02 + [-0.11,0.16,0.98]
// WE    4.85       Camera @[-83.37,151.51,743.16] yaw -7.26 pitch 7.57 + [-0.13,0.13,0.98]
// WE 63 7.25 20.16 Camera @[-95.45,156.38,737.22] yaw -8.19 pitch 7.99 + [-0.14,0.14,0.98]
#[test]
fn test_find_coarse_position() {
    let cdb = CameraDatabase::from_json(CAMERA_DB_JSON).unwrap();
    let logitech_body = cdb.get_body("Logitech C270 640x480").unwrap();
    let logitech_lens = cdb.get_lens("Logitech C270").unwrap();
    let camera = CameraPolynomial::new(logitech_body, logitech_lens, 1_000_000_000.0);
    let camera = CameraMapping::new(
        Rc::new(camera),
        [0., 0., 0.].into(),
        quat::look_at(&[-220., -310., -630.], &[0.10, -1., -0.1]).into(),
    );
    let mappings: Vec<PointMapping> = C1_DATA_ALL
        .iter()
        .map(|(model, screen)| {
            PointMapping::new(&Point3D::from_array(*model), &Point2D::from_array(*screen))
        })
        .collect();
    let cam = camera.find_coarse_position(&mappings, &[1000., 1000., 2000.], 11);
    let te = cam.total_error(&mappings);
    let we = cam.worst_error(&mappings);
    eprintln!("Final WE {:.2} {:.2} Camera {}", we, te, cam);
    assert!(
        we < 100.0,
        "Worst error should be about 53.82 but was {}",
        we
    );
    assert!(te < 250.0, "Total error should be about 220 but was {}", te);
}

//ft test_find_good
// CAM 0
// WE 28 4.12 20.86 Camera @[-197.71,-200.37,435.25] yaw -20.09 pitch -16.35 + [-0.33,-0.28,0.90]

// CAM 1
// WE 27 8.94 51.26 Camera @[-1.92,  -4.18,782.75] yaw -1.14 pitch -4.12 + [-0.02,-0.07,1.00]
// WE 99 3.58 22.17 Camera @[-21.98,112.49,768.56] yaw -2.56 pitch 4.43 + [-0.04,0.08,1.00]
// WE 92 5.81 31.90 Camera @[-29.29, 76.42,776.45] yaw -3.12 pitch 1.75 + [-0.05,0.03,1.00]
// WE 62 4.26 26.16 Camera @[-26.37, 85.59,773.16] yaw -2.89 pitch 2.45 + [-0.05,0.04,1.00]
// WE  6 1.30  7.18 Camera @[-74.46,180.50,741.43] yaw -6.54 pitch 9.71 + [-0.11,0.17,0.98]
// WE 48 1.25  8.04 Camera @[-75.56,166.17,748.12] yaw -6.60 pitch 8.53 + [-0.11,0.15,0.98]
// WE 17 1.09  7.37 Camera @[-75.64,172.25,745.83] yaw -6.61 pitch 9.02 + [-0.11,0.16,0.98]
// WE    4.85       Camera @[-83.37,151.51,743.16] yaw -7.26 pitch 7.57 + [-0.13,0.13,0.98]
// WE 63 7.25 20.16 Camera @[-95.45,156.38,737.22] yaw -8.19 pitch 7.99 + [-0.14,0.14,0.98]
#[allow(dead_code)]
// #[test]
fn test_find_good() {
    let cdb = CameraDatabase::from_json(CAMERA_DB_JSON).unwrap();
    let logitech_body = cdb.get_body("Logitech C270 640x480").unwrap();
    let logitech_lens = cdb.get_lens("Logitech C270").unwrap();
    let camera = CameraPolynomial::new(logitech_body, logitech_lens, 1_000_000_000.0);
    let camera = CameraMapping::new(
        // // Rc::new(CameraLens::default()),
        Rc::new(camera),
        [0., 0., 0.].into(),
        quat::look_at(&[-220., -310., -630.], &[0.10, -1., -0.1]).into(),
    );
    let mappings: Vec<PointMapping> = C1_DATA_ALL
        .iter()
        .map(|(model, screen)| {
            PointMapping::new(&Point3D::from_array(*model), &Point2D::from_array(*screen))
        })
        .collect();
    // let cam = camera.find_coarse_position(&mappings, &[1000., 1000., 2000.], 11);
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
    let mut worst_data = (1_000_000.0, 0, cam.clone(), 0.);
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
                    &|c, m, n| c.get_pm_sq_error(&m[n]),
                    &mappings,
                    last_n,
                    n,
                )
                .0;
            last_n = n;
        }
        for pm in mappings.iter() {
            cam.show_pm_error(pm);
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
            cam.show_pm_error(pm);
        }
        eprintln!("WE {} {:.2}", i, we);
        if we < worst_data.0 {
            eprintln!("WE {} {:.2} Camera {}", i, we, cam);
            worst_data = (we, i, cam.clone(), cam.total_error(&mappings));
        }
    }
    eprintln!(
        "Lowest WE {} {:.2} {:.2} Camera {}",
        worst_data.1, worst_data.0, worst_data.3, worst_data.2
    );
    assert!(false);
}

//ft test_optimize
#[allow(dead_code)]
// #[test]
fn test_optimize() {
    // let camera0 = CameraMapping::new(
    //     [-80., -120., 280.].into(), // 540 mm fromm model 280 for fov 35
    //     quat::look_at(&[-33., -30., -570.], &[0.10, -1., -0.1]).into(),
    // );
    let cdb = CameraDatabase::from_json(CAMERA_DB_JSON).unwrap();
    let logitech_body = cdb.get_body("Logitech C270 640x480").unwrap();
    let logitech_lens = cdb.get_lens("Logitech C270").unwrap();
    let camera = CameraPolynomial::new(logitech_body, logitech_lens, 1_000_000_000.0);
    let camera0 = CameraMapping::new(
        // for C0_DATA_ALL
        // [-196., -204., 435.].into(), // 540 mm fromm model origin?
        // for -201.77,-292.29,648.1
        // [54.10, -32.0, 781.].into(),
        // [-32.10, -7.0, 784.].into(),
        Rc::new(camera),
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
    let mut worst_data = (1_000_000.0, 0, cam.clone(), 0.);
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
                    &|c, m, n| c.get_pm_sq_error(&m[n]),
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
                    // &|c, m, n| c.get_pm_sq_error(&m[n]),
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
        dbg!(&cam);
        for pm in mappings.iter() {
            cam.show_pm_error(&pm);
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
            cam.show_pm_error(&pm);
        }
        if we < worst_data.0 {
            worst_data = (we, i, cam.clone(), cam.total_error(&mappings));
        }
    }
    eprintln!(
        "Lowest WE {} {:.2} {:.2} Camera {}",
        worst_data.1, worst_data.0, worst_data.3, worst_data.2
    );
    assert!(false);
}
