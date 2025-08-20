//a Imports
use std::path::{Path, PathBuf};

use crate::Result;

//a PathSet
//tp PathGlob
#[derive(Default, Clone, Copy)]
pub enum PathGlob {
    #[default]
    Skip,
    Add,
    Push,
    PushAdd,
}
impl PathGlob {
    pub fn is_push(self) -> bool {
        matches!(self, PathGlob::Push | PathGlob::PushAdd)
    }
    pub fn is_add(self) -> bool {
        matches!(self, PathGlob::Add | PathGlob::PushAdd)
    }
    pub fn is_skip(self) -> bool {
        matches!(self, PathGlob::Skip)
    }
    pub fn push(self) -> Self {
        match self {
            PathGlob::Skip => PathGlob::Push,
            PathGlob::Add => PathGlob::PushAdd,
            _ => panic!("Cannot add 'push' to a PathGlob that already has push"),
        }
    }
    pub fn add(self) -> Self {
        match self {
            PathGlob::Skip => PathGlob::Add,
            PathGlob::Push => PathGlob::PushAdd,
            _ => panic!("Cannot add 'add' to a PathGlob that already has add"),
        }
    }
}

//tp PathSet
#[derive(Default, Debug, Clone)]
pub struct PathSet {
    paths: Vec<PathBuf>,
}

//ip PathSet
impl PathSet {
    //mp glob_path
    pub fn glob_path<D, F>(
        &self,
        mut paths: Vec<PathBuf>,
        max: usize,
        max_depth: usize,
        dir_filter: &D,
        file_filter: &F,
        path: &Path,
    ) -> Vec<PathBuf>
    where
        F: Fn(&Path) -> bool,
        D: Fn(&Path) -> PathGlob,
    {
        if paths.len() >= max {
            return paths;
        }
        if path.is_dir() {
            let d_ops = dir_filter(path);
            if d_ops.is_add() {
                paths.push(path.into());
            }
            if max_depth == 0 || paths.len() >= max {
                return paths;
            }
            if d_ops.is_push() {
                let Ok(contents) = std::fs::read_dir(path) else {
                    return paths;
                };
                for p in contents {
                    match p {
                        Ok(p) => {
                            paths = self.glob_path(
                                paths,
                                max,
                                max_depth - 1,
                                dir_filter,
                                file_filter,
                                &p.path(),
                            );
                        }
                        _ => (),
                    }
                }
            }
        } else if path.is_file() {
            if file_filter(path) {
                paths.push(path.into());
            }
        }
        paths
    }

    //mp glob
    pub fn glob<D, F>(
        &self,
        max: usize,
        max_depth: usize,
        dir_filter: &D,
        file_filter: &F,
    ) -> Vec<PathBuf>
    where
        F: Fn(&Path) -> bool,
        D: Fn(&Path) -> PathGlob,
    {
        let mut paths = vec![];
        for p in self.paths.iter() {
            paths = self.glob_path(paths, max, max_depth, dir_filter, file_filter, p);
        }
        paths
    }

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
    pub fn find_file<P: AsRef<Path>>(&self, path: P) -> Option<PathBuf> {
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
    pub fn find_file_err<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        if let Some(path) = self.find_file(&path) {
            Ok(path)
        } else {
            let path = path.as_ref().to_string_lossy();
            Err(format!("Failed to find '{path}' on the search path").into())
        }
    }
}
