//a Imports
use std::cell::Ref;
use std::io::Write;
use std::rc::Rc;

use clap::{Arg, ArgAction, ArgMatches, Command};
use star_catalog::{Catalog, StarFilter};
use thunderclap::{ArgCount, CommandArgs, CommandBuilder};

use ic_base::{json, Ray, Rrc};
use ic_base::{Error, Result};
use ic_camera::CameraInstance;
use ic_camera::{CalibrationMapping, CameraDatabase, CameraProjection, LensPolys};
use ic_image::{Color, Image, ImagePt, ImageRgb8};
use ic_mapping::{CameraPtMapping, PointMapping};
use ic_mapping::{NamedPointSet, PointMappingSet};
use ic_project::{Cip, Project};
use ic_stars::StarMapping;

//a Modules
pub mod camera;
pub mod file_system;
pub mod image;
pub mod kernels;
pub mod mapping;
pub mod project;
pub mod threads;

//a CmdResult
pub type CmdResult = std::result::Result<String, ic_base::Error>;
pub fn cmd_ok() -> CmdResult {
    Ok("".into())
}

//a CmdArgs
//tp CmdArgs

#[derive(Default)]
pub struct CmdArgs {
    verbose: bool,

    project: Project,

    // Camera database that is part of the project
    cdb: Rrc<CameraDatabase>,

    // nps that is part of the project
    nps: Rrc<NamedPointSet>,

    // pms that is part of the project
    pms: Rrc<PointMappingSet>,

    // CIP that is part of the project
    cip: Rrc<Cip>,

    // camera is a *specific* camera, not part of a CIP or project
    camera: CameraInstance,

    calibration_mapping: CalibrationMapping,

    star_catalog: Option<Box<Catalog>>,
    star_mapping: StarMapping,

    read_img: Vec<String>,

    write_camera: Option<String>,
    write_calibration_mapping: Option<String>,
    write_star_mapping: Option<String>,
    write_polys: Option<String>,
    write_img: Option<String>,
    write_svg: Option<String>,

    bg_color: Option<Color>,
    pms_color: Option<Color>,
    model_color: Option<Color>,

    np: Vec<String>, // could be name, 3D, pixel XY (from camera mapping of 3D); might need at least 3

    named_rays: Vec<(String, Ray)>,

    kernels: Vec<String>,
    kernel_size: usize,
    scale: f64,
    angle: f64,
    px: usize,
    py: usize,
    flags: usize,
    use_deltas: bool,
    use_pts: usize,
    yaw_min: f64,
    yaw_max: f64,
    poly_degree: usize,
    triangle_closeness: f64,
    closeness: f64,
    yaw_error: f64,
    within: f64,
    brightness: f32,
}

//ip CommandArgs for CmdArgs
impl CommandArgs for CmdArgs {
    type Error = ic_base::Error;
    type Value = String;

    fn reset_args(&mut self) {
        self.read_img = vec![];
        self.write_img = None;
        self.write_calibration_mapping = None;
        self.write_star_mapping = None;
        self.write_polys = None;
        self.write_svg = None;
        self.write_camera = None;
        self.use_pts = 0;
        self.use_deltas = false;
        self.flags = 0;
        self.scale = 1.0;
        self.angle = 0.0;
    }
}

//ip CmdArgs accessors
impl CmdArgs {
    //mi project
    pub fn project(&self) -> &Project {
        &self.project
    }

    //mi cdb
    pub fn cdb(&self) -> &Rrc<CameraDatabase> {
        &self.cdb
    }

    //mi nps
    pub fn nps(&self) -> &Rrc<NamedPointSet> {
        self.project.nps()
    }

    //mi pms
    pub fn pms(&self) -> &Rrc<PointMappingSet> {
        &self.pms
    }

    //mi cip
    pub fn cip(&self) -> &Rrc<Cip> {
        &self.cip
    }

    //mi np_names
    pub fn np_names(&self) -> &[String] {
        &self.np
    }

    //mi camera
    /// Note - if there is a project loaded then (such as for orient) the camera might want to come from the CIP?
    pub fn camera(&self) -> &CameraInstance {
        &self.camera
    }

    //mi camera_mut
    pub fn camera_mut(&mut self) -> &mut CameraInstance {
        &mut self.camera
    }

