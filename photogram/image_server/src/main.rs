//a Imports
use std::net::TcpListener;
use std::sync::OnceLock;

use clap::Command;
use thunderclap::CommandBuilder;

use image_server::{CmdArgs, CmdResult, ProjectSet, cmd_ok};

use ic_base::Result;
// use ic_cache::{Cache, CacheEntry, Cacheable};
use ic_http::HttpServer;
use ic_threads::ThreadPool;

//a Main
//si HTTP_SRV
/// This is the configuration of the http server; it is set *once* in main before threads are created
///
/// One instance of this is created with a [OnceLock]
static HTTP_SRV: OnceLock<HttpServer<ProjectSet>> = OnceLock::new();

//fp serve_cmd
fn serve_cmd() -> CommandBuilder<CmdArgs> {
    let command = Command::new("serve").about("Start an HTTP server");

    let mut build = CommandBuilder::with_handler(command, serve_fn);
    CmdArgs::add_arg_num_threads(&mut build);
    CmdArgs::add_arg_port(&mut build);
    CmdArgs::add_arg_background(&mut build);

    build
}

fn run_server(cmd_args: CmdArgs) {
    let num_threads = cmd_args.num_threads();
    let port = cmd_args.port();
    let pool = ThreadPool::new(num_threads);
    let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{port}")) else {
        eprintln!("Failed to bind to port {port}");
        return;
    };
    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.issue_work(|| {
            let http_srv = HTTP_SRV.get().unwrap();
            http_srv.handle_connection(stream);
        });
    }
}

//fi serve_fn
fn serve_fn(cmd_args: &mut CmdArgs) -> CmdResult {
    ensure_http_server(cmd_args);
    cmd_args.server_running();
    let cmd_args_clone = cmd_args.clone();
    cmd_args.server_run(|| run_server(cmd_args_clone));
    if !cmd_args.background() {
        eprintln!("*******************************************************************");
        eprintln!("*** Running server in foreground - interrupt to stop the server ***");
        eprintln!("*******************************************************************");
        loop {
            std::thread::sleep(std::time::Duration::new(1, 0));
        }
    } else {
        eprintln!("************************************");
        eprintln!("*** Running server in background ***");
        eprintln!("************************************");
    }
    cmd_ok()
}

fn ensure_http_server(cmd_args: &CmdArgs) {
    HTTP_SRV.get_or_init(|| {
        let mut project_set = ProjectSet::new(cmd_args.clone());
        project_set.fill_from_project_path().unwrap();
        HttpServer::new(cmd_args.verbose(), project_set)
    });
}

//fp main
fn main() -> Result<()> {
    let command = Command::new("image_server")
        .about("Image calibration/correlation server")
        .version("0.1.0");

    let mut build = CommandBuilder::new(command);

    CmdArgs::add_arg_verbose(&mut build);
    CmdArgs::add_arg_pretty_json(&mut build);
    CmdArgs::add_arg_file_path(&mut build);
    CmdArgs::add_arg_image_path(&mut build);
    CmdArgs::add_arg_project_path(&mut build);

    build.add_subcommand(serve_cmd());

    let mut cmd_args = CmdArgs::default();
    let mut command = build.main(true, true);
    command
        .execute_env(&mut cmd_args)
        .map_err(|e| format!("Error {e:?}"))?;

    Ok(())
}
