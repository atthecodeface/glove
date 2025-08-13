//a Imports

use clap::Command;
use thunderclap::CommandBuilder;

use crate::cmd::{CmdArgs, CmdResult};

//a Help
//a Interrogate (show_mappings etc)
//fi list_cmd
fn list_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("list").about("Show point mappings");

    let mut build = CommandBuilder::new(command, Some(Box::new(list_fn)));

    CmdArgs::add_arg_named_point(&mut build, (0,));

    build
}

//fi list_fn
fn list_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms_n = cmd_args.get_pms_indices_of_nps()?;
    let pms = cmd_args.pms().borrow();
    let mappings = pms.mappings();

    for i in pms_n {
        let m = &mappings[i];
        println!(
            "{} : {} -> [{:.1}, {:.1}] @ {:.1}",
            m.name(),
            m.model(),
            m.screen()[0],
            m.screen()[1],
            m.error()
        );
    }
    Ok("".into())
}

//fi add_cmd
fn add_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("add").about("Add a new CIP");

    let mut build = CommandBuilder::new(command, Some(Box::new(add_fn)));
    CmdArgs::add_arg_positional_string(&mut build, "name", "Named point name", Some(1), None);
    CmdArgs::add_arg_positional_string(
        &mut build,
        "pixelxy",
        "Location of point on the sensor",
        Some(1),
        None,
    );
    CmdArgs::add_arg_positional_f64(
        &mut build,
        "error",
        "Error in the location",
        Some(0),
        Some("0.0"),
    );

    build
}

//fi add_fn
fn add_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let name = cmd_args.get_string_arg(0).unwrap();
    let pxy = cmd_args.arg_as_point2d(1)?;
    let error = cmd_args.get_f64_arg(0).unwrap_or(0.0);
    if !cmd_args
        .pms()
        .borrow_mut()
        .add_mapping(&cmd_args.nps().borrow(), name, &pxy, error)
    {
        Err(format!("Failed to add mapping for '{name}' to the point mapping set; it is probably not in the named point set").into())
    } else {
        Ok("".into())
    }
}

//a point_mappings command
//fp point_mappings_cmd
pub fn point_mappings_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("point_mappings").about("Operate on a point mapping set");

    let mut build = CommandBuilder::new(command, None);
    CmdArgs::add_arg_cip(&mut build, false);

    build.add_subcommand(list_cmd());
    build.add_subcommand(add_cmd());

    build
}
