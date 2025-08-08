//a Imports
use std::collections::HashMap;

use clap::{value_parser, Arg, ArgAction, Command};

use crate::{ArgFn, ArgResetFn, CommandArgs, CommandFn, CommandHandlerSet, CommandSet};

//a CommandBuilder
//tp CommandBuilder
pub struct CommandBuilder<C: CommandArgs> {
    command: Command,
    handler_set: CommandHandlerSet<C>,
    sub_cmds: HashMap<String, CommandBuilder<C>>,
}

//ip Default for CommandBuilder
impl<C: CommandArgs> std::default::Default for CommandBuilder<C> {
    fn default() -> Self {
        let command = Command::default();
        let handler_set = CommandHandlerSet::default();
        let sub_cmds = HashMap::default();
        Self {
            command,
            handler_set,
            sub_cmds,
        }
    }
}

//ip CommandBuilder
impl<C: CommandArgs> CommandBuilder<C> {
    //cp new
    pub fn new(mut command: Command, handler: Option<Box<dyn CommandFn<C>>>) -> Self {
        if handler.is_none() {
            command = command.subcommand_required(true);
        }
        let handler_set = CommandHandlerSet::new(handler);
        let sub_cmds = HashMap::default();
        Self {
            command,
            handler_set,
            sub_cmds,
        }
    }

    //mp set_arg_reset
    pub fn set_arg_reset(&mut self, handler: Box<dyn ArgResetFn<C>>) -> &mut Self {
        self.handler_set.set_arg_reset(handler);
        self
    }

    //mp add_arg
    pub fn add_arg(&mut self, arg: Arg, handler: Box<dyn ArgFn<C>>) -> &mut Self {
        let name = arg.get_id().as_str().into();
        self.command = std::mem::take(&mut self.command).arg(arg);
        self.handler_set.add_arg(name, handler);
        self
    }

    //mp add_subcommand
    pub fn add_subcommand(&mut self, subcommand: Self) -> &mut Self {
        self.sub_cmds
            .insert(subcommand.command.get_name().into(), subcommand);
        self
    }

    //mp build_subcommand
    pub fn build_subcommand(&mut self, subcommand: &mut Self) -> &mut Self {
        self.add_subcommand(std::mem::take(subcommand));
        self
    }

    //mc take
    pub(crate) fn take(self) -> (Command, CommandHandlerSet<C>) {
        let mut command = self.command;
        let mut handler_set = self.handler_set;
        for (k, sc) in self.sub_cmds.into_iter() {
            let (sc, schs) = sc.take();
            handler_set.add_subcommand(k, schs);
            command = command.subcommand(sc);
        }
        (command, handler_set)
    }

    //mp main
    /// Convert the builder into an actual [CommandSet] to be used by 'main'
    pub fn main(self, allow_batch: bool, allow_interactive: bool) -> CommandSet<C> {
        CommandSet::main(self, allow_batch, allow_interactive)
    }

    //mp add_flag
    pub fn add_flag<F>(
        &mut self,
        tag: &'static str,
        short: Option<char>,
        help: &'static str,
        set: F,
    ) where
        F: Fn(&mut C, bool) -> Result<(), C::Error> + 'static,
    {
        let mut arg = Arg::new(tag)
            .long(tag)
            .help(help)
            .action(ArgAction::SetTrue);
        if let Some(short) = short {
            arg = arg.short(short);
        }
        self.add_arg(
            arg,
            Box::new(move |args, matches| set(args, *matches.get_one::<bool>(tag).unwrap())),
        );
    }
}

//a ArgCount
#[derive(Debug, Default, Clone, Copy)]
pub enum ArgCount {
    #[default]
    Optional, // 0 or 1, not required, Set, num_args None
    Required,       // 1, required, Set, num_args None
    Exactly(usize), // n >= 1, required, Append, num args Some(n)
    Any,            // 0 to inf; not required, Append, num_args None
    Min(usize),     // n to inf, required, Append, num_args(n..)
    Max(usize),     // 0 to max, not required, Append, num_args(0..=n)
}

impl From<usize> for ArgCount {
    #[track_caller]
    fn from(n: usize) -> ArgCount {
        assert!(n != 0, "Cannot require exactly 0 occurrences");
        ArgCount::Exactly(n)
    }
}

