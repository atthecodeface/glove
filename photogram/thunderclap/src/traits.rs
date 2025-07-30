use clap::ArgMatches;
pub trait CommandArgs {
    type Error: std::convert::From<String> + std::fmt::Display;
    type Value: std::default::Default;
}

pub trait CommandFn<C: CommandArgs>: Fn(&mut C) -> Result<C::Value, C::Error> + 'static {}

pub trait ArgFn<C: CommandArgs>: Fn(&mut C, &ArgMatches) -> Result<(), C::Error> + 'static {}

impl<C: CommandArgs, T: Fn(&mut C) -> Result<C::Value, C::Error> + 'static> CommandFn<C> for T {}
impl<C: CommandArgs, T: Fn(&mut C, &ArgMatches) -> Result<(), C::Error> + 'static> ArgFn<C> for T {}
