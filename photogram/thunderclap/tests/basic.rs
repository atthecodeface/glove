//a Imports
use clap::Command;
use thunderclap::{CommandArgs, CommandArgsValue, CommandBuilder};

//a Value
trait Value:
    std::str::FromStr<Err: std::error::Error>
    + CommandArgsValue
    + 'static
    + std::ops::AddAssign
    + std::ops::SubAssign
    + Copy
{
}

impl<V> Value for V where
    V: std::str::FromStr<Err: std::error::Error>
        + CommandArgsValue
        + 'static
        + std::ops::AddAssign
        + std::ops::SubAssign
        + Copy
{
}

//a Op
#[derive(Debug, Default)]
enum Op {
    #[default]
    Add,
    Sub,
}

//a CmdArgs
#[derive(Debug, Default)]
struct CmdArgs<V>
where
    V: Value,
{
    op: Op,
    value: V,
    args: Vec<V>,
}

type Error<V> = <V as std::str::FromStr>::Err;

impl<V> CommandArgs for CmdArgs<V>
where
    V: Value,
{
    type Error = Error<V>;
    type Value = V;
    const PROPERTIES: &[thunderclap::CmdProperty<'static, Self, Self::Value, Self::Error>] = &[];
    fn value_from_str(s: &str) -> Result<V, Error<V>> {
        s.parse()
    }
    fn reset_args(&mut self) {
        self.args.clear();
        self.op = Op::Add;
    }
}

impl<V> CmdArgs<V>
where
    V: Value,
{
    fn push_arg(&mut self, value: V) -> Result<(), Error<V>> {
        self.args.push(value);
        Ok(())
    }

    fn clear(&mut self, f: bool) -> Result<(), Error<V>> {
        if f {
            self.value = V::default();
        }
        Ok(())
    }

    fn set_sub(&mut self, f: bool) -> Result<(), Error<V>> {
        if f {
            self.op = Op::Sub;
        } else {
            self.op = Op::Add;
        }
        Ok(())
    }

    fn calc(&mut self) -> Result<V, Error<V>> {
        for v in self.args.drain(..) {
            match self.op {
                Op::Add => {
                    self.value += v;
                }
                Op::Sub => {
                    self.value -= v;
                }
            }
        }
        Ok(self.value)
    }
}

//a Useful functions
//ft test_build
fn test_build<V: Value>() -> CommandBuilder<CmdArgs<V>> {
    let mut build = CommandBuilder::<CmdArgs<V>>::with_handler(Command::new("calc"), |x| x.calc());
    // A flag is *always* invoked
    build.add_flag(
        "clear",
        Some('c'),
        "If provided will clear the accumulator before using the new values",
        |x, v| x.clear(v),
    );
    // A flag is *always* invoked
    build.add_flag(
        "sub",
        Some('s'),
        "If provided will subtract, not add",
        |x, v| x.set_sub(v),
    );
    build
}

//a Simple tests for different basic types
//ft test_f32
#[test]
fn test_f32() -> Result<(), Box<dyn std::error::Error>> {
    let mut build = test_build();
    build.add_arg_f32(
        "value",
        None,
        "Values to add",
        (None, true),
        None,
        CmdArgs::push_arg,
    );
    let mut main = build.main(false, false);
    let mut args = CmdArgs::default();

    let _ = main.execute(&mut args, ["1.0"], false)?;
    assert_eq!(args.value, 1.0);
    let _ = main.execute(&mut args, ["1.0"], false)?;
    assert_eq!(args.value, 2.0);
    let _ = main.execute(&mut args, ["-s", "1.0"], false)?;
    assert_eq!(args.value, 1.0);
    let _ = main.execute(&mut args, ["1.0"], false)?; // Will add again
    assert_eq!(args.value, 2.0);

    Ok(())
}

//ft test_f64
#[test]
fn test_f64() -> Result<(), Box<dyn std::error::Error>> {
    let mut build = test_build();
    build.add_arg_f64(
        "value",
        None,
        "Values to add",
        (None, true),
        None,
        CmdArgs::push_arg,
    );
    let mut main = build.main(false, false);
    let mut args = CmdArgs::default();

    let _ = main.execute(&mut args, ["3.0"], false)?;
    assert_eq!(args.value, 3.0);

    Ok(())
}

//ft test_usize
#[test]
fn test_usize() -> Result<(), Box<dyn std::error::Error>> {
    let mut build = test_build();
    build.add_arg_usize(
        "value",
        None,
        "Values to add",
        (None, true),
        None,
        CmdArgs::push_arg,
    );
    let mut main = build.main(false, false);
    let mut args = CmdArgs::default();

    // Cannot do this, it *exits*
    // assert!(main.execute(&mut args, ["3.0"], false).is_err());
    let _ = main.execute(&mut args, ["3"], false)?;
    assert_eq!(args.value, 3);

    Ok(())
}

//ft test_u64
#[test]
fn test_u64() -> Result<(), Box<dyn std::error::Error>> {
    let mut build = test_build();
    build.add_arg_u64(
        "value",
        None,
        "Values to add",
        (None, true),
        None,
        CmdArgs::push_arg,
    );
    let mut main = build.main(false, false);
    let mut args = CmdArgs::default();

    // Cannot do this, it *exits*
    // assert!(main.execute(&mut args, ["3.0"], false).is_err());
    let _ = main.execute(&mut args, ["2"], false)?;
    assert_eq!(args.value, 2);

    Ok(())
}

//ft test_u32
#[test]
fn test_u32() -> Result<(), Box<dyn std::error::Error>> {
    let mut build = test_build();
    build.add_arg_u32(
        "value",
        None,
        "Values to add",
        (None, true),
        None,
        CmdArgs::push_arg,
    );
    let mut main = build.main(false, false);
    let mut args = CmdArgs::default();

    // Cannot do this, it *exits*
    // assert!(main.execute(&mut args, ["3.0"], false).is_err());
    let _ = main.execute(&mut args, ["2", "5", "8"], false)?;
    assert_eq!(args.value, 15);

    Ok(())
}

//ft test_isize
#[test]
fn test_isize() -> Result<(), Box<dyn std::error::Error>> {
    let mut build = test_build();
    build.add_arg_isize(
        "value",
        None,
        "Values to add",
        (None, true),
        None,
        CmdArgs::push_arg,
    );
    let mut main = build.main(false, false);
    let mut args = CmdArgs::default();

    assert!(main.execute(&mut args, ["3.0"], true).is_err());
    let _ = main.execute(&mut args, ["3"], false)?;
    assert_eq!(args.value, 3);

    Ok(())
}

//ft test_i64
#[test]
fn test_i64() -> Result<(), Box<dyn std::error::Error>> {
    let mut build = test_build();
    build.add_arg_i64(
        "value",
        None,
        "Values to add",
        (None, true),
        None,
        CmdArgs::push_arg,
    );
    let mut main = build.main(false, false);
    let mut args = CmdArgs::default();

    // Cannot do this, it *exits*
    // assert!(main.execute(&mut args, ["3.0"], false).is_err());
    let _ = main.execute(&mut args, ["2", "--sub"], false)?;
    assert_eq!(args.value, -2);

    Ok(())
}

//ft test_i32
#[test]
fn test_i32() -> Result<(), Box<dyn std::error::Error>> {
    let mut build = test_build();
    build.add_arg_i32(
        "value",
        None,
        "Values to add",
        (None, true),
        None,
        CmdArgs::push_arg,
    );
    let mut main = build.main(false, false);
    let mut args = CmdArgs::default();

    // Cannot do this, it *exits*
    // assert!(main.execute(&mut args, ["3.0"], false).is_err());
    let _ = main.execute(&mut args, ["2", "5", "8"], false)?;
    assert_eq!(args.value, 15);

    Ok(())
}

//a Batch
//ft test_isize_batch
#[test]
fn test_isize_batch() -> Result<(), Box<dyn std::error::Error>> {
    let mut build = test_build();
    build.add_arg_isize(
        "value",
        None,
        "Values to add",
        (None, true),
        None,
        CmdArgs::push_arg,
    );
    let mut main = build.main(true, false);
    let mut args = CmdArgs::default();

    // Can do this, as it does not exit in batch mode
    assert!(main.execute(&mut args, ["3.0"], true).is_err());
    let _ = main.execute(&mut args, ["3"], false)?;
    assert_eq!(args.value, 3);
    let _ = main.execute_batch(
        &mut args,
        "test_isize_batch",
        r#"
"#,
    )?;
    assert_eq!(args.value, 3);

    let _ = main.execute_batch(
        &mut args,
        "test_isize_batch",
        r#"
1 4 5
"#,
    )?;
    assert_eq!(args.value, 3 + 1 + 4 + 5);
    let _ = main.execute_batch(
        &mut args,
        "test_isize_batch",
        r#"
stack_push ${0} 17
echo "The top value is ${1}"
echo "The second value is ${2}"
stack_show
-c ${1} ${2}
-s 1 2 3
5
set x ${0}
6
-c ${x}
"#,
    )?;
    assert_eq!(args.value, 17 + 13 - 1 - 2 - 3 + 5);

    Ok(())
}
