//a Imports
use std::collections::HashMap;
use std::ffi::OsString;
use std::io::Write;
use std::rc::Rc;

use clap::{Arg, ArgAction, ArgMatches, Command};

use crate::interactive::{CmdRequest, CmdResponse, shell};
use crate::{
    CmdProperty, CommandArgs, CommandArgsValue, CommandBuilder, CommandHandlerSet, ExecError,
};

//a CommandSet
//tp CommandSet
/// This is a 'built' command with its handlers, and handlers for the
/// hierarchy of subcommands.
///
/// This is created using a [crate::CommandBuilder], and its `main`
/// method.
pub struct CommandSet<C: CommandArgs> {
    command: Command,
    /// Command, but with the extra stuff for batch/interactive mode
    batch_command: Command,
    handler_set: CommandHandlerSet<C>,
    cmd_stack: Vec<(String, Option<usize>)>,
    variables: HashMap<String, Rc<C::Value>>,
    result_history: Vec<Rc<C::Value>>,
    use_batch: bool,
    use_builtins: bool,
    show_result: bool,
    allow_interactive: bool,
}

//ip CommandSet
impl<C: CommandArgs> CommandSet<C> {
    //cp new
    /// Create a new command set, for a subcommand
    pub(crate) fn new(
        command: Command,
        batch_command: Command,
        handler_set: CommandHandlerSet<C>,
        use_builtins: bool,
        use_batch: bool,
        allow_interactive: bool,
    ) -> Self {
        Self {
            command,
            batch_command,
            handler_set,
            cmd_stack: vec![],
            variables: HashMap::default(),
            result_history: vec![],
            use_builtins,
            use_batch,
            allow_interactive,
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
        if allow_batch {
            command = command.subcommand_required(false);
            command = command.arg(
                Arg::new("batch")
                    .long("batch")
                    .help("Execute a batch set of commands")
                    .action(ArgAction::Append),
            );
        }
        let (use_builtins, batch_command) = {
            if allow_interactive || allow_batch {
                (true, Self::add_builtins(command.clone()))
            } else {
                (false, command.clone())
            }
        };
        if allow_interactive {
            command = command.subcommand_required(false);
            command = command.arg(
                Arg::new("interactive")
                    .long("interactive")
                    .help("Run comamnds from stdin after executing any batches or other provided commands")
                    .action(ArgAction::SetTrue),
            );
        }
        Self::new(
            command,
            batch_command,
            handler_set,
            use_builtins,
            allow_batch,
            allow_interactive,
        )
    }

    //mi fold_matches
    pub fn fold_matches<F, T>(
        &self,
        cmd_args: &mut C,
        matches: &ArgMatches,
        tag: &str,
        mut acc: T,
        mut fold: F,
    ) -> Result<T, ExecError<C>>
    where
        F: FnMut(T, &mut C, &Rc<C::Value>) -> Result<T, ExecError<C>>,
    {
        for v in matches.get_many::<String>(tag).unwrap() {
            let (_rest, opt_value) = self.substitute_var(cmd_args, v)?;
            let value = {
                if let Some(value) = opt_value {
                    value
                } else {
                    Rc::new(C::value_from_str(v).map_err(ExecError::eval)?)
                }
            };
            acc = fold(acc, cmd_args, &value)?;
        }
        Ok(acc)
    }

    //mi add_builtins
    fn add_builtins(command: Command) -> Command {
        command
            .arg(
                Arg::new("ignore_errors")
                    .long("ignore_errors")
                    .help("Ignore errors - i.e. do not exit if an error occurs, but return an empty result")
                    .action(ArgAction::SetTrue))
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
    ) -> Result<C::Value, ExecError<C>> {
        let mut file = {
            if let Some(filename) = matches.get_one::<String>("file") {
                let mut options = std::fs::File::options();
                if matches.get_one::<bool>("append") == Some(&true) {
                    options.append(true);
                    options.create(true);
                } else {
                    options.write(true);
                    options.truncate(true);
                    options.create(true);
                }
                Some(options.open(filename).map_err(|e| {
                    format!("Failed to create '{filename}' to echo output to ({e})")
                })?)
            } else {
                None
            }
        };
        if let Some(file) = &mut file {
            for v in matches.get_many::<String>("values").unwrap() {
                writeln!(file, "{v}")?;
            }
        } else {
            for v in matches.get_many::<String>("values").unwrap() {
                print!("{v}");
            }
            println!();
        }
        ExecError::cmd_ok()
    }

    //mi set_variable_value_str
    fn set_variable_value_str(&mut self, k: &str, v: &str) -> Result<(), ExecError<C>> {
        // eprintln!("Set variable '{k}' to value '{v}'");
        self.variables.insert(
            k.into(),
            Rc::new(C::value_from_str(v).map_err(ExecError::eval)?),
        );
        Ok(())
    }

    //mi handle_builtin_set
    fn handle_builtin_set(
        &mut self,
        cmd_args: &mut C,
        matches: &ArgMatches,
    ) -> Result<C::Value, ExecError<C>> {
        let k = matches.get_one::<String>("key").unwrap();
        let value = self
            .fold_matches(cmd_args, matches, "value", None, |_, _, v| {
                Ok(Some(v.clone()))
            })?
            .unwrap();

        // If the client does not handle the 'set' then set a local variable
        if let Some(property) = CmdProperty::find_property(C::PROPERTIES, k) {
            if (property.set_value_fn)(cmd_args, &value).map_err(ExecError::set_arg)? {
                return ExecError::cmd_ok();
            }
        }

        self.variables.insert(k.into(), value);
        ExecError::cmd_ok()
    }

    //mi handle_builtin_show
    fn handle_builtin_show(
        &self,
        cmd_args: &mut C,
        matches: &ArgMatches,
    ) -> Result<C::Value, ExecError<C>> {
        if let Some(keys) = matches.get_many::<String>("key") {
            for k in keys {
                let Some(property) = CmdProperty::find_property(C::PROPERTIES, k) else {
                    return Err(format!("Argument set does not have a value for '{k}'").into());
                };
                let Some(v) = (property.get_fn)(cmd_args) else {
                    return Err(format!("Argument set does not have a value for '{k}'").into());
                };
                println!("{k:20}: {}", v.value_string());
            }
        } else {
            for property in C::PROPERTIES {
                let k = property.name;
                let Some(v) = (property.get_fn)(cmd_args) else {
                    return Err(format!("Argument set does not have a value for '{k}'").into());
                };
                println!("{k:20}: {}", v.value_string());
            }
        }
        ExecError::cmd_ok()
    }

    //mi handle_builtin_stack_show
    fn handle_builtin_stack_show(
        &mut self,
        _cmd_args: &mut C,
        _matches: &ArgMatches,
    ) -> Result<C::Value, ExecError<C>> {
        for (i, v) in self.result_history.iter().rev().enumerate() {
            println!("{i:4} : {}", v.value_string());
        }
        ExecError::cmd_ok()
    }

    //mi handle_builtin_stack_clear
    fn handle_builtin_stack_clear(
        &mut self,
        _cmd_args: &mut C,
        _matches: &ArgMatches,
    ) -> Result<C::Value, ExecError<C>> {
        if self.result_history.len() > 1 {
            let _ = self.result_history.drain(1..);
        }
        ExecError::cmd_ok()
    }

    //mi handle_builtin_stack_pop
    fn handle_builtin_stack_pop(
        &mut self,
        _cmd_args: &mut C,
        _matches: &ArgMatches,
    ) -> Result<C::Value, ExecError<C>> {
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
    ) -> Result<C::Value, ExecError<C>> {
        if let Some(values) = matches.get_many::<String>("values") {
            if !self.result_history.is_empty() {
                self.result_history.pop();
            }
            for v in values {
                dbg!(&v);
                self.result_history
                    .push(Rc::new(C::value_from_str(v).map_err(ExecError::eval)?));
            }
            if !self.result_history.is_empty() {
                self.result_history
                    .push(self.result_history.last().unwrap().clone());
            }
        } else if !self.result_history.is_empty() {
            self.result_history
                .push(self.result_history.last().unwrap().clone());
        }
        ExecError::cmd_ok()
    }

    //mi handle_builtins
    fn handle_builtins(
        &mut self,
        cmd_args: &mut C,
        matches: &ArgMatches,
    ) -> Result<Option<C::Value>, ExecError<C>> {
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

    //mi substitute_var
    /// Substitute variables etc
    pub fn substitute_var<'a>(
        &self,
        cmd_args: &C,
        s: &'a str,
    ) -> Result<(&'a str, Option<Rc<C::Value>>), ExecError<C>> {
        if s.is_empty() || s.as_bytes()[0] != b'{' {
            return Ok((s, None));
        }
        let Some((name, rest)) = s.split_at(1).1.split_once('}') else {
            return Err("Bad variable specification - no closing '}'"
                .to_string()
                .into());
        };
        let mut value = {
            if let Ok(v) = name.parse::<usize>() {
                let n = self.result_history.len();
                if v >= n {
                    return Err(format!("Result stack is only {n} deep but requested {v}").into());
                }
                self.result_history[n - 1 - v].clone()
            } else if let Some(v) = self.variables.get(name) {
                v.clone()
            } else if let Some(property) = CmdProperty::find_property(C::PROPERTIES, name) {
                let Some(v) = (property.get_fn)(cmd_args) else {
                    return Err(format!("Failed to get proerty value for {name}'").into());
                };
                Rc::new(v)
            } else {
                return Err(format!("Failed to evaluate '${{{name}}}'").into());
            }
        };
        let mut rest = rest;
        while (C::Value::CAN_GET || C::Value::CAN_INDEX) && !rest.is_empty() {
            value = {
                if rest.as_bytes()[0] == b'[' {
                    let Some((index, new_rest)) = rest.split_at(1).1.split_once(']') else {
                        return Err(
                            "Unterminated index (no ']') in variable substitution in script"
                                .to_string()
                                .into(),
                        );
                    };
                    rest = new_rest;
                    if let Ok(index) = index.parse::<usize>() {
                        let Some(value) = value.index(index) else {
                            return Err(format!(
                                "Failed to get value at index {index} of value in script"
                            )
                            .into());
                        };
                        Rc::new(value)
                    } else {
                        let Some(value) = value.get(index) else {
                            return Err(
                                format!("Failed to get key '{index}' of value in script").into()
                            );
                        };
                        Rc::new(value)
                    }
                } else {
                    break;
                }
            };
        }
        Ok((rest, Some(value)))
    }

    //mi substitute
    /// Substitute variables etc
    ///
    /// If the supplied string starts with a double-quote then replace any variable with unquoted values (if a string value)
    ///
    /// If the supplied string does not start with a double-quote then replace directly
    fn substitute(
        &self,
        cmd_args: &C,
        s: String,
        in_dbl_quote: bool,
    ) -> Result<String, ExecError<C>> {
        if s.is_empty() || !s.contains('$') {
            return Ok(s);
        }
        let mut result = String::new();
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            if c != '$' {
                result.push(c);
                continue;
            }
            let (rest, opt_value) = self.substitute_var(cmd_args, chars.as_str())?;
            if let Some(value) = opt_value {
                let value_string = value.value_string();
                if in_dbl_quote && value_string.len() > 2 && value_string.as_bytes()[0] == b'"' {
                    result += value_string.trim_matches('"');
                } else {
                    result += &value_string;
                }
                chars = rest.chars();
            } else {
                result.push(c);
            }
        }
        Ok(result)
    }

