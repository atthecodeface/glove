//a Imports
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use star_catalog::Catalog as StarCatalog;
use thunderclap::{ArgCount, ArgDescriptor, CommandArgs, CommandBuilder};

use ic_base::{Error, PathSet, Result};

//a CmdResult
pub type CmdResult = std::result::Result<String, ic_base::Error>;
pub fn cmd_ok() -> CmdResult {
    Ok("".into())
}

#[derive(Default)]
pub struct CmdArgsInner {
    verbose: bool,
    pretty_json: bool,
    background: bool,

    port: usize,
    num_threads: usize,
    file_path_set: PathSet,
    image_path_set: PathSet,
    project_path_set: PathSet,

    // star_catalog is in a Mutex not a RwLock as it is not sync, as some of the star filters require RefCell
    star_catalog: Option<Mutex<StarCatalog>>,

    // Positional string / f64 / usize arguments
    arg_strings: Vec<String>,
    arg_f64s: Vec<f64>,
    arg_usizes: Vec<usize>,
}

//ip Debug for CmdArgsInner
impl std::fmt::Debug for CmdArgsInner {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "CmdArgs{{")?;
        if self.verbose {
            write!(fmt, "verbose, ")?;
        }
        if self.pretty_json {
            write!(fmt, "pretty_json")?;
        }
        if self.background {
            write!(fmt, "background, ")?;
        }
        Ok(())
    }
}

//a ServerState
#[derive(Default)]
pub struct ServerState {
    started: bool,
    server: ic_threads::ThreadPool,
}

//a CmdArgs
//tp CmdArgs
#[derive(Clone, Default)]
pub struct CmdArgs {
    arw: Arc<RwLock<CmdArgsInner>>,
    server: Arc<RwLock<ServerState>>,
}

//ip Debug for CmdArgs
impl std::fmt::Debug for CmdArgs {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self.arw.read() {
            Ok(inner) => inner.fmt(fmt),
            _ => {
                write!(fmt, "CmdArgs::<could not read>")
            }
        }
    }
}

//ip CmdArgs
impl CmdArgs {
    //mp server_running
    pub fn server_running(&self) -> bool {
        let server = self.server.read().unwrap();
        server.started
    }

    //mp server_run
    pub fn server_run<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut server = self.server.write().unwrap();
        server.server.add_thread();
        server.server.issue_work(f);
    }

    //mi map_mut
    fn map_mut<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&mut CmdArgsInner) -> T,
    {
        let mut inner = self.arw.write().unwrap();
        f(&mut inner)
    }
    //mi map
    fn map<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&CmdArgsInner) -> T,
    {
        let inner = self.arw.read().unwrap();
        f(&inner)
    }
}

//a CmdArgs setters
//ip CmdArgs setters
impl CmdArgs {
    //mi set_verbose
    pub(crate) fn set_verbose(&mut self, verbose: bool) -> Result<()> {
        self.map_mut(|inner| {
            inner.verbose = verbose;
        });
        Ok(())
    }

    //mi set_pretty_json
    pub(crate) fn set_pretty_json(&mut self, pretty_json: bool) -> Result<()> {
        self.map_mut(|inner| {
            inner.pretty_json = pretty_json;
        });
        Ok(())
    }

    //mi set_background
    pub(crate) fn set_background(&mut self, background: bool) -> Result<()> {
        self.map_mut(|inner| {
            inner.background = background;
        });
        Ok(())
    }

    //mi set_num_threads
    pub(crate) fn set_num_threads(&mut self, num_threads: usize) -> Result<()> {
        if num_threads == 0 || num_threads > 20 {
            return Err(format!(
                "Number of threads {num_threads} must be non-zero and no more than 20"
            )
            .into());
        }
        self.map_mut(|inner| {
            inner.num_threads = num_threads;
        });
        Ok(())
    }

    //mi set_port
    pub(crate) fn set_port(&mut self, port: usize) -> Result<()> {
        if !(1024..=60000).contains(&port) {
            return Err(format!("Port {port} must be in the range 1024..60000").into());
        }
        self.map_mut(|inner| {
            inner.port = port;
        });
        Ok(())
    }

