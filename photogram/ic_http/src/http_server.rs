//a Imports
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use crate::{HttpRequest, HttpResponse, HttpResponseType, HttpServerExt, MIME_TYPES};

//a HttpServer
//tp HttpServer
pub struct HttpServer<T: HttpServerExt> {
    verbose: bool,
    mime_types: HashMap<&'static str, &'static str>,
    data: T,
}
//ip HttpServer
impl<T: HttpServerExt> HttpServer<T> {
    //cp new
    pub fn new(verbose: bool, data: T) -> Self {
        let mime_types: HashMap<&'static str, &'static str> = MIME_TYPES.iter().copied().collect();
        HttpServer {
            verbose,
            mime_types,
            data,
        }
    }

    //ap data
    pub fn data(&self) -> &T {
        &self.data
    }

    //ap verbose
    #[inline]
    pub fn verbose(&self) -> bool {
        self.verbose
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
        let Some(path) = request.uri().path() else {
            response.resp_type = HttpResponseType::MalformedRequest;
            return false;
        };
        let Some(mut path) = self.data.find_file(path) else {
            response.resp_type = HttpResponseType::FileNotFound;
            eprintln!("Failed to find {path:?}");
            return false;
        };
        if path.is_dir() {
            path.push("index.html");
        }
        self.verbose().then(|| eprintln!("Fetching path {path:?}"));
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
                    break r_cs;
                }
                if n == 0 {
                    // Connection shut down without a full header
                    return;
                }
            }
        };
        if request.content_length() > 16 * 1024 * 1024 {
            return;
        }
        let mut response = HttpResponse::default();
        let mut content_buffer;
        if request.content_length() > content.len() {
            let mut extra_bytes = request.content_length() - content.len();
            content_buffer = Vec::with_capacity(request.content_length());
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
