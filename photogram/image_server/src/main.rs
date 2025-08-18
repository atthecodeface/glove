//a Imports
use std::collections::HashMap;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use clap::Command;

use image_server::ProjectSet;

use ic_base::{Mesh, Result};
// use ic_cache::{Cache, CacheEntry, Cacheable};
use ic_camera::CameraProjection;
use ic_http::{
    HttpRequest, HttpRequestType, HttpResponse, HttpResponseType, HttpServer, HttpServerExt,
};
use ic_image::{Image, ImageDrawable, ImageGray16, ImageRgb8, Patch};
use ic_kernel::{KernelArgs, Kernels};
use ic_threads::ThreadPool;
use image_server::ic_cmdline as cmdline_args;

//a Main
//si HTTP_SRV
/// This is the configuration of the http server; it is set *once* in main before threads are created
///
/// One instance of this is created with a [OnceLock]
static HTTP_SRV: OnceLock<HttpServer<ProjectSet>> = OnceLock::new();

//fp main
fn main() -> Result<()> {
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
        )
        .into());
    }
    if !(1024..=60000).contains(&port) {
        return Err(format!("Port {port} must be in the range 1024..60000").into());
    }

    let mut project_set = ProjectSet::new();
    project_set.set_image_root(image_root);
    project_set.fill_from_project_dir(project_root)?;
    HTTP_SRV
        .set(HttpServer::new(verbose, file_root, project_set))
        .map_err(|_| "Bug - faiiled to config server".to_string())?;

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
