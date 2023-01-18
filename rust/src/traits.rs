use crate::Value;

pub trait Function<'arg>: 'arg + std::fmt::Display {
    fn clone(&self) -> Box<dyn Function<'arg>>;
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
    fn differentiate(&self, arg: &Value) -> Option<Box<dyn Function<'arg>>>;
    fn evaluate(&self, arg_to_value: &dyn Fn(&Value) -> f64) -> f64;
    fn simplified(self: Box<Self>) -> Box<dyn Function<'arg>>;
    fn as_products(self: Box<Self>) -> (f64, Vec<Box<dyn Function<'arg>>>) {
        (1., vec![self.clone()])
    }
    fn as_sums(self: Box<Self>) -> Vec<Box<dyn Function<'arg>>> {
        vec![self.clone()]
    }
}
