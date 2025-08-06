//a Imports
use clap::ArgMatches;

//a CommandArgs
//tt CommandArgs
/// Trait that describes to the library the types used for argument and command functions
///
/// This should be implemented by a type that is used to hold and
/// build the arguments for the execution of commands
pub trait CommandArgs {
    /// Error type returned as an error by all [ArgFn] and [CommandFn]
    type Error: std::convert::From<String> + std::fmt::Display;

    /// Value type returned by commands
    ///
    /// This must provide `ToString` for use in batch mode and
    /// interactive operation, where the results of commands can be
    /// stored for future command invocations
    type Value: std::default::Default + std::string::ToString;

    /// Function invoked before every batch or interactive command to reset temporary options
    fn reset_args(&mut self) {}
}

//a ArgFn
//tt ArgFn
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
pub trait ArgFn<C: CommandArgs>: Fn(&mut C, &ArgMatches) -> Result<(), C::Error> + 'static {}

//ip ArgFn for Fn(CommandArgs, ArgMatches)
impl<C: CommandArgs, T: Fn(&mut C, &ArgMatches) -> Result<(), C::Error> + 'static> ArgFn<C> for T {}

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
