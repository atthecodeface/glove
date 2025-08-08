//a Imports
use clap::ArgAction;

//a ArgCount
//tp ArgCount
#[derive(Debug, Default, Clone, Copy)]
pub enum ArgCount {
    #[default]
    Optional, // false; 0 or 1, not required, Set, num_args None
    Required,                  // true; 1, required, Set, num_args None
    Exactly(usize),            // usize n; n >= 1, required, Append, num args Some(n)
    Any,                       // None; 0 to inf; not required, Append, num_args None
    Max(usize),                // Some(usize); 0 to max, not required, Append, num_args(0..=n)
    Min(usize),                // (n>1,); n to inf, required, Append, num_args(n..)
    PositionalOptional,        // (Some(0),true);  optional, Set, num_args Some(n) *positional*
    PositionalRequired(usize), // (Some(n),true);  required, Set, num_args Some(n) *positional*
    PositionalAny,             // (None,true);     optional, Append, num_args None *positional*
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

//ip From<(Option<usize>, bool)> for ArgCount {
impl From<(Option<usize>, bool)> for ArgCount {
    fn from((opt_n, positional): (Option<usize>, bool)) -> ArgCount {
        match (opt_n, positional) {
            (None, true) => ArgCount::PositionalAny,
            (Some(0), true) => ArgCount::PositionalOptional,
            (Some(n), true) => ArgCount::PositionalRequired(n),
            (Some(0), _) => ArgCount::Min(0),
            (Some(n), _) => ArgCount::Exactly(n),
            _ => ArgCount::Any,
        }
    }
}

//ip ArgCount {
impl ArgCount {
    pub(crate) fn uses_tag(&self) -> bool {
        use ArgCount::*;
        !matches!(
            self,
            PositionalOptional | PositionalRequired(_) | PositionalAny
        )
    }
    pub(crate) fn required(&self) -> bool {
        use ArgCount::*;
        !matches!(
            self,
            Optional | Any | Max(_) | PositionalOptional | PositionalAny
        )
    }
    pub(crate) fn action(&self) -> ArgAction {
        use ArgCount::*;
        match self {
            Optional => ArgAction::Set,
            Required => ArgAction::Set,
            PositionalOptional => ArgAction::Set,
            PositionalRequired(1) => ArgAction::Set,
            _ => ArgAction::Append,
        }
    }
    pub(crate) fn num_args(&self) -> Option<clap::builder::ValueRange> {
        use ArgCount::*;
        match self {
            Exactly(n) => Some((*n).into()),
            Min(n) => Some((*n..).into()),
            Max(n) => Some((0..=*n).into()),
            PositionalRequired(0) => None,
            PositionalRequired(1) => None,
            PositionalRequired(n) => Some((*n).into()),
            _ => None,
        }
    }
}
