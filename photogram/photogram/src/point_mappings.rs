//a Imports

use clap::Command;
use thunderclap::CommandBuilder;

use ic_base::JsonParsable;
use ic_mapping::PointMappingSet;

use crate::cmd::{CmdArgs, CmdResult};

//a Help
//a Interrogate (show_mappings etc)
//fi as_json_cmd
fn as_json_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("as_json").about("Generate the JSON for the PMS");

    let build = CommandBuilder::new(command, Some(Box::new(as_json_fn)));
    build
}

//fi as_json_fn
fn as_json_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    cmd_args.pms().borrow().to_json(cmd_args.pretty_json())
}

//fi add_json_cmd
fn add_json_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("add_json").about("Generate the JSON for the PMS");

    let mut build = CommandBuilder::new(command, Some(Box::new(add_json_fn)));
    CmdArgs::add_arg_positional_string(
        &mut build,
        "json_filename",
        "Filename of JSON PMS file to read",
        Some(1),
        None,
    );
    build
}

//fi add_json_fn
fn add_json_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms_filename = cmd_args.get_string_arg(0).unwrap();
    let (_, (pms, pms_not_found)) = PointMappingSet::load_json_file(
        &cmd_args.path_set,
        pms_filename,
        &*cmd_args.nps().borrow(),
    )?;
    if !pms_not_found.is_empty() {
        eprintln!("Warning: {pms_not_found:?}");
    }
    cmd_args.pms().borrow_mut().merge(pms);
    Ok("".into())
}

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

//fi remove_cmd
fn remove_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("remove").about("Remove a CIP");

    let mut build = CommandBuilder::new(command, Some(Box::new(remove_fn)));
    CmdArgs::add_arg_named_point(&mut build, (None, true));

    build
}

//fi remove_fn
fn remove_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let mut pms_n = cmd_args.get_pms_indices_of_nps()?;
    pms_n.sort();

    for pms_n in pms_n.into_iter().rev() {
        if !cmd_args.pms().borrow_mut().remove_mapping(pms_n) {
            return Err(
                format!("Failed to remove mapping '{pms_n}' from the point mapping set").into(),
            );
        }
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

    build.add_subcommand(as_json_cmd());
    build.add_subcommand(add_json_cmd());
    build.add_subcommand(list_cmd());
    build.add_subcommand(add_cmd());
    build.add_subcommand(remove_cmd());

    build
}