    //mi parse_str
    /// Parse a str into a Vec<String>
    ///
    /// This is provided with a complete batch file line, and should
    /// produce the appropriate result
    ///
    /// Parsing happens by left-to-right analysis, looking for tokens
    /// which are (generally) copied verbatim - a token being
    /// non-whitespace.
    ///
    /// However, a '#' introduces a comment, and so the '#' and the
    /// rest of the line are discarded and parsing finishes
    ///
    /// A double quote introduces a string that continues until the
    /// next non-escaped double quote; such a string goes through
    /// string variable subsitution.
    ///
    /// A single quote introduces a string that continues until the
    /// next non-escaped single quote; such a string does *NOT* go through
    /// variable subsitution.
    ///
    /// A '$' introduces a variable substituion
    fn parse_str(&mut self, cmd_args: &C, l: &str) -> Result<Vec<String>, ExecError<C>> {
        let mut parsed = vec![];
        let mut token: Option<String> = None;
        let mut delimiter: Option<char> = None;
        let mut escape = false;
        for c in l.chars() {
            if token.is_none() {
                // Nothing in hand
                if c.is_whitespace() {
                    continue;
                } else if c == '#' {
                    break;
                } else if c == '"' || c == '\'' {
                    delimiter = Some(c);
                    token = Some(String::new());
                } else {
                    token = Some(String::new());
                    token.as_mut().unwrap().push(c);
                }
            } else if escape {
                // escape a character (e.g. inside quotes)
                token.as_mut().unwrap().push(c);
                escape = false;
            } else if let Some(dc) = delimiter {
                // Closing delimiter
                if c == dc {
                    if dc == '"' {
                        parsed.push(self.substitute(cmd_args, token.take().unwrap(), true)?);
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
                parsed.push(self.substitute(cmd_args, token.take().unwrap(), false)?);
            } else {
                token.as_mut().unwrap().push(c);
            }
        }
        // Should check delimiter is none, escape is false
        if let Some(token) = token {
            if let Some(delimiter) = delimiter {
                return Err(format!("unclosed delimiter {delimiter} in line").into());
            } else {
                parsed.push(self.substitute(cmd_args, token, false)?);
            }
        }
        Ok(parsed)
    }

    //mi execute_str_line
    /// Execute commands from a single-line[str]
    pub fn execute_str_line(&mut self, cmd_args: &mut C, l: &str) -> Result<(), ExecError<C>> {
        let l = l.trim();
        let s = self.parse_str(cmd_args, l)?;
        if !s.is_empty() {
            if s[0].as_bytes()[0] == b'#' {
                return Ok(());
            }
            self.execute(cmd_args, s, true)?;
        }
        Ok(())
    }

    //mi executed_result
    fn executed_result(&mut self, result: C::Value) {
        if !result.is_none() {
            if !self.result_history.is_empty() {
                self.result_history.pop();
            }
            self.result_history.push(Rc::new(result));
        }
    }

    //mi execute_given_matches
    fn execute_given_matches(
        &mut self,
        cmd_args: &mut C,
        matches: &ArgMatches,
    ) -> Result<(), ExecError<C>> {
        self.handler_set.handle_args(self, cmd_args, matches)?;

        if self.use_builtins {
            if let Some(result) = self.handle_builtins(cmd_args, matches)? {
                self.executed_result(result);
                return Ok(());
            }
        }
        if self.use_batch && matches.contains_id("batch") {
            self.show_result = false;
            let batches: Vec<_> = matches
                .get_many::<String>("batch")
                .unwrap()
                .map(|filename| {
                    (
                        filename.clone(),
                        std::fs::read_to_string(filename)
                            .map_err(|e| format!("failed to load batch file {filename}: {e}")),
                    )
                })
                .collect();
            for b in &batches {
                if let Err(err) = &b.1 {
                    return Err(err.clone().into());
                }
            }
            for (filename, s) in batches {
                self.execute_batch(cmd_args, filename, &s.unwrap())?;
            }
        }
        let result = self
            .handler_set
            .handle_cmd(self, cmd_args, matches)
            .map_err(|e| {
                format!(
                    "{}:{} {e}",
                    self.cmd_stack.last().unwrap().0,
                    self.cmd_stack.last().unwrap().1.unwrap_or_default(),
                )
            })?;
        self.executed_result(result);
        Ok(())
    }

    //mi execute
    /// Execute at the top level, given an iterator that provides the arguments
    ///
    /// It is deemed to be executed from 'cmd_stack.last()';
    pub fn execute<I, T>(
        &mut self,
        cmd_args: &mut C,
        itr: I,
        in_batch: bool,
    ) -> Result<bool, ExecError<C>>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let mut be_interactive = false;
        cmd_args.reset_args();
        let mut cmd = {
            if in_batch {
                self.batch_command.clone()
            } else {
                self.command.clone()
            }
        };

        if let Some((name, opt_line)) = self.cmd_stack.last() {
            if let Some(line) = opt_line {
                cmd = cmd.bin_name(format!("{name} line {line}"));
            } else {
                cmd = cmd.bin_name(name);
            }
        }
        match cmd.try_get_matches_from_mut(itr) {
            Err(e) => {
                // eprintln!("thunderclap:handler:cmd matches err: {} {}", in_batch, e);
                if !in_batch {
                    e.exit();
                }
                return Err(e.to_string().into());
            }
            Ok(matches) => {
                // eprintln!(
                // "thunderclap:handler:cmd matches ok: {} {}",
                // self.use_builtins, in_batch
                // );
                let ignore_errors = {
                    if self.use_builtins && in_batch {
                        matches.get_one::<bool>("ignore_errors") == Some(&true)
                    } else {
                        false
                    }
                };
                if !in_batch
                    && self.allow_interactive
                    && matches.get_one::<bool>("interactive") == Some(&true)
                {
                    be_interactive = true;
                }
                if let Err(e) = self.execute_given_matches(cmd_args, &matches) {
                    if !ignore_errors {
                        return Err(e);
                    }
                }
            }
        }
        Ok(be_interactive)
    }

    //mi exec_as_cmd_response
    fn exec_as_cmd_response<T>(&self, r: Result<T, ExecError<C>>) -> CmdResponse {
        match r {
            Ok(_) => {
                if self.result_history.is_empty() {
                    CmdResponse::ExecOk("".to_string())
                } else {
                    CmdResponse::ExecOk(self.result_history.first().unwrap().value_string())
                }
            }
            Err(e) => CmdResponse::ExecError(e.to_string()),
        }
    }

    //mp set_env
    /// Set the environment from the OS environment
    ///
    /// This is public as it is used by thunderclap_httpd
    pub fn set_env(&mut self) -> Result<(), ExecError<C>> {
        let mut iter = std::env::args_os();
        let cmd_name = iter.next().unwrap();
        self.cmd_stack
            .push((cmd_name.to_str().unwrap().into(), None));
        self.variables.clear();

        for (k, v) in std::env::vars() {
            // Ignore errors importing the environment
            //
            // If the value is integers, importing a Path will produce an error, but...
            let _ = self.set_variable_value_str(&k, &v);
        }
        Ok(())
    }

    //mp execute_request
    pub fn execute_request(&mut self, cmd_args: &mut C, cmd_request: CmdRequest) -> CmdResponse {
        match cmd_request {
            CmdRequest::Prompt => {
                CmdResponse::Prompt(format!("{} > ", self.cmd_stack.last().unwrap().0))
            }
            CmdRequest::Exec(line) => {
                let result = self.execute_str_line(cmd_args, &line);
                self.exec_as_cmd_response(result)
            }
            CmdRequest::ExecVec(strings) => {
                let result = self.execute(cmd_args, strings.iter(), true);
                self.exec_as_cmd_response(result)
            }
            CmdRequest::ExecVecSubst(strings) => {
                let mut exec_strings = vec![];
                for s in strings.into_iter() {
                    match self.substitute(cmd_args, s, false) {
                        Ok(s) => {
                            exec_strings.push(s);
                        }
                        Err(e) => {
                            return CmdResponse::ExecError(e.to_string());
                        }
                    }
                }
                let result = self.execute(cmd_args, exec_strings.iter(), true);
                self.exec_as_cmd_response(result)
            }
            _ => CmdResponse::Finish,
        }
    }

    //mp execute_batch
    /// Execute a batch of commands from a [str], one per line
    pub fn execute_batch<I: Into<String>>(
        &mut self,
        cmd_args: &mut C,
        filename: I,
        s: &str,
    ) -> Result<(), ExecError<C>> {
        self.cmd_stack.push((filename.into(), Some(0)));
        for l in s.lines() {
            *self.cmd_stack.last_mut().unwrap().1.as_mut().unwrap() += 1;
            self.execute_str_line(cmd_args, l)?;
        }
        self.cmd_stack.pop();
        Ok(())
    }

    //mp execute_env
    /// Execute using the environment, running batch files and interactively if so requested
    ///
    /// Return the result of the last operation
    pub fn execute_env(&mut self, cmd_args: &mut C) -> Result<Rc<C::Value>, ExecError<C>> {
        self.set_env()?;
        let iter = std::env::args_os().skip(1);

        let interactive = match self.execute(cmd_args, iter, false) {
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(4);
            }
            Ok(interactive) => interactive,
        };

        // Move 'self' into 's' so that we can throw it to the shell
        // *and* use it afterwards by taking it back from the shell
        let mut s = self;
        if interactive {
            let (new_s, _cmd_args) = shell((s, cmd_args), |(s, cmd_args), request| {
                s.execute_request(cmd_args, request)
            });
            s = new_s;
        }
        Ok(s.result_history.first().cloned().unwrap_or_default())
    }

    //mp execute_env_noninteractive
    /// Execute using the environment, running batch files if so requested
    ///
    /// Returns any error, Ok(true) if interactive is required
    pub fn execute_env_noninteractive(&mut self, cmd_args: &mut C) -> Result<bool, ExecError<C>> {
        self.set_env()?;
        let iter = std::env::args_os().skip(1);
        self.execute(cmd_args, iter, false)
    }
}
