//a Imports
use std::path::Path;

use serde::Deserialize;

use crate::{Error, Result};

//a Public functions
//fp remove_comments
pub fn remove_comments(s: &str) -> String {
    let mut json_string = String::new();
    for j in s.lines() {
        if let Some(n) = j.find("//") {
            json_string.push_str(j.split_at(n).0);
            json_string.push('\n');
        } else {
            json_string.push_str(j);
            json_string.push('\n');
        }
    }
    json_string
}

//fp read_file
pub fn read_file<P: AsRef<Path> + std::fmt::Display>(path: P) -> Result<String> {
    let file_text = std::fs::read_to_string(&path).map_err(|e| Error::from((path, e)))?;
    Ok(remove_comments(&file_text))
}

//fi json_error
fn json_error(reason: &str, json: &str, err: serde_json::Error) -> Error {
    let line = err.line();
    let column = err.column();
    let mut result =
        format!("Error in parsing {reason} json at line {line} column {column}: {err}",);
    let start_line = line.saturating_sub(5);
    let end_line = line + 5;
    for (i, l) in json.lines().enumerate() {
        if i >= start_line {
            result.push_str(&format!("\n{:4}: {}", i + 1, l));
        }
        if i >= end_line {
            break;
        }
    }
    Error::JsonCtxt(result)
}

//fp from_json
pub fn from_json<'de, P: Deserialize<'de>>(reason: &str, json: &'de str) -> Result<P> {
    serde_json::from_str(json).map_err(|e| json_error(reason, json, e))
}
