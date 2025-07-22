//a Imports
use std::collections::HashMap;
use std::ffi::OsString;

use clap::{Arg, ArgAction, ArgMatches, Command};

pub trait CommandArgs {
    type Error: std::convert::From<String>;
    type Value: std::default::Default;
}

pub trait CommandFn<C: CommandArgs>: Fn(&mut C) -> Result<C::Value, C::Error> + 'static {}

pub trait ArgFn<C: CommandArgs>: Fn(&mut C, &ArgMatches) -> Result<(), C::Error> + 'static {}

impl<C: CommandArgs, T: Fn(&mut C) -> Result<C::Value, C::Error> + 'static> CommandFn<C> for T {}
impl<C: CommandArgs, T: Fn(&mut C, &ArgMatches) -> Result<(), C::Error> + 'static> ArgFn<C> for T {}

//a CommandBuilder
//tp CommandBuilder
pub struct CommandBuilder<C: CommandArgs> {
    command: Command,
    handler: Option<Box<dyn CommandFn<C>>>,
    sub_cmds: HashMap<String, CommandBuilder<C>>,
    args: HashMap<String, Box<dyn ArgFn<C>>>,
}

impl<C: CommandArgs> CommandBuilder<C> {
    pub fn new(mut command: Command, handler: Option<Box<dyn CommandFn<C>>>) -> Self {
        if handler.is_none() {
            command = command.subcommand_required(true);
        }
        let sub_cmds = HashMap::default();
        let args = HashMap::default();
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
        self.args.insert(name, handler);
    }

    pub fn add_subcommand(&mut self, subcommand: Self) {
        self.sub_cmds
            .insert(subcommand.command.get_name().into(), subcommand);
    }

    pub fn build(self) -> (Command, CommandHandlerSet<C>) {
        let mut command = self.command;
        let handler = self.handler;
        let args = self.args;
        let mut sub_cmds = HashMap::default();
        for (k, sc) in self.sub_cmds.into_iter() {
            let (sc, schs) = sc.build();
            sub_cmds.insert(k, schs);
            command = command.subcommand(sc);
        }
        (command, CommandHandlerSet::new(handler, sub_cmds, args))
    }
}

//a CommandHandlerSet
pub struct CommandHandlerSet<C: CommandArgs> {
    handler: Option<Box<dyn CommandFn<C>>>,
    sub_cmds: HashMap<String, CommandHandlerSet<C>>,
    args: HashMap<String, Box<dyn ArgFn<C>>>,
}

impl<C: CommandArgs> CommandHandlerSet<C> {
    fn new(
        handler: Option<Box<dyn CommandFn<C>>>,
        sub_cmds: HashMap<String, CommandHandlerSet<C>>,
        args: HashMap<String, Box<dyn ArgFn<C>>>,
    ) -> Self {
        Self {
            handler,
            sub_cmds,
            args,
        }
    }

    fn handle_args(&self, cmd_args: &mut C, matches: &ArgMatches) -> Result<(), C::Error> {
        for (a, f) in self.args.iter() {
            if matches.contains_id(a) {
                f(cmd_args, matches)?;
            }
        }
        Ok(())
    }

    fn execute_sub_cmd(
        &self,
        subcommand: &str,
        cmd_args: &mut C,
        sub_matches: &ArgMatches,
    ) -> Result<C::Value, C::Error> {
        let Some(sub_handler_set) = self.sub_cmds.get(subcommand) else {
            panic!("Subcommand was added to clap so there should be a match in the table");
        };
        sub_handler_set.handle_matches(cmd_args, sub_matches)
    }

    fn execute_cmd(&self, cmd_args: &mut C) -> Result<C::Value, C::Error> {
        self.handler.as_ref().unwrap()(cmd_args)
    }

    fn handle_matches(&self, cmd_args: &mut C, matches: &ArgMatches) -> Result<C::Value, C::Error> {
        self.handle_args(cmd_args, matches);
        if let Some((subcommand, submatches)) = matches.subcommand() {
            self.execute_sub_cmd(subcommand, cmd_args, submatches)
        } else {
            self.execute_cmd(cmd_args)
        }
    }
}

//a CommandSet
pub struct CommandSet<C: CommandArgs> {
    command: Command,
    handler_set: CommandHandlerSet<C>,
}

impl<C: CommandArgs> CommandSet<C> {
    pub fn new(builder: CommandBuilder<C>) -> Self {
        let (command, handler_set) = builder.build();
        let command = command.no_binary_name(true);
        Self {
            command,
            handler_set,
        }
    }

    pub fn execute<I, T>(&mut self, itr: I, cmd_args: &mut C) -> Result<C::Value, C::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        match self.command.try_get_matches_from_mut(itr) {
            Err(e) if e.kind() == clap::error::ErrorKind::DisplayHelp => Ok(C::Value::default()),
            Err(e) if e.kind() == clap::error::ErrorKind::DisplayVersion => Ok(C::Value::default()),
            Err(e) => Ok(C::Value::default()),
            Ok(matches) => self.handler_set.handle_matches(cmd_args, &matches),
        }
    }

    pub fn execute_env(&mut self, cmd_args: &mut C) -> Result<C::Value, C::Error> {
        self.execute(std::env::args_os().skip(1), cmd_args)
    }
}

impl<C: CommandArgs> std::convert::From<CommandBuilder<C>> for CommandSet<C> {
    fn from(cb: CommandBuilder<C>) -> CommandSet<C> {
        CommandSet::new(cb)
    }
}
