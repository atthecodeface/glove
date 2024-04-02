//a Imports
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

//a MimeTypes
//ci MIME_TYPES
pub const MIME_TYPES: &[(&str, &str)] = &[
    ("css", "text/css"),
    ("htm", "text/html"),
    ("html", "text/html"),
    ("js", "text/javascript"),
    ("txt", "text/plain"),
    ("gif", "image/gif"),
    ("ico", "image/vnd.microsoft.icon"),
    ("jpeg", "image/jpeg"),
    ("jpg", "image/jpeg"),
    ("png", "image/png"),
    ("svg", "image/svg+xml"),
    ("tif", "image/tiff"),
    ("tiff", "image/tiff"),
    ("json", "application/json"),
    ("pdf", "application/pdf"),
    ("wasm", "application/wasm"),
    ("xml", "application/xml"),
];

//a UriDecode
//tp UriDecode
#[derive(Debug, Default)]
pub struct UriDecode {
    uri: Option<String>,
    path: Option<PathBuf>,
    action: Option<String>,
    args: Vec<(String, Option<String>)>,
}

//ip UriDecode
impl UriDecode {
    //cp of_uri
    fn of_uri(uri: &str) -> Self {
        let uri = uri.to_string();
        Self {
            uri: Some(uri),
            path: None,
            action: None,
            args: vec![],
        }
    }

    //cp of_path
    fn of_path(path: PathBuf) -> Self {
        Self {
            uri: None,
            path: Some(path),
            action: None,
            args: vec![],
        }
    }

    //mp set_action
    fn set_action(&mut self, action: Option<&str>) {
        self.action = action.map(|a| a.to_owned());
    }

    //mp add_arg
    fn add_arg(&mut self, arg: &str, value: Option<&str>) {
        self.args
            .push((arg.to_string(), value.map(|a| a.to_owned())));
    }

    //ap path
    /// Get the [Path] of the decoded URI if it was valid, else None
    pub fn path(&self) -> Option<&Path> {
        match &self.path {
            Some(p) => Some(p.as_path()),
            None => None,
        }
    }
    //ap action
    /// Get the decoded action of the URI if the path was valid and it
    /// had an action
    ///
    /// This returns Some<action> if the Uri was '<path> ? <action> [ & <k> = <v> ] *'
    pub fn action(&self) -> Option<&str> {
        match &self.action {
            Some(p) => Some(p),
            None => None,
        }
    }

    //fp canonicalize_path
    pub fn canonicalize_path(path: &str) -> Option<PathBuf> {
        let mut pb = PathBuf::new();
        for pc in PathBuf::from(path).components() {
            match pc {
                Component::RootDir => {
                    pb = PathBuf::new();
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    if !pb.pop() {
                        return None;
                    }
                }
                Component::Normal(pc) => {
                    pb.push(pc);
                }
                _ => {
                    // C: for example on Windows
                    return None;
                }
            }
        }
        Some(pb)
    }

    //cp decode_uri
    /// Parse a URI as a path optionally followed by ? action [& k=v]*
    ///
    /// If the decode fails, produce a plain Uri
    pub fn decode_uri(uri: &str) -> UriDecode {
        let mut split = uri.splitn(2, '?');
        let Some(uri) = Self::canonicalize_path(split.next().unwrap()) else {
            return UriDecode::of_uri(uri);
        };

        let mut ud = UriDecode::of_path(uri);
        if let Some(action_args) = split.next() {
            let mut aa_split = action_args.split('&');
            ud.set_action(aa_split.next());
            for args in aa_split {
                let mut arg_split = args.splitn(2, '=');
                let arg = arg_split.next().unwrap();
                ud.add_arg(arg, arg_split.next());
            }
        }
        ud
    }

    //zz All done
}

//a HttpResponse
//tp HttpResponseType
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum HttpResponseType {
    FileRead,
    FileNotFound,
    #[default]
    MalformedRequest,
}

