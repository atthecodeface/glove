use image_calibrate::{CameraDatabase, CameraPolynomial, CameraProjection, CAMERA_DB_JSON};

#[test]
fn test() {
    let cdb = CameraDatabase::from_json(CAMERA_DB_JSON).unwrap();
    let logitech_body = cdb.get_body("Logitech C270 640x480").unwrap();
    let logitech_lens = cdb.get_lens("Logitech C270").unwrap();
    let camera = CameraPolynomial::new(logitech_body, logitech_lens, 1_000_000_000.0);
    eprintln!("{}", camera);
    eprintln!("{:?}", camera);
    println!("{}", serde_json::to_string(&camera).unwrap());
    assert_eq!(camera.px_abs_xy_to_px_rel_xy([320., 240.].into())[0], 0.);
    assert_eq!(camera.px_abs_xy_to_px_rel_xy([320., 240.].into())[1], 0.);
    assert_eq!(camera.px_abs_xy_to_px_rel_xy([0., 0.].into())[0], -320.);
    assert_eq!(camera.px_abs_xy_to_px_rel_xy([0., 0.].into())[1], 240.);
    assert_eq!(camera.px_abs_xy_to_px_rel_xy([640., 480.].into())[0], 320.);
    assert_eq!(camera.px_abs_xy_to_px_rel_xy([640., 480.].into())[1], -240.);
    for i in -100..100 {
        assert_eq!(
            camera.px_rel_xy_to_px_abs_xy(
                camera.px_abs_xy_to_px_rel_xy([i as f64, i as f64 * 3.].into())
            )[0],
            i as f64
        );
        assert_eq!(
            camera.px_rel_xy_to_px_abs_xy(
                camera.px_abs_xy_to_px_rel_xy([i as f64, i as f64 * 3.].into())
            )[1],
            i as f64 * 3.
        );
    }
    let txty = camera.px_rel_xy_to_txty([320., 240.].into());
    let fov_x2 = txty[0].atan().to_degrees();
    dbg!(txty, fov_x2);
    assert!(fov_x2 > 22.6);
    assert!(fov_x2 < 22.65);
    let xy_ratio = txty[0] / txty[1];
    dbg!(xy_ratio);
    assert!(xy_ratio > 1.332);
    assert!(xy_ratio < 1.334);
}
