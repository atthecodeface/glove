use crate::Value;

pub trait Function<'arg>: 'a + std::fmt::Display {
    fn clone<'a>(&'a self) -> Box<dyn Function + 'arg>;
    fn as_constant(&self) -> Option<f64> {
        None
    }
    fn is_zero(&self) -> bool {
        self.as_constant() == Some(0.)
    }
    fn is_one(&self) -> bool {
        self.as_constant() == Some(1.)
    }
    fn has_arg(&self, arg: &Value) -> bool;
    fn differentiate<'arg>(&'arg self, arg: &'arg Value) -> Option<Box<dyn Function + 'arg>>;
    fn evaluate<'e>(&'e self, arg_to_value: &'e dyn Fn(&'e Value) -> f64) -> f64;
    fn simplify<'arg>(&'arg self) -> Option<Box<dyn Function + 'arg>> {
        None
    }
    fn simplified<'arg>(&'arg self) -> Box<dyn Function + 'arg> {
        if let Some(f) = self.simplify() {
            f
        } else {
            self.clone()
        }
    }
    fn as_products<'s, 'arg>(&'s self) -> Option<(f64, Vec<Box<dyn Function + 'arg>>)> {
        None
    }
    fn as_sums<'arg>(&'arg self) -> Option<Vec<Box<dyn Function + 'arg>>> {
        None
    }
}