    //fp set_star_catalog
    pub fn set_star_catalog(&mut self, filename: &str) -> Result<()> {
        let mut catalog = StarCatalog::load_catalog(filename, 99.)?;
        catalog.derive_data();
        self.map_mut(|inner| {
            inner.star_catalog = Some(Mutex::new(catalog));
        });
        Ok(())
    }

    //mi add_file_path
    pub(crate) fn add_file_path(&mut self, s: &str) -> Result<()> {
        self.map_mut(|inner| inner.file_path_set.add_path(s))
    }

    //mi add_image_path
    pub(crate) fn add_image_path(&mut self, s: &str) -> Result<()> {
        self.map_mut(|inner| inner.image_path_set.add_path(s))
    }

    //mi add_project_path
    pub(crate) fn add_project_path(&mut self, s: &str) -> Result<()> {
        self.map_mut(|inner| inner.project_path_set.add_path(s))
    }

    //mi add_string_arg
    pub(crate) fn add_string_arg(&mut self, s: &str) -> Result<()> {
        self.map_mut(|inner| {
            inner.arg_strings.push(s.to_owned());
        });
        Ok(())
    }

    //mi add_f64_arg
    pub(crate) fn add_f64_arg(&mut self, v: f64) -> Result<()> {
        self.map_mut(|inner| {
            inner.arg_f64s.push(v);
        });
        Ok(())
    }

    //mi add_usize_arg
    pub(crate) fn add_usize_arg(&mut self, v: usize) -> Result<()> {
        self.map_mut(|inner| {
            inner.arg_usizes.push(v);
        });
        Ok(())
    }
}

impl CmdArgs {
    const ARG_VERBOSE: ArgDescriptor<Self> = ArgDescriptor::arg_flag(
        "verbose",
        Some('v'),
        "Enable verbose output",
        &Self::set_verbose,
    );

    const ARG_PRETTY_JSON: ArgDescriptor<Self> = ArgDescriptor::arg_flag(
        "pretty_json",
        None,
        "Use pretty-printing for Json output",
        &Self::set_pretty_json,
    );

    const ARG_FILE_PATH: ArgDescriptor<Self> = ArgDescriptor::arg_string(
        "file_path",
        None,
        "Add a directory to the search path",
        ArgCount::Any,
        None,
        &Self::add_file_path,
    );

    const ARG_IMAGE_PATH: ArgDescriptor<Self> = ArgDescriptor::arg_string(
        "image_path",
        None,
        "Add a directory to the search path",
        ArgCount::Any,
        None,
        &Self::add_image_path,
    );

    const ARG_PROJECT_PATH: ArgDescriptor<Self> = ArgDescriptor::arg_string(
        "project_path",
        None,
        "Add a directory to the search path",
        ArgCount::Any,
        None,
        &Self::add_project_path,
    );

    const ARG_BACKGROUND: ArgDescriptor<Self> = ArgDescriptor::arg_flag(
        "background",
        Some('b'),
        "Enable background operation",
        &Self::set_background,
    );

    const ARG_NUM_THREADS: ArgDescriptor<Self> = ArgDescriptor::arg_usize(
        "num_threads",
        None,
        "num_threads parameter for (e.g.) a kernel",
        ArgCount::Optional,
        Some("0"),
        &Self::set_num_threads,
    );

    const ARG_PORT: ArgDescriptor<Self> = ArgDescriptor::arg_usize(
        "port",
        None,
        "Port parameter for (e.g.) a kernel",
        ArgCount::Optional,
        Some("8020"),
        &Self::set_port,
    );

    pub const ARGS_MAIN: &[ArgDescriptor<Self>] = &[
        Self::ARG_VERBOSE,
        Self::ARG_PRETTY_JSON,
        Self::ARG_FILE_PATH,
        Self::ARG_IMAGE_PATH,
        Self::ARG_PROJECT_PATH,
    ];

    pub const ARGS_SERVE: &[ArgDescriptor<Self>] =
        &[Self::ARG_NUM_THREADS, Self::ARG_PORT, Self::ARG_BACKGROUND];
}

