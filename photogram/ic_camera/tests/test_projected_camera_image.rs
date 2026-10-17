use geo_nd::Vector;
use ic_base::Point2D;
use ic_camera::{CameraLensProjection, ProjectedCameraImage, RectilinearLens, SimpleSensorCamera};
use ic_image::{ImageDraw, ImageLumaF32};

fn check_image_to_sensor_mappings<C, I>(
    pci: &ProjectedCameraImage<C, I>,
    mappings: &[([f64; 2], [f64; 2])],
) where
    C: CameraLensProjection,
    I: ImageDraw,
{
    for (pxy, espxy) in mappings {
        let pxy: Point2D = pxy.into();
        let espxy: Point2D = espxy.into();
        let spxy = pci.map_image_region_pxy_to_sensor_abs_pxy(pxy);
        if spxy.distance(espxy) > 1E-6 {
            assert!(
                false,
                "Mapping image {pxy} had {spxy} was expected to be at {espxy}"
            )
        }
        let tpxy = pci.map_sensor_abs_pxy_to_image_region_pxy(spxy);
        if tpxy.distance(pxy) > 1E-6 {
            assert!(
                false,
                "Mapping sensor {spxy} had {tpxy} was expected to be at {pxy}"
            )
        }
        let opt_tpxy = pci.map_sensor_abs_pxy_to_opt_image_region_pxy(spxy);
        if let Some(otpxy) = opt_tpxy {
            assert!(
                tpxy[0] >= 0.0
                    && tpxy[0] < pci.width() as f64
                    && tpxy[1] >= 0.0
                    && tpxy[1] < pci.height() as f64,
                "tpxy of {tpxy} was outside the image region but opt returned Some {otpxy:?}"
            );
        } else {
            assert!(
                tpxy[0] < 0.0
                    || tpxy[0] >= pci.width() as f64
                    || tpxy[1] < 0.0
                    || tpxy[1] >= pci.height() as f64,
                "tpxy of {tpxy} was inside the image region but opt returned None"
            );
        }
    }
}
/// Test the sized sensor and the CameraSensor trait, quite simply
#[test]
fn test_0() -> ic_base::Result<()> {
    let image = ImageLumaF32::new(120, 90);
    let camera = SimpleSensorCamera::new(240, 180, 0.1, RectilinearLens::default(), 50.0);

    let pci = ProjectedCameraImage::mapping_camera_to_image(&image, &camera);
    check_image_to_sensor_mappings(
        &pci,
        &[
            ([1.0, 2.0], [1.0, 2.0]),
            ([2.0, 8.0], [2.0, 8.0]),
            ([9.0, 3.1], [9.0, 3.1]),
        ],
    );

    let pci = ProjectedCameraImage::mapping_camera_to_image(&image, &camera)
        .with_sensor_translation([6.1, 9.2].into());
    check_image_to_sensor_mappings(
        &pci,
        &[
            ([10.0, 12.0], [3.9, 2.8]),
            ([20.0, 18.0], [13.9, 8.8]),
            ([19.0, 33.1], [12.9, 23.9]),
        ],
    );

    let pci = ProjectedCameraImage::mapping_camera_to_image(&image, &camera).with_scaling(0.5);
    check_image_to_sensor_mappings(
        &pci,
        &[
            ([10.0, 12.0], [20.0, 24.0]),
            ([20.0, 18.0], [40.0, 36.0]),
            ([19.0, 33.1], [38.0, 66.2]),
        ],
    );

    // Image of the whole sensor reduce by a factor of 2
    let pci = ProjectedCameraImage::mapping_camera_to_image(&image, &camera)
        .of_sensor_region([120., 90.0].into(), 240.0);
    check_image_to_sensor_mappings(
        &pci,
        &[
            ([10.0, 12.0], [20.0, 24.0]),
            ([20.0, 18.0], [40.0, 36.0]),
            ([19.0, 33.1], [38.0, 66.2]),
        ],
    );

    // Image of the center of the sensor at factor of 1
    let pci = ProjectedCameraImage::mapping_camera_to_image(&image, &camera)
        .of_sensor_region([120., 90.0].into(), 120.0);
    check_image_to_sensor_mappings(
        &pci,
        &[
            ([10.0, 12.0], [70.0, 57.0]),
            ([20.0, 18.0], [80.0, 63.0]),
            ([19.0, 33.1], [79.0, 78.1]),
        ],
    );

    // Image of the center of the sensor at factor of 1.5
    //
    // Image distance from 60,45 should be scaled up by 1.5 and added to 120,90
    let pci = ProjectedCameraImage::mapping_camera_to_image(&image, &camera)
        .of_sensor_region([120., 90.0].into(), 180.0);
    check_image_to_sensor_mappings(
        &pci,
        &[
            ([60.0, 45.0], [120.0, 90.0]),
            ([20.0, 30.0], [60.0, 67.5]),
            ([120., 90.0], [210.0, 157.5]),
            ([0., 0.0], [30.0, 22.5]),
            ([240., 90.0], [390.0, 157.5]),
            ([-120., 0.0], [-150.0, 22.5]),
            ([120., 180.0], [210.0, 292.5]),
            ([0., -90.0], [30.0, 22.5 - 135.0]),
        ],
    );
    Ok(())
}
