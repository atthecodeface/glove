//a Imports
use serde::{Deserialize, Serialize};

use ic_base::json;
use ic_base::{Point2D, Point3D, Result, RollYaw};

use crate::{CameraInstance, CameraProjection};

//a CalibrationMapping
//tp CalibrationMapping
/// Should probably store this as a vec of Point3D and a vec of same length of Point2D
#[derive(Debug, Clone, Default)]
pub struct CalibrationMapping {
    /// Mappings from world coordinates to absolute camera pixel values
    mappings: Vec<(f64, f64, f64, usize, usize)>,
}

//ip Serialize for CalibrationMapping
impl Serialize for CalibrationMapping {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.mappings.len()))?;
        for m in &self.mappings {
            seq.serialize_element(m)?;
        }
        seq.end()
    }
}

//ip Deserialize for CalibrationMapping
impl<'de> Deserialize<'de> for CalibrationMapping {
    fn deserialize<DE>(deserializer: DE) -> std::result::Result<Self, DE::Error>
    where
        DE: serde::Deserializer<'de>,
    {
        let mappings = Vec::<_>::deserialize(deserializer)?;
        Ok(Self { mappings })
    }
}

//ip CalibrationMapping
impl CalibrationMapping {
    //cp new
    pub fn new(world: Vec<Point3D>, sensor: Vec<Point2D>) -> Self {
        assert_eq!(world.len(), sensor.len());
        let mappings = world
            .into_iter()
            .zip(sensor.into_iter())
            .map(|(w, s)| (w[0], w[1], w[2], s[0] as usize, s[1] as usize))
            .collect();
        Self { mappings }
    }

    //cp from_json
    pub fn from_json(json: &str) -> Result<Self> {
        json::from_json("calibration mapping", json)
    }

    //dp to_json
    pub fn to_json(self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self)?)
    }

    //ap len
    pub fn len(&self) -> usize {
        self.mappings.len()
    }

    //ap get_pairings
    /// Get pairings between grid points, their camera-relative Point3Ds, and the roll-yaw described by
    /// the camera focus distance and lens type (not using its
    /// polynomial)
    pub fn get_pairings(&self, camera: &CameraInstance) -> Vec<(Point3D, Point3D, RollYaw)> {
        let mut result = vec![];
        for (kx, ky, kz, vx, vy) in &self.mappings {
            let grid_world: Point3D = [*kx, *ky, *kz].into();
            let grid_camera = camera.world_xyz_to_camera_xyz(grid_world);
            let pxy_abs: Point2D = [*vx as f64, *vy as f64].into();
            let pxy_rel = camera.px_abs_xy_to_px_rel_xy(pxy_abs);
            let ry = camera.px_rel_xy_to_ry(pxy_rel);
            result.push((grid_world, grid_camera, ry));
        }
        result
    }

    //ap get_xyz_pairings
    /// Get XY pairings
    pub fn get_xyz_pairings(&self) -> Vec<(Point3D, Point2D)> {
        let mut result = vec![];
        for (kx, ky, kz, vx, vy) in &self.mappings {
            let grid_world: Point3D = [*kx, *ky, *kz].into();
            let pxy_abs: Point2D = [*vx as f64, *vy as f64].into();
            result.push((grid_world, pxy_abs));
        }
        result
    }

    //ap get_pxys
    /// Get XY pairings
    pub fn get_pxys(&self) -> Vec<Point2D> {
        let mut result = vec![];
        for (_, _, _, vx, vy) in &self.mappings {
            let pxy_abs: Point2D = [*vx as f64, *vy as f64].into();
            result.push(pxy_abs);
        }
        result
    }
}
