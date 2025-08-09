//a Imports
use std::collections::HashMap;
use std::ffi::OsString;
use std::io::Write;
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
    use_builtins: bool,
    show_result: bool,
}

//ip CommandSet
impl<C: CommandArgs> CommandSet<C> {
    //cp new
    /// Create a new command set, for a subcommand
    pub(crate) fn new(
        command: Command,
        handler_set: CommandHandlerSet<C>,
        use_builtins: bool,
    ) -> Self {
        Self {
            command,
            handler_set,
            cmd_stack: vec![],
            variables: HashMap::default(),
            result_history: vec![],
            use_builtins,
            show_result: true,
        }
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
        let mut use_builtins = false;
        if allow_interactive || allow_batch {
            command = Self::add_builtins(command);
            use_builtins = true;
        }
        if allow_batch {
            command = command.subcommand_required(false);
            command = command.arg(
                Arg::new("batch")
                    .long("batch")
                    .help("Execute a batch set of commands")
                    .action(ArgAction::Append),
            );
        }
        Self::new(command, handler_set, use_builtins)
    }

    //mi add_builtins
    fn add_builtins(command: Command) -> Command {
        command
            .subcommand(
                Command::new("set")
                    .about("Set a thunderclap variable to a value")
                    .arg(Arg::new("key").help("Variable name to set").required(true))
                    .arg(
                        Arg::new("value")
                            .help("Value to set the variable name to")
                            .required(true),
                    ),
            )
            .subcommand(
                Command::new("show")
                    .about("Show a value from the command argument set")
                    .arg(Arg::new("key").help("Keys to show").required(false).action(ArgAction::Append)),
            )
            .subcommand(
                Command::new("echo")
                    .about("Print to a file or stdout")
                    .arg(
                        Arg::new("file")
                            .long("file")
                            .short('f')
                            .help("File to write output to")
                            .required(false),
                    )
                    .arg(
                        Arg::new("append")
                            .long("append")
                            .short('a')
                            .help("If writing to file, then append, don't overwrite")
                            .required(false)
                            .action(ArgAction::SetTrue),
                    )
                    .arg(
                        Arg::new("values")
                            .help("Values to print out")
                            .required(true)
                            .action(ArgAction::Append),
                    ),
            )
            .subcommand(
                Command::new("stack_show")
                    .about("Show the values on the value history stack")
            )
            .subcommand(
                Command::new("stack_push")
                    .about("Push values onto the value history stack")
                    .arg(
                        Arg::new("values")
                            .help("Values to push onto the stack; default is to push the last nonempty result")
                            .required(false)
                            .action(ArgAction::Append),
                    ),
            )
            .subcommand(
                Command::new("stack_clear")
                    .about("Clear the value history stack")
            )
            .subcommand(
                Command::new("stack_pop")
                    .about("Pop one (or more) values from the stack")
                    .arg(
                        Arg::new("n")
                            .help("Number of value to pop from the stacks")
                            .default_value("1")
                            .action(ArgAction::Set),
                    ),
            )
    }

    //mi handle_builtin_echo
    fn handle_builtin_echo(
        &self,
        _cmd_args: &mut C,
        matches: &ArgMatches,
    ) -> Result<String, C::Error> {
        let mut file = {
            if let Some(filename) = matches.get_one::<String>("file") {
                let mut options = std::fs::File::options();
                if matches.get_one::<bool>("append") == Some(&true) {
                    options.append(true);
                    options.create(true);
                } else {
                    options.write(true);
                    options.create(true);
                }
                Some(options.open(filename).map_err(|e| {
                    format!("Failed to create '{filename}' to echo output to ({e})")
                })?)
            } else {
                None
            }
        };
        for v in matches.get_many::<String>("values").unwrap() {
            if let Some(file) = &mut file {
                writeln!(file, "{v}")
                    .map_err(|_e| "Failed to write to echo output file".to_string())?;
            } else {
                println!("{v}");
            }
        }
        Ok("".into())
    }

    //mi handle_builtin_set
    fn handle_builtin_set(
        &mut self,
        cmd_args: &mut C,
        matches: &ArgMatches,
    ) -> Result<String, C::Error> {
        let k = matches.get_one::<String>("key").unwrap();
        let v = matches.get_one::<String>("value").unwrap();
        if !cmd_args.value_set(k, v)? {
            self.variables.insert(k.into(), Rc::new(v.into()));
        }
        Ok("".into())
    }

