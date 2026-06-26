//a Modules
mod arg_count;
mod arg_descriptor;
mod builder;
mod cmd_descriptor;
mod command_set;
mod handler;
pub mod interactive;
mod property;
mod traits;

use thiserror::Error;

pub mod json;
pub use arg_count::ArgCount;
pub use arg_descriptor::ArgDescriptor;
pub use builder::CommandBuilder;
pub use cmd_descriptor::CmdDescriptor;
pub use property::CmdProperty;
pub use traits::{CommandArgs, CommandArgsValue};

pub use command_set::CommandSet;
pub(crate) use handler::CommandHandlerSet;
pub(crate) use traits::{ArgFn, ArgResetFn, CommandFn};

pub use clap;
pub use clap::{Arg, Command};

pub fn bound<F, V>(v: V, min: Option<V>, max: Option<V>, f: F) -> Result<V, String>
where
    V: PartialOrd,
    F: FnOnce(V, bool) -> String,
{
    if let Some(min) = min {
        if v < min {
            return Err(f(v, false));
        }
    }
    if let Some(max) = max {
        if v > max {
            return Err(f(v, true));
        }
    }
    Ok(v)
}

#[derive(Error)]
pub enum ExecError<C>
where
    C: CommandArgs,
{
    #[error("failed to evaluate string: {0}")]
    Eval(C::Error),
    #[error("failed to set argument to command: {0}")]
    SetArg(C::Error),
    #[error("failed to output: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to execute command: {0}")]
    Exec(C::Error),
    #[error("failed to execute builtin command: {0}")]
    Builtin(String),
    #[error("some message: {0}")]
    Msg(String),
}

//ip Debug for ExecError<C>
impl<C> std::fmt::Debug for ExecError<C>
where
    C: CommandArgs,
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        <Self as std::fmt::Display>::fmt(self, fmt)
    }
}

impl<C> From<String> for ExecError<C>
where
    C: CommandArgs,
{
    fn from(s: std::string::String) -> Self {
        ExecError::Msg(s)
    }
}

impl<C> ExecError<C>
where
    C: CommandArgs,
{
    fn eval(e: C::Error) -> Self {
        Self::Eval(e)
    }
    fn exec(e: C::Error) -> Self {
        Self::Exec(e)
    }
    fn set_arg(e: C::Error) -> Self {
        Self::SetArg(e)
    }
    fn cmd_ok() -> Result<C::Value, Self> {
        C::cmd_ok().map_err(ExecError::exec)
    }
}
