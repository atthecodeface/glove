//a Modules
use clap::{Arg, ArgAction, ArgMatches, Command};

use ic_camera::CameraInstance;
use ic_mapping::{CameraPtMapping, PointMapping};
use thunderclap::{ArgCount, CommandArgs, CommandBuilder};

pub mod camera;
pub mod file_system;
pub mod image;
pub mod kernels;
pub mod mapping;
pub mod project;
pub mod threads;

use ic_base::{Error, Result};

//a CmdResult
type CmdResult = std::result::Result<String, ic_base::Error>;
fn cmd_ok() -> CmdResult {
    Ok("".into())
}

//a CmdArgs
//tp CmdArgs
use ic_camera::CameraDatabase;
use ic_image::Color;
use ic_mapping::NamedPointSet;
use ic_mapping::PointMappingSet;
use ic_project::Project;
use std::cell::{Ref, RefMut};
use std::rc::Rc;
#[derive(Default)]
pub struct CmdArgs {
    verbose: bool,
    project: Project,

    cdb: Option<CameraDatabase>,

    // camera: CameraInstance,
    write_camera: Option<String>,
    write_mapping: Option<String>,
    write_star_mapping: Option<String>,
    write_polys: Option<String>,
    read_img: Vec<String>,
    write_img: Option<String>,
    write_svg: Option<String>,

    nps: Rc<NamedPointSet>,
    pms: PointMappingSet,
    camera: CameraInstance,
    pms_color: Option<Color>,
    model_color: Option<Color>,
    np: Vec<String>, // could be name, 3D, pixel XY (from camera mapping of 3D); might need at least 3
}

//ip CommandArgs for CmdArgs
impl CommandArgs for CmdArgs {
    type Error = ic_base::Error;
    type Value = String;
}

//ip CmdArgs
impl CmdArgs {
    //a Types
    fn project(&self) -> &Project {
        &self.project
    }
    fn nps(&self) -> Ref<NamedPointSet> {
        self.project.nps_ref()
    }
    fn cdb(&self) -> Ref<CameraDatabase> {
        self.project.cdb_ref()
    }

    fn camera_mut(&mut self) -> &mut CameraInstance {
        &mut self.camera
    }

    fn get_cdb(&self) -> &CameraDatabase {
        self.cdb.as_ref().unwrap()
    }

    fn set_verbose(&mut self, verbose: bool) -> Result<()> {
        self.verbose = verbose;
        Ok(())
    }
    fn set_cdb(&mut self, cdb: CameraDatabase) -> Result<()> {
        self.cdb = Some(cdb);
        Ok(())
    }
    fn set_camera(&mut self, camera: CameraInstance) -> Result<()> {
        self.camera = camera;
        Ok(())
    }
    fn set_read_img(&mut self, v: Vec<String>) -> Result<()> {
        self.read_img = v;
        Ok(())
    }
    fn set_write_img(&mut self, s: &str) -> Result<()> {
        self.write_img = Some(s.to_owned());
        Ok(())
    }
    fn set_write_camera(&mut self, s: &str) -> Result<()> {
        self.write_camera = Some(s.to_owned());
        Ok(())
    }
    fn set_write_mapping(&mut self, s: &str) -> Result<()> {
        self.write_mapping = Some(s.to_owned());
        Ok(())
    }
    fn set_write_star_mapping(&mut self, s: &str) -> Result<()> {
        self.write_star_mapping = Some(s.to_owned());
        Ok(())
    }
    fn set_write_polys(&mut self, s: &str) -> Result<()> {
        self.write_polys = Some(s.to_owned());
        Ok(())
    }
    fn set_write_svg(&mut self, s: &str) -> Result<()> {
        self.write_svg = Some(s.to_owned());
        Ok(())
    }

    fn set_pms_color(&mut self, s: &str) -> Result<()> {
        let c: Color = s.try_into()?;
        self.pms_color = Some(c);
        Ok(())
    }

    fn set_model_color(&mut self, s: &str) -> Result<()> {
        let c: Color = s.try_into()?;
        self.model_color = Some(c);
        Ok(())
    }

    //fp add_arg_pms
    // fn add_arg_pms(build: &mut CommandBuilder<Self>) {
    // build.add_arg_string(
    // "pms",
    // None,
    // "Maximum brightness of stars to use in the catalog",
    // Some("5.0"),
    // CmdArgs::set_brightness,
    // );
    // }

    //fp add_arg_camera
    fn add_arg_camera(build: &mut CommandBuilder<Self>) {
        camera::add_arg_camera(
            build,
            Self::get_cdb,
            Self::set_camera,
            Self::camera_mut,
            false,
        );
    }

    fn add_arg_pms_color(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "pms_color",
            None,
            "Color for PMS points",
            ArgCount::Optional,
            None,
            CmdArgs::set_pms_color,
        );
    }
    fn add_arg_model_color(build: &mut CommandBuilder<Self>) {
        build.add_arg_string(
            "model_color",
            None,
            "Color for mapped model crosses",
            ArgCount::Optional,
            None,
            CmdArgs::set_model_color,
        );
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
