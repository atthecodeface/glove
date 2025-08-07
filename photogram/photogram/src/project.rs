//a Imports

use clap::Command;

use thunderclap::CommandBuilder;

use crate::cmd::{CmdArgs, CmdResult};

//a Help
//hi PROJECT_LONG_HELP
const PROJECT_LONG_HELP: &str = "\
Project help";

//a Project as a whole
//fp project_cmd
pub fn project_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("project")
        .about("Operate on a project as a whole")
        .long_about(PROJECT_LONG_HELP);

    let mut build = CommandBuilder::new(command, Some(Box::new(project_fn)));
    CmdArgs::add_arg_project(&mut build, false);
    CmdArgs::add_arg_nps(&mut build);

    build
}

//fi project_fn
fn project_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let camera = cmd_args.project().cip(0).borrow().camera().clone();

    eprintln!("Camera {camera:?}");
    eprintln!("Mapping {}", camera.borrow().map_model([0., 0., 0.].into()));
    println!(
        "{}",
        serde_json::to_string_pretty(cmd_args.project()).unwrap()
    );
    Ok("".into())
}
