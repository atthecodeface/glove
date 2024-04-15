//a Imports
use std::collections::HashMap;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use clap::Command;

use ic_base::Mesh;
// use ic_cache::{Cache, CacheEntry, Cacheable};
use ic_cmdline as cmdline_args;
use ic_http::{
    HttpRequest, HttpRequestType, HttpResponse, HttpResponseType, HttpServer, HttpServerExt,
};
use ic_image::{Image, ImageGray16, ImageRgb8, Patch};
use ic_kernel::{KernelArgs, Kernels};
use ic_threads::ThreadPool;

mod project_decode;
mod project_entry;

use project_decode::ProjectDecode;
use project_entry::NamedProject;
mod image_cache;
use image_cache::{ImageCache, ImageCacheEntry};

//a ProjectSet
//ti ProjectSet
#[derive(Debug)]
struct ProjectSet {
    image_root: PathBuf,
    projects: Vec<NamedProject>,
    index_by_name: HashMap<String, usize>,
    kernels: Kernels,
    image_cache: ImageCache,
}

//ip ProjectSet
impl ProjectSet {
    fn new() -> Self {
        let kernels = Kernels::new();
        let image_root = "".into();
        let projects = vec![];
        let index_by_name = HashMap::new();
        let image_cache = ImageCache::new();
        Self {
            image_root,
            projects,
            index_by_name,
            kernels,
            image_cache,
        }
    }

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
        let mut pd = ProjectDecode::decode_request(request)?;
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
        let up = self.projects[pd.idx].ensure_loaded()?;
        let p = up.as_ref();
        if cip >= p.ncips() {
            return Err("Cip out of range".into());
        }
        let cip = p.cip(cip).clone();
        let cip_r = cip.borrow();
        let pms = cip_r.pms();
        let mesh = Mesh::optimized(pms.borrow().mappings().iter().map(|p| p.screen()));
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
        let up = self.projects[pd.idx].ensure_loaded()?;
        let p = up.as_ref();
        if cip >= p.ncips() {
            return Err("Cip out of range".into());
        }
        let cip = p.cip(cip).clone();
        let cip_r = cip.borrow();
        let path = self.image_root.as_path().join(cip_r.image());
        server.verbose().then(|| eprintln!("Open image {path:?}"));

        let src_img_ref = self.image_cache.src_image(&path)?;
        let src_img = ImageCacheEntry::cr_as_rgb8(&src_img_ref);

        let src_size = src_img.size();
        let src_size = (src_size.0 as f64, src_size.1 as f64);
        let x_scale = pd.width.map(|w| src_size.0 / w).unwrap_or(1.0);
        let y_scale = pd.height.map(|h| src_size.1 / h).unwrap_or(1.0);
        let scale = x_scale.max(y_scale);
        let width = (src_size.0 / scale) as usize;
        let height = (src_size.1 / scale) as usize;
        let mut scaled_img = ImageRgb8::read_or_create_image(width, height, None).unwrap();
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

