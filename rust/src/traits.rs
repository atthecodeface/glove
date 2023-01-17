use crate::Value;

pub trait Function<'arg>: 'arg + std::fmt::Display {
    fn clone(&self) -> Box<dyn Function<'arg> + 'arg>;
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
    fn differentiate<'s: 'arg>(
        &'s self,
        arg: &'arg Value,
    ) -> Option<Box<dyn Function<'arg> + 'arg>>;
    fn evaluate<'e: 'arg>(&'e self, arg_to_value: &'e dyn Fn(&'e Value) -> f64) -> f64;
    fn simplify<'s: 'arg>(&'s self) -> Option<Box<dyn Function<'arg> + 'arg>> {
        None
    }
    fn simplified<'s: 'arg>(&'s self) -> Box<dyn Function<'arg> + 'arg> {
        if let Some(f) = self.simplify() {
            f
        } else {
            self.clone()
        }
    }
    fn as_products(self: Box<Self>) -> Option<(f64, Vec<Box<dyn Function<'arg> + 'arg>>)> {
        None
    }
    fn as_sums<'s: 'arg>(&'s self) -> Option<Vec<Box<dyn Function<'arg> + 'arg>>> {
        None
    }
}
