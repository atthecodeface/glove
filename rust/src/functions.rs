//a Imports
use crate::{Function, Value};

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

    fn evaluate(&self, arg_to_value: &dyn Fn(&Value) -> f64) -> f64 {
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

//a Sum
#[derive(Default)]
pub struct Sum<'a> {
    fns: Vec<Box<dyn Function + 'a>>,
}

impl<'arg> Sum<'arg> {
    pub fn add_fn(&mut self, f: Box<dyn Function + 'arg>) {
        if !f.is_zero() {
            self.fns.push(f);
        }
    }
}

impl<'arg> std::fmt::Display for Sum<'arg> {
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

impl<'a> Function for Sum<'a> {
    fn clone<'arg>(&'arg self) -> Box<dyn Function + 'arg> {
        let fns = self.fns.iter().map(|f| (*f).clone()).collect();
        Box::new(Sum::<'arg> { fns })
    }

    fn as_constant(&self) -> Option<f64> {
        if self.fns.is_empty() {
            Some(0.0)
        } else {
            None
        }
    }

    fn evaluate(&self, arg_to_value: &dyn Fn(&Value) -> f64) -> f64 {
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

    fn has_arg(&self, arg: &Value) -> bool {
        for f in self.fns.iter() {
            if f.has_arg(arg) {
                return true;
            }
        }
        return false;
    }
    fn differentiate<'arg>(&'arg self, arg: &'arg Value) -> Option<Box<dyn Function + 'arg>> {
        let mut sum = Sum::<'arg> { fns: vec![] };
        for f in self.fns.iter() {
            if let Some(df) = f.differentiate(arg) {
                sum.add_fn(df)
            }
        }
        if sum.fns.is_empty() {
            None
        } else {
            Some(Box::new(sum))
        }
    }
}

//a Product
#[derive(Default)]
pub struct Product<'a> {
    fns: Vec<Box<dyn Function + 'a>>,
}

impl<'arg> std::fmt::Display for Product<'arg> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self.fns.len() {
            0 => write!(fmt, "1"),
            1 => self.fns[0].fmt(fmt),
            _ => {
                write!(fmt, "(")?;
                for (i, f) in self.fns.iter().enumerate() {
                    if i > 0 {
                        write!(fmt, "*")?;
                    }
                    f.fmt(fmt)?;
                }
                write!(fmt, ")")
            }
        }
    }
}

impl<'arg> Product<'arg> {
    pub fn add_fn(&mut self, f: Box<dyn Function + 'arg>) {
        if let Some(x) = f.as_constant() {
            if x == 0. {
                self.fns.clear();
                self.fns.push(f);
            } else if x != 1. {
                self.fns.push(f);
            }
        } else {
            self.fns.push(f);
        }
    }
}

impl<'a> Function for Product<'a> {
    fn clone<'arg>(&'arg self) -> Box<dyn Function + 'arg> {
        let fns = self.fns.iter().map(|f| (*f).clone()).collect();
        Box::new(Product::<'arg> { fns })
    }

    fn as_constant(&self) -> Option<f64> {
        if self.fns.is_empty() {
            Some(1.0)
        } else {
            None
        }
    }

    fn evaluate(&self, arg_to_value: &dyn Fn(&Value) -> f64) -> f64 {
        if let Some(x) = self.as_constant() {
            x
        } else {
            let mut r = 1.;
            for f in self.fns.iter() {
                r *= f.evaluate(arg_to_value);
            }
            r
        }
    }

    fn has_arg(&self, arg: &Value) -> bool {
        for f in self.fns.iter() {
            if f.has_arg(arg) {
                return true;
            }
        }
        return false;
    }

    fn differentiate<'arg>(&'arg self, arg: &'arg Value) -> Option<Box<dyn Function + 'arg>> {
        let mut sum = Sum::<'arg> { fns: vec![] };
        for (i, f) in self.fns.iter().enumerate() {
            if let Some(df) = f.differentiate(arg) {
                let mut product = Product::<'arg> { fns: vec![] };
                for (j, f) in self.fns.iter().enumerate() {
                    if i != j {
                        product.add_fn((*f).clone());
                    }
                }
                product.add_fn(df);
                sum.add_fn(Box::new(product));
            }
        }
        if sum.fns.is_empty() {
            None
        } else {
            Some(Box::new(sum))
        }
    }
}

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
    fn evaluate(&self, arg_to_value: &dyn Fn(&Value) -> f64) -> f64 {
        let v = self.g.evaluate(arg_to_value);
        let f_value = |_| v;
        self.f.evaluate(&f_value)
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
}

//ip Display for FunctionOfFunction
impl<'arg> std::fmt::Display for FunctionOfFunction<'arg> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "{}o{}", self.f, self.g)
    }
}
