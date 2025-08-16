//a Imports
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::json::json_error;
use crate::json::remove_comments;
use crate::{Error, Result};

//a PathSet
//tp PathSet
#[derive(Default, Debug, Clone)]
pub struct PathSet {
    paths: Vec<PathBuf>,
}

//ip PathSet
impl PathSet {
    //mp add_path
    pub fn add_path<P: AsRef<Path> + std::fmt::Display>(&mut self, path: P) -> Result<()> {
        if !path.as_ref().exists() {
            Err(format!("Path {path} cannot be added to seach chain as it does not exist").into())
        } else {
            self.paths.push(path.as_ref().into());
            Ok(())
        }
    }

    //mp find_file
    pub fn find_file<P: AsRef<Path> + std::fmt::Display>(&self, path: P) -> Option<PathBuf> {
        if path.as_ref().exists() {
            Some(path.as_ref().into())
        } else {
            for p in &self.paths {
                let try_path = p.join(path.as_ref());
                if try_path.exists() {
                    return Some(try_path);
                }
            }
            None
        }
    }

    //fp read_json_file
    pub fn read_json_file<P: AsRef<Path> + std::fmt::Display>(&self, path: P) -> Result<String> {
        if let Some(path) = self.find_file(&path) {
            let file_text = std::fs::read_to_string(&path)?;
            Ok(remove_comments(&file_text))
        } else {
            Err(format!("Failed to find '{path}' on the search path").into())
        }
    }

    //mp load_from_json_file
    pub fn load_from_json_file<P: AsRef<Path> + std::fmt::Display, T: DeserializeOwned>(
        &self,
        reason: &str,
        path: P,
    ) -> Result<T> {
        let json = self
            .read_json_file(&path)
            .map_err(|e| (e, reason.to_owned()))?;
        serde_json::from_str(&json).map_err(|e| json_error(&format!("{reason} '{path}'"), &json, e))
    }
}
