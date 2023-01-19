//a Imports
use crate::{Arg, Function, Node, Product};

//a FunctionOfFunction
//tp FunctionOfFunction
pub struct FunctionOfFunction<A: Arg> {
    // For f(g())
    f: Node<A>,
    f_arg: A,
    g: Node<A>,
}

//ip FunctionOfFunction
impl<A: Arg> FunctionOfFunction<A> {
    pub fn new(f: Node<A>, f_arg: &A, g: Node<A>) -> FunctionOfFunction<A> {
        let f_arg = f_arg.clone();
        FunctionOfFunction { f, f_arg, g }
    }
}

//ip Function for FunctionOfFunction
impl<A: Arg> Function<A> for FunctionOfFunction<A> {
    //fp clone
    fn clone(&self) -> Node<A> {
        Node::new(FunctionOfFunction {
            f: self.f.clone(),
            f_arg: self.f_arg.clone(),
            g: self.g.clone(),
        })
    }

    //fp as_constant
    fn as_constant(&self) -> Option<f64> {
        if let Some(c) = self.f.as_constant() {
            Some(c)
        } else {
            None
        }
    }

    //fp evaluate
    fn evaluate(&self, arg_to_value: &dyn Fn(&A) -> f64) -> f64 {
        let v = self.g.evaluate(arg_to_value);
        fn f_value<A: Arg>(v: f64, _: &A) -> f64 {
            v
        }
        self.f.evaluate(&|a| f_value(v, a))
        //        0.
    }

    //fp has_arg
    fn has_arg(&self, arg: &A) -> bool {
        self.g.has_arg(arg)
    }

    //fp differentiate
    fn differentiate(&self, arg: &A) -> Option<Node<A>> {
        let dg = self.g.differentiate(arg);
        if dg.is_none() {
            return None;
        }
        let dg = dg.unwrap();
        if let Some(df) = self.f.differentiate(&self.f_arg) {
            let mut product = Product::default();
            product.add_fn(dg);
            product.add_fn(Node::new(FunctionOfFunction::new(
                df,
                &self.f_arg,
                self.g.clone(),
            )));
            Some(Node::new(product))
        } else {
            None
        }
    }

    //fp simplified
    fn simplified(self: Box<Self>) -> Node<A> {
        let f_simp = self.f.simplified();
        let g_simp = self.g.simplified();
        Node::new(FunctionOfFunction::new(f_simp, &self.f_arg, g_simp))
    }
}

//ip Display for FunctionOfFunction
impl<A: Arg> std::fmt::Display for FunctionOfFunction<A> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "[{} o {}]", self.f, self.g)
    }
}
