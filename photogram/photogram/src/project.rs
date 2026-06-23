use clap::Command;
use geo_nd::Vector;
use thunderclap::{CommandArgs, CommandBuilder};

use ic_camera::CameraProjection;

use crate::cmd::{CmdArgs, CmdResult};

//a Help
//hi LIST_LONG_HELP
const LIST_LONG_HELP: &str = "\
List help";

//fi as_json_fn
fn as_json_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    cmd_args.project().to_json(cmd_args.pretty_json())
}

//fi list_fn
fn list_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let ncips = cmd_args.project().ncips();
    for i in 0..ncips {
        let name = cmd_args.project().cip_name(i).unwrap();
        let cip = cmd_args.project().cip(&name).unwrap().clone();
        let cip = cip.borrow();
        let camera = cip.camera();
        let position = camera.borrow().position().clone();
        let is_placed = !position.is_zero();
        if is_placed {
            println!("Cip: '{name}' @ {position}",);
        } else {
            println!("Cip: '{name}' camera unplaced");
        }
    }
    CmdArgs::cmd_ok()
}

fn as_json_cmd() -> CommandBuilder<CmdArgs> {
    CommandBuilder::with_handler(
        Command::new("as_json")
            .about("As_Json the project as a *single* JSON file")
            .long_about(LIST_LONG_HELP),
        as_json_fn,
    )
}

fn list_cmd() -> CommandBuilder<CmdArgs> {
    CommandBuilder::with_handler(
        Command::new("list")
            .about("Operate on a list as a whole")
            .long_about(LIST_LONG_HELP),
        list_fn,
    )
}

pub fn project_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("project")
        .about("Operate on a camera/image/point mapping set")
        .version("0.1.0");

    let mut build = CommandBuilder::new(command);

    build.add_subcommand(list_cmd());
    build.add_subcommand(as_json_cmd());

    build
}
