pub trait Arg: Clone + std::fmt::Display + Eq + 'static {}

impl Arg for String {}

pub struct Node<A: Arg> {
    bf: Box<dyn Function<A>>,
}

impl<A: Arg> Node<A> {
    pub fn new<F: Function<A> + 'static>(bf: F) -> Node<A> {
        let bf = Box::new(bf);
        Self { bf }
    }
}

impl<A: Arg> std::fmt::Display for Node<A> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.bf.fmt(fmt)
    }
}

pub trait Function<A: Arg>: std::fmt::Display {
    fn clone(&self) -> Node<A>;
    fn as_constant(&self) -> Option<f64> {
        None
    }
    fn is_zero(&self) -> bool {
        self.as_constant() == Some(0.)
    }
    fn is_one(&self) -> bool {
        self.as_constant() == Some(1.)
    }
    fn has_arg(&self, arg: &A) -> bool;
    fn differentiate(&self, arg: &A) -> Option<Node<A>>;
    fn evaluate(&self, arg_to_value: &dyn Fn(&A) -> f64) -> f64;
    fn simplified(self: Box<Self>) -> Node<A>;
    fn as_products(self: Box<Self>) -> (f64, Vec<Node<A>>) {
        (1., vec![self.clone()])
    }
    fn as_sums(self: Box<Self>) -> Vec<Node<A>> {
        vec![self.clone()]
    }
}

impl<A: Arg> Node<A> {
    pub fn is_zero(&self) -> bool {
        self.as_constant() == Some(0.)
    }
    pub fn is_one(&self) -> bool {
        self.as_constant() == Some(1.)
    }
    pub fn clone_node(&self) -> Node<A> {
        self.bf.clone()
    }
    pub fn as_constant(&self) -> Option<f64> {
        self.bf.as_constant()
    }
    pub fn has_arg(&self, arg: &A) -> bool {
        self.bf.has_arg(arg)
    }
    pub fn differentiate(&self, arg: &A) -> Option<Node<A>> {
        self.bf.differentiate(arg)
    }
    pub fn evaluate(&self, arg_to_value: &dyn Fn(&A) -> f64) -> f64 {
        self.bf.evaluate(arg_to_value)
    }
    pub fn simplified(self) -> Node<A> {
        self.bf.simplified()
    }
    pub fn as_products(self) -> (f64, Vec<Node<A>>) {
        self.bf.as_products()
    }
    pub fn as_sums(self) -> Vec<Node<A>> {
        self.bf.as_sums()
    }
}