//tp HttpResponse
#[derive(Debug, Default)]
pub struct HttpResponse {
    pub resp_type: HttpResponseType,
    pub content: Vec<u8>,
    pub mime_type: Option<String>,
    pub is_utf8: bool,
}

//a HttpRequest
//tp HttpRequestType
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum HttpRequestType {
    Get,
    Put,
    Post,
    #[default]
    Unknown,
}
//tp HttpRequest
#[derive(Debug, Default)]
pub struct HttpRequest {
    pub req_type: HttpRequestType,
    pub uri: UriDecode,
    pub content_type: String,
    pub content_length: usize,
}
//ip HttpRequest
impl HttpRequest {
    //fi split_at_crlf
    fn split_at_crlf(buffer: &[u8]) -> Option<(&[u8], &[u8])> {
        let n = buffer.len();
        let Some(cr) = buffer
            .iter()
            .enumerate()
            .find_map(|(n, b)| (*b == b'\r').then_some(n))
        else {
            return None;
        };
        if cr + 1 < n && buffer[cr + 1] == b'\n' {
            let (start, end) = buffer.split_at(cr);
            Some((start, &end[2..]))
        } else {
            None
        }
    }

    //mp parse_req_hdr
    fn parse_req_hdr<'buf>(&mut self, buffer: &'buf [u8]) -> Option<&'buf [u8]> {
        let Some((b_req, b_rest)) = Self::split_at_crlf(buffer) else {
            return None;
        };
        if b_req.iter().any(|b| !b.is_ascii()) {
            return None;
        }

        let mut req_fields = b_req.splitn(3, |b| *b == b' ');
        let Some(b_req_type) = req_fields.next() else {
            return None;
        };
        let Some(b_uri) = req_fields.next() else {
            return None;
        };
        let Some(b_http) = req_fields.next() else {
            return None;
        };
        if b_http != b"HTTP/1.1" {
            return None;
        }
        if b_req_type == b"GET" {
            self.req_type = HttpRequestType::Get;
        } else if b_req_type == b"PUT" {
            self.req_type = HttpRequestType::Put;
        } else if b_req_type == b"POST" {
            self.req_type = HttpRequestType::Post;
        }
        self.uri = UriDecode::decode_uri(std::str::from_utf8(b_uri).unwrap());
        Some(b_rest)
    }

    //cp parse_request
    pub fn parse_request(buffer: &[u8]) -> Option<(HttpRequest, &[u8])> {
        let mut request = HttpRequest::default();
        let Some(mut rest) = request.parse_req_hdr(buffer) else {
            return None;
        };
        loop {
            let Some((b_req, b_rest)) = Self::split_at_crlf(rest) else {
                break;
            };
            if b_req.is_empty() {
                return Some((request, b_rest));
            }
            let Ok(line) = std::str::from_utf8(b_req) else {
                break;
            };
            if let Some((k, v)) = line.split_once(": ") {
                if k == "Content-Length" {
                    if let Ok(n) = v.parse::<usize>() {
                        request.content_length = n;
                    }
                }
                if k == "Content-Type" {
                    request.content_type = v.into();
                }
            }
            rest = b_rest;
        }
        None
    }
}

//a HttpServer
//tt HttpServerExt
/// This is the type of the configuration of an http server that is set *once* and then is immutable.
///
/// One instance of this is created with a [OnceLock]
pub trait HttpServerExt: Sized {
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

//tp HttpServer
pub struct HttpServer<T: HttpServerExt> {
    file_root: PathBuf,
    mime_types: HashMap<&'static str, &'static str>,
    data: T,
}
//ip HttpServer
impl<T: HttpServerExt> HttpServer<T> {
    //cp new
    pub fn new<I: Into<PathBuf>>(file_root: I, data: T) -> Self {
        let mime_types: HashMap<&'static str, &'static str> = MIME_TYPES.iter().copied().collect();
        let file_root = file_root.into();
        HttpServer {
            mime_types,
            file_root,
            data,
        }
    }

