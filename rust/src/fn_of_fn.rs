//a Imports
use crate::{Function, Product, Value};

//a FunctionOfFunction
//tp FunctionOfFunction
pub struct FunctionOfFunction<'arg> {
    // For f(g())
    f: Box<dyn Function + 'arg>,
    f_arg: &'arg Value<'arg>,
    g: Box<dyn Function + 'arg>,
}

//ip FunctionOfFunction
impl<'arg> FunctionOfFunction<'arg> {
    pub fn new(
        f: Box<dyn Function + 'arg>,
        f_arg: &'arg Value<'arg>,
        g: Box<dyn Function + 'arg>,
    ) -> Self {
        Self { f, f_arg, g }
    }
}

//ip Function for FunctionOfFunction
impl<'arg> Function for FunctionOfFunction<'arg> {
    //fp clone
    fn clone<'a>(&'a self) -> Box<dyn Function + 'a> {
        Box::new(FunctionOfFunction::<'a> {
            f: self.f.clone(),
            f_arg: self.f_arg,
            g: self.g.clone(),
        })
    }

    //fp evaluate
    fn evaluate<'e>(&'e self, arg_to_value: &'e dyn Fn(&'e Value) -> f64) -> f64 {
        let v = self.g.evaluate(arg_to_value);
        fn f_value<'a>(v: f64, _: &'a Value<'a>) -> f64 {
            v
        }
        self.f.evaluate(&|a| f_value(v, a))
    }

    //fp has_arg
    fn has_arg(&self, arg: &Value) -> bool {
        self.g.has_arg(arg)
    }

    //fp differentiate
    fn differentiate<'a>(&'a self, arg: &'a Value) -> Option<Box<dyn Function + 'a>> {
        let dg = self.g.differentiate(arg);
        if dg.is_none() {
            return None;
        }
        let dg = dg.unwrap();
        if let Some(df) = self.f.differentiate(self.f_arg) {
            let mut product = Product::default();
            product.add_fn(dg);
            product.add_fn(Box::new(FunctionOfFunction::new(
                df,
                self.f_arg,
                self.g.clone(),
            )));
            Some(Box::new(product))
        } else {
            None
        }
    }

    //fp simplify
    fn simplify<'s>(&'s self) -> Option<Box<dyn Function + 's>> {
        if let Some(f_simp) = self.f.simplify() {
            if let Some(g_simp) = self.g.simplify() {
                Some(Box::new(FunctionOfFunction::new(
                    f_simp, self.f_arg, g_simp,
                )))
            } else {
                Some(Box::new(FunctionOfFunction::new(
                    f_simp,
                    self.f_arg,
                    self.g.clone(),
                )))
            }
        } else {
            if let Some(g_simp) = self.g.simplify() {
                Some(Box::new(FunctionOfFunction::new(
                    self.f.clone(),
                    self.f_arg,
                    g_simp,
                )))
            } else {
                None
            }
        }
    }
}

//ip Display for FunctionOfFunction
impl<'arg> std::fmt::Display for FunctionOfFunction<'arg> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "[{} o {}]", self.f, self.g)
    }
}
