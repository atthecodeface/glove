//a Imports
use std::collections::HashMap;

use clap::{value_parser, Arg, ArgAction, Command};

use crate::{ArgFn, CommandArgs, CommandFn, CommandHandlerSet, CommandSet};

//a CommandBuilder
//tp CommandBuilder
pub struct CommandBuilder<C: CommandArgs> {
    command: Command,
    handler: Option<Box<dyn CommandFn<C>>>,
    sub_cmds: HashMap<String, CommandBuilder<C>>,
    args: Vec<(String, Box<dyn ArgFn<C>>)>,
}

impl<C: CommandArgs> CommandBuilder<C> {
    pub fn new(mut command: Command, handler: Option<Box<dyn CommandFn<C>>>) -> Self {
        if handler.is_none() {
            command = command.subcommand_required(true);
        }
        let sub_cmds = HashMap::default();
        let args = vec![];

        Self {
            command,
            handler,
            sub_cmds,
            args,
        }
    }

    pub fn add_arg(&mut self, arg: Arg, handler: Box<dyn ArgFn<C>>) {
        let name = arg.get_id().as_str().into();
        self.command = std::mem::take(&mut self.command).arg(arg);
        self.args.push((name, handler));
    }

    pub fn add_subcommand(&mut self, subcommand: Self) {
        self.sub_cmds
            .insert(subcommand.command.get_name().into(), subcommand);
    }

    pub(crate) fn take(self) -> (Command, CommandHandlerSet<C>) {
        let mut command = self.command;
        let handler = self.handler;
        let args = self.args;
        let mut sub_cmds = HashMap::default();
        for (k, sc) in self.sub_cmds.into_iter() {
            let (sc, schs) = sc.take();
            sub_cmds.insert(k, schs);
            command = command.subcommand(sc);
        }
        (command, CommandHandlerSet::new(handler, sub_cmds, args))
    }

    pub fn build(self) -> CommandSet<C> {
        CommandSet::new(self)
    }

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

//ap add_arg
macro_rules! add_arg {
    ($m:ident, $t: ty, ref $ft:ty ) => {
        impl<C: CommandArgs> CommandBuilder<C> {
            pub fn $m<F>(
                &mut self,
                tag: &'static str,
                short: Option<char>,
                help: &'static str,
                default_value: Option<&'static str>,
                set: F,
                required: bool,
            ) where
                F: Fn(&mut C, &$ft) -> Result<(), C::Error> + 'static,
            {
                let mut arg = Arg::new(tag)
                    .long(tag)
                    .help(help)
                    .value_parser(value_parser!($t))
                    .required(required)
                    .action(ArgAction::Set);
                if let Some(short) = short {
                    arg = arg.short(short);
                }
                if let Some(default_value) = default_value {
                    arg = arg.default_value(default_value);
                }
                self.add_arg(
                    arg,
                    Box::new(move |args, matches| {
                        let v = &*matches.get_one::<$t>(tag).unwrap();
                        set(args, v)
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
                default_value: Option<&'static str>,
                set: F,
                required: bool,
            ) where
                F: Fn(&mut C, $ft) -> Result<(), C::Error> + 'static,
            {
                let mut arg = Arg::new(tag)
                    .long(tag)
                    .help(help)
                    .value_parser(value_parser!($t))
                    .required(required)
                    .action(ArgAction::Set);
                if let Some(short) = short {
                    arg = arg.short(short);
                }
                if let Some(default_value) = default_value {
                    arg = arg.default_value(default_value);
                }
                self.add_arg(
                    arg,
                    Box::new(move |args, matches| {
                        let v = *matches.get_one::<$t>(tag).unwrap();
                        set(args, v)
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
