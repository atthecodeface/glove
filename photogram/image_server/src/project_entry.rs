//a Imports
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use ic_base::json;
use ic_project::Project;

//a ProjectPath
//tp ProjectPath
#[derive(Debug)]
pub struct ProjectPath(Box<Path>);

//ip Display for ProjectPath
impl std::fmt::Display for ProjectPath {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        std::fmt::Debug::fmt(&self.0, fmt)
    }
}
//ip AsRef<Path> for ProjectPath {
impl std::convert::AsRef<Path> for ProjectPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
//ip Deref for ProjectPath {
impl std::ops::Deref for ProjectPath {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.0
    }
}

//a ProjectWrap
//ti ProjectWrap
#[derive(Debug, Default)]
struct ProjectWrap(Project);

//ip Send for ProjectWrap
unsafe impl Send for ProjectWrap {}

//ii ProjectWrap
impl ProjectWrap {
    //mp of_json
    /// Set to be a project from some Json
    fn of_json(&mut self, project_json: &str) -> Result<(), String> {
        self.0 = json::from_json("project", project_json)?;
        Ok(())
    }

    //mp load
    /// Load the project from a path - it drops the old project
    fn load<P: AsRef<Path> + std::fmt::Display>(&mut self, path: P) -> Result<(), String> {
        let project_json = json::read_file(path)?;
        self.of_json(&project_json)
    }

    //mp save
    /// Save the project to a path
    fn save<P: AsRef<Path> + std::fmt::Display>(&self, path: P) -> Result<(), String> {
        let mut f = File::create(&path).map_err(|e| format!("Failed to open file {path}: {e}"))?;
        let json = self.0.to_json(true)?;
        f.write(json.as_bytes())
            .map_err(|e| format!("Failed to write Json to {path}: {e}"))?;
        Ok(())
    }

    //zz All done
}

impl std::convert::AsRef<Project> for ProjectWrap {
    fn as_ref(&self) -> &Project {
        &self.0
    }
}

//a UniqueProjectRef
#[derive(Debug)]
pub struct UniqueProjectRef<'r> {
    mg_project: MutexGuard<'r, Option<ProjectWrap>>,
}
impl<'a> UniqueProjectRef<'a> {
    pub fn as_ref(&self) -> &Project {
        self.mg_project.as_ref().unwrap().as_ref()
    }
}

//a NamedProject
//tp NamedProject
/// A project with a name; it is Sync and Send
///
/// Only a single operation can occur on the project at any time,
/// through a mutex inside
#[derive(Debug)]
pub struct NamedProject {
    name: String,
    path: ProjectPath,
    project: Mutex<Option<ProjectWrap>>,
}

//ii NamedProject
impl NamedProject {
    //ap name
    pub fn name(&self) -> &str {
        &self.name
    }

    //cp new
    /// Create a new [NamedProject] given a path
    ///
    /// The project is not loaded by default
    pub fn new(path: Box<Path>) -> Result<Self, String> {
        let path = ProjectPath(path);
        let Some(name) = path.file_stem() else {
            return Err(format!("Could not get name of file from path {path}"));
        };
        let name = name.to_string_lossy().to_string();
        let project = None.into();
        Ok(Self {
            name,
            path,
            project,
        })
    }

    //ap is_mapped
    #[allow(dead_code)]
    pub fn is_mapped(&self) -> bool {
        self.project.lock().unwrap().is_some()
    }

    //ap ensure_loaded
    pub fn ensure_loaded(&self) -> Result<UniqueProjectRef, String> {
        let mut p = self.project.lock().unwrap();
        if p.is_none() {
            let mut project = ProjectWrap::default();
            project.load(&self.path)?;
            *p = Some(project);
        }
        Ok(UniqueProjectRef { mg_project: p })
    }

    //mp of_json
    pub fn of_json(&self, json: &str) -> Result<(), String> {
        let mut p = self.project.lock().unwrap();
        let mut project = ProjectWrap::default();
        project.of_json(json)?;
        *p = Some(project);
        Ok(())
    }

    //mp map
    /// Apply a function to the enclosed project, if it has been loaded
    pub fn map<R, F>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&Project) -> R,
    {
        let opt_project = self.project.lock().unwrap();
        opt_project.as_ref().map(|p| f(&p.0))
    }

    //mp save
    /// Save the Project back to its Path, if it has been loaded
    pub fn save(&self) -> Option<Result<(), String>> {
        let opt_project = self.project.lock().unwrap();
        opt_project.as_ref().map(|p| p.save(&self.path))
    }

    //mp load
    /// Load the Project from to its Path; this creates it
    #[allow(dead_code)]
    pub fn load(&self) -> Option<Result<(), String>> {
        let mut opt_project = self.project.lock().unwrap();
        opt_project.as_mut().map(|p| p.load(&self.path))
    }
}
