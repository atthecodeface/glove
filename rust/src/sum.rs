//a Imports
use crate::{Function, Value};

//a Sum
//tp Sum
#[derive(Default)]
pub struct Sum<'arg> {
    fns: Vec<Box<dyn Function<'arg> + 'arg>>,
}

//ip Sum
impl<'arg> Sum<'arg> {
    pub fn add_fn(&mut self, f: Box<dyn Function<'arg> + 'arg>) {
        if !f.is_zero() {
            self.fns.push(f);
        }
    }
}

//ip Display for Sum
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

//ip Function for Sum
impl<'arg> Function<'arg> for Sum<'arg> {
    //fp clone
    fn clone(&self) -> Box<dyn Function<'arg> + 'arg> {
        let fns = self.fns.iter().map(|f| (*f).clone()).collect();
        Box::new(Sum::<'arg> { fns })
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
    fn evaluate<'e: 'arg>(&'e self, arg_to_value: &'e dyn Fn(&'e Value) -> f64) -> f64 {
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
    fn has_arg(&self, arg: &Value) -> bool {
        for f in self.fns.iter() {
            if f.has_arg(arg) {
                return true;
            }
        }
        return false;
    }

    //fp differentiate
    fn differentiate<'s: 'arg>(
        &'s self,
        arg: &'arg Value,
    ) -> Option<Box<dyn Function<'arg> + 'arg>> {
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

    //fp simplify
    fn simplify<'a: 'arg>(&'a self) -> Option<Box<dyn Function<'arg> + 'arg>> {
        let mut fns = Vec::new();
        let mut constant = 0.;
        let mut simplified = false;
        for f in self.fns.iter() {
            if let Some(c) = f.as_constant() {
                constant += c;
            } else if let Some(f) = f.simplify() {
                if let Some(c) = f.as_constant() {
                    constant += c;
                } else {
                    fns.push(f);
                    simplified = true;
                }
            } else if let Some(ps) = f.as_sums() {
                for p in ps.into_iter() {
                    fns.push(p);
                }
                simplified = true;
            } else {
                fns.push((*f).clone());
            }
        }
        if fns.is_empty() {
            Some(Box::new(Value::constant(constant)))
        } else if constant == 0. && fns.len() == 1 {
            let f = fns.pop().unwrap();
            Some(f)
        } else if simplified {
            let mut sum = Sum::default();
            sum.add_fn(Box::new(Value::constant(constant)));
            for f in fns {
                sum.add_fn(f);
            }
            Some(Box::new(sum))
        } else {
            None
        }
    }
}
