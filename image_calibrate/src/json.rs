//a Imports
use std::path::Path;

use serde::Deserialize;

//a Public functions
//fp read_file
pub fn read_file<P: AsRef<Path> + std::fmt::Display>(path: P) -> Result<String, String> {
    let file_text = std::fs::read_to_string(&path)
        .map_err(|e| format!("Error reading json file {}: {}", path, e))?;
    let mut json_string = String::new();
    for j in file_text.lines() {
        if let Some(n) = j.find("//") {
            json_string.push_str(j.split_at(n).0);
            json_string.push('\n');
        } else {
            json_string.push_str(j);
            json_string.push('\n');
        }
    }
    Ok(json_string)
}

//fi json_error
fn json_error(reason: &str, json: &str, err: serde_json::Error) -> String {
    let line = err.line();
    let column = err.column();
    let mut result = format!(
        "Error in parsing {} json at line {} column {}: {}",
        reason, line, column, err
    );
    let start_line = if line > 5 { line - 5 } else { 0 };
    let end_line = line + 5;
    for (i, l) in json.lines().enumerate() {
        if i >= start_line {
            result.push_str(&format!("\n{:4}: {}", i + 1, l));
        }
        if i >= end_line {
            break;
        }
    }
    result
}

//fp from_json
pub fn from_json<'de, P: Deserialize<'de>>(reason: &str, json: &'de str) -> Result<P, String> {
    serde_json::from_str(json).map_err(|e| json_error(reason, json, e))
}
