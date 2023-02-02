use image_calibrate::{CameraProjection, CameraRectilinear};

#[test]
fn test() {
    let x = CameraRectilinear::new_logitech_c270_640();
    eprintln!("{}", x);
    dbg!(x.x_px_from_tan_sc);
    dbg!(x.y_px_from_tan_sc);
    assert_eq!(x.px_abs_xy_to_px_rel_xy([320., 240.].into())[0], 0.);
    assert_eq!(x.px_abs_xy_to_px_rel_xy([320., 240.].into())[1], 0.);
    assert_eq!(x.px_abs_xy_to_px_rel_xy([0., 0.].into())[0], -320.);
    assert_eq!(x.px_abs_xy_to_px_rel_xy([0., 0.].into())[1], 240.);
    assert_eq!(x.px_abs_xy_to_px_rel_xy([640., 480.].into())[0], 320.);
    assert_eq!(x.px_abs_xy_to_px_rel_xy([640., 480.].into())[1], -240.);
    for i in -100..100 {
        assert_eq!(
            x.px_rel_xy_to_px_abs_xy(x.px_abs_xy_to_px_rel_xy([i as f64, i as f64 * 3.].into()))[0],
            i as f64
        );
        assert_eq!(
            x.px_rel_xy_to_px_abs_xy(x.px_abs_xy_to_px_rel_xy([i as f64, i as f64 * 3.].into()))[1],
            i as f64 * 3.
        );
    }
    let txty = x.px_rel_xy_to_txty([320., 240.].into());
    let fov_x2 = txty[0].atan().to_degrees();
    dbg!(fov_x2);
    assert!(fov_x2 > 22.619);
    assert!(fov_x2 < 22.631);
    let xy_ratio = txty[0] / txty[1];
    dbg!(xy_ratio);
    assert!(xy_ratio > 1.332);
    assert!(xy_ratio < 1.334);
}
