//a Imports
use clap::ArgAction;

//a ArgCount
//tp ArgCount
#[derive(Debug, Default, Clone, Copy)]
pub enum ArgCount {
    #[default]
    Optional, // false; 0 or 1, not required, Set, num_args None
    Required,          // true; 1, required, Set, num_args None
    Exactly(usize),    // usize n; n >= 1, required, Append, num args Some(n)
    Any,               // None; 0 to inf; not required, Append, num_args None
    Max(usize),        // Some(usize); 0 to max, not required, Append, num_args(0..=n)
    Min(usize),        // (n>1,); n to inf, required, Append, num_args(n..)
    Positional(usize), // (n,true);  0 => not required, 1+ required, Append, num_args Some(n) *positional*
}

//ip From<usize> for ArgCount {
impl From<usize> for ArgCount {
    #[track_caller]
    fn from(n: usize) -> ArgCount {
        assert!(n != 0, "Cannot require exactly 0 occurrences");
        ArgCount::Exactly(n)
    }
}

//ip From<Option<usize>> for ArgCount {
impl From<Option<usize>> for ArgCount {
    #[track_caller]
    fn from(opt_n: Option<usize>) -> ArgCount {
        assert!(opt_n != Some(0), "Cannot require at most 0 occurrences");
        match opt_n {
            Some(n) => ArgCount::Max(n),
            _ => ArgCount::Any,
        }
    }
}

//ip From<(usize,)> for ArgCount {
impl From<(usize,)> for ArgCount {
    #[track_caller]
    fn from((min,): (usize,)) -> ArgCount {
        match min {
            0 => ArgCount::Any,
            n => ArgCount::Min(n),
        }
    }
}

//ip From<bool> for ArgCount {
impl From<bool> for ArgCount {
    fn from(required: bool) -> ArgCount {
        if required {
            ArgCount::Required
        } else {
            ArgCount::Optional
        }
    }
}

//ip From<(usize, bool)> for ArgCount {
impl From<(usize, bool)> for ArgCount {
    fn from((n, positional): (usize, bool)) -> ArgCount {
        if positional {
            ArgCount::Positional(n)
        } else if n > 0 {
            ArgCount::Exactly(n)
        } else {
            ArgCount::Any
        }
    }
}

//ip ArgCount {
impl ArgCount {
    pub(crate) fn uses_tag(&self) -> bool {
        use ArgCount::*;
        !matches!(self, Positional(_))
    }
    pub(crate) fn required(&self) -> bool {
        use ArgCount::*;
        !matches!(self, Optional | Any | Max(_) | Positional(0))
    }
    pub(crate) fn action(&self) -> ArgAction {
        use ArgCount::*;
        match self {
            Optional => ArgAction::Set,
            Required => ArgAction::Set,
            Positional(0) => ArgAction::Set,
            Positional(1) => ArgAction::Set,
            _ => ArgAction::Append,
        }
    }
    pub(crate) fn num_args(&self) -> Option<clap::builder::ValueRange> {
        use ArgCount::*;
        match self {
            Exactly(n) => Some((*n).into()),
            Min(n) => Some((*n..).into()),
            Max(n) => Some((0..=*n).into()),
            Positional(0) => None,
            Positional(1) => None,
            Positional(n) => Some((*n).into()),
            _ => None,
        }
    }
}