    //mi calibration_mapping
    pub fn calibration_mapping(&self) -> &CalibrationMapping {
        &self.calibration_mapping
    }

    //mi star_catalog
    pub fn star_catalog(&self) -> &Catalog {
        self.star_catalog.as_ref().unwrap()
    }

    //mi star_catalog_mut
    pub fn star_catalog_mut(&mut self) -> &mut Catalog {
        self.star_catalog.as_mut().unwrap()
    }

    //mi star_mapping
    pub fn star_mapping(&self) -> &StarMapping {
        &self.star_mapping
    }

    //mi bg_color
    pub fn bg_color(&self) -> Option<&Color> {
        self.bg_color.as_ref()
    }

    //mi pms_color
    pub fn pms_color(&self) -> Option<&Color> {
        self.pms_color.as_ref()
    }

    //mi model_color
    pub fn model_color(&self) -> Option<&Color> {
        self.model_color.as_ref()
    }

    //mi named_rays
    pub fn named_rays(&self) -> &[(String, Ray)] {
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
        self.write_img.as_ref().map(|s| s.as_str())
    }

    //mi write_svg
    pub fn write_svg(&self) -> Option<&str> {
        self.write_svg.as_ref().map(|s| s.as_str())
    }
}

//ip CmdArgs setters
impl CmdArgs {
    //mi set_verbose
    fn set_verbose(&mut self, verbose: bool) -> Result<()> {
        self.verbose = verbose;
        Ok(())
    }

