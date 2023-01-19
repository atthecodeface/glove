mod traits;
pub use traits::{Arg, Function, Node};

mod value;
pub use value::Value;
mod sum;
pub use sum::Sum;
mod product;
pub use product::Product;
mod power_of;
pub use power_of::PowerOf;
mod fn_of_fn;
pub use fn_of_fn::FunctionOfFunction;

#[test]
pub fn test_simple() {
    let a = "qi".to_string();
    let qi = Value::new_arg(&a);
    let f = PowerOf::new(qi.clone(), 2);
    let df_dqi = f.differentiate(&a).unwrap();
    println!("  f: {}", f);
    println!(" df: {}", df_dqi);
    let d2f_dqi = df_dqi.differentiate(&a).unwrap();
    println!("d2f: {}", d2f_dqi);

    let x: fn(&String) -> f64 = |_| 3.;
    let x_dx: fn(&String) -> f64 = |_| 3. + 0.0001;
    println!("  f(3): {}", f.evaluate(&x));
    println!("  f(3+): {}", f.evaluate(&x_dx));
    println!("  diff): {}", f.evaluate(&x_dx) - f.evaluate(&x));
    println!("  grad): {}", (f.evaluate(&x_dx) - f.evaluate(&x)) / 0.0001);
    println!(" df(3): {}", df_dqi.evaluate(&x));
    assert!(false);
}

#[test]
pub fn test_simple2() {
    let a = "qi".to_string();
    let qi = Value::new_arg(&a);
    let mut two_qi = Product::default();
    two_qi.add_fn(qi.clone());
    two_qi.add_fn(Node::new(Value::constant(2.)));
    let f = PowerOf::new(two_qi.clone(), 2);
    let df_dqi = f.differentiate(&a).unwrap();
    println!("  f: {}", f);
    println!(" df: {}", df_dqi);
    let d2f_dqi = df_dqi.differentiate(&a).unwrap();
    println!("d2f: {}", d2f_dqi);
    let d3f_dqi = d2f_dqi.differentiate(&a);
    assert!(!d3f_dqi.is_none());
}

#[test]
pub fn test_fn_of_fn() {
    let ax = "x".to_string();
    let a = "qi".to_string();
    let x = Value::new_arg(&ax);
    let x_sq = PowerOf::new(x.clone(), 2);
    let qi = Value::new_arg(&a);
    let mut two_qi = Product::default();
    two_qi.add_fn(qi.clone());
    two_qi.add_fn(Node::new(Value::constant(2.)));

    let x_sq_2_qi = FunctionOfFunction::new(Node::new(x_sq), &ax, Node::new(two_qi));

    let df_dqi = x_sq_2_qi.differentiate(&a).unwrap();
    println!("  f: {}", x_sq_2_qi);
    println!(" df: {}", df_dqi);
    let df_dqi_simp = df_dqi.clone().simplified();
    println!("simp: {}", df_dqi_simp);
    let d2f_dqi = df_dqi.differentiate(&a).unwrap();
    println!("d2f: {}", d2f_dqi);
    let d2f_dqi_simp = d2f_dqi.clone().simplified();
    println!("simp d2f: {}", d2f_dqi_simp);
    let d3f_dqi = d2f_dqi.differentiate(&a);
    assert!(!d3f_dqi.is_none());
}
