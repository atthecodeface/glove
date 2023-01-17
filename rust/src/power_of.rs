//a Imports
use crate::{Function, Product, Value};

//a PowerOf
pub struct PowerOf<'arg> {
    x: Box<dyn Function + 'arg>,
    n: isize,
}

impl<'arg> PowerOf<'arg> {
    pub fn new(x: Box<dyn Function + 'arg>, n: isize) -> Self {
        Self { x, n }
    }
}

impl<'arg> Function for PowerOf<'arg> {
    fn clone<'a>(&'a self) -> Box<dyn Function + 'a> {
        Box::new(PowerOf::<'a> {
            x: self.x.clone(),
            n: self.n,
        })
    }

    //fp evaluate
    fn evaluate<'e>(&'e self, arg_to_value: &'e dyn Fn(&'e Value) -> f64) -> f64 {
        self.x.evaluate(arg_to_value).powf(self.n as f64)
    }

    fn has_arg(&self, arg: &Value) -> bool {
        self.x.has_arg(arg)
    }

    fn differentiate<'a>(&'a self, arg: &'a Value) -> Option<Box<dyn Function + 'a>> {
        let dx = self.x.differentiate(arg);
        if dx.is_none() {
            return None;
        }
        let dx = dx.unwrap();
        let df: Box<dyn Function> = {
            match self.n {
                1 => Box::new(Value::one()),
                n => {
                    let mut f = Product::default();
                    f.add_fn(Box::new(Value::constant(n as f64)));
                    f.add_fn(self.x.clone());
                    Box::new(f)
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
            Some(Box::new(product))
        }
    }
}

impl<'arg> std::fmt::Display for PowerOf<'arg> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}^{}", self.x, self.n)
    }
}
