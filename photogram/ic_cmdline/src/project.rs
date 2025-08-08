//a Modules
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

use ic_base::{json, Result, Rrc};
use ic_project::{Cip, Project};

use crate::{camera, mapping};

//a Project
//fp add_project_arg
pub fn add_project_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("project")
            .long("proj")
            .alias("project")
            .required(required)
            .help("Project JSON")
            .action(ArgAction::Set),
    )
}

//fp get_project
pub fn get_project(matches: &ArgMatches) -> Result<Project> {
    if let Some(project_filename) = matches.get_one::<String>("project") {
        let project_json = json::read_file(project_filename)?;
        let project: Project = json::from_json("project", &project_json)?;
        Ok(project)
    } else {
        let mut project = Project::default();
        let cdb = camera::get_camera_database(matches)?;
        let nps = mapping::get_nps(matches)?;
        project.set_cdb(cdb);
        project.set_nps(nps.into());
        Ok(project)
    }
}

//a Cip
//fp add_cip_arg
pub fn add_cip_arg(cmd: Command, required: bool) -> Command {
    cmd.arg(
        Arg::new("cip")
            .long("cip")
            .value_parser(value_parser!(usize))
            .required(required)
            .help("CIP number (camera and PMS) within the project")
            .action(ArgAction::Set),
    )
}

//fp get_cip
pub fn get_cip(matches: &ArgMatches, project: &Project) -> Result<Rrc<Cip>> {
    if let Some(cip) = matches.get_one::<usize>("cip") {
        let cip = *cip;
        if cip < project.ncips() {
            Ok(project.cip(cip).clone())
        } else {
            Err(format!(
                "CIP {cip} is too large for the project (it has {} cips)",
                project.ncips()
            )
            .into())
        }
    } else {
        let cip = Cip::default();
        let camera = camera::get_camera(matches, project)?;
        let pms = mapping::get_pms(matches, &project.nps_ref())?;
        *cip.camera_mut() = camera;
        *cip.pms_mut() = pms;
        let cip = cip.into();
        Ok(cip)
    }
}