    //mi set_project
    fn set_project(&mut self, project_filename: &str) -> Result<()> {
        let project_json = json::read_file(project_filename)?;
        let project: Project = json::from_json("project", &project_json)?;
        self.project = project;
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

    //mi set_camera_db
    fn set_camera_db(&mut self, camera_db_filename: &str) -> Result<()> {
        let camera_db_json = json::read_file(camera_db_filename)?;
        let mut camera_db: CameraDatabase = json::from_json("camera database", &camera_db_json)?;
        camera_db.derive();
        self.project.set_cdb(camera_db);
        Ok(())
    }

    //mi set_camera_file
    fn set_camera_file(&mut self, camera_filename: &str) -> Result<()> {
        let camera_json = json::read_file(camera_filename)?;
        let camera = CameraInstance::from_json(&self.cdb.borrow(), &camera_json)?;
        self.set_camera(camera);
        Ok(())
    }

    //mi set_camera_body
    fn set_camera_body(&mut self, body: &str) -> Result<()> {
        let body = self.cdb.borrow().get_body_err(body)?.clone();
        self.camera.set_body(body);
        Ok(())
    }

    //mi set_camera_lens
    fn set_camera_lens(&mut self, lens: &str) -> Result<()> {
        let lens = self.cdb.borrow().get_lens_err(lens)?.clone();
        self.camera.set_lens(lens);
        Ok(())
    }

    //mi set_camera_focus_distance
    fn set_camera_focus_distance(&mut self, focus_distance: f64) -> Result<()> {
        self.camera.set_mm_focus_distance(focus_distance);
        Ok(())
    }

    //mi set_camera_polys
    fn set_camera_polys(&mut self, polys: &str) -> Result<()> {
        let json = json::read_file(polys)?;
        let lens_polys: LensPolys = json::from_json("lens polynomials", &json)?;
        let mut lens = self.camera.lens().clone();
        lens.set_polys(lens_polys);
        self.camera.set_lens(lens);
        Ok(())
    }

    //mi add_nps
    /// Adds the named point set into the current NPS...
    ///
    /// Could perhaps do with a way to reset the nps for batch mode
    fn add_nps(&mut self, nps_filename: &str) -> Result<()> {
        let nps_json = json::read_file(nps_filename)?;
        self.project
            .nps_mut()
            .merge(&NamedPointSet::from_json(&nps_json)?);
        Ok(())
    }

    //mi add_pms
    /// Adds a point mapping set
    fn add_pms(&mut self, pms_filename: &str) -> Result<()> {
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

    //mi add_read_img
    fn add_read_img(&mut self, s: &str) -> Result<()> {
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
    fn set_cip(&mut self, cip: usize) -> Result<()> {
        if cip >= self.project.ncips() {
            return Err(format!(
                "CIP {cip} is too large for the project (it has {} cips)",
                self.project.ncips()
            )
            .into());
        }
        self.cip = self.project.cip(cip).clone();
        self.pms = self.cip.borrow().pms().clone();
        Ok(())
    }

    //mi set_np
    fn set_np(&mut self, s: &str) -> Result<()> {
        self.np.push(s.into());
        Ok(())
    }

    //mi set_ray_file
    fn set_ray_file(&mut self, ray_filename: &str) -> Result<()> {
        let r_json = json::read_file(ray_filename)?;
        let mut named_rays: Vec<(String, Ray)> = json::from_json("ray list", &r_json)?;
        self.named_rays.append(&mut named_rays);
        Ok(())
    }

    //mi add_kernel
    /// Add a kernel to run
    fn add_kernel(&mut self, kernel: &str) -> Result<()> {
        self.kernels.push(kernel.to_owned());
        Ok(())
    }

    //mi set_bg_color
    fn set_bg_color(&mut self, s: &str) -> Result<()> {
        let c: Color = s.try_into()?;
        self.bg_color = Some(c);
        Ok(())
    }

    //mi set_pms_color
    fn set_pms_color(&mut self, s: &str) -> Result<()> {
        let c: Color = s.try_into()?;
        self.pms_color = Some(c);
        Ok(())
    }

    //mi set_model_color
    fn set_model_color(&mut self, s: &str) -> Result<()> {
        let c: Color = s.try_into()?;
        self.model_color = Some(c);
        Ok(())
    }

    //mi set_scale
    fn set_scale(&mut self, v: f64) -> Result<()> {
        self.scale = v;
        Ok(())
    }

    //mi set_angle
    fn set_angle(&mut self, v: f64) -> Result<()> {
        self.angle = v;
        Ok(())
    }

    //mi set_px
    fn set_px(&mut self, v: usize) -> Result<()> {
        self.px = v;
        Ok(())
    }

    //mi set_py
    fn set_py(&mut self, v: usize) -> Result<()> {
        self.py = v;
        Ok(())
    }

    //mi set_flags
    fn set_flags(&mut self, v: usize) -> Result<()> {
        self.flags = v;
        Ok(())
    }

    //mi set_kernel_size
    fn set_kernel_size(&mut self, v: usize) -> Result<()> {
        self.kernel_size = v;
        Ok(())
    }

    //mi set_write_img
    fn set_write_img(&mut self, s: &str) -> Result<()> {
        self.write_img = Some(s.to_owned());
        Ok(())
    }

    //mi set_write_camera
    fn set_write_camera(&mut self, s: &str) -> Result<()> {
        self.write_camera = Some(s.to_owned());
        Ok(())
    }

    //mi set_write_calibration_mapping
    fn set_write_calibration_mapping(&mut self, s: &str) -> Result<()> {
        self.write_calibration_mapping = Some(s.to_owned());
        Ok(())
    }

    //mi set_write_star_mapping
    fn set_write_star_mapping(&mut self, s: &str) -> Result<()> {
        self.write_star_mapping = Some(s.to_owned());
        Ok(())
    }

    //mi set_write_polys
    fn set_write_polys(&mut self, s: &str) -> Result<()> {
        self.write_polys = Some(s.to_owned());
        Ok(())
    }

    //mi set_write_svg
    fn set_write_svg(&mut self, s: &str) -> Result<()> {
        self.write_svg = Some(s.to_owned());
        Ok(())
    }

    // mi set_use_deltas
    fn set_use_deltas(&mut self, use_deltas: bool) -> Result<()> {
        self.use_deltas = use_deltas;
        Ok(())
    }

    // mi set_use_pts
    fn set_use_pts(&mut self, v: usize) -> Result<()> {
        self.use_pts = thunderclap::bound(v, Some(6), None, |v, _| {
            format!("Number of points ({v}) must be at least six")
        })?;
        Ok(())
    }

    // mi set_yaw_min
    fn set_yaw_min(&mut self, v: f64) -> Result<()> {
        self.yaw_min = thunderclap::bound(v, Some(0.0), Some(90.0), |v, _| {
            format!("Minimum yaw {v} must be in the range 0 to 90")
        })?;
        Ok(())
    }

    // mi set_yaw_max
    fn set_yaw_max(&mut self, v: f64) -> Result<()> {
        self.yaw_max = thunderclap::bound(v, Some(self.yaw_min), Some(90.0), |v, _| {
            format!(
                "Maximum yaw {v} must be between yaw_min ({}) and 90",
                self.yaw_min
            )
        })?;
        Ok(())
    }

    // mi set_poly_degree
    fn set_poly_degree(&mut self, v: usize) -> Result<()> {
        self.poly_degree = thunderclap::bound(v, Some(2), Some(12), |v, _| {
            format!("The polynomial degree {v} should be between 2 and 12 for reliability",)
        })?;
        Ok(())
    }

    // mi set_triangle_closeness
    fn set_triangle_closeness(&mut self, closeness: f64) -> Result<()> {
        self.triangle_closeness = closeness;
        Ok(())
    }

    // mi set_closeness
    fn set_closeness(&mut self, closeness: f64) -> Result<()> {
        self.closeness = closeness;
        Ok(())
    }

    // mi set_yaw_error
    fn set_yaw_error(&mut self, v: f64) -> Result<()> {
        self.yaw_error = thunderclap::bound(v, Some(0.0), Some(1.0), |v, _| {
            format!("The maximum yaw error {v} must be between 0 and 1 degree",)
        })?;
        Ok(())
    }

    // mi set_within
    fn set_within(&mut self, v: f64) -> Result<()> {
        self.within = thunderclap::bound(v, Some(0.0), Some(90.0), |v, _| {
            format!("The 'within' yaw {v} must be between 0 and 90 degree",)
        })?;
        Ok(())
    }

    // mi set_brightness
    fn set_brightness(&mut self, v: f32) -> Result<()> {
        self.brightness = thunderclap::bound(v, Some(0.0), Some(16.0), |v, _| {
            format!("Brightness (magnitude of stars) {v} must be between 0 and 16",)
        })?;
        Ok(())
    }

    // mi use_pts
    pub fn use_pts(&self, n: usize) -> usize {
        if self.use_pts != 0 {
            n.min(self.use_pts)
        } else {
            n
        }
    }

    // mi ensure_star_catalog
    pub fn ensure_star_catalog(&self) -> Result<()> {
        if self.star_catalog.is_none() {
            Err("Star catalog *must* have been specified".into())
        } else {
            Ok(())
        }
    }
}

//ip CmdArgs arg build methods
impl CmdArgs {
    //mp add_arg_verbose
    pub fn add_arg_verbose(build: &mut CommandBuilder<Self>) {
        build.add_flag(
            "verbose",
            Some('v'),
            "Enable verbose output",
            CmdArgs::set_verbose,
        );
    }

    //fp add_arg_read_image
    pub fn add_arg_read_image<I: Into<ArgCount>>(build: &mut CommandBuilder<Self>, arg_count: I) {
        build.add_arg_string(
            "read",
            Some('r'),
            "Image to read",
            arg_count.into(),
            None,
            CmdArgs::add_read_img,
        );
    }

    //fp add_arg_write_image
    pub fn add_arg_write_image(build: &mut CommandBuilder<Self>, required: bool) {
        image::add_arg_write_img(build, CmdArgs::set_write_img, required);
    }

    //fp add_arg_kernel
    pub fn add_arg_kernel<I: Into<ArgCount>>(build: &mut CommandBuilder<Self>, arg_count: I) {
        build.add_arg_string(
            "kernel",
            None,
            "Add a kernel to run",
            arg_count.into(),
            None,
            CmdArgs::add_kernel,
        );
    }

    //fp add_arg_nps
    pub fn add_arg_nps(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "nps",
            None,
            "Add a named point set to the list",
            (0,).into(),
            None,
            CmdArgs::add_nps,
        );
    }

    //fp add_arg_camera_database
    pub fn add_arg_camera_database(build: &mut CommandBuilder<Self>, required: bool) {
        build.add_arg_string(
            "camera_db",
            None,
            "Camera database JSON filename",
            required.into(),
            None,
            CmdArgs::set_camera_db,
        );
    }

    //fp add_arg_project
    pub fn add_arg_project(build: &mut CommandBuilder<Self>, required: bool) {
        build.add_arg_string(
            "project",
            None,
            "Project JSON filename",
            required.into(),
            None,
            CmdArgs::set_project,
        );
    }

    //fp add_arg_pms
    pub fn add_arg_pms(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "pms",
            None,
            "Add a point mapping set",
            false.into(), // Perhaps should allow some in...
            None,
            CmdArgs::add_pms,
        );
    }

    //fp add_arg_cip
    pub fn add_arg_cip(build: &mut CommandBuilder<Self>, required: bool) {
        let arg_count = required.into();
        build.add_arg_usize(
            "cip",
            None,
            "CIP number (camera and PMS) within the project",
            arg_count,
            None,
            CmdArgs::set_cip,
        );
    }

    //fp add_arg_camera
    pub fn add_arg_camera(build: &mut CommandBuilder<Self>, required: bool) {
        build.add_arg_string(
            "camera",
            Some('c'),
            "Camera lens, placement and orientation JSON",
            required.into(),
            None,
            CmdArgs::set_camera_file,
        );

        build.add_arg_string(
            "use_body",
            None,
            "Specify which body to use in the camera",
            false.into(),
            None,
            CmdArgs::set_camera_body,
        );

        build.add_arg_string(
            "use_lens",
            None,
            "Specify which lens to use in the camera",
            false.into(),
            None,
            CmdArgs::set_camera_lens,
        );

        build.add_arg_f64(
            "use_focus",
            None,
            "Specify the focus distance in mm used for the image, in the camera",
            false.into(),
            None,
            CmdArgs::set_camera_focus_distance,
        );

        build.add_arg_string(
            "use_polys",
            None,
            "Specify an override for the lens polynomials in the camera",
            false.into(),
            None,
            CmdArgs::set_camera_polys,
        );
    }

    //fp add_arg_ray_file
    pub fn add_arg_ray_file<I: Into<ArgCount>>(build: &mut CommandBuilder<Self>, arg_count: I) {
        build.add_arg_string(
            "rays",
            None,
            "Model ray Json files (list of name, ray)",
            arg_count.into(),
            None,
            CmdArgs::set_ray_file,
        );
    }

    //fp add_arg_named_point
    pub fn add_arg_named_point<I: Into<ArgCount>>(build: &mut CommandBuilder<Self>, arg_count: I) {
        build.add_arg_string(
            "np",
            None,
            "Specifies named points for the patch",
            arg_count.into(),
            None,
            CmdArgs::set_np,
        );
    }

    //fp add_arg_px
    pub fn add_arg_px(build: &mut CommandBuilder<Self>, required: bool) {
        build.add_arg_usize(
            "px",
            None,
            "Pixel X value to use",
            required.into(),
            None,
            CmdArgs::set_px,
        );
    }

    //fp add_arg_py
    pub fn add_arg_py(build: &mut CommandBuilder<Self>, required: bool) {
        build.add_arg_usize(
            "py",
            None,
            "Pixel Y value to use",
            required.into(),
            None,
            CmdArgs::set_py,
        );
    }

    //fp add_arg_kernel_size
    pub fn add_arg_kernel_size(build: &mut CommandBuilder<Self>, required: bool) {
        build.add_arg_usize(
            "kernel_size",
            None,
            "Size parameter for a kernel",
            required.into(),
            Some("8"),
            CmdArgs::set_kernel_size,
        );
    }

    //fp add_arg_flags
    pub fn add_arg_flags(build: &mut CommandBuilder<Self>) {
        build.add_arg_usize(
            "flags",
            None,
            "Flags parameter for (e.g.) a kernel",
            false.into(),
            Some("0"),
            CmdArgs::set_flags,
        );
    }

    //fp add_arg_scale
    pub fn add_arg_scale(build: &mut CommandBuilder<Self>) {
        build.add_arg_f64(
            "scale",
            None,
            "Scale parameter for (e.g.) a kernel",
            false.into(),
            Some("1"),
            CmdArgs::set_scale,
        );
    }

    //fp add_arg_angle
    pub fn add_arg_angle(build: &mut CommandBuilder<Self>) {
        build.add_arg_f64(
            "angle",
            None,
            "Angle parameter for (e.g.) a kernel",
            false.into(),
            Some("0"),
            CmdArgs::set_angle,
        );
    }

    //fp add_arg_bg_color
    pub fn add_arg_bg_color(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "bg_color",
            None,
            "Background color",
            ArgCount::Optional,
            None,
            CmdArgs::set_bg_color,
        );
    }

