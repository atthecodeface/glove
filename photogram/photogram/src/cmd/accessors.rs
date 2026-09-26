use anyhow::anyhow;

use ic_photogram::SphericalImageShape;
use star_catalog::Catalog;

use crate::Result;

use ic_photogram::CameraDatabase;
use ic_photogram::CameraInstance;
use ic_photogram::Color8;
use ic_photogram::{Cip, Project};
use ic_photogram::{NamedPointSet, PointMappingSet};
use ic_photogram::{NamedRayList, Point2D, Point3D, Rrc};

use super::CmdArgs;

//a CmdArgs accessors
//ip CmdArgs accessors
impl CmdArgs {
    //mi project
    pub fn project(&self) -> &Project {
        &self.project
    }

    //mi pretty_json
    pub fn pretty_json(&self) -> bool {
        self.pretty_json
    }

    //mi project_mut
    pub fn project_mut(&mut self) -> &mut Project {
        &mut self.project
    }

    pub fn cdb(&self) -> &Rrc<CameraDatabase> {
        &self.project.cdb()
    }

    pub fn nps(&self) -> &Rrc<NamedPointSet> {
        self.project.nps()
    }

    pub fn cip(&self) -> Option<&Rrc<Cip>> {
        self.cip.as_ref()
    }

    pub fn pms(&self) -> Option<Rrc<PointMappingSet>> {
        self.cip.as_ref().map(|cip| cip.borrow().pms().clone())
    }

    pub fn np_names(&self) -> &[String] {
        &self.np
    }