impl From<Option<usize>> for ArgCount {
    #[track_caller]
    fn from(opt_n: Option<usize>) -> ArgCount {
        assert!(opt_n != Some(0), "Cannot require at most 0 occurrences");
        match opt_n {
            Some(n) => ArgCount::Max(n),
            _ => ArgCount::Any,
        }
    }
}

impl From<(usize,)> for ArgCount {
    #[track_caller]
    fn from((min,): (usize,)) -> ArgCount {
        match min {
            0 => ArgCount::Any,
            n => ArgCount::Min(n),
        }
    }
}

impl From<bool> for ArgCount {
    fn from(required: bool) -> ArgCount {
        if required {
            ArgCount::Required
        } else {
            ArgCount::Optional
        }
    }
}

impl ArgCount {
    fn required(&self) -> bool {
        use ArgCount::*;
        matches!(self, Required | Exactly(_) | Min(_))
    }
    fn action(&self) -> ArgAction {
        use ArgCount::*;
        match self {
            Optional => ArgAction::Set,
            Required => ArgAction::Set,
            _ => ArgAction::Append,
        }
    }
    fn num_args(&self) -> Option<clap::builder::ValueRange> {
        use ArgCount::*;
        match self {
            Exactly(n) => Some((*n).into()),
            Min(n) => Some((*n..).into()),
            Max(n) => Some((0..=*n).into()),
            _ => None,
        }
    }
}

//ap add_arg
macro_rules! add_arg {
    ($m:ident, $t: ty, ref $ft:ty ) => {
        impl<C: CommandArgs> CommandBuilder<C> {
            pub fn $m<F>(
                &mut self,
                tag: &'static str,
                short: Option<char>,
                help: &'static str,
                count: ArgCount,
                default_value: Option<&'static str>,
                set: F,
            ) where
                F: Fn(&mut C, &$ft) -> Result<(), C::Error> + 'static,
            {
                let required = count.required();
                let action = count.action();
                let num_args = count.num_args();
                let mut arg = Arg::new(tag)
                    .long(tag)
                    .help(help)
                    .value_parser(value_parser!($t))
                    .required(required)
                    .action(action);
                if let Some(num_args) = num_args {
                    arg = arg.num_args(num_args);
                }
                if let Some(short) = short {
                    arg = arg.short(short);
                }
                if let Some(default_value) = default_value {
                    arg = arg.default_value(default_value);
                }
                self.add_arg(
                    arg,
                    Box::new(move |args, matches| {
                        for v in matches.get_many::<$t>(tag).unwrap() {
                            set(args, &*v)?
                        }
                        Ok(())
                    }),
                );
            }
        }
    };
    ($m:ident, $t: ty, $ft:ty ) => {
        impl<C: CommandArgs> CommandBuilder<C> {
            pub fn $m<F>(
                &mut self,
                tag: &'static str,
                short: Option<char>,
                help: &'static str,
                count: ArgCount,
                default_value: Option<&'static str>,
                set: F,
            ) where
                F: Fn(&mut C, $ft) -> Result<(), C::Error> + 'static,
            {
                let required = count.required();
                let action = count.action();
                let num_args = count.num_args();
                let mut arg = Arg::new(tag)
                    .long(tag)
                    .help(help)
                    .value_parser(value_parser!($t))
                    .required(required)
                    .action(action);
                if let Some(num_args) = num_args {
                    arg = arg.num_args(num_args);
                }
                if let Some(short) = short {
                    arg = arg.short(short);
                }
                if let Some(default_value) = default_value {
                    arg = arg.default_value(default_value);
                }
                self.add_arg(
                    arg,
                    Box::new(move |args, matches| {
                        for v in matches.get_many::<$t>(tag).unwrap() {
                            set(args, *v)?
                        }
                        Ok(())
                    }),
                );
            }
        }
    };
    ($m:ident, $t: ty) => {
        add_arg!($m, $t, $t);
    };
}

add_arg!(add_arg_string, String, ref str);

add_arg!(add_arg_isize, isize);
add_arg!(add_arg_i128, i128);
add_arg!(add_arg_i64, i64);
add_arg!(add_arg_i32, i32);
add_arg!(add_arg_i16, i16);
add_arg!(add_arg_i8, i8);

add_arg!(add_arg_usize, usize);
add_arg!(add_arg_u128, u128);
add_arg!(add_arg_u64, u64);
add_arg!(add_arg_u32, u32);
add_arg!(add_arg_u16, u16);
add_arg!(add_arg_u8, u8);

add_arg!(add_arg_f64, f64);
add_arg!(add_arg_f32, f32);
