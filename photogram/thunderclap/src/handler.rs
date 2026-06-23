//a Imports
use std::collections::HashMap;

use clap::ArgMatches;

use crate::{ArgFn, ArgResetFn, CommandArgs, CommandFn, CommandSet, ExecError};

//a CommandHandlerSet
//tp CommandHandlerSet
/// A crate-only visible type that maps a single command and its
/// arguments to appropriate functions
///
/// Subcommands of the command each have their own
/// [CommandHandlerSet], held in a hash table,
pub struct CommandHandlerSet<C: CommandArgs> {
    handler: Option<Box<dyn CommandFn<C>>>,
    sub_cmds: HashMap<String, CommandHandlerSet<C>>,
    arg_reset: Option<Box<dyn ArgResetFn<C>>>,
    args: Vec<(String, Box<dyn ArgFn<C>>)>,
}

//ip Default for CommandHandlerSet
impl<C: CommandArgs> std::default::Default for CommandHandlerSet<C> {
    fn default() -> Self {
        Self {
            handler: None,
            sub_cmds: HashMap::default(),
            arg_reset: None,
            args: vec![],
        }
    }
}

//ip CommandHandlerSet
impl<C: CommandArgs> CommandHandlerSet<C> {
    //cp new
    /// Create a new [CommandHandlerSet], packaging the data provided
    pub fn new(handler: Option<Box<dyn CommandFn<C>>>) -> Self {
        let sub_cmds = HashMap::default();
        let arg_reset = None;
        let args = vec![];

        Self {
            handler,
            sub_cmds,
            arg_reset,
            args,
        }
    }

    //mp set_arg_reset
    pub fn set_arg_reset(&mut self, handler: Box<dyn ArgResetFn<C>>) {
        self.arg_reset = Some(handler);
    }

    //mp add_arg
    pub fn add_arg(&mut self, name: String, handler: Box<dyn ArgFn<C>>) {
        self.args.push((name, handler));
    }

    //mp add_subcommand
    pub fn add_subcommand(&mut self, name: String, handler_set: Self) {
        self.sub_cmds.insert(name, handler_set);
    }

    //mp handle_args
    /// Handle all of the arguments in the application-specified order
    ///
    /// Each argument is expected to update 'cmd_args'; if an
    /// argument's [ArgFn] returns an error then all processing is
    /// stopped and that error is returned.
    pub fn handle_args(
        &self,
        cmd_set: &CommandSet<C>,
        cmd_args: &mut C,
        matches: &ArgMatches,
    ) -> Result<(), ExecError<C>> {
        if let Some(arg_reset_fn) = &self.arg_reset {
            (*arg_reset_fn)(cmd_args);
        }
        for (a, f) in self.args.iter() {
            if matches.contains_id(a) {
                f(cmd_set, cmd_args, matches)?;
            }
        }
        Ok(())
    }

    //mi execute_sub_cmd
    /// Execute a named subcommand of this handler
    ///
    /// The subcommand's handler is invoked.
    fn execute_sub_cmd(
        &self,
        cmd_set: &CommandSet<C>,
        subcommand: &str,
        cmd_args: &mut C,
        sub_matches: &ArgMatches,
    ) -> Result<C::Value, ExecError<C>> {
        let sub_handler_set = self
            .sub_cmds
            .get(subcommand)
            .expect("Subcommand was added to clap so there should be a match in the table");
        sub_handler_set.handle_args(cmd_set, cmd_args, sub_matches)?;
        sub_handler_set.handle_cmd(cmd_set, cmd_args, sub_matches)
    }

    //mi execute_cmd
    /// Execute the command function of this handler
    fn execute_cmd(
        &self,
        _cmd_set: &CommandSet<C>,
        cmd_args: &mut C,
    ) -> Result<C::Value, ExecError<C>> {
        self.handler
            .as_ref()
            .map(|handler| handler(cmd_args))
            .unwrap_or_else(|| C::cmd_ok())
            .map_err(ExecError::exec)
    }

    //mp handle_cmd
    /// Handle an 'ArgMatches' for this command, with a current set of 'CommandArgs'
    ///
    /// Either a subcommand of the handler is invoked, or if none
    /// is provided then the function for this handler is invoked
    pub fn handle_cmd(
        &self,
        cmd_set: &CommandSet<C>,
        cmd_args: &mut C,
        matches: &ArgMatches,
    ) -> Result<C::Value, ExecError<C>> {
        if let Some((subcommand, submatches)) = matches.subcommand() {
            self.execute_sub_cmd(cmd_set, subcommand, cmd_args, submatches)
        } else {
            self.execute_cmd(cmd_set, cmd_args)
        }
    }
}
