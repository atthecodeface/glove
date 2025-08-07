//a Imports
use std::collections::HashMap;
use std::ffi::OsString;
use std::rc::Rc;

use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::{ArgFn, ArgResetFn, CommandArgs, CommandBuilder, CommandFn};

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

    pub fn set_arg_reset(&mut self, handler: Box<dyn ArgResetFn<C>>) {
        self.arg_reset = Some(handler);
    }

    pub fn add_arg(&mut self, name: String, handler: Box<dyn ArgFn<C>>) {
        self.args.push((name, handler));
    }

    pub fn add_subcommand(&mut self, name: String, handler_set: Self) {
        self.sub_cmds.insert(name, handler_set);
    }

    //mi handle_args
    /// Handle all of the arguments in the application-specified order
    ///
    /// Each argument is expected to update 'cmd_args'; if an
    /// argument's [ArgFn] returns an error then all processing is
    /// stopped and that error is returned.
    fn handle_args(&self, cmd_args: &mut C, matches: &ArgMatches) -> Result<(), C::Error> {
        if let Some(arg_reset_fn) = &self.arg_reset {
            (*arg_reset_fn)(cmd_args);
        }
        for (a, f) in self.args.iter() {
            if matches.contains_id(a) {
                if false {
                    for (i, mut r) in matches.get_raw_occurrences(a).unwrap().enumerate() {
                        if r.len() == 1 {
                            eprintln!(
                                "Arg '{a}' occurrence {} to value {:?}",
                                i + 1,
                                r.next().unwrap()
                            );
                        } else {
                            let mut l = format!("Arg '{a}' occurrence {} to value [", i + 1);
                            for v in r {
                                l += &format!("{v:?}, ");
                            }
                            eprintln!("{l}]");
                        }
                    }
                }
                f(cmd_args, matches)?;
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
        subcommand: &str,
        cmd_args: &mut C,
        sub_matches: &ArgMatches,
    ) -> Result<String, C::Error> {
        let Some(sub_handler_set) = self.sub_cmds.get(subcommand) else {
            panic!("Subcommand was added to clap so there should be a match in the table");
        };
        sub_handler_set.handle_args(cmd_args, sub_matches)?;
        sub_handler_set.handle_cmd(cmd_args, sub_matches)
    }

    //mi execute_cmd
    /// Execute the command function of this handler
    fn execute_cmd(&self, cmd_args: &mut C) -> Result<String, C::Error> {
        if self.handler.is_none() {
            Ok("".to_string())
        } else {
            let result = self.handler.as_ref().unwrap()(cmd_args)?;
            Ok(result.to_string())
        }
    }

    //mi handle_cmd
    /// Handle an 'ArgMatches' for this command, with a current set of 'CommandArgs'
    ///
    /// Either a subcommand of the handler is invoked, or if none
    /// is provided then the function for this handler is invoked
    fn handle_cmd(&self, cmd_args: &mut C, matches: &ArgMatches) -> Result<String, C::Error> {
        if let Some((subcommand, submatches)) = matches.subcommand() {
            self.execute_sub_cmd(subcommand, cmd_args, submatches)
        } else {
            self.execute_cmd(cmd_args)
        }
    }
}

//a CommandSet
//tp CommandSet
/// This is a 'built' command with its handlers, and handlers for the
/// hierarchy of subcommands.
///
/// This is created using a [crate::CommandBuilder], and its `main`
/// method.
pub struct CommandSet<C: CommandArgs> {
    command: Command,
    handler_set: CommandHandlerSet<C>,
    cmd_stack: Vec<(String, Option<usize>)>,
    variables: HashMap<String, Rc<String>>,
    result_history: Vec<Rc<String>>,
}

//ip CommandSet
impl<C: CommandArgs> CommandSet<C> {
    //cp new
    /// Create a new command set, for a subcommand
    pub(crate) fn new(command: Command, handler_set: CommandHandlerSet<C>) -> Self {
        Self {
            command,
            handler_set,
            cmd_stack: vec![],
            variables: HashMap::default(),
            result_history: vec![],
        }
    }

    //cp subcmd
    /// Create a new command set, for a subcommand
    pub(crate) fn subcmd(builder: CommandBuilder<C>) -> Self {
        let (command, handler_set) = builder.take();
        let command = command.no_binary_name(true);
        Self::new(command, handler_set)
    }

    //cp main
    /// Create a new command set as a 'main' command handler
    ///
    /// This is the toplevel command handler
    pub(crate) fn main(
        builder: CommandBuilder<C>,
        allow_batch: bool,
        allow_interactive: bool,
    ) -> Self {
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
        Self::new(command, handler_set)
    }

    //mi execute_str_line
    /// Execute commands from a single-line[str]
    fn execute_str_line(&mut self, cmd_args: &mut C, l: &str) -> Result<(), C::Error> {
        let l = l.trim();
        if !l.is_empty() {
            if l.as_bytes()[0] == b'#' {
                return Ok(());
            }
            let v = self.execute(cmd_args, l.split_whitespace())?;
            self.result_history.push(Rc::new(v));
        }
        Ok(())
    }

    //mi execute_str
    /// Execute commands from a [str]
    fn execute_str(&mut self, cmd_args: &mut C, s: &str) -> Result<(), C::Error> {
        for l in s.lines() {
            if let Some(c_l) = self.cmd_stack.last_mut() {
                c_l.1 = c_l.1.map(|x| x + 1);
            }
            self.execute_str_line(cmd_args, l)?;
        }
        Ok(())
    }

    //mi execute
    /// Execute at the top level, given an iterator that provides the arguments
    ///
    /// It is deemed to be executed from 'cmd_stack.last()';
    fn execute<I, T>(&mut self, cmd_args: &mut C, itr: I) -> Result<String, C::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        cmd_args.reset_args();
        let mut cmd = self.command.clone();
        if let Some((name, opt_line)) = self.cmd_stack.last() {
            if let Some(line) = opt_line {
                cmd = cmd.bin_name(format!("{name} line {line}"));
            } else {
                cmd = cmd.bin_name(name);
            }
        }
        match cmd.try_get_matches_from_mut(itr) {
            Err(e) => {
                e.exit();
            }
            Ok(matches) => {
                self.handler_set.handle_args(cmd_args, &matches)?;
                if matches.contains_id("batch") {
                    let batches: Vec<_> = matches
                        .get_many::<String>("batch")
                        .unwrap()
                        .map(|filename| {
                            (
                                filename.clone(),
                                std::fs::read_to_string(filename).map_err(|e| {
                                    format!("failed to load batch file {filename}: {e}")
                                }),
                            )
                        })
                        .collect();
                    for b in &batches {
                        if let Err(err) = &b.1 {
                            return Err(err.clone().into());
                        }
                    }
                    for (filename, s) in batches {
                        self.cmd_stack.push((filename, Some(0)));
                        self.execute_str(cmd_args, &s.unwrap())?;
                        self.cmd_stack.pop();
                    }
                }
                self.handler_set.handle_cmd(cmd_args, &matches)
            }
        }
    }

    //mp execute_env
    pub fn execute_env(&mut self, cmd_args: &mut C) -> Result<String, C::Error> {
        let mut iter = std::env::args_os();
        let cmd_name = iter.next().unwrap();
        self.cmd_stack
            .push((cmd_name.to_str().unwrap().into(), None));
        match self.execute(cmd_args, iter) {
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(4);
            }
            x => x,
        }
    }
}
