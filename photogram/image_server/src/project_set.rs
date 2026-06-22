//a Imports
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ic_base::{PathGlob, Result};
use ic_http::{
    HttpRequest, HttpRequestType, HttpResponse, HttpResponseType, HttpServer, HttpServerExt,
};
use ic_image::{Image, ImageDrawable, ImageGray16, ImageRgb8};
use ic_kernel::{KernelArgs, Kernels};
use ic_mapping::Patch;
use ic_mesh::Mesh;

use crate::CmdArgs;
use crate::NamedProject;
use crate::ProjectDecode;
use crate::{ImageCache, ImageCacheEntry};

//a ProjectSet
//tp ProjectSet
/// The ProjectSet is created once, and is owned by the HTTP server
///
/// It has access to the command line arguments through an Arc<RwLock<args>>
#[derive(Debug)]
pub struct ProjectSet {
    /// CmdArgs is an Arc<RwLock<args>> to permit access to arguments after the threads kick in
    cmd_args: CmdArgs,
    /// projects is filled before multiple threads kick in, so is read-only thereafter
    projects: Vec<NamedProject>,
    /// index_by_name is filled before multiple threads kick in, so is read-only thereafter
    index_by_name: HashMap<String, usize>,
    kernels: Kernels,
    /// ImageCache is a Mutex<Cache<>>
    image_cache: ImageCache,
}

//ip ProjectSet
impl ProjectSet {
    pub fn new(cmd_args: CmdArgs) -> Self {
        let kernels = Kernels::new();
        let projects = vec![];
        let index_by_name = HashMap::new();
        let image_cache = ImageCache::new();
        Self {
            cmd_args,
            projects,
            index_by_name,
            kernels,
            image_cache,
        }
    }

    //mp fill_from_project_path
    pub fn fill_from_project_path(&mut self) -> Result<()> {
        let paths = self.cmd_args.map_project_path(|ps| {
            ps.glob(100, 20, &|_| PathGlob::Push, &|f| {
                f.extension().is_some_and(|x| x == "json")
                    && f.file_stem()
                        .and_then(|x| x.to_str())
                        .is_some_and(|x| x.ends_with("_proj"))
            })
        });
        for p in paths {
            eprintln!("Adding project JSON {p:?}");
            self.add_project(p.into_boxed_path())?;
        }
        Ok(())
    }

