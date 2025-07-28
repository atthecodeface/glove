//a Modules
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};

use ic_camera::CameraInstance;
use ic_mapping::{CameraPtMapping, PointMapping};

pub mod camera;
pub mod file_system;
pub mod image;
pub mod kernels;
pub mod mapping;
pub mod project;
pub mod threads;

pub mod builder;

use builder::{CommandArgs, CommandBuilder};
use ic_base::{Error, Result};
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

//mp add_arg_usize
pub fn add_arg_usize<C, F>(
    build: &mut CommandBuilder<C>,
    tag: &'static str,
    short: Option<char>,
    help: &'static str,
    default_value: Option<&'static str>,
    set: F,
    required: bool,
) where
    C: CommandArgs<Error = Error>,
    F: Fn(&mut C, usize) -> Result<()> + 'static,
{
    let mut arg = Arg::new(tag)
        .long(tag)
        .help(help)
        .value_parser(value_parser!(usize))
        .required(required)
        .action(ArgAction::Set);
    if let Some(short) = short {
        arg = arg.short(short);
    }
    if let Some(default_value) = default_value {
        arg = arg.default_value(default_value);
    }
    build.add_arg(
        arg,
        Box::new(move |args, matches| set(args, *matches.get_one::<usize>(tag).unwrap())),
    );
}

//mp add_arg_isize
pub fn add_arg_isize<C, F>(
    build: &mut CommandBuilder<C>,
    tag: &'static str,
    short: Option<char>,
    help: &'static str,
    default_value: Option<&'static str>,
    set: F,
    required: bool,
) where
    C: CommandArgs<Error = Error>,
    F: Fn(&mut C, isize) -> Result<()> + 'static,
{
    let mut arg = Arg::new(tag)
        .long(tag)
        .help(help)
        .value_parser(value_parser!(isize))
        .required(required)
        .action(ArgAction::Set);
    if let Some(short) = short {
        arg = arg.short(short);
    }
    if let Some(default_value) = default_value {
        arg = arg.default_value(default_value);
    }
    build.add_arg(
        arg,
        Box::new(move |args, matches| set(args, *matches.get_one::<isize>(tag).unwrap())),
    );
}

//mp add_arg_f32
pub fn add_arg_f32<C, F>(
    build: &mut CommandBuilder<C>,
    tag: &'static str,
    short: Option<char>,
    help: &'static str,
    default_value: Option<&'static str>,
    set: F,
    required: bool,
) where
    C: CommandArgs<Error = Error>,
    F: Fn(&mut C, f32) -> Result<()> + 'static,
{
    let mut arg = Arg::new(tag)
        .long(tag)
        .help(help)
        .value_parser(value_parser!(f32))
        .required(required)
        .action(ArgAction::Set);
    if let Some(short) = short {
        arg = arg.short(short);
    }
    if let Some(default_value) = default_value {
        arg = arg.default_value(default_value);
    }
    build.add_arg(
        arg,
        Box::new(move |args, matches| set(args, *matches.get_one::<f32>(tag).unwrap())),
    );
}

//mp add_arg_f64
pub fn add_arg_f64<C, F>(
    build: &mut CommandBuilder<C>,
    tag: &'static str,
    short: Option<char>,
    help: &'static str,
    default_value: Option<&'static str>,
    set: F,
    required: bool,
) where
    C: CommandArgs<Error = Error>,
    F: Fn(&mut C, f64) -> Result<()> + 'static,
{
    let mut arg = Arg::new(tag)
        .long(tag)
        .help(help)
        .value_parser(value_parser!(f64))
        .required(required)
        .action(ArgAction::Set);
    if let Some(short) = short {
        arg = arg.short(short);
    }
    if let Some(default_value) = default_value {
        arg = arg.default_value(default_value);
    }
    build.add_arg(
        arg,
        Box::new(move |args, matches| set(args, *matches.get_one::<f64>(tag).unwrap())),
    );
}
//mp add_arg_string
pub fn add_arg_string<C, F>(
    build: &mut CommandBuilder<C>,
    tag: &'static str,
    short: Option<char>,
    help: &'static str,
    default_value: Option<&'static str>,
    set: F,
    required: bool,
) where
    C: CommandArgs<Error = Error>,
    F: Fn(&mut C, &str) -> Result<()> + 'static,
{
    let mut arg = Arg::new(tag)
        .long(tag)
        .help(help)
        .required(required)
        .action(ArgAction::Set);
    if let Some(short) = short {
        arg = arg.short(short);
    }
    if let Some(default_value) = default_value {
        arg = arg.default_value(default_value);
    }
    build.add_arg(
        arg,
        Box::new(move |args, matches| set(args, matches.get_one::<String>(tag).unwrap().as_ref())),
    );
}
