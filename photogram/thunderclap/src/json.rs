//!
//! JsonValue implements
//!
//! JsonValue::to_string()
//!
//! pub fn from_value<T>(value: Value) -> Result<T, Error>
//!    where T: DeserializeOwned,
//!
//! pub fn to_value<T>(value: T) -> Result<Value, Error>
//! where T: Serialize,

use crate::CommandArgsValue;

pub use serde_json::from_value;
pub use serde_json::to_value;
pub use serde_json::Value;

//
//     fn from_str(s: &str) -> Result<Self, Self::FromStrError> {
//        eprintln!("Json from str '{s}'");
//        if let Ok(v) = serde_json::from_str::<Value>(s) {
//            eprintln!("Value {v:?}");
//            return Ok(v);
//        }
//        let v = serde_json::to_value(s)?;
//        eprintln!("Value {v:?}");
//        Ok(v)
//    }

pub trait JsonValueConvert: Sized {
    type Error: Sized;
    fn value_from_str(s: &str) -> Result<Self, Self::Error>;
}
impl JsonValueConvert for Value {
    type Error = serde_json::Error;
    fn value_from_str(s: &str) -> Result<Self, Self::Error> {
        if let Ok(v) = serde_json::from_str::<Value>(s) {
            return Ok(v);
        }
        let v = serde_json::to_value(s)?;
        Ok(v)
    }
}

//ip CommandArgsValue for Value
impl CommandArgsValue for Value {
    const CAN_INDEX: bool = true;
    const CAN_GET: bool = true;
    fn is_none(&self) -> bool {
        self == &serde_json::Value::Null
    }
    fn value_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    fn is_array(&self) -> bool {
        mod x {
            pub(super) fn is_array(v: &super::Value) -> bool {
                v.is_array()
            }
        }
        x::is_array(self)
    }
    fn is_map(&self) -> bool {
        mod x {
            pub(super) fn is_map(v: &super::Value) -> bool {
                v.is_map()
            }
        }
        x::is_map(self)
    }
    fn is_empty(&self) -> bool {
        if let Some(array) = self.as_array() {
            array.is_empty()
        } else if let Some(obj) = self.as_object() {
            obj.is_empty()
        } else {
            true
        }
    }
    fn len(&self) -> Option<usize> {
        if let Some(array) = self.as_array() {
            Some(array.len())
        } else {
            self.as_object().map(|obj| obj.len())
        }
    }
    fn index(&self, n: usize) -> Option<Self> {
        if !self.is_array() {
            None
        } else {
            self.get(n).cloned()
        }
    }
    fn key(&self, n: usize) -> Option<&str> {
        if let Some(obj) = self.as_object() {
            obj.keys().nth(n).map(|x| x.as_str())
        } else {
            None
        }
    }
    fn get(&self, _s: &str) -> Option<Self> {
        None
    }
}
