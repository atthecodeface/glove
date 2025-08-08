//a Imports
use star_catalog::Catalog;

use ic_base::Result;
use ic_base::{json, Ray, Rrc};
use ic_camera::CameraInstance;
use ic_camera::{CalibrationMapping, CameraDatabase, LensPolys};
use ic_image::Color;
use ic_mapping::{NamedPointSet, PointMappingSet};
use ic_project::{Cip, Project, ProjectFileDesc};
use ic_stars::StarMapping;

use super::CmdArgs;

//a CmdArgs setters
//ip CmdArgs setters
impl CmdArgs {
    //mi set_verbose
    pub(crate) fn set_verbose(&mut self, verbose: bool) -> Result<()> {
        self.verbose = verbose;
        Ok(())
    }

    //mi set_camera_db
    pub(crate) fn set_camera_db(&mut self, filename: &str) -> Result<()> {
        let mut camera_db: CameraDatabase = self
            .path_set
            .load_from_json_file("camera database", filename)?;
        camera_db.derive();
        self.project.set_cdb(camera_db);
        self.cdb = self.project.cdb().clone();
        Ok(())
    }

    //mi set_project
    pub(crate) fn set_project(&mut self, filename: &str) -> Result<()> {
        self.project = self.path_set.load_from_json_file("project", filename)?;
        self.nps = self.project.nps().clone();
        self.cdb = self.project.cdb().clone();
        Ok(())
    }

    //mi set_project_desc
    pub(crate) fn set_project_desc(&mut self, project_filename: &str) -> Result<()> {
        let project_file_desc: ProjectFileDesc = self
            .path_set
            .load_from_json_file("project descriptor", project_filename)?;
        self.project = project_file_desc.load_project(&self.path_set)?;
        self.nps = self.project.nps().clone();
        self.cdb = self.project.cdb().clone();
        Ok(())
    }

    //mi set_calibration_mapping_file
    pub fn set_calibration_mapping_file(&mut self, filename: &str) -> Result<()> {
        let json = json::read_file(filename)?;
        self.calibration_mapping = CalibrationMapping::from_json(&json)?;
        Ok(())
    }

    //mi set_calibration_mapping
    pub fn set_calibration_mapping(&mut self, mapping: CalibrationMapping) {
        self.calibration_mapping = mapping;
    }

    //mi set_camera_json
    pub(crate) fn set_camera_json(&mut self, camera_json: &str) -> Result<()> {
        let camera = CameraInstance::from_json(&self.cdb.borrow(), camera_json)?;
        self.set_camera(camera);
        Ok(())
    }

    //mi set_camera_file
    pub(crate) fn set_camera_file(&mut self, camera_filename: &str) -> Result<()> {
        let camera_json = json::read_file(camera_filename)?;
        let camera = CameraInstance::from_json(&self.cdb.borrow(), &camera_json)?;
        self.set_camera(camera);
        Ok(())
    }

    //mi set_camera_body
    pub(crate) fn set_camera_body(&mut self, body: &str) -> Result<()> {
        let body = self.cdb.borrow().get_body_err(body)?.clone();
        self.camera.set_body(body);
        Ok(())
    }

    //mi set_camera_lens
    pub(crate) fn set_camera_lens(&mut self, lens: &str) -> Result<()> {
        let lens = self.cdb.borrow().get_lens_err(lens)?.clone();
        self.camera.set_lens(lens);
        Ok(())
    }

    //mi set_camera_focus_distance
    pub(crate) fn set_camera_focus_distance(&mut self, focus_distance: f64) -> Result<()> {
        self.camera.set_mm_focus_distance(focus_distance);
        Ok(())
    }

    //mi set_camera_polys
    pub(crate) fn set_camera_polys(&mut self, polys: &str) -> Result<()> {
        let json = json::read_file(polys)?;
        let lens_polys: LensPolys = json::from_json("lens polynomials", &json)?;
        let mut lens = self.camera.lens().clone();
        lens.set_polys(lens_polys);
        self.camera.set_lens(lens);
        Ok(())
    }

    //mi add_path
    /// Adds a directory to the search ath
    pub(crate) fn add_path(&mut self, s: &str) -> Result<()> {
        if let Err(e) = self.path_set.add_path(s) {
            eprintln!("Warning: {e}");
        };
        Ok(())
    }

    //mi add_nps
    /// Adds the named point set into the current NPS...
    ///
    /// Could perhaps do with a way to reset the nps for batch mode
    pub(crate) fn add_nps(&mut self, nps_filename: &str) -> Result<()> {
        let nps_json = json::read_file(nps_filename)?;
        self.project
            .nps_mut()
            .merge(&NamedPointSet::from_json(&nps_json)?);
        Ok(())
    }

