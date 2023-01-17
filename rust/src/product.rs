//a Imports
use crate::{Function, Sum, Value};

//a Product
//tp Product
pub struct Product<'a> {
    s: f64,
    fns: Vec<Box<dyn Function + 'a>>,
}

//ip Default for Product
impl<'arg> std::default::Default for Product<'arg> {
    fn default() -> Self {
        Self {
            s: 1.,
            fns: Vec::new(),
        }
    }
}

//ip Display for Product
impl<'arg> std::fmt::Display for Product<'arg> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self.fns.len() {
            0 => write!(fmt, "{}", self.s),
            _ => {
                let mut pre = "(";
                if self.s != 1. {
                    write!(fmt, "{}", self.s)?;
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
impl<'arg> Product<'arg> {
    pub fn scale(&mut self, c: f64) {
        self.s *= c;
    }
    pub fn add_fn(&mut self, f: Box<dyn Function + 'arg>) {
        if let Some(x) = f.as_constant() {
            self.s *= x;
        } else {
            self.fns.push(f);
        }
    }
}

//ip Function for Product
impl<'a> Function for Product<'a> {
    //fp clone
    fn clone<'arg>(&'arg self) -> Box<dyn Function + 'arg> {
        let fns = self.fns.iter().map(|f| (*f).clone()).collect();
        Box::new(Product::<'arg> { s: self.s, fns })
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
    fn evaluate<'e>(&'e self, arg_to_value: &'e dyn Fn(&'e Value) -> f64) -> f64 {
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
    fn has_arg(&self, arg: &Value) -> bool {
        for f in self.fns.iter() {
            if f.has_arg(arg) {
                return true;
            }
        }
        return false;
    }

    //fp differentiate
    fn differentiate<'arg>(&'arg self, arg: &'arg Value) -> Option<Box<dyn Function + 'arg>> {
        let mut sum = Sum::default();
        for (i, f) in self.fns.iter().enumerate() {
            if let Some(df) = f.differentiate(arg) {
                let mut product = Product::<'arg> {
                    s: self.s,
                    fns: vec![],
                };
                for (j, f) in self.fns.iter().enumerate() {
                    if i != j {
                        product.add_fn((*f).clone());
                    }
                }
                product.add_fn(df);
                sum.add_fn(Box::new(product));
            }
        }
        if sum.is_zero() {
            None
        } else {
            Some(Box::new(sum))
        }
    }

    //fp as_products
    fn as_products<'s, 'arg>(&'s self) -> Option<(f64, Vec<Box<dyn Function + 'arg>>)> {
        Some((self.s, self.fns.iter().map(|f| (*f).clone()).collect()))
    }

    //fp simplify
    fn simplify<'arg>(&'arg self) -> Option<Box<dyn Function + 'arg>> {
        println!("Simplify {}", self);
        let mut fns = Vec::new();
        let mut constant = self.s;
        let mut simplified = false;
        for f in self.fns.iter() {
            if let Some(c) = f.as_constant() {
                constant *= c;
            } else if let Some(f) = f.simplify() {
                if let Some(c) = f.as_constant() {
                    constant *= c;
                } else if let Some((c, ps)) = f.as_products() {
                    constant *= c;
                    for p in ps.into_iter() {
                        fns.push(p.clone());
                    }
                    simplified = true;
                } else {
                    fns.push((*f).clone());
                }
            } else if let Some((c, ps)) = f.as_products() {
                constant *= c;
                for p in ps.into_iter() {
                    fns.push(p);
                }
                simplified = true;
            } else {
                fns.push((*f).clone());
            }
        }
        println!("{} {}", constant, fns.len());
        if fns.is_empty() {
            Some(Box::new(Value::constant(constant)))
        } else if constant == 1. && fns.len() == 1 {
            let f = fns.pop().unwrap();
            Some(f)
        } else if simplified {
            let mut product = Product::default();
            product.scale(constant);
            println!("Simplified to {}", constant);
            for f in fns {
                println!(" * {}", f);
                product.add_fn(f);
            }
            Some(Box::new(product))
        } else {
            None
        }
    }
}