    //mp add_project
    pub fn add_project(&mut self, path: Box<Path>) -> Result<()> {
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
    ) -> Result<()> {
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
    ) -> Result<()> {
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
    ) -> Result<()> {
        let mut str_content = "";
        let mut e = match std::str::from_utf8(content) {
            Ok(c) => {
                str_content = c;
                None
            }
            Err(_e) => Some("Bad UTF8 in JSon".to_string().into()),
        };
        if e.is_none() {
            e = self.projects[idx].of_json(str_content).err();
        }
        if e.is_none() {
            e = self.projects[idx].save().unwrap().err();
        }
        if let Some(e) = e {
            Err(format!("Failed to save project {idx} with json {e}:").into())
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
    ) -> Result<()> {
        let up = self.projects[pd.idx].ensure_loaded()?;
        let p = up.as_ref();

        let cip = pd.cip().unwrap_or_default();
        let Some(cip) = p.cip(cip).cloned() else {
            return Err("Cip could not be found".into());
        };
        let cip_r = cip.borrow();

        let pms = cip_r.pms();
        let mesh = Mesh::optimized(pms.borrow().mappings().iter().map(|p| *p.screen()), 1E-2);
        let triangles: Vec<_> = mesh.triangles().collect();
        eprintln!("Triangles of mesh {triangles:?}");
        let json = serde_json::to_string(&triangles).unwrap();
        eprintln!("Json of mesh {json}");
        response.content = json.into_bytes();
        response.mime_type = server.mime_type("json");
        response.resp_type = HttpResponseType::FileRead;
        Ok(())
    }

    //mi http_cip_image
    fn http_cip_image(
        &self,
        server: &HttpServer<Self>,
        _request: &HttpRequest,
        _content: &[u8],
        response: &mut HttpResponse,
        pd: &ProjectDecode,
    ) -> Result<()> {
        let up = self.projects[pd.idx].ensure_loaded()?;
        let p = up.as_ref();

        let cip = pd.cip().unwrap_or_default();
        let Some(cip) = p.cip(cip).cloned() else {
            return Err("Cip could not be found".into());
        };
        let cip_r = cip.borrow();

        let Some(path) = self.cmd_args.find_image_file(cip_r.image_filename()) else {
            return Err(format!("Could not find image file {}", cip_r.image_filename()).into());
        };
        server.verbose().then(|| eprintln!("Open image {path:?}"));

        let src_img_ref = self.image_cache.src_image(&path)?;
        let src_img = ImageCacheEntry::cr_as_rgb8(&src_img_ref);

        let img_bytes = src_img.encode("jpeg")?;
        response.content = img_bytes;
        response.mime_type = server.mime_type("jpeg");
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
    ) -> Result<()> {
        let up = self.projects[pd.idx].ensure_loaded()?;
        let p = up.as_ref();

        let cip = pd.cip().unwrap_or_default();
        let Some(cip) = p.cip(cip).cloned() else {
            return Err("Cip could not be found".into());
        };
        let cip_r = cip.borrow();

        let Some(path) = self.cmd_args.find_image_file(cip_r.image_filename()) else {
            return Err(format!("Could not find image file {}", cip_r.image_filename()).into());
        };
        server.verbose().then(|| eprintln!("Open image {path:?}"));

        let src_img_ref = self.image_cache.src_image(&path)?;
        let src_img = ImageCacheEntry::cr_as_rgb8(&src_img_ref);

        let src_size = src_img.size();
        let src_size = (src_size.0 as f64, src_size.1 as f64);
        let x_scale = pd.width.map(|w| src_size.0 / w).unwrap_or(1.0);
        let y_scale = pd.height.map(|h| src_size.1 / h).unwrap_or(1.0);
        let scale = x_scale.max(y_scale);
        let width = (src_size.0 / scale) as u32;
        let height = (src_size.1 / scale) as u32;
        let mut scaled_img = ImageRgb8::new(width, height);
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
    /// cip, width, window
    fn http_cip_patch(
        &self,
        server: &HttpServer<Self>,
        _request: &HttpRequest,
        _content: &[u8],
        response: &mut HttpResponse,
        pd: &ProjectDecode,
    ) -> Result<()> {
        let up = self.projects[pd.idx].ensure_loaded()?;
        let p = up.as_ref();

        let nps = pd.nps();
        let nps = p.nps().borrow().select(nps.iter().map(|s| s.as_str()))?;

        let cip = pd.cip().unwrap_or_default();
        let Some(cip) = p.cip(cip).cloned() else {
            return Err("Cip could not be found".into());
        };
        let cip_r = cip.borrow();

        let Some(path) = self.cmd_args.find_image_file(cip_r.image_filename()) else {
            return Err(format!("Could not find image file {}", cip_r.image_filename()).into());
        };

        let src_img_ref = self.image_cache.src_image(&path)?;
        let src_img = ImageCacheEntry::cr_as_rgb8(&src_img_ref);

        let camera = cip_r.camera_ref();

        let Some(mut patch) = Patch::create(nps.iter().cloned()) else {
            return Err(format!("Failed to create patch for nps with {} points", nps.len()).into());
        };
        patch.set_render_px_per_model(25.0);
        patch.set_expansion_factor(1.1);
        patch.update_data();

        let patch_img = patch.create_img(&*camera, src_img).unwrap();

        let to_width = pd.width.map(|x| x as usize).unwrap_or(200);
        let ws = pd.window.unwrap_or(4) as u32;
        let (w, h, mut img_data) = patch_img.as_vec_gray_f32(Some(to_width));
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
    fn find_file<A: AsRef<Path>>(&self, file: A) -> Option<PathBuf> {
        self.cmd_args.find_file(file)
    }
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
                if request.action_is("list") && request.req_type() == HttpRequestType::Get {
                    self.http_list_projects(server, request, content, response)
                } else {
                    Err("Unknown project action".into())
                }
            } else if let Some(idx) = pd.project_idx() {
                match (request.req_type(), request.action()) {
                    (HttpRequestType::Get, Some("load")) => {
                        self.http_load_project(server, request, content, response, idx)
                    }
                    (HttpRequestType::Put, Some("save")) => {
                        self.http_save_project(server, request, content, response, idx)
                    }
                    (HttpRequestType::Get, Some("mesh")) => {
                        self.http_cip_pms_mesh(server, request, content, response, &pd)
                    }
                    (HttpRequestType::Get, Some("image")) => {
                        self.http_cip_image(server, request, content, response, &pd)
                    }
                    (HttpRequestType::Get, Some("thumbnail")) => {
                        self.http_cip_thumbnail(server, request, content, response, &pd)
                    }
                    (HttpRequestType::Get, Some("patch")) => {
                        self.http_cip_patch(server, request, content, response, &pd)
                    }
                    _ => Err("Bad request type".into()),
                }
            } else {
                Err(format!("Failed to find project {}", pd.project().unwrap()).into())
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