    //mi handle_builtin_show
    fn handle_builtin_show(
        &self,
        cmd_args: &mut C,
        matches: &ArgMatches,
    ) -> Result<String, C::Error> {
        if let Some(keys) = matches.get_many::<String>("key") {
            for k in keys {
                let Some(v) = cmd_args.value_str(k) else {
                    return Err("Argument set does not have a value for 'k'"
                        .to_string()
                        .into());
                };
                println!("{k:20}: {v}");
            }
            Ok("".into())
        } else {
            for k in cmd_args.keys() {
                if let Some(v) = cmd_args.value_str(k) {
                    println!("{k:20}: {v}");
                }
            }
            Ok("".into())
        }
    }

    //mi handle_builtin_stack_show
    fn handle_builtin_stack_show(
        &mut self,
        _cmd_args: &mut C,
        _matches: &ArgMatches,
    ) -> Result<String, C::Error> {
        for (i, v) in self.result_history.iter().rev().enumerate() {
            println!("{i:4} : {v}");
        }
        Ok("".into())
    }

    //mi handle_builtin_stack_clear
    fn handle_builtin_stack_clear(
        &mut self,
        _cmd_args: &mut C,
        _matches: &ArgMatches,
    ) -> Result<String, C::Error> {
        if self.result_history.len() > 1 {
            let _ = self.result_history.drain(1..);
        }
        Ok("".into())
    }

    //mi handle_builtin_stack_pop
    fn handle_builtin_stack_pop(
        &mut self,
        _cmd_args: &mut C,
        _matches: &ArgMatches,
    ) -> Result<String, C::Error> {
        if self.result_history.len() > 1 {
            Ok(Rc::into_inner(self.result_history.remove(1)).unwrap())
        } else {
            Err("Value stack underflow in pop".to_owned().into())
        }
    }

    //mi handle_builtin_stack_push
    fn handle_builtin_stack_push(
        &mut self,
        _cmd_args: &mut C,
        matches: &ArgMatches,
    ) -> Result<String, C::Error> {
        if let Some(values) = matches.get_many::<String>("values") {
            if !self.result_history.is_empty() {
                self.result_history.pop();
            }
            for v in values {
                self.result_history.push(Rc::new(v.to_owned()));
            }
            if !self.result_history.is_empty() {
                self.result_history
                    .push(self.result_history.last().unwrap().clone());
            }
        } else if !self.result_history.is_empty() {
            self.result_history
                .push(self.result_history.last().unwrap().clone());
        }
        Ok("".into())
    }

    //mi handle_builtins
    fn handle_builtins(
        &mut self,
        cmd_args: &mut C,
        matches: &ArgMatches,
    ) -> Result<Option<String>, C::Error> {
        match matches.subcommand_name() {
            Some("echo") => self
                .handle_builtin_echo(cmd_args, matches.subcommand().unwrap().1)
                .map(Some),
            Some("show") => self
                .handle_builtin_show(cmd_args, matches.subcommand().unwrap().1)
                .map(Some),
            Some("set") => self
                .handle_builtin_set(cmd_args, matches.subcommand().unwrap().1)
                .map(Some),
            Some("stack_show") => self
                .handle_builtin_stack_show(cmd_args, matches.subcommand().unwrap().1)
                .map(Some),
            Some("stack_push") => self
                .handle_builtin_stack_push(cmd_args, matches.subcommand().unwrap().1)
                .map(Some),
            Some("stack_pop") => self
                .handle_builtin_stack_pop(cmd_args, matches.subcommand().unwrap().1)
                .map(Some),
            Some("stack_clear") => self
                .handle_builtin_stack_clear(cmd_args, matches.subcommand().unwrap().1)
                .map(Some),
            _ => Ok(None),
        }
    }

