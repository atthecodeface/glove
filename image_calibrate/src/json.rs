//a Imports
use std::path::Path;

use serde::Deserialize;

//a Public functions
//fp read_file
pub fn read_file<P: AsRef<Path> + std::fmt::Display>(path: P) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("Error reading json file {}: {}", path, e))
}

//fp from_json
pub fn from_json<'de, P: Deserialize<'de>>(reason: &str, json: &'de str) -> Result<P, String> {
    serde_json::from_str(json).map_err(|e| {
        format!(
            "Error in parsing {} json at line {} column {}: {}",
            reason,
            e.line(),
            e.column(),
            e
        )
    })
}
