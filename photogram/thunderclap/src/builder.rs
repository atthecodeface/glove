//a Imports
use std::collections::HashMap;

use clap::{Arg, Command};

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
}
