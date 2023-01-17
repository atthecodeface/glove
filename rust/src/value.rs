use crate::Function;

pub enum Value<'f> {
    Argument(String),
    Constant(f64),
    Function(Box<dyn Function + 'f>),
}

impl<'f> Value<'f> {
    pub fn new_arg<S: Into<String>>(name: S) -> Self {
        let name = name.into();
        Self::Argument(name)
    }
    pub fn new_fn(f: Box<dyn Function + 'f>) -> Self {
        Self::Function(f)
    }
    pub fn one() -> Self {
        Self::Constant(1.0)
    }
    pub fn constant(f: f64) -> Self {
        Self::Constant(f)
    }
}

impl<'f> std::fmt::Display for Value<'f> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Argument(name) => name.fmt(fmt),
            Self::Constant(x) => x.fmt(fmt),
            Self::Function(f) => f.fmt(fmt),
        }
    }
}

impl<'f> PartialEq<Value<'f>> for Value<'f> {
    fn eq(&self, other: &Value<'f>) -> bool {
        match self {
            Self::Argument(name) => {
                if let Self::Argument(other_name) = other {
                    name == other_name
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

impl<'f> Function for Value<'f> {
    fn evaluate<'e>(&'e self, arg_to_value: &'e dyn Fn(&'e Value) -> f64) -> f64 {
        match self {
            Self::Constant(x) => *x,
            Self::Function(f) => f.evaluate(arg_to_value),
            Self::Argument(_) => arg_to_value(self),
        }
    }

    fn clone<'arg>(&'arg self) -> Box<dyn Function + 'arg> {
        use Value::*;
        Box::new(match self {
            Argument(name) => Value::new_arg(name),
            Constant(f) => Value::constant(*f),
            Function(f) => Value::Function((*f).clone()),
        })
    }

    fn as_constant(&self) -> Option<f64> {
        match self {
            Self::Constant(x) => Some(*x),
            Self::Function(f) => f.as_constant(),
            _ => None,
        }
    }

    fn has_arg(&self, arg: &Value) -> bool {
        match self {
            Self::Argument(_) => self == arg,
            _ => false,
        }
    }
    fn differentiate<'arg>(&'arg self, arg: &'arg Value) -> Option<Box<dyn Function>> {
        if self.has_arg(arg) {
            Some(Box::new(Value::one()))
        } else {
            None
        }
    }
    fn simplify<'arg>(&'arg self) -> Option<Box<dyn Function + 'arg>> {
        if let Self::Function(f) = self {
            if let Some(c) = f.as_constant() {
                Some(Box::new(Value::constant(c)))
            } else {
                None
            }
        } else {
            None
        }
    }
}
