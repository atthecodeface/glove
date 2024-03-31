//a Imports
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::path::Path;

//a MimeTypes
//ci MIME_TYPES
pub const MIME_TYPES: &[(&'static str, &'static str)] = &[
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

//a HttpResponse
//tp HttpResponseType
#[derive(Debug, Default, Clone, Copy)]
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

//a HttpServer
//tp HttpServer
/// This is the type of the configuration of an http server that is set *once* and then is immutable.
///
/// One instance of this is created with a [OnceLock]
pub trait HttpServerExt: Sized {
    fn set_http_response(
        &self,
        _server: &HttpServer<Self>,
        _request: &str,
        _response: &mut HttpResponse,
    ) -> bool {
        false
    }
}
impl HttpServerExt for () {}
pub struct HttpServer<T: HttpServerExt> {
    file_root: String,
    mime_types: HashMap<&'static str, &'static str>,
    data: T,
}

//ip HttpServer
impl<T: HttpServerExt> HttpServer<T> {
    //cp new
    pub fn new<I: Into<String>>(file_root: I, data: T) -> Self {
        let mime_types: HashMap<&'static str, &'static str> = MIME_TYPES.iter().copied().collect();
        let file_root = file_root.into();
        HttpServer {
            mime_types,
            file_root,
            data,
        }
    }

    //fi decode_get_filename
    pub fn decode_get_filename(&self, s: &str) -> Result<Option<String>, String> {
        if s.starts_with("GET /") {
            let mut filename = String::new();
            for c in s.chars().skip(5) {
                if ('0'..='9').contains(&c)
                    || ('a'..='z').contains(&c)
                    || ('A'..='Z').contains(&c)
                    || c == '_'
                    || c == '/'
                    || c == '.'
                {
                    filename.push(c);
                } else if c == ' ' {
                    return Ok(Some(filename));
                } else {
                    return Err(format!("Bad filename in request {s}"));
                }
            }
            Ok(Some(filename))
        } else {
            Ok(None)
        }
    }

    //mp mime_type
    pub fn mime_type(&self, extension: &str) -> Option<String> {
        self.mime_types.get(extension).map(|mt| mt.to_string())
    }

    //fi set_file_response
    pub fn set_file_response(&self, request: &str, response: &mut HttpResponse) -> bool {
        match self.decode_get_filename(request) {
            Ok(Some(filename)) => {
                let mut filename = format!("{}{}", self.file_root, filename);
                if filename.chars().last().unwrap() == '/' {
                    filename.push_str("index.html");
                };
                let path = Path::new(&filename);
                // eprintln!("Path {path:?}");
                if let Some(ext) = path.extension() {
                    response.mime_type = self.mime_type(ext.to_str().unwrap());
                    if let Ok(bytes) = fs::read(&filename) {
                        response.is_utf8 = std::str::from_utf8(&bytes).is_ok();
                        response.content = bytes;
                        response.resp_type = HttpResponseType::FileRead;
                    } else {
                        response.resp_type = HttpResponseType::FileNotFound;
                        eprintln!("Failed to open {filename}");
                    }
                }
                true
            }
            Ok(None) => {
                response.resp_type = HttpResponseType::MalformedRequest;
                false
            }
            Err(e) => {
                response.resp_type = HttpResponseType::MalformedRequest;
                eprintln!("Error {e}");
                true
            }
        }
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
        let buf_reader = BufReader::new(&mut stream);
        let request = buf_reader.lines().next().unwrap().unwrap();
        // eprintln!("Request: {:#?}", request);

        let mut response = HttpResponse::default();
        if !self.data.set_http_response(self, &request, &mut response) {
            self.set_file_response(&request, &mut response);
        }
        let _ = self.send_response(&mut stream, response);
    }

    //zz All done
}
