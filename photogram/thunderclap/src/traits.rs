//a Imports
use clap::ArgMatches;

use crate::{CommandSet, ExecError};

//a CommandArgs
//tt CommandArgsValue
/// This must provide `ToString` for use in batch mode and
/// interactive operation, where the results of commands can be
/// stored for future command invocations
pub trait CommandArgsValue: std::default::Default {
    const CAN_INDEX: bool;
    const CAN_GET: bool;
    fn value_string(&self) -> String;
    /// Return true if the value is effectively 'NULL', so should not be pusehd to the result stack
    fn is_none(&self) -> bool;
    fn is_empty(&self) -> bool {
        true
    }
    fn len(&self) -> Option<usize> {
        None
    }
    fn index(&self, _n: usize) -> Option<Self> {
        None
    }
    fn get(&self, _s: &str) -> Option<Self> {
        None
    }
    fn key(&self, _n: usize) -> Option<&str> {
        None
    }
    fn is_array(&self) -> bool {
        false
    }
    fn is_map(&self) -> bool {
        false
    }
}

//ip CommandArgsValue for ()
impl CommandArgsValue for () {
    const CAN_INDEX: bool = false;
    const CAN_GET: bool = false;
    fn is_none(&self) -> bool {
        true
    }

    fn value_string(&self) -> String {
        "".into()
    }
}

//mi command_args_value
macro_rules! command_args_value {
    {$t:ty} => {
        impl $crate :: CommandArgsValue for $t {
            const CAN_INDEX: bool = false;
            const CAN_GET: bool = false;
            fn is_none(&self) -> bool { false }
            fn value_string(&self) -> String { std::string::ToString::to_string(self) }
        }
    };
}

//ip CommandArgsValue for String
command_args_value! {String}

//ip CommandArgsValue for u8 to usize
command_args_value! {usize}
command_args_value! {u64}
command_args_value! {u32}
command_args_value! {u16}
command_args_value! {u8}

//ip CommandArgsValue for i8 to isize
command_args_value! {isize}
command_args_value! {i64}
command_args_value! {i32}
command_args_value! {i16}
command_args_value! {i8}

//ip CommandArgsValue for f32, f64
command_args_value! {f32}
command_args_value! {f64}

//ip CommandArgsValue for bool
command_args_value! {bool}

//tt CommandArgs
/// Trait that describes to the library the types used for argument and command functions
///
/// This should be implemented by a type that is used to hold and
/// build the arguments for the execution of commands
pub trait CommandArgs: 'static {
    /// Error type returned as an error by all [ArgFn] and [CommandFn]
    type Error: std::error::Error;
    // type Error: std::convert::From<String> + std::error::Error;

    /// Value type returned by commands
    type Value: CommandArgsValue;

    fn value_from_str(s: &str) -> Result<Self::Value, Self::Error>;

    fn cmd_ok() -> Result<Self::Value, Self::Error> {
        Ok(Self::Value::default())
    }

    /// Function invoked before every batch or interactive command to reset temporary options
    fn reset_args(&mut self) {}

    /// Get the keys (elements) of the arguments - used in batch and interactive only
    fn keys(&self) -> Box<dyn Iterator<Item = &str>> {
        const KEYS: [String; 0] = [];
        Box::new(KEYS.iter().map(|s| s.as_str()))
    }

    /// Retrieve the value of a key, in some form, from the arguments - used in batch and interactive only
    ///
    /// Return None if the key is not provided by the args
    fn value_str(&self, _key: &str) -> Option<Self::Value> {
        None
    }

    /// Set the value to a value from a string
    ///
    /// Return Ok(false) if the key is not provided by the args
    ///
    /// Return Ok(true) if the key value was set correctly
    ///
    /// Return Err() if the key was known but could not be set
    fn value_set(&mut self, _key: &str, _value: &Self::Value) -> Result<bool, Self::Error> {
        Ok(false)
    }
}

//a ArgResetFn, ArgFn
//tt ArgResetFn
/// Trait of functions submitted to reset [CommandArgs] prior to a (sub)command
///
/// This is invoked for a subcommand prior to setting its matches
///
/// It can be used to reset once-only arguments; on the command line
/// this will generally have no effect, as the arguments are yet to be
/// set by ArgFn invocations; in batch mode or interactive mode this
/// may reset values that are used once only (on previous invocations)
///
/// This function need not be provided if the [CommandArgs] are
/// autoreset at the *end* of a command by the application.
pub trait ArgResetFn<C: CommandArgs>: Fn(&mut C) + 'static {}

//ip ArgResetFn for Fn(CommandArgs)
impl<C: CommandArgs, T: Fn(&mut C) + 'static> ArgResetFn<C> for T {}

//tt ArgFn
/// Trait of functions submitted to update [CommandArgs] with a value from the [ArgMatches]
///
/// This is invoked for a specific argument when it is provided in the
/// [ArgMatches]; the function should parse the value(s) and update
/// the [CommandArgs] appropriately.
///
/// All argument functions are invoked in the order in which they are
/// provided to the command builder; so if one argument is required
/// and creates the main data structure for an application, and other
/// arguments modify that, then the main data structure argument
/// should be supplied first, and its [ArgFn] will be invoked first,
/// permitting later argument functions to just modify the main data
/// structure.
pub trait ArgFn<C: CommandArgs>:
    Fn(&CommandSet<C>, &mut C, &ArgMatches) -> Result<(), ExecError<C>> + 'static
{
}

//ip ArgFn for Fn(CommandArgs, ArgMatches)
impl<
        C: CommandArgs,
        T: Fn(&CommandSet<C>, &mut C, &ArgMatches) -> Result<(), ExecError<C>> + 'static,
    > ArgFn<C> for T
{
}

//a CommandFn
//tt CommandFn
/// Trait of functions submitted to be executed as a command or subcommand
///
/// The function is invoked after all the arguments for the command
/// have been added; if the command itself has subcommands, and a
/// subcommand is specified, then the subcommand function is invoked
/// and not the command function
///
/// The arguments for the function should all be defined in the
/// [CommandArgs] structure, which can be modified; if batch or
/// interactive operation is used then the updated [CommandArgs] is
/// seen by later commands
///
/// The return value of the command is available in batch and
/// interactive operation for later commands
pub trait CommandFn<C: CommandArgs>: Fn(&mut C) -> Result<C::Value, C::Error> + 'static {}

//ip ArgFn for Fn(CommandArgs, ArgMatches)
impl<C: CommandArgs, T: Fn(&mut C) -> Result<C::Value, C::Error> + 'static> CommandFn<C> for T {}
