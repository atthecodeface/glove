//a Imports
use std::net::TcpListener;
use std::sync::{Arc, Mutex, OnceLock};

use clap::Command;
use image_calibrate::cmdline_args;
use image_calibrate::http::{HttpResponse, HttpResponseType, HttpServer, HttpServerExt};
use image_calibrate::thread_pool::ThreadPool;
use image_calibrate::Project;

//a Main
//si HTTP_SRV
/// This is the configuration of the http server; it is set *once* in main before threads are created
///
/// One instance of this is created with a [OnceLock]
static HTTP_SRV: OnceLock<HttpServer<ProjectSet>> = OnceLock::new();

//fp blah
unsafe impl Send for ProjectWrap {}
#[derive(Debug, Default)]
struct ProjectWrap(Project);
impl ProjectWrap {}

#[derive(Debug, Default)]
struct ProjectSet {
    project: Mutex<ProjectWrap>,
}
impl HttpServerExt for ProjectSet {
    fn set_http_response(
        &self,
        server: &HttpServer<Self>,
        request: &str,
        response: &mut HttpResponse,
    ) -> bool {
        if request.starts_with("GET /project") {
            let wrp = self.project.lock().unwrap();
            let json = wrp.0.to_json(false).unwrap();
            response.content = json.into_bytes();
            response.mime_type = server.mime_type("json");
            response.resp_type = HttpResponseType::FileRead;
            true
        } else {
            false
        }
    }
}

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

    HTTP_SRV
        .set(HttpServer::new(file_root, ProjectSet::default()))
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
