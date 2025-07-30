//a Imports
use std::collections::HashMap;
use std::ffi::OsString;

use clap::{ArgMatches, Command};

use crate::{ArgFn, CommandArgs, CommandBuilder, CommandFn};

//a CommandHandlerSet
pub struct CommandHandlerSet<C: CommandArgs> {
    handler: Option<Box<dyn CommandFn<C>>>,
    sub_cmds: HashMap<String, CommandHandlerSet<C>>,
    args: Vec<(String, Box<dyn ArgFn<C>>)>,
}

impl<C: CommandArgs> CommandHandlerSet<C> {
    pub fn new(
        handler: Option<Box<dyn CommandFn<C>>>,
        sub_cmds: HashMap<String, CommandHandlerSet<C>>,
        args: Vec<(String, Box<dyn ArgFn<C>>)>,
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
        self.handle_args(cmd_args, matches)?;
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
        let (command, handler_set) = builder.take();
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
            Err(e) => {
                e.exit();
            }
            Ok(matches) => self.handler_set.handle_matches(cmd_args, &matches),
        }
    }

    pub fn execute_env(&mut self, cmd_args: &mut C) -> Result<C::Value, C::Error> {
        match self.execute(std::env::args_os().skip(1), cmd_args) {
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(4);
            }
            x => x,
        }
    }
}
