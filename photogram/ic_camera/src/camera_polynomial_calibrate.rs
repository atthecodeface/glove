//a Imports
use serde::{Deserialize, Serialize};

use ic_base::json;
use ic_base::{Point2D, Point3D, Result, RollYaw};

use crate::{CameraDatabase, CameraPolynomial};
use crate::{CameraPolynomialDesc, CameraProjection};

//a CameraPolynomialCalibrateDesc
//tp CameraPolynomialCalibrateDesc
/// A description of a calibration for a camera and a lens, for an
/// image of a grid (e.g. graph paper)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraPolynomialCalibrateDesc {
    /// Camera description (body, lens, min focus distance)
    camera: CameraPolynomialDesc,
    /// Mappings from world coordinates to absolute camera pixel values
    mappings: Vec<(f64, f64, f64, usize, usize)>,
}

//a CameraPolynomialCalibrate
//tp CameraPolynomialCalibrate
#[derive(Debug, Clone, Default)]
pub struct CameraPolynomialCalibrate {
    /// Mappings from world coordinates to absolute camera pixel values
    mappings: Vec<(f64, f64, f64, usize, usize)>,
    /// Derived camera instance
    camera: CameraPolynomial,
}

//ip CameraPolynomialCalibrate
impl CameraPolynomialCalibrate {
    //ap camera
    pub fn camera(&self) -> &CameraPolynomial {
        &self.camera
    }

    //ap camera_mut
    pub fn camera_mut(&mut self) -> &mut CameraPolynomial {
        &mut self.camera
    }

    //cp new
    pub fn new(camera: CameraPolynomial, mappings: Vec<(f64, f64, f64, usize, usize)>) -> Self {
        Self { camera, mappings }
    }

    //cp from_desc
    pub fn from_desc(cdb: &CameraDatabase, desc: CameraPolynomialCalibrateDesc) -> Result<Self> {
        let camera = CameraPolynomial::from_desc(cdb, desc.camera)?;
        let s = Self {
            camera,
            mappings: desc.mappings,
        };
        Ok(s)
    }

    //cp from_json
    pub fn from_json(cdb: &CameraDatabase, json: &str) -> Result<Self> {
        let desc: CameraPolynomialCalibrateDesc =
            json::from_json("camera calibration descriptor", json)?;
        Self::from_desc(cdb, desc)
    }

    //dp to_desc
    pub fn to_desc(self) -> CameraPolynomialCalibrateDesc {
        let camera = self.camera.to_desc();
        let mappings = self.mappings;
        CameraPolynomialCalibrateDesc { camera, mappings }
    }

    //dp to_desc_json
    pub fn to_desc_json(self) -> Result<String> {
        Ok(serde_json::to_string(&self.to_desc())?)
    }

    //mp grid_as_model
    /*
    pub fn grid_as_model(&self, grid: Point2D) -> Point3D {
        let xy_rel = self.camera_poly.px_abs_xy_to_px_rel_xy(xy);
        let ry = self.camera_poly.px_rel_xy_to_ry(&self, xy_rel);
    }
     */

    //ap get_pairings
    /// Get pairings between grid points, their camera-relative Point3Ds, and the roll-yaw described by
    /// the camera focus distance and lens type (not using its
    /// polynomial)
    pub fn get_pairings(&self) -> Vec<(Point3D, Point3D, RollYaw)> {
        let mut result = vec![];
        for (kx, ky, kz, vx, vy) in &self.mappings {
            let grid_world: Point3D = [*kx, *ky, *kz].into();
            let grid_camera = self.camera.world_xyz_to_camera_xyz(grid_world);
            let pxy_abs: Point2D = [*vx as f64, *vy as f64].into();
            let pxy_rel = self.camera.px_abs_xy_to_px_rel_xy(pxy_abs);
            let ry = self.camera.px_rel_xy_to_ry(pxy_rel);
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
