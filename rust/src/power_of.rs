//a Imports
use crate::{Arg, Function, Node, Product, Value};

//a PowerOf
//tp PowerOf
pub struct PowerOf<A: Arg> {
    x: Node<A>,
    n: isize,
}

//ip PowerOf
impl<A: Arg> PowerOf<A> {
    pub fn new(x: Node<A>, n: isize) -> Self {
        Self { x, n }
    }
}

//ip Function for PowerOf
impl<A: Arg> Function<A> for PowerOf<A> {
    //fp clone
    fn clone(&self) -> Node<A> {
        Node::new(PowerOf {
            x: self.x.clone(),
            n: self.n,
        })
    }

    //fp evaluate
    fn evaluate(&self, arg_to_value: &dyn Fn(&A) -> f64) -> f64 {
        self.x.evaluate(arg_to_value).powf(self.n as f64)
    }

    //fp has_arg
    fn has_arg(&self, arg: &A) -> bool {
        self.x.has_arg(arg)
    }

    //fp differentiate
    fn differentiate(&self, arg: &A) -> Option<Node<A>> {
        let dx = self.x.differentiate(arg);
        if dx.is_none() {
            return None;
        }
        let dx = dx.unwrap();
        let df = {
            match self.n {
                1 => Node::new(Value::one()),
                n => {
                    let mut f = Product::default();
                    f.add_fn(Node::new(Value::constant(n as f64)));
                    f.add_fn(self.x.clone());
                    Node::new(f)
                }
            }
        };
        if dx.is_zero() || df.is_zero() {
            None
        } else if dx.is_one() {
            Some(df)
        } else if df.is_one() {
            Some(dx)
        } else {
            let mut product = Product::default();
            product.add_fn(dx);
            product.add_fn(df);
            Some(Node::new(product))
        }
    }
    //fp simplified
    fn simplified(self: Box<Self>) -> Node<A> {
        Node::new(*self)
    }
}

//ip Display for PowerOf
impl<A: Arg> std::fmt::Display for PowerOf<A> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}^{}", self.x, self.n)
    }
}
