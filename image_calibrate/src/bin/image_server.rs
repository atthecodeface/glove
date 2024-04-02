//a Imports
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

use clap::Command;
use image_calibrate::cmdline_args;
use image_calibrate::http::{
    HttpRequest, HttpRequestType, HttpResponse, HttpResponseType, HttpServer, HttpServerExt,
};
use image_calibrate::json;
use image_calibrate::thread_pool::ThreadPool;
use image_calibrate::Project;

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
//ip Send for ProjectWrap
unsafe impl Send for ProjectWrap {}

//ti ProjectWrap
#[derive(Debug, Default)]
struct ProjectWrap(Project);

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

//a NamedProject
//ti NamedProject
#[derive(Debug)]
struct NamedProject {
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
    pub fn is_mapped(&self) -> bool {
        self.project.lock().unwrap().is_some()
    }

    //ap ensure_loaded
    pub fn ensure_loaded(&self) -> Result<(), String> {
        let mut p = self.project.lock().unwrap();
        if p.is_none() {
            let mut project = ProjectWrap::default();
            project.load(&self.path)?;
            *p = Some(project);
        }
        Ok(())
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
    pub fn load(&self) -> Option<Result<(), String>> {
        let mut opt_project = self.project.lock().unwrap();
        opt_project.as_mut().map(|p| p.load(&self.path))
    }
}

//a ProjectSet
//ti ProjectSet
#[derive(Debug, Default)]
struct ProjectSet {
    projects: Vec<NamedProject>,
    index_by_name: HashMap<String, usize>,
}

//tp ProjectDecodeType
#[derive(Debug)]
pub enum ProjectDecodeType {
    Failed,
    Root,
    UnknownProject,
    Project,
}

//tp ProjectDecode
#[derive(Debug)]
pub struct ProjectDecode<'a> {
    dec_type: ProjectDecodeType,
    project: &'a str,
    idx: usize,
    cip: Option<usize>,
    action: Option<&'a str>,
    args: Vec<(&'a str, Option<&'a str>)>,
}

//ip ProjectDecode
impl<'a> ProjectDecode<'a> {
    //cp new
    fn new() -> Self {
        Self {
            dec_type: ProjectDecodeType::Failed,
            project: "",
            idx: 0,
            cip: None,
            action: None,
            args: vec![],
        }
    }
    //cp failed
    fn failed() -> Self {
        let mut pd = Self::new();
        pd.dec_type = ProjectDecodeType::Failed;
        pd
    }
    //cp root
    fn root() -> Self {
        let mut pd = Self::new();
        pd.dec_type = ProjectDecodeType::Root;
        pd
    }
    //cp of_project
    fn of_project<'c>(project: &'c str, idx: Option<usize>) -> ProjectDecode<'c> {
        let mut pd = ProjectDecode::new();
        pd.project = project;
        if let Some(idx) = idx {
            pd.dec_type = ProjectDecodeType::Project;
            pd.idx = idx;
        } else {
            pd.dec_type = ProjectDecodeType::UnknownProject;
        }
        pd
    }
    //mp set_action
    pub fn set_action(&mut self, action: Option<&'a str>) {
        self.action = action;
    }

    //mp add_arg
    pub fn add_arg(&mut self, arg: &'a str, value: Option<&'a str>) {
        self.args.push((arg, value));
    }

    //ap is_root
    fn is_root(&self) -> bool {
        matches!(self.dec_type, ProjectDecodeType::Root)
    }
    //ap is_failed
    fn is_failed(&self) -> bool {
        matches!(self.dec_type, ProjectDecodeType::Failed)
    }
    //ap is_project
    fn is_project(&self) -> bool {
        matches!(self.dec_type, ProjectDecodeType::Project)
    }
    //ap project_idx
    fn project_idx(&self) -> Option<usize> {
        self.is_project().then_some(self.idx)
    }
    //ap project
    fn project(&self) -> Option<&str> {
        match self.dec_type {
            ProjectDecodeType::Project => Some(self.project),
            ProjectDecodeType::UnknownProject => Some(self.project),
            _ => None,
        }
    }
}

//ip ProjectSet
impl ProjectSet {
    //mp fill_from_project_dir
    pub fn fill_from_project_dir<P: AsRef<Path> + std::fmt::Display>(
        &mut self,
        path: P,
    ) -> Result<(), String> {
        for d in path
            .as_ref()
            .read_dir()
            .map_err(|e| format!("Failed to read directory {path}: {e}"))?
        {
            if d.is_err() {
                continue;
            }
            let d = d.unwrap();
            let Ok(ft) = d.file_type() else {
                continue;
            };
            if !ft.is_file() {
                continue;
            }
            let pb = d.path();
            if pb.extension().is_some_and(|x| x == "json") {
                if pb
                    .file_stem()
                    .and_then(|x| x.to_str())
                    .is_some_and(|x| x.ends_with("_proj"))
                {
                    self.add_project(pb.into_boxed_path());
                }
            }
        }
        Ok(())
    }

    //mp add_project
    pub fn add_project(&mut self, path: Box<Path>) -> Result<(), String> {
        let named_project = NamedProject::new(path)?;
        let n = self.projects.len();
        self.index_by_name
            .insert(named_project.name().to_string(), n);
        self.projects.push(named_project);
        Ok(())
    }

    //mp find_project
    pub fn find_project(&self, name: &str) -> Option<usize> {
        self.index_by_name.get(name).as_deref().copied()
    }

