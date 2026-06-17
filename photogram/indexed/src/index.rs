/// The trait for an index
///
/// This *explicitly* does not support `From<usize>`; to create an
/// index type from a usize, use the `from_usize()` method. Supporting
/// From, and hence Into, would weaken the protection provided by the
/// index type, so it *must* be opt-in by the type.
pub trait Idx:
    Copy + std::fmt::Debug + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash + 'static
{
    const NONE: Option<Self>;

    /// Generate an index from a usize value
    fn from_usize(idx: usize) -> Self;

    /// Retrieve the index as a usize
    #[track_caller]
    fn index(self) -> usize {
        assert!(
            Self::NONE.is_none(),
            "Bug: Attempt to retrieve index from an *optional* Idx value; use opt_index instead"
        );
        self.opt_index().unwrap()
    }

    /// If the type supports 'Option' for the index, return true if it is None, else false
    #[track_caller]
    fn is_none(self) -> bool {
        assert!(
            Self::NONE.is_some(),
            "Cannot use 'is_none()' on an index that does not support 'None'"
        );
        self == Self::NONE.unwrap()
    }

    /// Return the index as an option - if the index type does not support Option, then always returns Some(index)
    fn opt_index(self) -> Option<usize>;

    /// Return 'None', for an index type that supports None
    #[track_caller]
    fn none() -> Self {
        Self::NONE.expect("Cannot retrieve 'none' from an index type that does not support it")
    }

    /// Get the *next* index value
    ///
    /// If invoked on 'None' then return None
    fn next(self) -> Self {
        if let Some(index) = self.opt_index() {
            Self::from_usize(index + 1)
        } else {
            self
        }
    }
}

impl Idx for usize {
    const NONE: Option<usize> = None;
    fn from_usize(n: usize) -> usize {
        n
    }
    fn opt_index(self) -> Option<usize> {
        Some(self)
    }
}

/// Make an index type given an underlying unsigned integer, e.g. make_index!{Index, u32, false}
///
/// The $opt is true then the MAX of the underlying signed integer is used to
/// indicate None and the index supports (effectively) and option; if $opt is
/// false then the index is always Some
#[macro_export]
macro_rules! make_index {
    {$(#[$outer:meta])* $id: ident, $t:ty, $opt:expr} => {

        $(#[$outer])*

        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
        pub struct $id($t);

        impl std::default::Default for $id {
            fn default() -> Self {
                if $opt { Self (< $t > :: MAX) } else {Self(0_usize as $t)}
            }
        }
        impl$crate :: Idx for $id {
            const NONE: Option<Self> = {if $opt {Some(Self ( < $t > :: MAX ))}  else {None}};
            fn from_usize(n: usize) -> Self { Self(n as $t)}
            fn opt_index(self) -> Option<usize> {
                if $opt && self.0 == < $t > :: MAX {None} else {Some(self.0 as usize)}
            }
        }

    }
}