    //mp add_arg_pms_color
    pub fn add_arg_pms_color(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "pms_color",
            None,
            "Color for PMS points",
            ArgCount::Optional,
            None,
            CmdArgs::set_pms_color,
        );
    }

    //mp add_arg_model_color
    pub fn add_arg_model_color(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "model_color",
            None,
            "Color for mapped model crosses",
            ArgCount::Optional,
            None,
            CmdArgs::set_model_color,
        );
    }

    //mp add_arg_calibration_mapping
    pub fn add_arg_calibration_mapping(build: &mut CommandBuilder<Self>, required: bool) {
        build.add_arg_string(
            "calibration_mapping",
            Some('m'),
            "Camera calibration mapping JSON",
            required.into(),
            None,
            CmdArgs::set_calibration_mapping_file,
        );
    }

    //fp add_arg_write_camera
    pub fn add_arg_write_camera(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "write_camera",
            None,
            "File to write the final camera JSON to",
            ArgCount::Optional,
            None,
            CmdArgs::set_write_camera,
        );
    }

    //fp add_arg_write_calibration_mapping
    pub fn add_arg_write_calibration_mapping(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "write_calibration_mapping",
            None,
            "File to write a derived mapping JSON to",
            ArgCount::Optional,
            None,
            CmdArgs::set_write_calibration_mapping,
        );
    }

    //fp add_arg_write_star_mapping
    pub fn add_arg_write_star_mapping(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "write_star_mapping",
            None,
            "File to write a derived star mapping JSON to",
            ArgCount::Optional,
            None,
            CmdArgs::set_write_star_mapping,
        );
    }

    //fp add_arg_write_polys
    pub fn add_arg_write_polys(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "write_polys",
            None,
            "File to write a derived polynomials JSON to",
            ArgCount::Optional,
            None,
            CmdArgs::set_write_polys,
        );
    }

    //fp add_arg_write_svg
    pub fn add_arg_write_svg(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "write_svg",
            None,
            "File to write an output SVG to",
            ArgCount::Optional,
            None,
            CmdArgs::set_write_svg,
        );
    }
    //fp add_arg_poly_degree
    pub fn add_arg_poly_degree(build: &mut CommandBuilder<Self>) {
        build.add_arg_usize(
            "poly_degree",
            None,
            "Degree of polynomial to use for the lens calibration (5 for 50mm)",
            ArgCount::Optional,
            Some("5"),
            CmdArgs::set_poly_degree,
        );
    }

    //fp add_arg_use_deltas
    pub fn add_arg_use_deltas(build: &mut CommandBuilder<Self>) {
        build.add_flag(
            "use_deltas",
            None,
            "Use deltas for plotting rather than absolute values",
            CmdArgs::set_use_deltas,
        );
    }

    //fp add_arg_num_pts
    pub fn add_arg_num_pts(build: &mut CommandBuilder<Self>) {
        build.add_arg_usize(
            "num_pts",
            Some('n'),
            "Number of points to use (from start of mapping); if not specified, use all",
            ArgCount::Optional,
            None,
            CmdArgs::set_use_pts,
        );
    }

    //fp add_arg_yaw_min_max
    pub fn add_arg_yaw_min_max(
        build: &mut CommandBuilder<Self>,
        min: Option<&'static str>,
        max: Option<&'static str>,
    ) {
        build.add_arg_f64(
            "yaw_min",
            None,
            "Minimim yaw to use for plotting or updating the star mapping, in degrees",
            ArgCount::Optional,
            min,
            CmdArgs::set_yaw_min,
        );
        build.add_arg_f64(
            "yaw_max",
            None,
            "Maximim yaw to use for plotting or updating the star mapping, in degrees",
            ArgCount::Optional,
            max,
            CmdArgs::set_yaw_max,
        );
    }

    //fp add_arg_yaw_error
    pub fn add_arg_yaw_error(build: &mut CommandBuilder<Self>) {
        build.add_arg_f64(
            "yaw_error",
            None,
            "Maximum relative error in yaw to permit a closest match for",
            ArgCount::Optional,
            Some("0.03"),
            CmdArgs::set_yaw_error,
        );
    }

    //fp add_arg_within
    pub fn add_arg_within(build: &mut CommandBuilder<Self>) {
        build.add_arg_f64(
            "within",
            None,
            "Only use catalog stars Within this angle (degrees) for mapping",
            ArgCount::Optional,
            Some("15"),
            CmdArgs::set_within,
        );
    }

    //fp add_arg_closeness
    pub fn add_arg_closeness(build: &mut CommandBuilder<Self>) {
        build.add_arg_f64(
            "closeness",
            None,
            "Closeness (degrees) to find triangles of stars or degress for calc cal mapping, find stars, map_stars etc",
            ArgCount::Optional,
            Some("0.2"),
            CmdArgs::set_closeness,
        );
    }

    //fp add_arg_triangle_closeness
    pub fn add_arg_triangle_closeness(build: &mut CommandBuilder<Self>) {
        build.add_arg_f64(
            "triangle_closeness",
            None,
            "Closeness (degrees) to find triangles of stars",
            ArgCount::Optional,
            Some("0.2"),
            CmdArgs::set_triangle_closeness,
        );
    }

    //fp add_arg_star_mapping
    pub fn add_arg_star_mapping(build: &mut CommandBuilder<Self>) {
        build.add_arg(
            Arg::new("star_mapping")
                .required(false)
                .help("File mapping sensor coordinates to catalog identifiers")
                .action(ArgAction::Set),
            Box::new(CmdArgs::arg_star_mapping),
        );
    }

    //fp add_arg_star_catalog
    pub fn add_arg_star_catalog(build: &mut CommandBuilder<Self>) {
        build.add_arg(
            Arg::new("star_catalog")
                .long("catalog")
                .required(false)
                .help("Star catalog to use")
                .action(ArgAction::Set),
            Box::new(CmdArgs::arg_star_catalog),
        );
    }

    //fp add_arg_brightness
    pub fn add_arg_brightness(build: &mut CommandBuilder<Self>) {
        build.add_arg_f32(
            "brightness",
            None,
            "Maximum brightness of stars to use in the catalog",
            ArgCount::Optional,
            Some("5.0"),
            CmdArgs::set_brightness,
        );
    }

    //fp arg_star_mapping
    pub fn arg_star_mapping(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
        let filename = matches.get_one::<String>("star_mapping").unwrap();
        let json = json::read_file(filename)?;
        args.star_mapping = StarMapping::from_json(&json)?;
        Ok(())
    }

    //fp arg_star_catalog
    pub fn arg_star_catalog(args: &mut CmdArgs, matches: &ArgMatches) -> Result<()> {
        let catalog_filename = matches.get_one::<String>("star_catalog").unwrap();
        let mut catalog = Catalog::load_catalog(catalog_filename, 99.)?;
        catalog.derive_data();
        args.star_catalog = Some(Box::new(catalog));
        Ok(())
    }
}

