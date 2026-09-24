use geo_nd::Vector;
use ic_base::Point2D;
use ic_camera::{CameraSensor, SizedSensor};

/// Test the sized sensor and the CameraSensor trait, quite simply
#[test]
fn test_sensor_sized() -> ic_base::Result<()> {
    let mut sensor = SizedSensor::default();
    sensor.height = 3;
    sensor.width = 4;
    sensor.mm_per_pixel = 0.2;

    assert_eq!(sensor.sensor_px_size(), (4.0, 3.0));
    assert_eq!(sensor.sensor_px_center().as_ref(), &[2.0, 1.5]);
    assert_eq!(sensor.sensor_mm_single_pixel_width(), 0.2);
    assert_eq!(sensor.sensor_mm_single_pixel_height(), 0.2);
    assert!((sensor.sensor_mm_size().0 - 0.8).abs() < 1E-5);
    assert!((sensor.sensor_mm_size().1 - 0.6).abs() < 1E-5);

    // centre is (2, 1.5); pxy relative is (-1, 1)
    let pxy: Point2D = [1.0, 0.5].into();
    assert!(
        sensor
            .sensor_px_abs_to_px_rel(pxy)
            .distance_sq_arr(&[-1.0, 1.0])
            < 1E-5
    );
    assert!(
        sensor
            .sensor_px_rel_to_px_abs(pxy)
            .distance_sq_arr(&[3.0, 1.0])
            < 1E-5
    );

    let t = sensor.sensor_px_rel_to_px_abs(sensor.sensor_px_abs_to_px_rel(pxy));
    assert!(t.distance(pxy) < 1E-5);
    let t = sensor.sensor_px_abs_to_px_rel(sensor.sensor_px_rel_to_px_abs(pxy));
    assert!(t.distance(pxy) < 1E-5);

    // sensor relative is from pixel is /10.0 * 0.2, from pxy itself (note, note from pxy relative...)
    assert!((sensor.optical_xy_to_optical_txty(pxy, 10.0).tanx() - 0.02).abs() < 1E-5);
    assert!((sensor.optical_xy_to_optical_txty(pxy, 10.0).tany() - 0.01).abs() < 1E-5);

    let t = sensor.optical_txty_to_optical_xy(sensor.optical_xy_to_optical_txty(pxy, 10.0), 10.0);
    assert!(t.distance(pxy) < 1E-5);

    let t = sensor.optical_txty_to_optical_xy(sensor.optical_xy_to_optical_txty(pxy, 20.0), 20.0);
    assert!(t.distance(pxy) < 1E-5);

    Ok(())
}
