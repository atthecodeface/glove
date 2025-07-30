//a Modules
mod builder;
mod handler;
mod traits;

pub use builder::CommandBuilder;
pub use traits::CommandArgs;

pub(crate) use handler::{CommandHandlerSet, CommandSet};
pub(crate) use traits::{ArgFn, CommandFn};

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
