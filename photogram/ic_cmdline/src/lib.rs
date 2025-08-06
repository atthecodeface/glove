//a Modules
use std::cell::Ref;
use std::io::Write;
use std::rc::Rc;

use clap::{Arg, ArgAction, ArgMatches, Command};
use thunderclap::{ArgCount, CommandArgs, CommandBuilder};

use ic_base::{json, Ray, Rrc};
use ic_camera::CameraInstance;
use ic_camera::{CameraDatabase, CameraProjection, LensPolys};
use ic_image::{Color, Image, ImagePt, ImageRgb8};
use ic_mapping::{CameraPtMapping, PointMapping};
use ic_mapping::{NamedPointSet, PointMappingSet};
use ic_project::{Cip, Project};

pub mod camera;
pub mod file_system;
pub mod image;
pub mod kernels;
pub mod mapping;
pub mod project;
pub mod threads;

use ic_base::{Error, Result};

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

    cips: Vec<Rrc<Cip>>,

    // Change to Rrc
    nps: Rc<NamedPointSet>,
    // Change to Rrc
    pms: PointMappingSet,
    // Keep
    camera: CameraInstance,

    write_camera: Option<String>,
    write_mapping: Option<String>,
    write_star_mapping: Option<String>,
    write_polys: Option<String>,
    read_img: Vec<String>,
    write_img: Option<String>,
    write_svg: Option<String>,

    bg_color: Option<Color>,
    pms_color: Option<Color>,
    model_color: Option<Color>,
    np: Vec<String>, // could be name, 3D, pixel XY (from camera mapping of 3D); might need at least 3
    cip: Rrc<Cip>,

    named_rays: Vec<(String, Ray)>,

    kernels: Vec<String>,
    kernel_size: usize,
    scale: f64,
    angle: f64,
    px: usize,
    py: usize,
    flags: usize,
}

//ip CommandArgs for CmdArgs
impl CommandArgs for CmdArgs {
    type Error = ic_base::Error;
    type Value = String;
}

//ip CmdArgs accessors
impl CmdArgs {
    //mi project
    pub fn project(&self) -> &Project {
        &self.project
    }

    //mi cdb
    pub fn cdb(&self) -> Ref<CameraDatabase> {
        self.project.cdb_ref()
    }

    //mi nps
    pub fn nps(&self) -> Ref<NamedPointSet> {
        self.project.nps_ref()
    }

    //mi pms
    pub fn pms(&self) -> &PointMappingSet {
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

    //mi write_img
    pub fn write_img(&self) -> Option<&str> {
        self.write_img.as_ref().map(|s| s.as_str())
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
        Ok(())
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
        let camera = CameraInstance::from_json(&self.cdb(), &camera_json)?;
        self.set_camera(camera)
    }

    //mi set_camera_body
    fn set_camera_body(&mut self, body: &str) -> Result<()> {
        let body = self.cdb().get_body_err(body)?.clone();
        self.camera.set_body(body);
        Ok(())
    }

    //mi set_camera_lens
    fn set_camera_lens(&mut self, lens: &str) -> Result<()> {
        let lens = self.cdb().get_lens_err(lens)?.clone();
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

    //mi set_camera
    fn set_camera(&mut self, camera: CameraInstance) -> Result<()> {
        if self.project.ncips() == 0 {
            let cip: Rrc<Cip> = Cip::default().into();
            self.cip = cip.clone();
            self.project.add_cip(cip);
        }
        self.cip.borrow_mut().set_camera(camera.clone().into());
        self.camera = camera;
        Ok(())
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

    //mi set_write_mapping
    fn set_write_mapping(&mut self, s: &str) -> Result<()> {
        self.write_mapping = Some(s.to_owned());
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

    //fp add_arg_write_mapping
    pub fn add_arg_write_mapping(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "write_mapping",
            None,
            "File to write a derived mapping JSON to",
            ArgCount::Optional,
            None,
            CmdArgs::set_write_mapping,
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
