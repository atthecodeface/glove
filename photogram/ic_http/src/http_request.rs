//a Imports
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use crate::UriDecode;

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
    req_type: HttpRequestType,
    uri: UriDecode,
    content_type: String,
    content_length: usize,
}
//ip HttpRequest
impl HttpRequest {
    //ap req_type
    pub fn req_type(&self) -> HttpRequestType {
        self.req_type
    }

    //ap uri
    pub fn uri(&self) -> &UriDecode {
        &self.uri
    }

    //ap content_type
    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    //ap content_length
    pub fn content_length(&self) -> usize {
        self.content_length
    }

    //fi split_at_crlf
    fn split_at_crlf(buffer: &[u8]) -> Option<(&[u8], &[u8])> {
        let n = buffer.len();

        let cr = buffer
            .iter()
            .enumerate()
            .find_map(|(n, b)| (*b == b'\r').then_some(n))?;

        if cr + 1 < n && buffer[cr + 1] == b'\n' {
            let (start, end) = buffer.split_at(cr);
            Some((start, &end[2..]))
        } else {
            None
        }
    }

    //mp action
    pub fn action(&self) -> Option<&str> {
        self.uri.action().map(|x| x.as_str())
    }

    //mp action_is
    pub fn action_is(&self, action: &str) -> bool {
        self.uri.action_is(action)
    }

    //mp get_one
    pub fn get_one<T>(&self, id: &str) -> Option<Result<T, String>>
    where
        T: std::str::FromStr + 'static,
    {
        for (k, v) in self.uri.args() {
            if k == id && v.is_some() {
                return Some(
                    T::from_str(v.as_ref().unwrap()).map_err(|_e| "Failed to parse".into()),
                );
            }
        }
        None
    }

    //mp get_many
    pub fn get_many<'a, T>(&'a self, id: &'a str) -> impl Iterator<Item = Result<T, String>> + 'a
    where
        T: std::str::FromStr + 'static,
    {
        self.uri.args().iter().filter_map(move |(k, v)| {
            if k == id && v.is_some() {
                Some(T::from_str(v.as_ref().unwrap()).map_err(|_e| "Failed to parse".into()))
            } else {
                None
            }
        })
    }

    //mp parse_req_hdr
    fn parse_req_hdr<'buf>(&mut self, buffer: &'buf [u8]) -> Option<&'buf [u8]> {
        let (b_req, b_rest) = Self::split_at_crlf(buffer)?;
        if b_req.iter().any(|b| !b.is_ascii()) {
            return None;
        }

        let mut req_fields = b_req.splitn(3, |b| *b == b' ');
        let b_req_type = req_fields.next()?;
        let b_uri = req_fields.next()?;
        let b_http = req_fields.next()?;
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
        let mut rest = request.parse_req_hdr(buffer)?;
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
