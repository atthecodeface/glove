use crate::CommandArgs;

pub struct CmdProperty<'a, C, V, E> {
    pub name: &'a str,
    /// Retrieve the value of a key, in some form, from the arguments - used in batch and interactive only
    ///
    /// Return None if the value is not retrievable
    pub get_fn: &'a dyn Fn(&C) -> Option<V>,

    /// Set the value to a known value
    ///
    /// Return Ok(false) if the key is not provided by the args
    ///
    /// Return Ok(true) if the key value was set correctly
    ///
    /// Return Err() if the key was known but could not be set
    pub set_value_fn: &'a dyn Fn(&mut C, &V) -> Result<bool, E>,
}

impl<'a, C, V, E> CmdProperty<'a, C, V, E> {
    pub fn find_property<'f>(properties: &'f [Self], k: &str) -> Option<&'f Self> {
        for p in properties {
            if p.name == k {
                return Some(p);
            }
        }
        None
    }
}