    /// Note that the camera is NOT part of the CIP
    pub fn camera(&self) -> &CameraInstance {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut CameraInstance {
        &mut self.camera
    }

    pub fn star_catalog(&self) -> &Catalog {
        self.star_catalog.as_ref().unwrap()
    }
    pub fn star_catalog_mut(&mut self) -> &mut Catalog {
        self.star_catalog.as_mut().unwrap()
    }

    //mi get_string_arg
    pub fn get_string_arg(&self, n: usize) -> Option<&str> {
        self.arg_strings.get(n).map(|n| n.as_str())
    }

    //mi get_f64_arg
    pub fn get_f64_arg(&self, n: usize) -> Option<f64> {
        self.arg_f64s.get(n).copied()
    }

    //mi get_usize_arg
    pub fn get_usize_arg(&self, n: usize) -> Option<usize> {
        self.arg_usizes.get(n).copied()
    }

    //mi arg_usizes
    pub fn arg_usizes(&self) -> &[usize] {
        &self.arg_usizes
    }

    //mi arg_strings
    pub fn arg_strings(&self) -> impl Iterator<Item = &str> {
        self.arg_strings.iter().map(|s| s.as_str())
    }

    //mi arg_as_point3d
    #[track_caller]
    pub fn arg_as_point3d(&self, n: usize) -> Result<Point3D> {
        assert!(n < self.arg_strings.len());
        let coords: Vec<_> = self.arg_strings[n].split(',').collect();
        if coords.len() != 3 {
            return Err(anyhow!(
                "Expected 3 coordinates for a 3D point, got {coords:?}"
            ));
        }
        Ok([
            coords[0].parse::<f64>()?,
            coords[1].parse::<f64>()?,
            coords[2].parse::<f64>()?,
        ]
        .into())
    }

    //mi arg_as_point2d
    #[track_caller]
    pub fn arg_as_point2d(&self, n: usize) -> Result<Point2D> {
        assert!(n < self.arg_strings.len());
        let coords: Vec<_> = self.arg_strings[n].split(',').collect();
        if coords.len() != 2 {
            return Err(anyhow!(
                "Expected 2 coordinates for a 2D point, got {coords:?}"
            ));
        }
        Ok([coords[0].parse::<f64>()?, coords[1].parse::<f64>()?].into())
    }

    //mi bg_color
    pub fn bg_color(&self) -> Option<&Color8> {
        self.bg_color.as_ref()
    }

    //mi pms_color
    pub fn pms_color(&self) -> Option<&Color8> {
        self.pms_color.as_ref()
    }

    //mi model_color
    pub fn model_color(&self) -> Option<&Color8> {
        self.model_color.as_ref()
    }

    //mi named_rays
    pub fn named_rays(&self) -> &NamedRayList {
        &self.named_rays
    }

    //mi kernels
    pub fn kernels(&self) -> &[String] {
        &self.kernels
    }

    //mi kernel_size
    pub fn kernel_size(&self) -> usize {
        self.kernel_size
    }

    //mi flags
    pub fn flags(&self) -> usize {
        self.flags
    }

    //mi pxy
    pub fn pxy(&self) -> (usize, usize) {
        (self.px, self.py)
    }

    //mi scale
    pub fn scale(&self) -> f64 {
        self.scale
    }

    //mi angle
    pub fn angle(&self) -> f64 {
        self.angle
    }

    //mi range
    pub fn range(&self) -> f64 {
        self.range
    }

    //mi steps
    pub fn steps(&self) -> usize {
        self.steps
    }

    //mi max_error
    pub fn max_error(&self) -> f64 {
        self.max_error
    }

    //mi max_points
    pub fn max_points(&self) -> usize {
        self.max_points
    }

    //mi max_pairs
    pub fn max_pairs(&self) -> usize {
        self.max_pairs
    }

    //mi yaw_min
    pub fn yaw_min(&self) -> f64 {
        self.yaw_min
    }

    //mi yaw_max
    pub fn yaw_max(&self) -> f64 {
        self.yaw_max
    }

    //mi within
    pub fn within(&self) -> f64 {
        self.within
    }

    //mi brightness
    pub fn brightness(&self) -> f32 {
        self.brightness
    }

    //mi closeness
    pub fn closeness(&self) -> f64 {
        self.closeness
    }

    //mi yaw_error
    pub fn yaw_error(&self) -> f64 {
        self.yaw_error
    }

    //mi triangle_closeness
    pub fn triangle_closeness(&self) -> f64 {
        self.triangle_closeness
    }

    //mi use_deltas
    pub fn use_deltas(&self) -> bool {
        self.use_deltas
    }

    //mi from_camera
    pub fn from_camera(&self) -> bool {
        self.from_camera
    }

    //mi poly_degree
    pub fn poly_degree(&self) -> usize {
        self.poly_degree
    }

    //mi read_img
    pub fn read_img(&self) -> &[String] {
        &self.read_img
    }

    //mi write_img
    pub fn write_img(&self) -> Option<&str> {
        self.write_img.as_deref()
    }

    //mi write_svg
    pub fn write_svg(&self) -> Option<&str> {
        self.write_svg.as_deref()
    }

    //mp use_pts
    pub fn use_pts(&self, n: usize) -> usize {
        if self.use_pts != 0 {
            n.min(self.use_pts)
        } else {
            n
        }
    }

    pub fn shape(&self) -> SphericalImageShape {
        self.shape
    }

    pub fn get_point2d(&self, n: usize) -> Option<&Point2D> {
        self.xy.get(n)
    }
    pub fn get_point3d(&self, n: usize) -> Option<&Point3D> {
        self.xyz.get(n)
    }

    //mp ensure_star_catalog
    pub fn ensure_star_catalog(&self) -> Result<()> {
        if self.star_catalog.is_none() {
            Err(anyhow!("Star catalog *must* have been specified"))
        } else {
            Ok(())
        }
    }
    pub fn validate_cip(&self) -> Result<()> {
        if self.cip().is_none() {
            Err(anyhow!("No CIP selected"))
        } else {
            Ok(())
        }
    }
    pub fn validate_spherical_image(&self) -> Result<()> {
        if self.spherical_image.is_none() {
            Err(anyhow!("No spherical image has been added"))
        } else {
            Ok(())
        }
    }
}
