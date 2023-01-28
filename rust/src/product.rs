//a Imports
use crate::{Arg, Function, Node, Sum, Value};

//a Product
//tp Product
pub struct Product<A: Arg> {
    s: f64,
    fns: Vec<Node<A>>,
}

//ip Default for Product
impl<A: Arg> std::default::Default for Product<A> {
    fn default() -> Self {
        Self {
            s: 1.,
            fns: Vec::new(),
        }
    }
}

//ip Display for Product
impl<A: Arg> std::fmt::Display for Product<A> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self.fns.len() {
            0 => write!(fmt, "{}", self.s),
            _ => {
                let mut pre = "(";
                if self.s != 1. {
                    write!(fmt, "({}", self.s)?;
                    pre = "*";
                }
                for f in self.fns.iter() {
                    pre.fmt(fmt)?;
                    f.fmt(fmt)?;
                    pre = "*";
                }
                write!(fmt, ")")
            }
        }
    }
}

//ip Product
impl<A: Arg> Product<A> {
    pub fn scale(&mut self, c: f64) {
        self.s *= c;
    }
    pub fn add_fn(&mut self, f: Node<A>) {
        if let Some(x) = f.as_constant() {
            self.s *= x;
        } else {
            self.fns.push(f);
        }
    }
}

//ip Function for Product
impl<A: Arg> Function<A> for Product<A> {
    //fp clone
    fn clone(&self) -> Node<A> {
        let fns = self.fns.iter().map(|f| (*f).clone_node()).collect();
        Node::new(Product { s: self.s, fns })
    }

    //fp as_constant
    fn as_constant(&self) -> Option<f64> {
        if self.fns.is_empty() {
            Some(self.s)
        } else {
            None
        }
    }

    //fp evaluate
    fn evaluate(&self, arg_to_value: &dyn Fn(&A) -> f64) -> f64 {
        if let Some(x) = self.as_constant() {
            x
        } else {
            let mut r = self.s;
            for f in self.fns.iter() {
                r *= f.evaluate(arg_to_value);
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
        false
    }

    //fp differentiate
    fn differentiate(&self, arg: &A) -> Option<Node<A>> {
        let mut sum = Sum::default();
        for (i, f) in self.fns.iter().enumerate() {
            if let Some(df) = f.differentiate(arg) {
                let mut product = Product {
                    s: self.s,
                    fns: vec![],
                };
                for (j, f) in self.fns.iter().enumerate() {
                    if i != j {
                        product.add_fn((*f).clone_node());
                    }
                }
                product.add_fn(df);
                sum.add_fn(Node::new(product));
            }
        }
        if sum.is_zero() {
            None
        } else {
            Some(Node::new(sum))
        }
    }

    //fp as_products
    fn as_products(self: Box<Self>) -> (f64, Vec<Node<A>>) {
        (
            self.s,
            self.fns.into_iter().map(|f| f.clone_node()).collect(),
        )
    }

    //fp simplified
    fn simplified(self: Box<Self>) -> Node<A> {
        println!("Simplify {}", self);
        let mut fns = Vec::new();
        let mut constant = self.s;
        for f in self.fns.into_iter() {
            let f = f.simplified();
            if let Some(c) = f.as_constant() {
                constant *= c;
            } else {
                let (c, ps) = f.as_products();
                constant *= c;
                for p in ps.into_iter() {
                    fns.push(p);
                }
            }
        }
        // println!("{} {}", constant, fns.len());
        if fns.is_empty() {
            Node::new(Value::constant(constant))
        } else if constant == 1. && fns.len() == 1 {
            fns.pop().unwrap()
        } else {
            let mut product = Product::default();
            product.scale(constant);
            println!("Simplified to {}", constant);
            for f in fns {
                println!(" * {}", f);
                product.add_fn(f);
            }
            Node::new(product)
        }
    }
}