    //mi http_cip_patch
    fn http_cip_patch(
        &self,
        server: &HttpServer<Self>,
        _request: &HttpRequest,
        _content: &[u8],
        response: &mut HttpResponse,
        pd: &ProjectDecode,
    ) -> Result<(), String> {
        let cip = pd.cip().unwrap_or_default();
        let up = self.projects[pd.idx].ensure_loaded()?;
        let p = up.as_ref();
        if cip >= p.ncips() {
            return Err("Cip out of range".into());
        }
        let cip = p.cip(cip).clone();
        let cip_r = cip.borrow();
        let path = self.image_root.as_path().join(cip_r.image());

        let src_img_ref = self.image_cache.src_image(&path)?;
        let src_img = ImageCacheEntry::cr_as_rgb8(&src_img_ref);
        // let src_img = ImageRgb8::read_image(path)?;

        let nps = p.nps_ref();
        let camera = cip_r.camera_ref();

        let mut model_pts = vec![];
        for name in &pd.nps {
            if let Some(n) = nps.get_pt(name) {
                let model = n.model().0;
                model_pts.push((name, model, camera.map_model(model)))
            } else {
                return Err(format!("Could not find NP {name} in the project"));
            }
        }
        if model_pts.len() < 3 {
            return Err(format!(
                "Need at least 3 points for a patch, got {}",
                model_pts.len()
            ));
        }

        for m in &model_pts {
            eprintln!("{} {} {}", m.0, m.1, m.2);
        }
        let model_pts: Vec<_> = model_pts.into_iter().map(|(_, m, _)| m).collect();

        let px_per_model = pd.px_per_model.unwrap_or(10.0);
        let Some(patch) =
            Patch::create(src_img, px_per_model, &model_pts, &|m| camera.map_model(m))?
        else {
            return Err("Failled to create patch".into());
        };

        let to_width = pd.width.map(|x| x as usize).unwrap_or(200);
        let ws = pd.window.unwrap_or(4) as u32;
        let img = patch.img();
        let (w, h, mut img_data) = img.as_vec_gray_f32(Some(to_width));
        let mut img_data_sq = img_data.clone();
        let args: KernelArgs = (w, h).into();

        // sum(x)^2 - sum(x^2)

        let args = args.with_size(ws as usize);
        let ws_f = ws as f32;
        let args_mean = args.with_scale(1.0 / ws_f);
        self.kernels
            .run_shader("square", &args, w * h, None, img_data_sq.as_mut_slice())?;
        self.kernels.run_shader(
            "window_sum_x",
            &args_mean,
            w * h,
            None,
            img_data_sq.as_mut_slice(),
        )?;
        self.kernels.run_shader(
            "window_sum_y",
            &args_mean,
            w * h,
            None,
            img_data_sq.as_mut_slice(),
        )?;
        self.kernels.run_shader(
            "window_sum_x",
            &args_mean,
            w * h,
            None,
            img_data.as_mut_slice(),
        )?;
        self.kernels.run_shader(
            "window_sum_y",
            &args_mean,
            w * h,
            None,
            img_data.as_mut_slice(),
        )?;
        self.kernels
            .run_shader("square", &args, w * h, None, img_data.as_mut_slice())?;

        self.kernels.run_shader(
            "sub_scaled",
            &args,
            w * h,
            Some(img_data.as_slice()),
            img_data_sq.as_mut_slice(),
        )?;
        self.kernels.run_shader(
            "sqrt",
            &args.with_scale(2.0),
            w * h,
            None,
            img_data_sq.as_mut_slice(),
        )?;

        // minus
        // square sum sum
        let img = ImageGray16::of_vec_f32(w, h, img_data_sq, 1.0);
        let img_bytes = img.encode("png")?;
        response.content = img_bytes;
        response.mime_type = server.mime_type("png");
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
                } else if request.action_is("patch") && request.req_type == HttpRequestType::Get {
                    self.http_cip_patch(server, request, content, response, &pd)
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
    let cmd = cmdline_args::threads::add_threads_arg(cmd);
    let cmd = cmdline_args::threads::add_port_arg(cmd);
    let cmd = cmdline_args::file_system::add_file_root_arg(cmd, true);
    let cmd = cmdline_args::file_system::add_image_root_arg(cmd, true);
    let cmd = cmdline_args::file_system::add_project_root_arg(cmd, true);

    let matches = cmd.get_matches();
    let verbose = cmdline_args::get_verbose(&matches);
    let num_threads = cmdline_args::threads::get_threads(&matches)?;
    let port = cmdline_args::threads::get_port(&matches)?;
    let file_root = cmdline_args::file_system::get_file_root(&matches)?;
    let image_root = cmdline_args::file_system::get_image_root(&matches)?;
    let project_root = cmdline_args::file_system::get_project_root(&matches)?;
    if num_threads == 0 || num_threads > 20 {
        return Err(format!(
            "Number of threads {num_threads} must be non-zero and no more than 20"
        ));
    }
    if !(1024..=60000).contains(&port) {
        return Err(format!("Port {port} must be in the range 1024..60000"));
    }

    let mut project_set = ProjectSet::new();
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
