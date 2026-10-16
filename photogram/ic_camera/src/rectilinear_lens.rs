use ic_base::TanXTanY;

use crate::LensProjection;

#[derive(Debug, Clone, Copy, Default)]
pub struct RectilinearLens();

impl LensProjection for RectilinearLens {
    fn camera_txty_to_optical_txty(&self, camera_txty: TanXTanY) -> TanXTanY {
        camera_txty
    }
    fn optical_txty_to_camera_txty(&self, sensor_txty: TanXTanY) -> TanXTanY {
        sensor_txty
    }
}