    //mi add_pms
    /// Adds a point mapping set
    pub(crate) fn add_pms(&mut self, pms_filename: &str) -> Result<()> {
        let mut pms = PointMappingSet::new();
        let pms_json = json::read_file(pms_filename)?;
        let nf = pms.read_json(&self.project.nps_ref(), &pms_json, true)?;
        if !nf.is_empty() {
            eprintln!("Warning: {nf}");
        }
        if self.project.ncips() == 0 {
            let cip: Rrc<Cip> = Cip::default().into();
            self.cip = cip.clone();
            self.project.add_cip(cip);
        }
        self.cip.borrow_mut().pms_mut().merge(pms);
        Ok(())
    }

    //mp set_camera
    pub fn set_camera(&mut self, camera: CameraInstance) {
        if self.project.ncips() == 0 {
            let cip: Rrc<Cip> = Cip::default().into();
            self.cip = cip.clone();
            self.project.add_cip(cip);
        }
        self.cip.borrow_mut().set_camera(camera.clone().into());
        self.camera = camera;
    }

    //mp set_star_mapping_file
    pub(crate) fn set_star_mapping_file(&mut self, filename: &str) -> Result<()> {
        let json = json::read_file(filename)?;
        self.star_mapping = StarMapping::from_json(&json)?;
        Ok(())
    }

    //fp set_star_catalog
    pub fn set_star_catalog(&mut self, filename: &str) -> Result<()> {
        let mut catalog = Catalog::load_catalog(filename, 99.)?;
        catalog.derive_data();
        self.star_catalog = Some(Box::new(catalog));
        Ok(())
    }

    //mi add_read_img
    pub(crate) fn add_read_img(&mut self, s: &str) -> Result<()> {
        self.read_img.push(s.into());
        Ok(())
    }

    //mi set_cip
    /// Set to use CIP 'n' of the project
    // let cip = Cip::default();
    // let camera = camera::get_camera(matches, project)?;
    // let pms = mapping::get_pms(matches, &project.nps_ref())?;
    // *cip.camera_mut() = camera;
    // *cip.pms_mut() = pms;
    // let cip = cip.into();
    pub(crate) fn set_cip(&mut self, cip: usize) -> Result<()> {
        if cip >= self.project.ncips() {
            return Err(format!(
                "CIP {cip} is too large for the project (it has {} cips)",
                self.project.ncips()
            )
            .into());
        }
        self.cip = self.project.cip(cip).clone();
        self.pms = self.cip.borrow().pms().clone();
        self.camera = self.cip.borrow().camera().borrow().clone();
        Ok(())
    }

    //mi add_np
    pub(crate) fn add_np(&mut self, s: &str) -> Result<()> {
        self.np.push(s.into());
        Ok(())
    }

    //mi set_ray_file
    pub(crate) fn set_ray_file(&mut self, ray_filename: &str) -> Result<()> {
        let r_json = json::read_file(ray_filename)?;
        let mut named_rays: Vec<(String, Ray)> = json::from_json("ray list", &r_json)?;
        self.named_rays.append(&mut named_rays);
        Ok(())
    }

    //mi add_kernel
    /// Add a kernel to run
    pub(crate) fn add_kernel(&mut self, kernel: &str) -> Result<()> {
        self.kernels.push(kernel.to_owned());
        Ok(())
    }

    //mi set_bg_color
    pub(crate) fn set_bg_color(&mut self, s: &str) -> Result<()> {
        let c: Color = s.try_into()?;
        self.bg_color = Some(c);
        Ok(())
    }

    //mi set_pms_color
    pub(crate) fn set_pms_color(&mut self, s: &str) -> Result<()> {
        let c: Color = s.try_into()?;
        self.pms_color = Some(c);
        Ok(())
    }

    //mi set_model_color
    pub(crate) fn set_model_color(&mut self, s: &str) -> Result<()> {
        let c: Color = s.try_into()?;
        self.model_color = Some(c);
        Ok(())
    }

    //mi set_scale
    pub(crate) fn set_scale(&mut self, v: f64) -> Result<()> {
        self.scale = v;
        Ok(())
    }

    //mi set_angle
    pub(crate) fn set_angle(&mut self, v: f64) -> Result<()> {
        self.angle = v;
        Ok(())
    }

    //mi set_px
    pub(crate) fn set_px(&mut self, v: usize) -> Result<()> {
        self.px = v;
        Ok(())
    }

    //mi set_py
    pub(crate) fn set_py(&mut self, v: usize) -> Result<()> {
        self.py = v;
        Ok(())
    }

    //mi set_flags
    pub(crate) fn set_flags(&mut self, v: usize) -> Result<()> {
        self.flags = v;
        Ok(())
    }

    //mi add_string_arg
    pub(crate) fn add_string_arg(&mut self, s: &str) -> Result<()> {
        self.arg_strings.push(s.to_owned());
        Ok(())
    }

    //mi add_f64_arg
    pub(crate) fn add_f64_arg(&mut self, v: f64) -> Result<()> {
        self.arg_f64s.push(v);
        Ok(())
    }