    //mi substitute
    /// Substitute variables etc
    fn substitute(&self, cmd_args: &C, s: String) -> Result<String, C::Error> {
        if !s.contains('$') {
            return Ok(s);
        }
        let mut result = String::new();
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            if c != '$' {
                result.push(c);
                continue;
            }
            let Some(nc) = chars.next() else {
                result.push(c);
                return Ok(s);
            };
            if nc != '{' {
                result.push(c);
                continue;
            }
            if let Some((name, rest)) = chars.as_str().split_once('}') {
                if let Some(v) = self.variables.get(name) {
                    result += v;
                } else if let Some(v) = cmd_args.value_str(name) {
                    result += &v;
                } else if let Ok(v) = name.parse::<usize>() {
                    let n = self.result_history.len();
                    if v < n {
                        result += &self.result_history[n - 1 - v];
                    }
                } else {
                    return Err(format!("Failed to evaulate ${{{name}}}").into());
                }
                chars = rest.chars();
            } else {
                result.push('$');
                result.push('{');
                continue;
            }
        }
        Ok(result)
    }

    //mi parse_str
    /// Parse a str into a Vec<String>
    fn parse_str(&mut self, cmd_args: &C, l: &str) -> Result<Vec<String>, C::Error> {
        let mut parsed = vec![];
        let mut token: Option<String> = None;
        let mut delimiter: Option<char> = None;
        let mut escape = false;
        for c in l.chars() {
            if token.is_none() {
                if c.is_whitespace() {
                    continue;
                } else if c == '"' || c == '\'' {
                    delimiter = Some(c);
                    token = Some(String::new());
                } else {
                    token = Some(String::new());
                    token.as_mut().unwrap().push(c);
                }
            } else if escape {
                token.as_mut().unwrap().push(c);
            } else if let Some(dc) = delimiter {
                if c == dc {
                    if dc == '"' {
                        parsed.push(self.substitute(cmd_args, token.take().unwrap())?);
                    } else {
                        parsed.push(token.take().unwrap());
                    }
                    delimiter = None;
                } else if c == '\\' {
                    escape = true;
                } else {
                    token.as_mut().unwrap().push(c);
                }
            } else if c == '\\' {
                escape = true;
            } else if c.is_whitespace() {
                parsed.push(self.substitute(cmd_args, token.take().unwrap())?);
            } else {
                token.as_mut().unwrap().push(c);
            }
        }
        // Should check delimiter is none, escape is false
        if let Some(token) = token {
            if delimiter != Some('\'') {
                parsed.push(self.substitute(cmd_args, token)?);
            } else {
                parsed.push(token);
            }
        }
        Ok(parsed)
    }

    //mi execute_str_line
    /// Execute commands from a single-line[str]
    fn execute_str_line(&mut self, cmd_args: &mut C, l: &str) -> Result<(), C::Error> {
        let l = l.trim();
        let s = self.parse_str(cmd_args, l)?;
        if !s.is_empty() {
            if s[0].as_bytes()[0] == b'#' {
                return Ok(());
            }
            self.execute(cmd_args, s)?;
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

    //mi executed_result
    fn executed_result(&mut self, result: String) {
        if !result.is_empty() {
            if !self.result_history.is_empty() {
                self.result_history.pop();
            }
            self.result_history.push(Rc::new(result));
        }
    }

    //mi execute
    /// Execute at the top level, given an iterator that provides the arguments
    ///
    /// It is deemed to be executed from 'cmd_stack.last()';
    fn execute<I, T>(&mut self, cmd_args: &mut C, itr: I) -> Result<(), C::Error>
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
                if self.use_builtins {
                    if let Some(result) = self.handle_builtins(cmd_args, &matches)? {
                        self.executed_result(result);
                        return Ok(());
                    }
                }
                if matches.contains_id("batch") {
                    self.show_result = false;
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
                let result = self.handler_set.handle_cmd(cmd_args, &matches)?;
                self.executed_result(result);
                Ok(())
            }
        }
    }

    //mp execute_env
    pub fn execute_env(&mut self, cmd_args: &mut C) -> Result<String, C::Error> {
        let mut iter = std::env::args_os();
        let cmd_name = iter.next().unwrap();
        self.cmd_stack
            .push((cmd_name.to_str().unwrap().into(), None));
        self.variables.clear();
        for (k, v) in std::env::vars() {
            self.variables.insert(k, Rc::new(v));
        }
        match self.execute(cmd_args, iter) {
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(4);
            }
            _x => {
                let result = {
                    if self.result_history.is_empty() {
                        "".into()
                    } else {
                        Rc::into_inner(self.result_history.remove(0)).unwrap()
                    }
                };
                if self.show_result {
                    println!("{result}");
                }
                Ok(result)
            }
        }
    }
}
