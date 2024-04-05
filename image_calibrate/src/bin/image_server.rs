//a Imports
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use clap::Command;
use image_calibrate::cmdline_args;
use image_calibrate::http::{
    HttpRequest, HttpRequestType, HttpResponse, HttpResponseType, HttpServer, HttpServerExt,
};
use image_calibrate::image;
use image_calibrate::image::Image;
use image_calibrate::json;
use image_calibrate::thread_pool::ThreadPool;
use image_calibrate::{Mesh, Project};

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
    pub fn ensure_loaded(&self) -> Result<MutexGuard<Option<ProjectWrap>>, String> {
        let mut p = self.project.lock().unwrap();
        if p.is_none() {
            let mut project = ProjectWrap::default();
            project.load(&self.path)?;
            *p = Some(project);
        }
        Ok(p)
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

//a ProjectDecode
//tp ProjectDecodeType
#[derive(Debug, Default)]
pub enum ProjectDecodeType {
    #[default]
    Root,
    UnknownProject,
    Project,
}

//tp ProjectDecode
#[derive(Debug, Default)]
pub struct ProjectDecode {
    dec_type: ProjectDecodeType,
    project: String,
    idx: usize,
    cip: Option<usize>,
    width: Option<f64>,
    height: Option<f64>,
}

//ip ProjectDecode
impl ProjectDecode {
    //cp decode_request
    pub fn decode_request(request: &HttpRequest) -> Option<Self> {
        let Some(path) = request.uri.path() else {
            return None;
        };
        if !path.starts_with("project") {
            return None;
        }
        let project = path.strip_prefix("project").unwrap();
        let Some(project) = project.to_str() else {
            return None;
        };
        let mut pd = ProjectDecode {
            dec_type: ProjectDecodeType::Root,
            ..Default::default()
        };
        if !project.is_empty() {
            pd.dec_type = ProjectDecodeType::UnknownProject;
            pd.project = project.into();
        }
        if let Some(Ok(cip)) = request.get_one::<usize>("cip") {
            pd.cip = Some(cip);
        }
        if let Some(Ok(width)) = request.get_one::<f64>("width") {
            pd.width = Some(width);
        }
        if let Some(Ok(height)) = request.get_one::<f64>("height") {
            pd.height = Some(height);
        }
        Some(pd)
    }

    //mp set_project_idx
    fn set_project_idx(&mut self, idx: Option<usize>) {
        if let Some(idx) = idx {
            self.dec_type = ProjectDecodeType::Project;
            self.idx = idx;
        } else {
            self.dec_type = ProjectDecodeType::UnknownProject;
            self.idx = 0;
        }
    }

    //ap is_root
    fn is_root(&self) -> bool {
        matches!(self.dec_type, ProjectDecodeType::Root)
    }

    //ap is_project
    fn is_project(&self) -> bool {
        matches!(self.dec_type, ProjectDecodeType::Project)
    }

    //ap might_be_project
    fn might_be_project(&self) -> bool {
        matches!(
            self.dec_type,
            ProjectDecodeType::Project | ProjectDecodeType::UnknownProject
        )
    }

    //ap project_idx
    fn project_idx(&self) -> Option<usize> {
        self.is_project().then_some(self.idx)
    }

    //ap project
    fn project(&self) -> Option<&str> {
        match self.dec_type {
            ProjectDecodeType::Project => Some(&self.project),
            ProjectDecodeType::UnknownProject => Some(&self.project),
            _ => None,
        }
    }
    //ap cip
    fn cip(&self) -> Option<usize> {
        self.cip
    }

    //zz All done
}

//a ProjectSet
//ti ProjectSet
#[derive(Debug, Default)]
struct ProjectSet {
    image_root: PathBuf,
    projects: Vec<NamedProject>,
    index_by_name: HashMap<String, usize>,
}

//ip ProjectSet
impl ProjectSet {
    //mp set_image_root
    pub fn set_image_root<I: Into<PathBuf>>(&mut self, image_root: I) {
        self.image_root = image_root.into();
    }

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
            if pb.extension().is_some_and(|x| x == "json")
                && pb
                    .file_stem()
                    .and_then(|x| x.to_str())
                    .is_some_and(|x| x.ends_with("_proj"))
            {
                self.add_project(pb.into_boxed_path())?;
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
        self.index_by_name.get(name).copied()
    }

    //mp decode_project
    pub fn decode_project(&self, request: &HttpRequest) -> Option<ProjectDecode> {
        let Some(mut pd) = ProjectDecode::decode_request(request) else {
            return None;
        };
        if pd.might_be_project() {
            let opt_idx = self.find_project(pd.project().unwrap());
            pd.set_project_idx(opt_idx);
        }
        Some(pd)
    }

    //mi http_list_projects
    fn http_list_projects(
        &self,
        server: &HttpServer<Self>,
        _request: &HttpRequest,
        _content: &[u8],
        response: &mut HttpResponse,
    ) -> Result<(), String> {
        let names: Vec<String> = self.index_by_name.keys().cloned().collect();
        let json = serde_json::to_string(&names).unwrap();
        response.content = json.into_bytes();
        response.mime_type = server.mime_type("json");
        response.resp_type = HttpResponseType::FileRead;
        Ok(())
    }

    //mi http_load_project
    fn http_load_project(
        &self,
        server: &HttpServer<Self>,
        _request: &HttpRequest,
        _content: &[u8],
        response: &mut HttpResponse,
        idx: usize,
    ) -> Result<(), String> {
        self.projects[idx]
            .ensure_loaded()
            .map(|_x| ())
            .and_then(|_| self.projects[idx].map(|p| p.to_json(false)).unwrap())
            .map(|json| {
                response.content = json.into_bytes();
                response.mime_type = server.mime_type("json");
                response.resp_type = HttpResponseType::FileRead;
            })
    }

    //mi http_save_project
    fn http_save_project(
        &self,
        _server: &HttpServer<Self>,
        _request: &HttpRequest,
        content: &[u8],
        response: &mut HttpResponse,
        idx: usize,
    ) -> Result<(), String> {
        let mut str_content = "";
        let mut e = match std::str::from_utf8(content) {
            Ok(c) => {
                str_content = c;
                None
            }
            Err(_e) => Some("Bad UTF8 in JSon".to_string()),
        };
        if e.is_none() {
            e = self.projects[idx].of_json(str_content).err();
        }
        if e.is_none() {
            e = self.projects[idx].save().unwrap().err();
        }
        if let Some(e) = e {
            Err(format!("Failed to save project {idx} with json {e}:"))
        } else {
            response.resp_type = HttpResponseType::FileRead;
            Ok(())
        }
    }

    //mi http_cip_pms_mesh
    fn http_cip_pms_mesh(
        &self,
        server: &HttpServer<Self>,
        _request: &HttpRequest,
        _content: &[u8],
        response: &mut HttpResponse,
        pd: &ProjectDecode,
    ) -> Result<(), String> {
        let cip = pd.cip().unwrap_or_default();
        let p = self.projects[pd.idx].ensure_loaded()?;
        let p = &p.as_ref().unwrap().0;
        if cip >= p.ncips() {
            return Err("Cip out of range".into());
        }
        let cip = p.cip(cip).clone();
        let cip_r = cip.borrow();
        let pms = cip_r.pms();
        let mesh = Mesh::optimized(&pms.borrow());
        let triangles: Vec<_> = mesh.triangles().collect();
        eprintln!("Triangles of mesh {triangles:?}");
        let json = serde_json::to_string(&triangles).unwrap();
        eprintln!("Json of mesh {json}");
        response.content = json.into_bytes();
        response.mime_type = server.mime_type("json");
        response.resp_type = HttpResponseType::FileRead;
        Ok(())
    }

    //mi http_cip_thumbnail
    fn http_cip_thumbnail(
        &self,
        server: &HttpServer<Self>,
        _request: &HttpRequest,
        _content: &[u8],
        response: &mut HttpResponse,
        pd: &ProjectDecode,
    ) -> Result<(), String> {
        let cip = pd.cip().unwrap_or_default();
        let p = self.projects[pd.idx].ensure_loaded()?;
        let p = &p.as_ref().unwrap().0;
        if cip >= p.ncips() {
            return Err("Cip out of range".into());
        }
        let cip = p.cip(cip).clone();
        let cip_r = cip.borrow();
        let path = self.image_root.as_path().join(cip_r.image());
        server.verbose().then(|| eprintln!("Open image {path:?}"));
        let src_img = image::read_image(&path)?;
        let src_size = src_img.size();
        let src_size = (src_size.0 as f64, src_size.1 as f64);
        let x_scale = pd.width.map(|w| src_size.0 / w).unwrap_or(1.0);
        let y_scale = pd.height.map(|h| src_size.1 / h).unwrap_or(1.0);
        let scale = x_scale.max(y_scale);
        let width = (src_size.0 / scale) as usize;
        let height = (src_size.1 / scale) as usize;
        let mut scaled_img = image::read_or_create_image(width, height, None).unwrap();
        for y in 0..height {
            let sy = (y as f64 + 0.5) * scale;
            for x in 0..width {
                let sx = (x as f64 + 0.5) * scale;
                let c = src_img.get(sx as u32, sy as u32);
                scaled_img.put(x as u32, y as u32, &c);
            }
        }
        let img_bytes = scaled_img.encode("jpeg")?;
        response.content = img_bytes;
        response.mime_type = server.mime_type("jpeg");
        response.resp_type = HttpResponseType::FileRead;
        Ok(())
    }

    //zz All done
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
        let Some(pd) = self.decode_project(request) else {
            return false;
        };
        server.verbose().then(|| {
            eprintln!("ImageServer: {request:?}");
            eprintln!("    Decoded: {pd:?}");
        });
        let result = {
            if pd.is_root() {
                if request.action_is("list") && request.req_type == HttpRequestType::Get {
                    self.http_list_projects(server, request, content, response)
                } else {
                    Err("Unknown project action".into())
                }
            } else if let Some(idx) = pd.project_idx() {
                if request.action_is("load") && request.req_type == HttpRequestType::Get {
                    self.http_load_project(server, request, content, response, idx)
                } else if request.action_is("save") && request.req_type == HttpRequestType::Put {
                    self.http_save_project(server, request, content, response, idx)
                } else if request.action_is("mesh") && request.req_type == HttpRequestType::Get {
                    self.http_cip_pms_mesh(server, request, content, response, &pd)
                } else if request.action_is("thumbnail") && request.req_type == HttpRequestType::Get
                {
                    self.http_cip_thumbnail(server, request, content, response, &pd)
                } else {
                    Err("Bad request type".into())
                }
            } else {
                Err(format!("Failed to find project {}", pd.project().unwrap()))
            }
        };
        match result {
            Err(e) => {
                eprintln!("Failed to handle request: {e}\n  {pd:?}");
                false
            }
            _ => true,
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
    let cmd = cmdline_args::add_verbose_arg(cmd);
    let cmd = cmdline_args::add_threads_arg(cmd);
    let cmd = cmdline_args::add_port_arg(cmd);
    let cmd = cmdline_args::add_file_root_arg(cmd, true);
    let cmd = cmdline_args::add_image_root_arg(cmd, true);
    let cmd = cmdline_args::add_project_root_arg(cmd, true);

    let matches = cmd.get_matches();
    let verbose = cmdline_args::get_verbose(&matches);
    let num_threads = cmdline_args::get_threads(&matches)?;
    let port = cmdline_args::get_port(&matches)?;
    let file_root = cmdline_args::get_file_root(&matches)?;
    let image_root = cmdline_args::get_image_root(&matches)?;
    let project_root = cmdline_args::get_project_root(&matches)?;
    if num_threads == 0 || num_threads > 20 {
        return Err(format!(
            "Number of threads {num_threads} must be non-zero and no more than 20"
        ));
    }
    if !(1024..=60000).contains(&port) {
        return Err(format!("Port {port} must be in the range 1024..60000"));
    }

    let mut project_set = ProjectSet::default();
    project_set.set_image_root(image_root);
    project_set.fill_from_project_dir(project_root)?;
    HTTP_SRV
        .set(HttpServer::new(verbose, file_root, project_set))
        .map_err(|_| "Bug - faiiled to config server")?;

    let pool = ThreadPool::new(4);
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
        .map_err(|_a| (format!("Failed to bind to port {port}")))?;
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.issue_work(|| {
            let http_srv = HTTP_SRV.get().unwrap();
            http_srv.handle_connection(stream);
        });
    }
    Ok(())
}
