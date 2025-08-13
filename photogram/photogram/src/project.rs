//a Imports

use clap::Command;

use thunderclap::CommandBuilder;

use ic_base::Point3D;
use ic_camera::CameraProjection;

use crate::cmd::{CmdArgs, CmdResult};

//a Help
//hi LIST_LONG_HELP
const LIST_LONG_HELP: &str = "\
List help";

//a Create JSON for the whole project
//fp as_json_cmd
pub fn as_json_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("as_json")
        .about("As_Json the project as a *single* JSON file")
        .long_about(LIST_LONG_HELP);

    let build = CommandBuilder::new(command, Some(Box::new(as_json_fn)));

    build
}

//fi as_json_fn
fn as_json_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    cmd_args.project().to_json(cmd_args.pretty_json())
}

//a List as a whole
//fp list_cmd
pub fn list_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("list")
        .about("Operate on a list as a whole")
        .long_about(LIST_LONG_HELP);

    let build = CommandBuilder::new(command, Some(Box::new(list_fn)));

    build
}

//fi list_fn
fn list_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let camera = cmd_args.project().cip(0).borrow().camera().clone();

    eprintln!("Camera {camera:?}");
    eprintln!(
        "Mapping {}",
        camera.borrow().world_xyz_to_px_abs_xy(Point3D::default())
    );
    println!(
        "{}",
        serde_json::to_string_pretty(cmd_args.project()).unwrap()
    );
    Ok("".into())
}

//a project command
//fp project_cmd
pub fn project_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("project")
        .about("Operate on a camera/image/point mapping set")
        .version("0.1.0");

    let mut build = CommandBuilder::new(command, None);

    build.add_subcommand(list_cmd());
    build.add_subcommand(as_json_cmd());

    build
}