//ip CmdArgs methods to get
impl CmdArgs {
    //mp get_image_read_or_create
    pub fn get_image_read_or_create(&self) -> ic_base::Result<ImageRgb8> {
        let read_filename = self.read_img.first();
        let img = ImageRgb8::read_or_create_image(
            self.camera.sensor_size().0 as usize,
            self.camera.sensor_size().1 as usize,
            read_filename.map(|x| x.as_str()),
        )?;
        Ok(img)
    }

    //mp get_read_image
    pub fn get_read_image(&self, n: usize) -> ic_base::Result<ImageRgb8> {
        let Some(read_filename) = self.read_img.get(n) else {
            return Err(format!("Required at least {} read images to be specified", n + 1).into());
        };
        let img = ImageRgb8::read_image(read_filename)?;
        Ok(img)
    }
}

//ip CmdArgs - Operations
impl CmdArgs {
    //mp pms_map
    pub fn pms_map<M, T>(&self, map: M) -> ic_base::Result<T>
    where
        M: FnOnce(&PointMappingSet) -> ic_base::Result<T>,
    {
        let cip = self.cip.borrow();
        let pms = cip.pms_ref();
        map(&*pms)
    }

    //mp convert_calibration_mapping
    pub fn convert_calibration_mapping(&self) {
        let v = self.calibration_mapping.get_xyz_pairings();

        //cb Add calibrations to NamedPointSet and PointMappingSet
        for (n, (model_xyz, pxy_abs)) in v.into_iter().enumerate() {
            let name = n.to_string();
            let color = [255, 255, 255, 255].into();
            self.nps
                .borrow_mut()
                .add_pt(name.clone(), color, Some(model_xyz), 0.);
            self.pms
                .borrow_mut()
                .add_mapping(&self.nps.borrow(), &name, &pxy_abs, 0.);
        }
    }

