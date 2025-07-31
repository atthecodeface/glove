//a Imports
use std::collections::HashMap;
use std::ffi::OsString;

use clap::{Arg, ArgAction, ArgMatches, Command};

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

    pub fn main(builder: CommandBuilder<C>, allow_batch: bool, allow_interactive: bool) -> Self {
        let (command, handler_set) = builder.take();
        let mut command = command.no_binary_name(true);
        if allow_batch {
            command = command.subcommand_required(false);
            command = command.arg(
                Arg::new("batch")
                    .long("batch")
                    .help("Execute a batch set of commands")
                    .action(ArgAction::Append),
            );
        }
        Self {
            command,
            handler_set,
        }
    }

    pub fn execute_str(
        &mut self,
        cmd_name: &str,
        s: &str,
        cmd_args: &mut C,
    ) -> Result<C::Value, C::Error> {
        let mut value_stack = vec![];
        for l in s.lines() {
            let l = l.trim();
            if l.is_empty() {
                continue;
            }
            let v = self.execute(cmd_name, l.split_whitespace(), cmd_args)?;
            value_stack.push(v);
        }
        if value_stack.is_empty() {
            Ok(C::Value::default())
        } else {
            Ok(value_stack.pop().unwrap())
        }
    }

    pub fn execute<I, T>(
        &mut self,
        cmd_name: &str,
        itr: I,
        cmd_args: &mut C,
    ) -> Result<C::Value, C::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let mut cmd = self.command.clone().bin_name(cmd_name);
        match cmd.try_get_matches_from_mut(itr) {
            Err(e) => {
                e.exit();
            }
            Ok(matches) => {
                if matches.contains_id("batch") {
                    let batches: Vec<_> = matches
                        .get_many::<String>("batch")
                        .unwrap()
                        .map(|filename| {
                            (
                                filename.clone(),
                                std::fs::read_to_string(filename)
                                    .map_err(|e| format!("Failed to load batch file {filename}")),
                            )
                        })
                        .collect();
                    for b in &batches {
                        if let Err(err) = &b.1 {
                            return Err(err.clone().into());
                        }
                    }
                    for (filename, s) in batches {
                        let _ = self.execute_str(&filename, &s.unwrap(), cmd_args)?;
                    }
                }
                self.handler_set.handle_matches(cmd_args, &matches)
            }
        }
    }

    pub fn execute_env(&mut self, cmd_args: &mut C) -> Result<C::Value, C::Error> {
        let mut iter = std::env::args_os();
        let cmd_name = iter.next().unwrap();
        match self.execute(cmd_name.to_str().unwrap(), iter, cmd_args) {
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(4);
            }
            x => x,
        }
    }
}
