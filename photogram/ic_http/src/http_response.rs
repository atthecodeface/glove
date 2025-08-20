//a Imports

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
