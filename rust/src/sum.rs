//a Imports
use crate::{Arg, Function, Node, Value};

//a Sum
//tp Sum
pub struct Sum<A: Arg> {
    fns: Vec<Node<A>>,
}

//ip Default for Sum
impl<A: Arg> std::default::Default for Sum<A> {
    fn default() -> Self {
        Sum { fns: Vec::new() }
    }
}

//ip Sum
impl<A: Arg> Sum<A> {
    pub fn add_fn(&mut self, f: Node<A>) {
        if !f.is_zero() {
            self.fns.push(f);
        }
    }
}

//ip Display for Sum
impl<A: Arg> std::fmt::Display for Sum<A> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self.fns.len() {
            0 => write!(fmt, "0"),
            1 => self.fns[0].fmt(fmt),
            _ => {
                write!(fmt, "(")?;
                for (i, f) in self.fns.iter().enumerate() {
                    if i > 0 {
                        write!(fmt, "+")?;
                    }
                    f.fmt(fmt)?;
                }
                write!(fmt, ")")
            }
        }
    }
}

//ip Function for Sum
impl<A: Arg> Function<A> for Sum<A> {
    //fp clone
    fn clone(&self) -> Node<A> {
        let fns = self.fns.iter().map(|f| (*f).clone()).collect();
        Node::new(Sum { fns })
    }

    //fp as_constant
    fn as_constant(&self) -> Option<f64> {
        if self.fns.is_empty() {
            Some(0.0)
        } else {
            None
        }
    }

    //fp evaluate
    fn evaluate(&self, arg_to_value: &dyn Fn(&A) -> f64) -> f64 {
        if let Some(x) = self.as_constant() {
            x
        } else {
            let mut r = 0.;
            for f in self.fns.iter() {
                r += f.evaluate(arg_to_value);
            }
            r
        }
    }

    //fp has_arg
    fn has_arg(&self, arg: &A) -> bool {
        for f in self.fns.iter() {
            if f.has_arg(arg) {
                return true;
            }
        }
        return false;
    }

    //fp differentiate
    fn differentiate(&self, arg: &A) -> Option<Node<A>> {
        let mut sum = Sum { fns: vec![] };
        for f in self.fns.iter() {
            if let Some(df) = f.differentiate(arg) {
                sum.add_fn(df)
            }
        }
        if sum.fns.is_empty() {
            None
        } else {
            Some(Node::new(sum))
        }
    }

    //fp simplified
    fn simplified(self: Box<Self>) -> Node<A> {
        let mut fns = Vec::new();
        let mut constant = 0.;
        for f in self.fns.into_iter() {
            let f = f.simplified();
            if let Some(c) = f.as_constant() {
                constant += c;
            } else {
                let ps = f.as_sums();
                for p in ps.into_iter() {
                    fns.push(p);
                }
            }
        }
        if fns.is_empty() {
            Node::new(Value::constant(constant))
        } else if constant == 0. && fns.len() == 1 {
            let f = fns.pop().unwrap();
            f
        } else {
            let mut sum = Sum::default();
            sum.add_fn(Node::new(Value::constant(constant)));
            for f in fns {
                sum.add_fn(f);
            }
            Node::new(sum)
        }
    }
}