//a CmdArgs accessors and operations
//ip CmdArgs - Operations
impl CmdArgs {
    //mi verbose
    pub fn verbose(&self) -> bool {
        self.map(|inner| inner.verbose)
    }

    //mi pretty_json
    pub fn pretty_json(&self) -> bool {
        self.map(|inner| inner.pretty_json)
    }

    //mi background
    pub fn background(&self) -> bool {
        self.map(|inner| inner.background)
    }

    //mi num_threads
    pub fn num_threads(&self) -> usize {
        self.map(|inner| inner.num_threads)
    }

    //mi port
    pub fn port(&self) -> usize {
        self.map(|inner| inner.port)
    }

    //mi get_f64_arg
    pub fn get_f64_arg(&self, n: usize) -> Option<f64> {
        self.map(|inner| inner.arg_f64s.get(n).copied())
    }

    //mi get_usize_arg
    pub fn get_usize_arg(&self, n: usize) -> Option<usize> {
        self.map(|inner| inner.arg_usizes.get(n).copied())
    }

    //mi get_string_arg
    pub fn get_string_arg(&self, n: usize) -> Option<String> {
        self.map(|inner| inner.arg_strings.get(n).cloned())
    }

    //mi find_image_file
    pub fn find_image_file<P: AsRef<Path>>(&self, path: P) -> Option<PathBuf> {
        self.map(|inner| inner.image_path_set.find_file(path))
    }

    //mi find_file
    pub fn find_file<P: AsRef<Path>>(&self, path: P) -> Option<PathBuf> {
        self.map(|inner| inner.file_path_set.find_file(path))
    }

    //mi find_project_file
    pub fn find_project_file<P: AsRef<Path>>(&self, path: P) -> Option<PathBuf> {
        self.map(|inner| inner.project_path_set.find_file(path))
    }

    //mi map_project_path
    pub fn map_project_path<F, T>(&self, f: F) -> T
    where
        F: Fn(&PathSet) -> T,
    {
        self.map(|inner| f(&inner.project_path_set))
    }

    //mp if_verbose
    pub fn if_verbose<F>(&self, f: F)
    where
        F: FnOnce(),
    {
        if self.verbose() {
            f()
        }
    }
}

//a CommandArgs for CmdArgs
//ip CommandArgs for CmdArgs
//ti KeyFn
struct KeyFn(
    &'static str,
    &'static dyn Fn(&CmdArgs) -> Option<String>,
    &'static dyn Fn(&mut CmdArgs, &str) -> Result<bool>,
);

//ci KEY_FNS
const KEY_FNS: &[KeyFn] = &[KeyFn(
    "num_threads",
    &|cmd_args| cmd_args.map(|inner| Some(inner.verbose.to_string())),
    &|mut _cmd_args, s| Err(format!("Failed to set key 'num_threads' to '{s}'").into()),
)];

impl CommandArgs for CmdArgs {
    type Error = Error;
    type Value = String;

    fn value_from_str(s: &str) -> Result<Self::Value> {
        Ok(s.into())
    }

    fn cmd_ok() -> CmdResult {
        Ok("".into())
    }

    fn reset_args(&mut self) {
        self.map_mut(|inner| {
            inner.arg_strings = vec![];
            inner.arg_f64s = vec![];
            inner.arg_usizes = vec![];
        });
    }

    /// Get the keys (elements) of the arguments - used in batch and interactive only
    fn keys(&self) -> Box<dyn Iterator<Item = &str>> {
        Box::new(KEY_FNS.iter().map(|k| k.0))
    }

    /// Retrieve the value of a key, in some form, from the arguments - used in batch and interactive only
    fn value_str(&self, key: &str) -> Option<String> {
        for k in KEY_FNS.iter() {
            if key == k.0 {
                return k.1(self);
            }
        }
        None
    }

    /// Set the value
    fn value_set(&mut self, key: &str, value: &String) -> Result<bool> {
        for k in KEY_FNS.iter() {
            if key == k.0 {
                return k.2(self, value);
            }
        }
        Ok(false)
    }
}
