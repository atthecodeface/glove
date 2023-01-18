//a Imports
use crate::Function;

//a Value
//tp Value
pub enum Value<'arg> {
    Argument(String),
    Constant(f64),
    Function(Box<dyn Function<'arg> + 'arg>),
}

//ip Value
impl<'arg> Value<'arg> {
    pub fn new_arg<S: Into<String>>(name: S) -> Self {
        let name = name.into();
        Self::Argument(name)
    }
    pub fn new_fn(f: Box<dyn Function<'arg> + 'arg>) -> Self {
        Self::Function(f)
    }
    pub fn one() -> Self {
        Self::Constant(1.0)
    }
    pub fn constant(f: f64) -> Self {
        Self::Constant(f)
    }
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Argument(n) => Some(n),
            _ => None,
        }
    }
}

//ip Display for Value
impl<'f> std::fmt::Display for Value<'f> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Argument(name) => name.fmt(fmt),
            Self::Constant(x) => x.fmt(fmt),
            Self::Function(f) => f.fmt(fmt),
        }
    }
}

//ip PartialEq for Value
impl<'f, 'other> PartialEq<Value<'other>> for Value<'f> {
    fn eq(&self, other: &Value<'other>) -> bool {
        self.name().is_some() && self.name() == other.name()
    }
}

//ip Function for Value
impl<'arg> Function<'arg> for Value<'arg> {
    //fp evaluate
    fn evaluate(&self, arg_to_value: &dyn Fn(&Value) -> f64) -> f64 {
        match self {
            Self::Constant(x) => *x,
            Self::Function(f) => f.evaluate(arg_to_value),
            Self::Argument(_) => arg_to_value(self),
        }
    }

    //fp clone
    fn clone(&self) -> Box<dyn Function<'arg> + 'arg> {
        use Value::*;
        Box::new(match self {
            Argument(name) => Value::new_arg(name),
            Constant(f) => Value::constant(*f),
            Function(f) => Value::Function((*f).clone()),
        })
    }

    //fp as_constant
    fn as_constant(&self) -> Option<f64> {
        match self {
            Self::Constant(x) => Some(*x),
            Self::Function(f) => f.as_constant(),
            _ => None,
        }
    }

    //fp has_arg
    fn has_arg(&self, arg: &Value) -> bool {
        match self {
            Self::Argument(_) => self == arg,
            _ => false,
        }
    }

    //fp differentiate
    fn differentiate(&self, arg: &Value) -> Option<Box<dyn Function<'arg>>> {
        if self.has_arg(arg) {
            Some(Box::new(Value::one()))
        } else {
            None
        }
    }

    //fp simplified
    fn simplified(self: Box<Self>) -> Box<dyn Function<'arg> + 'arg> {
        match *self {
            Self::Function(f) => {
                if let Some(c) = f.as_constant() {
                    Box::new(Value::constant(c))
                } else {
                    Box::new(Self::Function(f))
                }
            }
            s => Box::new(s),
        }
    }
}
