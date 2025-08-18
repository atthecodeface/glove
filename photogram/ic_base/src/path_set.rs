//a Imports
use std::path::{Path, PathBuf};


use crate::Result;

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

    //mp find_file_err
    pub fn find_file_err<P: AsRef<Path> + std::fmt::Display>(&self, path: P) -> Result<PathBuf> {
        if let Some(path) = self.find_file(&path) {
            Ok(path)
        } else {
            Err(format!("Failed to find '{path}' on the search path").into())
        }
    }
}
