//a Imports
use crate::{Arg, Function, Node};

//a Value
//tp Value
pub enum Value<A: Arg> {
    Argument(A),
    Constant(f64),
    Function(Node<A>),
}

//ip Value
impl<A: Arg> Value<A> {
    pub fn new_arg(arg: &A) -> Self {
        Self::Argument(arg.clone())
    }
    pub fn new_fn(f: Node<A>) -> Self {
        Self::Function(f)
    }
    pub fn one() -> Self {
        Self::Constant(1.0)
    }
    pub fn constant(f: f64) -> Self {
        Self::Constant(f)
    }
    pub fn arg(&self) -> Option<&A> {
        match self {
            Self::Argument(n) => Some(n),
            _ => None,
        }
    }
}

//ip Display for Value
impl<A: Arg> std::fmt::Display for Value<A> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::Argument(name) => name.fmt(fmt),
            Self::Constant(x) => x.fmt(fmt),
            Self::Function(f) => f.fmt(fmt),
        }
    }
}

//ip PartialEq for Value
impl<A: Arg> PartialEq<Value<A>> for Value<A> {
    fn eq(&self, other: &Value<A>) -> bool {
        self.arg().is_some() && self.arg() == other.arg()
    }
}

//ip Function for Value
impl<A: Arg> Function<A> for Value<A> {
    //fp clone
    fn clone(&self) -> Node<A> {
        use Value::*;
        Node::new(match self {
            Argument(name) => Value::new_arg(name),
            Constant(f) => Value::constant(*f),
            Function(f) => Value::Function((*f).clone_node()),
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
    fn has_arg(&self, arg: &A) -> bool {
        match self {
            Self::Argument(a) => a == arg,
            _ => false,
        }
    }

    //fp differentiate
    fn differentiate(&self, arg: &A) -> Option<Node<A>> {
        if self.has_arg(arg) {
            Some(Node::new(Value::one()))
        } else {
            None
        }
    }

    //fp evaluate
    fn evaluate(&self, arg_to_value: &dyn Fn(&A) -> f64) -> f64 {
        match self {
            Self::Constant(x) => *x,
            Self::Function(f) => f.evaluate(arg_to_value),
            Self::Argument(a) => arg_to_value(a),
        }
    }

    //fp simplified
    fn simplified(self: Box<Self>) -> Node<A> {
        match *self {
            Self::Function(f) => {
                if let Some(c) = f.as_constant() {
                    Node::new(Value::constant(c))
                } else {
                    Node::new(Self::Function(f))
                }
            }
            s => Node::new(s),
        }
    }
}