    //mi set_kernel_size
    pub(crate) fn set_kernel_size(&mut self, v: usize) -> Result<()> {
        self.kernel_size = v;
        Ok(())
    }

    //mi set_write_img
    pub(crate) fn set_write_img(&mut self, s: &str) -> Result<()> {
        self.write_img = Some(s.to_owned());
        Ok(())
    }

    //mi set_write_project
    pub(crate) fn set_write_project(&mut self, s: &str) -> Result<()> {
        self.write_project = Some(s.to_owned());
        Ok(())
    }

    //mi set_write_named_points
    pub(crate) fn set_write_named_points(&mut self, s: &str) -> Result<()> {
        self.write_named_points = Some(s.to_owned());
        Ok(())
    }

    //mi set_write_point_mapping
    pub(crate) fn set_write_point_mapping(&mut self, s: &str) -> Result<()> {
        self.write_point_mapping = Some(s.to_owned());
        Ok(())
    }
    //mi set_write_camera
    pub(crate) fn set_write_camera(&mut self, s: &str) -> Result<()> {
        self.write_camera = Some(s.to_owned());
        Ok(())
    }

    //mi set_write_calibration_mapping
    pub(crate) fn set_write_calibration_mapping(&mut self, s: &str) -> Result<()> {
        self.write_calibration_mapping = Some(s.to_owned());
        Ok(())
    }

    //mi set_write_star_mapping
    pub(crate) fn set_write_star_mapping(&mut self, s: &str) -> Result<()> {
        self.write_star_mapping = Some(s.to_owned());
        Ok(())
    }

    //mi set_write_polys
    pub(crate) fn set_write_polys(&mut self, s: &str) -> Result<()> {
        self.write_polys = Some(s.to_owned());
        Ok(())
    }

    //mi set_write_svg
    pub(crate) fn set_write_svg(&mut self, s: &str) -> Result<()> {
        self.write_svg = Some(s.to_owned());
        Ok(())
    }

    // mi set_use_deltas
    pub(crate) fn set_use_deltas(&mut self, use_deltas: bool) -> Result<()> {
        self.use_deltas = use_deltas;
        Ok(())
    }

    // mi set_use_pts
    pub(crate) fn set_use_pts(&mut self, v: usize) -> Result<()> {
        self.use_pts = thunderclap::bound(v, Some(6), None, |v, _| {
            format!("Number of points ({v}) must be at least six")
        })?;
        Ok(())
    }

    // mi set_yaw_min
    pub(crate) fn set_yaw_min(&mut self, v: f64) -> Result<()> {
        self.yaw_min = thunderclap::bound(v, Some(0.0), Some(90.0), |v, _| {
            format!("Minimum yaw {v} must be in the range 0 to 90")
        })?;
        Ok(())
    }

    // mi set_yaw_max
    pub(crate) fn set_yaw_max(&mut self, v: f64) -> Result<()> {
        self.yaw_max = thunderclap::bound(v, Some(self.yaw_min), Some(90.0), |v, _| {
            format!(
                "Maximum yaw {v} must be between yaw_min ({}) and 90",
                self.yaw_min
            )
        })?;
        Ok(())
    }

    // mi set_poly_degree
    pub(crate) fn set_poly_degree(&mut self, v: usize) -> Result<()> {
        self.poly_degree = thunderclap::bound(v, Some(2), Some(12), |v, _| {
            format!("The polynomial degree {v} should be between 2 and 12 for reliability",)
        })?;
        Ok(())
    }

    // mi set_triangle_closeness
    pub(crate) fn set_triangle_closeness(&mut self, closeness: f64) -> Result<()> {
        self.triangle_closeness = closeness;
        Ok(())
    }

    // mi set_closeness
    pub(crate) fn set_closeness(&mut self, closeness: f64) -> Result<()> {
        self.closeness = closeness;
        Ok(())
    }

    // mi set_yaw_error
    pub(crate) fn set_yaw_error(&mut self, v: f64) -> Result<()> {
        self.yaw_error = thunderclap::bound(v, Some(0.0), Some(1.0), |v, _| {
            format!("The maximum yaw error {v} must be between 0 and 1 degree",)
        })?;
        Ok(())
    }

    // mi set_within
    pub(crate) fn set_within(&mut self, v: f64) -> Result<()> {
        self.within = thunderclap::bound(v, Some(0.0), Some(90.0), |v, _| {
            format!("The 'within' yaw {v} must be between 0 and 90 degree",)
        })?;
        Ok(())
    }

    // mi set_brightness
    pub(crate) fn set_brightness(&mut self, v: f32) -> Result<()> {
        self.brightness = thunderclap::bound(v, Some(0.0), Some(16.0), |v, _| {
            format!("Brightness (magnitude of stars) {v} must be between 0 and 16",)
        })?;
        Ok(())
    }
}