    //mp draw_image
    pub fn draw_image(&self, pts: &[ImagePt]) -> Result<()> {
        if self.read_img.is_empty() || self.write_img.is_none() {
            return Ok(());
        }
        let mut img = ImageRgb8::read_image(&self.read_img[0])?;
        for p in pts {
            p.draw(&mut img);
        }
        img.write(self.write_img.as_ref().unwrap())?;
        Ok(())
    }

    //mp show_step
    pub fn show_step<S>(&self, s: S)
    where
        S: std::fmt::Display,
    {
        if self.verbose {
            eprintln!("\n{s}");
        }
    }

    //mp update_star_mappings
    pub fn update_star_mappings(&mut self) -> (usize, f64) {
        self.star_mapping.update_star_mappings(
            self.star_catalog.as_ref().unwrap(),
            &self.camera,
            self.closeness,
            self.yaw_error,
            self.yaw_min,
            self.yaw_max,
        )
    }

    //mp if_verbose
    pub fn if_verbose<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        if self.verbose {
            f()
        }
    }

    //mp write_outputs
    pub fn write_outputs(&self) -> Result<()> {
        if let Some(filename) = &self.write_camera {
            let s = self.camera.to_json()?;
            let mut f = std::fs::File::create(filename)?;
            f.write_all(s.as_bytes())?;
        }
        if let Some(filename) = &self.write_polys {
            let s = self.camera.lens().polys().to_json()?;
            let mut f = std::fs::File::create(filename)?;
            f.write_all(s.as_bytes())?;
        }
        Ok(())
    }

    //mp output_camera
    pub fn output_camera(&self) -> CmdResult {
        let s = self.camera.to_json()?;
        Ok(s.to_string())
    }

    //mp output_calibration_mapping
    pub fn output_calibration_mapping(&self) -> CmdResult {
        let s = self.calibration_mapping.clone().to_json()?;
        Ok(s.to_string())
    }

    //mp output_star_mapping
    pub fn output_star_mapping(&self) -> CmdResult {
        let s = self.star_mapping.clone().to_json()?;
        Ok(s.to_string())
    }

    //mp output_polynomials
    pub fn output_polynomials(&self) -> CmdResult {
        let s = self.camera.lens().polys().to_json()?;
        Ok(s.to_string())
    }
}

