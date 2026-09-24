use std::f64;

use geo_nd::{Quaternion, Vector};
use ic_base::{Point3D, Quat, TanXTanY};
use ic_camera::{BaseCamera, CameraProjection};

/// Test the sized sensor and the CameraSensor trait, quite simply
#[test]
fn test_base_camera() -> ic_base::Result<()> {
    let mut camera = BaseCamera::default();
    camera.position = [1., 2., 3.].into();
    // Orientation is 90 degrees around X axis; so Y maps to -Z, Z maps to Y; X is unchanged
    //
    // This is 'look at +Y with +Z as up'
    camera.orientation = Quat::of_axis_angle(&[1.0, 0., 0.], f64::consts::FRAC_PI_2);

    assert!(camera.position().distance(Point3D::from([1.0, 2.0, 3.0])) < 1E-4);
    assert!(camera.orientation().distance(&camera.orientation) < 1E-4);

    assert!(
        camera
            .camera_dir_to_world_dir([1.0, 0.0, 0.0].into())
            .distance(Point3D::from([1.0, 0.0, 0.0]))
            < 1E-4
    );
    assert!(
        camera
            .camera_dir_to_world_dir([0.0, 1.0, 0.0].into())
            .distance(Point3D::from([0.0, 0.0, -1.0]))
            < 1E-4
    );
    assert!(
        camera
            .camera_dir_to_world_dir([0.0, 0.0, 1.0].into())
            .distance(Point3D::from([0.0, 1.0, 0.0]))
            < 1E-4
    );
    // Of TanXTanY (0,0) is [0,0,-1]
    assert!(
        camera
            .camera_txty_to_world_dir(TanXTanY::of_tx_ty(0.0, 0.0))
            .distance(Point3D::from([0.0, -1.0, 0.0]))
            < 1E-4
    );

    // Of TanXTany(1,0) is (1/sqrt(2),0,-1/sqrt(2))
    assert!(
        camera
            .camera_txty_to_world_dir(TanXTanY::of_tx_ty(1.0, 0.0))
            .distance(Point3D::from([(0.5_f64).sqrt(), -(0.5_f64).sqrt(), 0.0]))
            < 1E-4
    );

    // Of TanXTany(1,1) is (1/sqrt(3),1/sqrt(3),-1/sqrt(3))
    assert!(
        camera
            .camera_txty_to_world_dir(TanXTanY::of_tx_ty(1.0, 1.0))
            .distance(Point3D::from([
                (1.0 / 3.0_f64).sqrt(),
                -(1.0 / 3.0_f64).sqrt(),
                -(1.0 / 3.0_f64).sqrt()
            ]))
            < 1E-4
    );

    // All these are *in front* of the camera (for a world_dir)
    for p in [
        [1.0, -0.1, 0.3],
        [0.0, -1.0, -0.3],
        [0.0, -0.1, 1.0],
        [1., -3., 5.],
    ]
    .into_iter()
    {
        let p: Point3D = p.into();
        let t = camera.camera_dir_to_world_dir(camera.world_dir_to_camera_dir(p));
        assert!(t.distance(p) < 1E-6);
        let t = camera.world_dir_to_world_xyz(camera.world_xyz_to_world_dir(p));
        assert!(t.distance(p) < 1E-6);

        let t = camera.world_dir_to_world_xyz(p);
        assert!(t.distance(p + camera.position) < 1E-6);

        let t = camera.world_xyz_to_world_dir(p);
        assert!(t.distance(p - camera.position) < 1E-6);

        eprintln!(
            "{} {:?}",
            camera.world_dir_to_camera_dir(p),
            camera.world_dir_to_camera_txty(p)
        );
        eprintln!(
            "{p} {}",
            camera.camera_txty_to_world_dir(camera.world_dir_to_camera_txty(p))
        );

        let t = camera.camera_txty_to_world_dir(camera.world_dir_to_camera_txty(p));
        assert!(t.distance(p.normalize()) < 1E-5);
        let t = camera.camera_txty_to_world_dir(camera.world_dir_to_camera_txty(-p));
        assert!(t.distance(p.normalize()) < 1E-5);
    }

    Ok(())
}
