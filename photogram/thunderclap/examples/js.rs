use thunderclap::json;

impl thunderclap::CommandArgs for CmdArgs {
    type Error = serde_json::Error;
    type Value = json::Value;

    fn value_from_str(s: &str) -> Result<Self::Value, Self::Error> {
        eprintln!("Json from str '{s}'");
        if let Ok(v) = serde_json::from_str::<Self::Value>(s) {
            eprintln!("Value {v:?}");
            return Ok(v);
        }
        let v = serde_json::to_value(s)?;
        eprintln!("Value {v:?}");
        Ok(v)
    }
}
#[derive(Debug, Default)]
pub struct CmdArgs {}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = thunderclap::Command::new("js").about("Example JsonValue");
    let build = thunderclap::CommandBuilder::<CmdArgs>::new(command);
    let mut cmd_args = CmdArgs::default();
    let mut command = build.main(true, true);
    command.execute_env(&mut cmd_args)?;
    Ok(())
}