    //mp mime_type
    pub fn mime_type(&self, extension: &str) -> Option<String> {
        self.mime_types.get(extension).map(|mt| mt.to_string())
    }

    //fi set_file_response
    pub fn set_file_response(
        &self,
        request: &HttpRequest,
        _content: &[u8],
        response: &mut HttpResponse,
    ) -> bool {
        let Some(path) = request.uri.path() else {
            response.resp_type = HttpResponseType::MalformedRequest;
            return false;
        };
        let mut path = Path::join(&self.file_root, path);
        if path.is_dir() {
            path.push("index.html");
        }
        eprintln!("Path {path:?}");
        if let Some(ext) = path.extension() {
            response.mime_type = self.mime_type(ext.to_str().unwrap());
            if let Ok(bytes) = fs::read(&path) {
                response.is_utf8 = std::str::from_utf8(&bytes).is_ok();
                response.content = bytes;
                response.resp_type = HttpResponseType::FileRead;
            } else {
                response.resp_type = HttpResponseType::FileNotFound;
                eprintln!("Failed to open {path:?}");
            }
        }
        true
    }

    //mp send_response
    pub fn send_response(
        &self,
        stream: &mut TcpStream,
        response: HttpResponse,
    ) -> Result<(), std::io::Error> {
        match response.resp_type {
            HttpResponseType::MalformedRequest => {
                stream.write_all("HTTP/1.1 400 BAD REQUEST\r\n\r\n".as_bytes())
            }
            HttpResponseType::FileNotFound => {
                stream.write_all("HTTP/1.1 404 NOT FOUND\r\n\r\n".as_bytes())
            }
            HttpResponseType::FileRead => {
                let length = response.content.len();
                let charset = if response.is_utf8 {
                    "; charset=utf-8"
                } else {
                    ""
                };
                let mime_type = response
                    .mime_type
                    .map(|mt| format!("Content-Type: {mt}{charset}\r\n")) // text/html; charset=utf-8
                    .unwrap_or_default();
                stream.write_all(
                    format!("HTTP/1.1 200 OK\r\n{mime_type}Content-Length: {length}\r\n\r\n")
                        .as_bytes(),
                )?;
                stream.write_all(&response.content)
            }
        }
    }

    //fp handle_connection
    pub fn handle_connection(&self, mut stream: TcpStream) {
        let mut buffer = vec![0_u8; 65536];
        let mut ofs = 0;
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let (request, mut content) = {
            loop {
                let Ok(n) = stream.read(buffer.as_mut_slice().split_at_mut(ofs).1) else {
                    return;
                };
                ofs += n;
                if let Some(r_cs) = HttpRequest::parse_request(&buffer[0..ofs]) {
                    break (r_cs);
                }
                if n == 0 {
                    // Connection shut down without a full header
                    return;
                }
            }
        };
        if request.content_length > 16 * 1024 * 1024 {
            return;
        }
        let mut response = HttpResponse::default();
        let mut content_buffer;
        if request.content_length > content.len() {
            let mut extra_bytes = request.content_length - content.len();
            content_buffer = Vec::with_capacity(request.content_length);
            content_buffer.extend_from_slice(content);
            while extra_bytes > 0 {
                let max_n = extra_bytes.min(buffer.len());
                let Ok(n) = stream.read(&mut buffer[0..max_n]) else {
                    return;
                };
                content_buffer.extend_from_slice(&buffer[0..n]);
                extra_bytes -= n;
                if n == 0 && extra_bytes > 0 {
                    // Connection shut down without full content
                    return;
                }
            }
            content = &content_buffer;
        }
        if self
            .data
            .set_http_response(self, &request, content, &mut response)
            || self.set_file_response(&request, content, &mut response)
        {
            let _ = self.send_response(&mut stream, response);
        } else {
            eprintln!("Request failed: send {response:?}");
            let _ = self.send_response(&mut stream, response);
        }
    }

    //zz All done
}
