//a Imports
use crate::{Function, Product, Value};

//a FunctionOfFunction
//tp FunctionOfFunction
pub struct FunctionOfFunction<'f, 'arg>
where
    'arg: 'f,
{
    // For f(g())
    f: Box<dyn Function<'arg>>,
    f_arg: &'f Value<'arg>,
    g: Box<dyn Function<'arg>>,
}

//ip FunctionOfFunction
impl<'f, 'arg> FunctionOfFunction<'f, 'arg>
where
    'arg: 'f,
{
    pub fn new<'a>(
        f: Box<dyn Function<'arg>>,
        f_arg: &'a Value<'arg>,
        g: Box<dyn Function<'arg>>,
    ) -> FunctionOfFunction<'a, 'arg>
    where
        'arg: 'a,
    {
        FunctionOfFunction::<'a> { f, f_arg, g }
    }
}

//ip Function for FunctionOfFunction
impl<'f, 'arg> Function<'arg> for FunctionOfFunction<'f, 'arg>
where
    'arg: 'f,
{
    //fp clone
    fn clone(&self) -> Box<dyn Function<'arg>> {
        Box::new(FunctionOfFunction {
            f: self.f.clone(),
            f_arg: self.f_arg,
            g: self.g.clone(),
        })
    }

    //fp evaluate
    fn evaluate(&self, arg_to_value: &dyn Fn(&Value) -> f64) -> f64 {
        let v = self.g.evaluate(arg_to_value);
        fn f_value<'a>(v: f64, _: &'a Value<'a>) -> f64 {
            v
        }
        // self.f.evaluate(&|a| f_value(v, a))
        0.
    }

    //fp has_arg
    fn has_arg(&self, arg: &Value) -> bool {
        self.g.has_arg(arg)
    }

    //fp differentiate
    fn differentiate(&self, arg: &Value) -> Option<Box<dyn Function<'arg>>> {
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

    //fp simplified
    fn simplified(self: Box<Self>) -> Box<dyn Function<'arg> + 'arg> {
        let f_simp = self.f.simplified();
        let g_simp = self.g.simplified();
        Box::new(FunctionOfFunction::new(f_simp, self.f_arg, g_simp))
    }
}

//ip Display for FunctionOfFunction
impl<'f, 'arg> std::fmt::Display for FunctionOfFunction<'f, 'arg> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "[{} o {}]", self.f, self.g)
    }
}