//a Errors
//fp add_errors_arg
pub fn add_errors_arg(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("worst_error")
            .long("worst")
            .help("Use worst error for resolving")
            .action(ArgAction::SetTrue),
    )
    .arg(
        Arg::new("total_error")
            .long("total")
            .help("Use total error for resolving")
            .action(ArgAction::SetTrue),
    )
}

//fp get_error_fn
pub fn get_error_fn(
    matches: &ArgMatches,
) -> for<'a, 'b> fn(&'a CameraInstance, &'b [PointMapping], usize) -> f64 {
    if matches.get_flag("worst_error") {
        let error_method: for<'a, 'b> fn(&'a CameraInstance, &'b [PointMapping], usize) -> f64 =
            |c, m, _n| c.worst_error(m);
        error_method
    } else {
        let error_method: for<'a, 'b> fn(&'a CameraInstance, &'b [PointMapping], usize) -> f64 =
            |c, m, _n| c.total_error(m);
        error_method
    }
}

//a General
//fp add_verbose_arg
pub fn add_verbose_arg(cmd: Command) -> Command {
    cmd.arg(
        Arg::new("verbose")
            .long("verbose")
            .short('v')
            .help("Set verbosity")
            .long_help("Set verbose")
            .action(ArgAction::Set),
    )
}

//fp get_verbose
pub fn get_verbose(matches: &ArgMatches) -> bool {
    matches.get_one::<String>("verbose").is_some()
}

//mp add_arg_verbose
pub fn add_arg_verbose<C, F>(build: &mut CommandBuilder<C>, set: F)
where
    C: CommandArgs<Error = Error>,
    F: Fn(&mut C, bool) -> Result<()> + 'static,
{
    build.add_arg(
        Arg::new("verbose")
            .long("verbose")
            .short('v')
            .help("Set verbosity")
            .long_help("Set verbose")
            .action(ArgAction::SetTrue),
        Box::new(move |args, matches| set(args, *matches.get_one::<bool>("verbose").unwrap())),
    );
}
