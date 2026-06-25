use clap::Command;

use crate::{ArgDescriptor, CommandArgs, CommandBuilder, CommandFn};

pub struct CmdDescriptor<C: CommandArgs> {
    name: &'static str,
    version: &'static str,
    about: &'static str,
    args: &'static [ArgDescriptor<C>],
    cmds: &'static [Self],
    handler: Option<&'static dyn CommandFn<C>>,
}

impl<C: CommandArgs> CmdDescriptor<C> {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            version: &"",
            about: &"",
            args: &[],
            cmds: &[],
            handler: None,
        }
    }
    pub const fn version(mut self, version: &'static str) -> Self {
        self.version = version;
        self
    }
    pub const fn about(mut self, about: &'static str) -> Self {
        self.about = about;
        self
    }
    pub const fn args(mut self, args: &'static [ArgDescriptor<C>]) -> Self {
        self.args = args;
        self
    }
    pub const fn handler(mut self, handler: &'static dyn CommandFn<C>) -> Self {
        self.handler = Some(handler);
        self
    }
    pub const fn cmds(mut self, cmds: &'static [Self]) -> Self {
        self.cmds = cmds;
        self
    }
    pub fn build(&self) -> CommandBuilder<C> {
        let mut command = Command::new(self.name);

        if self.version != "" {
            command = command.version(self.version);
        }
        if self.about != "" {
            command = command.about(self.about);
        }
        let mut build = if let Some(handler) = self.handler {
            CommandBuilder::with_handler(command, handler)
        } else {
            CommandBuilder::new(command)
        };
        build.add_args(self.args);
        for c in self.cmds {
            build.add_subcommand(c.build());
        }
        build
    }
}
