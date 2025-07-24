//a Imports
use geo_nd::{quat, Quaternion};
use serde::{Deserialize, Serialize};

use ic_base::json;
use ic_base::{Point2D, Point3D, Quat, Result, RollYaw};

use crate::{serialize_body_name, serialize_lens_name};
use crate::{CameraBody, CameraDatabase, CameraLens, CameraPolynomial};
use crate::{CameraPolynomialDesc, CameraProjection};

//a CameraPolynomialCalibrateDesc
//tp CameraPolynomialCalibrateDesc
/// A description of a calibration for a camera and a lens, for an
/// image of a grid (e.g. graph paper)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraPolynomialCalibrateDesc {
    /// Camera description (body, lens, min focus distance)
    camera: CameraPolynomialDesc,
    /// Distance of the camera from the graph paper for the photograph
    // distance: f64,
    /// Grid point the camera is centred on
    // centred_on: (f64, f64),
    /// Rotation around the X axis (after accounting for z-rotation),
    /// i.e. how much the camera is off left-right from straight-on
    // x_rotation: f64,
    /// Rotation around the X axis (after accounting for z-rotation),
    /// i.e. how much the camera is off left-right from straight-on
    // y_rotation: f64,
    /// Rotation around the Z axis, i.e. how much off-horiztonal the
    /// camera was
    // z_rotation: f64,
    /// Mappings from grid coordinates to absolute camera pixel values
    mappings: Vec<(isize, isize, usize, usize)>,
}

//a CameraPolynomialCalibrate
//tp CameraPolynomialCalibrate
#[derive(Debug, Clone, Default)]
pub struct CameraPolynomialCalibrate {
    /// Mappings from grid coordinates to absolute camera pixel values
    mappings: Vec<(isize, isize, usize, usize)>,
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

    //ap distance
    // pub fn distance(&self) -> f64 {
    // self.distance
    // }

    //cp from_desc
    pub fn from_desc(cdb: &CameraDatabase, desc: CameraPolynomialCalibrateDesc) -> Result<Self> {
        // let position = [desc.centred_on.0, desc.centred_on.1, desc.distance].into();
        // let direction: Quat = quat::look_at(&[0., 0., -1.], &[0., 1., 0.]).into();
        // let rotate_x: Quat = quat::of_axis_angle(&[1., 0., 0.], desc.x_rotation).into();
        // let rotate_y: Quat = quat::of_axis_angle(&[0., 1., 0.], desc.y_rotation).into();
        // let rotate_z: Quat = quat::of_axis_angle(&[0., 0., 1.], desc.z_rotation).into();
        // let direction = direction * rotate_z;
        // let direction = direction * rotate_x;
        // let direction = direction * rotate_y;
        // let position: Point3D = rotate_y.conjugate().apply3(&position);
        // let position: Point3D = rotate_x.conjugate().apply3(&position);

        let camera = CameraPolynomial::from_desc(cdb, desc.camera)?;
        //let body = cdb.get_body_err(&desc.camera.body)?.clone();
        // let lens = cdb.get_lens_err(&desc.camera.lens)?.clone();
        // let camera = CameraPolynomial::new(
        // body.clone(),
        // lens.clone(),
        // desc.camera.mm_focus_distance,
        // position,
        // direction,
        // );
        // eprintln!("{camera}");
        // let m: Point3D = camera.camera_xyz_to_world_xyz([0., 0., -desc.distance].into());
        // eprintln!("Camera {camera} focused on {m}");
        let s = Self {
            //            body,
            //            lens,
            camera,
            //            mm_focus_distance: desc.camera.mm_focus_distance,
            // centred_on: desc.centred_on,
            // distance: desc.distance,
            // x_rotation: desc.x_rotation,
            // y_rotation: desc.y_rotation,
            // z_rotation: desc.z_rotation,
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
    pub fn to_desc(mut self) -> CameraPolynomialCalibrateDesc {
        let camera = self.camera.to_desc();
        let mappings = self.mappings;
        // let distance = self.distance;
        // let centred_on = self.centred_on;
        // let x_rotation = self.x_rotation;
        // let y_rotation = self.y_rotation;
        // let z_rotation = self.z_rotation;
        CameraPolynomialCalibrateDesc {
            camera,
            // distance,
            // centred_on,
            // x_rotation,
            // y_rotation,
            // z_rotation,
            mappings,
        }
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
    pub fn get_pairings(&self) -> Vec<(Point2D, Point3D, RollYaw)> {
        let mut result = vec![];
        for (kx, ky, vx, vy) in &self.mappings {
            let grid: Point2D = [*kx as f64, *ky as f64].into();
            let grid_world: Point3D = [*kx as f64, *ky as f64, 0.].into();
            let grid_camera = self.camera.world_xyz_to_camera_xyz(grid_world);
            let pxy_abs: Point2D = [*vx as f64, *vy as f64].into();
            let pxy_rel = self.camera.px_abs_xy_to_px_rel_xy(pxy_abs);
            // eprintln!("{grid} : {grid_camera} : {pxy_abs} : {pxy_rel}");
            let ry = self.camera.px_rel_xy_to_ry(pxy_rel);
            result.push((grid, grid_camera, ry));
        }
        result
    }

    //ap get_xy_pairings
    /// Get XY pairings
    pub fn get_xy_pairings(&self) -> Vec<(Point2D, Point2D)> {
        let mut result = vec![];
        for (kx, ky, vx, vy) in &self.mappings {
            let grid: Point2D = [*kx as f64, *ky as f64].into();
            let pxy_abs: Point2D = [*vx as f64, *vy as f64].into();
            result.push((grid, pxy_abs));
        }
        result
    }
}