    //mp decode_project
    pub fn decode_project<'req>(&self, request: &'req HttpRequest) -> ProjectDecode<'req> {
        if !request.uri.starts_with("/project") {
            return ProjectDecode::failed();
        }
        let project = request.uri.strip_prefix("/project").unwrap();
        if project.len() > 0 && project.as_bytes()[0] != b'/' && project.as_bytes()[0] != b'?' {
            return ProjectDecode::failed();
        }
        if project.len() < 2 {
            return ProjectDecode::root();
        }
        let project = {
            if project.as_bytes()[0] == b'/' {
                &project[1..]
            } else {
                project
            }
        };
        // Look for ? action [& k=v]*
        let mut split = project.splitn(2, '?');
        let project = split.next().unwrap();
        let mut pd = ProjectDecode::of_project(project, self.find_project(project));
        if let Some(action_args) = split.next() {
            let mut aa_split = action_args.split('&');
            pd.set_action(aa_split.next());
            while let Some(args) = aa_split.next() {
                let mut arg_split = args.splitn(2, '=');
                let arg = arg_split.next().unwrap();
                pd.add_arg(arg, arg_split.next());
            }
        }
        pd
    }
}

//ip HttpServerExt for ProjectSet
impl HttpServerExt for ProjectSet {
    //mp set_http_response
    fn set_http_response(
        &self,
        server: &HttpServer<Self>,
        request: &HttpRequest,
        content: &[u8],
        response: &mut HttpResponse,
    ) -> bool {
        let pd = self.decode_project(request);
        eprintln!("ImageServer: {request:?}");
        eprintln!("    Decoded: {pd:?}");
        if pd.is_failed() {
            return false;
        };
        if request.req_type == HttpRequestType::Get {
            if pd.is_root() {
                let names: Vec<String> = self.index_by_name.keys().cloned().collect();
                let json = serde_json::to_string(&names).unwrap();
                response.content = json.into_bytes();
                response.mime_type = server.mime_type("json");
                response.resp_type = HttpResponseType::FileRead;
                true
            } else if let Some(idx) = pd.project_idx() {
                if let Err(e) = self.projects[idx].ensure_loaded() {
                    eprintln!("Failed to ensure project {idx} loaded {e}:");
                    return false;
                }
                match self.projects[idx].map(|p| p.to_json(false)).unwrap() {
                    Ok(json) => {
                        response.content = json.into_bytes();
                        response.mime_type = server.mime_type("json");
                        response.resp_type = HttpResponseType::FileRead;
                        true
                    }
                    Err(e) => {
                        eprintln!("Failed to create Json {e}:");
                        false
                    }
                }
            } else {
                eprintln!("Failed to find project {}", pd.project().unwrap());
                false
            } /*        } else if request.req_type == HttpRequestType::Post {
                          eprintln!("ImageServer: Post content {content:?}");
                          let wrp = self.project.lock().unwrap();
                          let json = wrp.0.to_json(false).unwrap();
                          response.content = json.into_bytes();
                          response.mime_type = server.mime_type("json");
                          response.resp_type = HttpResponseType::FileRead;
                          true
              */
        } else if request.req_type == HttpRequestType::Put {
            if let Some(idx) = pd.project_idx() {
                let mut str_content = "";
                let mut e = match std::str::from_utf8(content) {
                    Ok(c) => {
                        str_content = c;
                        None
                    }
                    Err(e) => Some("Bad UTF8 in JSon".to_string()),
                };
                if e.is_none() {
                    e = self.projects[idx].of_json(str_content).err();
                }
                if e.is_none() {
                    e = self.projects[idx].save().unwrap().err();
                }
                if let Some(e) = e {
                    eprintln!("Failed to save project {idx} with json {e}:");
                    return false;
                }
                response.resp_type = HttpResponseType::FileRead;
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

//a Main
//si HTTP_SRV
/// This is the configuration of the http server; it is set *once* in main before threads are created
///
/// One instance of this is created with a [OnceLock]
static HTTP_SRV: OnceLock<HttpServer<ProjectSet>> = OnceLock::new();

//fp main
fn main() -> Result<(), String> {
    let cmd = Command::new("image_server")
        .about("Image calibration/correlation server")
        .version("0.1.0");
    let cmd = cmdline_args::add_threads_arg(cmd);
    let cmd = cmdline_args::add_port_arg(cmd);
    let cmd = cmdline_args::add_file_root_arg(cmd, true);

    let matches = cmd.get_matches();
    let num_threads = cmdline_args::get_threads(&matches)?;
    let port = cmdline_args::get_port(&matches)?;
    let file_root = cmdline_args::get_file_root(&matches)?;
    if num_threads == 0 || num_threads > 20 {
        return Err(format!(
            "Number of threads must be non-zero and no more than 20"
        ));
    }
    if port < 1024 || port > 60000 {
        return Err(format!("Port must be in the range 1024..60000"));
    }

    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
        .map_err(|_a| (format!("Failed to bind to port {port}")))?;
    let pool = ThreadPool::new(4);

    let mut project_set = ProjectSet::default();
    project_set.fill_from_project_dir("/Users/gavinjstark/Git/glove/image_calibrate/nac");
    HTTP_SRV
        .set(HttpServer::new(file_root, project_set))
        .map_err(|_| "Bug - faiiled to config server")?;
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.issue_work(|| {
            let http_srv = HTTP_SRV.get().unwrap();
            http_srv.handle_connection(stream);
        });
    }
    Ok(())
}
