//a Imports

use clap::{Arg, ArgAction, ArgMatches, Command};
use thunderclap::{CommandArgs, CommandBuilder};

use ic_base::{Error, Result};
use ic_camera::CameraInstance;
use ic_mapping::{CameraPtMapping, PointMapping};

//a Modules
pub mod camera;
pub mod file_system;
pub mod image;
pub mod kernels;
pub mod mapping;
pub mod project;
pub mod threads;

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
