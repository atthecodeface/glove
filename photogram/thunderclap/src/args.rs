//a Imports
//a Modules
use clap::{value_parser, Arg, ArgAction};

use crate::{CommandArgs, CommandBuilder};

//mp add_flag
pub fn add_flag<C, F>(
    build: &mut CommandBuilder<C>,
    tag: &'static str,
    short: Option<char>,
    help: &'static str,
    set: F,
) where
    C: CommandArgs,
    F: Fn(&mut C, bool) -> Result<(), C::Error> + 'static,
{
    let mut arg = Arg::new(tag)
        .long(tag)
        .help(help)
        .value_parser(value_parser!(usize))
        .action(ArgAction::Set);
    if let Some(short) = short {
        arg = arg.short(short);
    }
    build.add_arg(
        arg,
        Box::new(move |args, matches| set(args, *matches.get_one::<bool>("verbose").unwrap())),
    );
}

//mp add_isize
pub fn add_isize<C, F>(
    build: &mut CommandBuilder<C>,
    tag: &'static str,
    short: Option<char>,
    help: &'static str,
    default_value: Option<&'static str>,
    set: F,
    required: bool,
) where
    C: CommandArgs,
    F: Fn(&mut C, isize) -> Result<(), C::Error> + 'static,
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

//mp add_u32
pub fn add_u32<C, F>(
    build: &mut CommandBuilder<C>,
    tag: &'static str,
    short: Option<char>,
    help: &'static str,
    default_value: Option<&'static str>,
    set: F,
    required: bool,
) where
    C: CommandArgs,
    F: Fn(&mut C, u32) -> Result<(), C::Error> + 'static,
{
    let mut arg = Arg::new(tag)
        .long(tag)
        .help(help)
        .value_parser(value_parser!(u32))
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
        Box::new(move |args, matches| set(args, *matches.get_one::<u32>(tag).unwrap())),
    );
}

//mp add_usize
pub fn add_usize<C, F>(
    build: &mut CommandBuilder<C>,
    tag: &'static str,
    short: Option<char>,
    help: &'static str,
    default_value: Option<&'static str>,
    set: F,
    required: bool,
) where
    C: CommandArgs,
    F: Fn(&mut C, usize) -> Result<(), C::Error> + 'static,
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

//mp add_f32
pub fn add_f32<C, F>(
    build: &mut CommandBuilder<C>,
    tag: &'static str,
    short: Option<char>,
    help: &'static str,
    default_value: Option<&'static str>,
    set: F,
    required: bool,
) where
    C: CommandArgs,
    F: Fn(&mut C, f32) -> Result<(), C::Error> + 'static,
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

//mp add_f64
pub fn add_f64<C, F>(
    build: &mut CommandBuilder<C>,
    tag: &'static str,
    short: Option<char>,
    help: &'static str,
    default_value: Option<&'static str>,
    set: F,
    required: bool,
) where
    C: CommandArgs,
    F: Fn(&mut C, f64) -> Result<(), C::Error> + 'static,
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
//mp add_string
pub fn add_string<C, F>(
    build: &mut CommandBuilder<C>,
    tag: &'static str,
    short: Option<char>,
    help: &'static str,
    default_value: Option<&'static str>,
    set: F,
    required: bool,
) where
    C: CommandArgs,
    F: Fn(&mut C, &str) -> Result<(), C::Error> + 'static,
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
