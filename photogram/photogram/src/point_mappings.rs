//a Imports

use clap::Command;
use thunderclap::{CommandArgs, CommandBuilder};

use ic_base::JsonParsable;
use ic_mapping::PointMappingSet;

use crate::cmd::{CmdArgs, CmdResult};

fn as_json_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    cmd_args.pms().borrow().to_json(cmd_args.pretty_json())
}

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

    CmdArgs::cmd_ok()
}

fn list_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let pms_n = cmd_args.get_pms_indices_of_nps()?;
    let pms = cmd_args.pms().borrow();
    let mappings = pms.mappings();

    for i in pms_n {
        let m = &mappings[i];
        println!(
            "{} : {} -> [{:.1}, {:.1}] @ {:.1}",
            m.named_point().ref_tag(),
            m.model(),
            m.screen()[0],
            m.screen()[1],
            m.error()
        );
    }
    CmdArgs::cmd_ok()
}

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
    CmdArgs::cmd_ok()
}

fn add_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    let name = cmd_args.get_string_arg(0).unwrap();
    let pxy = cmd_args.arg_as_point2d(1)?;
    let error = cmd_args.get_f64_arg(0).unwrap_or(0.0);
    let Some(_n) =
        cmd_args
            .pms()
            .borrow_mut()
            .add_mapping(&cmd_args.nps().borrow(), name, &pxy, error)
    else {
        return Err(format!("Failed to add mapping for '{name}' to the point mapping set; it is probably not in the named point set").into());
    };
    CmdArgs::cmd_ok()
}

fn remove_cmd() -> CommandBuilder<CmdArgs> {
    let mut build =
        CommandBuilder::with_handler(Command::new("remove").about("Remove a CIP"), remove_fn);
    CmdArgs::add_arg_named_point(&mut build, (None, true));

    build
}

fn add_cmd() -> CommandBuilder<CmdArgs> {
    let mut build =
        CommandBuilder::with_handler(Command::new("add").about("Add a new CIP"), add_fn);

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

fn list_cmd() -> CommandBuilder<CmdArgs> {
    let mut build =
        CommandBuilder::with_handler(Command::new("list").about("Show point mappings"), list_fn);

    CmdArgs::add_arg_named_point(&mut build, (0,));

    build
}

fn as_json_cmd() -> CommandBuilder<CmdArgs> {
    CommandBuilder::with_handler(
        Command::new("as_json").about("Generate the JSON for the PMS"),
        as_json_fn,
    )
}

fn add_json_cmd() -> CommandBuilder<CmdArgs> {
    let mut build = CommandBuilder::with_handler(
        Command::new("add_json").about("Generate the JSON for the PMS"),
        add_json_fn,
    );
    CmdArgs::add_arg_positional_string(
        &mut build,
        "json_filename",
        "Filename of JSON PMS file to read",
        Some(1),
        None,
    );
    build
}

pub fn point_mappings_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("point_mappings").about("Operate on a point mapping set");

    let mut build = CommandBuilder::new(command);
    CmdArgs::add_arg_cip(&mut build, false);

    build.add_subcommand(as_json_cmd());
    build.add_subcommand(add_json_cmd());
    build.add_subcommand(list_cmd());
    build.add_subcommand(add_cmd());
    build.add_subcommand(remove_cmd());

    build
}
