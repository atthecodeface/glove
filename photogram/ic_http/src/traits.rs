//a Imports
use std::path::{Path, PathBuf};

use crate::{HttpRequest, HttpResponse, HttpServer};

//tt HttpServerExt
/// This is the type of the configuration of an http server that is set *once* and then is immutable.
///
/// One instance of this is created with a [OnceLock]
pub trait HttpServerExt: Sized {
    fn find_file<A: AsRef<Path>>(&self, _file: A) -> Option<PathBuf> {
        None
    }
    fn set_http_response(
        &self,
        _server: &HttpServer<Self>,
        _request: &HttpRequest,
        _content: &[u8],
        _response: &mut HttpResponse,
    ) -> bool {
        false
    }
}

//ip HttpServerExt for ()
impl HttpServerExt for () {}
