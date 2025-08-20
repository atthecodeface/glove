//a Imports

mod http_request;
mod http_response;
mod http_server;
mod mime;
mod traits;
mod uri_decode;

pub use http_request::{HttpRequest, HttpRequestType};
pub use http_response::{HttpResponse, HttpResponseType};
pub use http_server::HttpServer;
pub use mime::MIME_TYPES;
pub use traits::HttpServerExt;
pub use uri_decode::UriDecode;

/*
//a UriArgValue
//ti UriArgValue
// trait UriArgValueKind = std::any::Any  + 'static
pub struct UriArgValue {
    inner: Box<dyn std::any::Any + 'static>,
    type_id: std::any::TypeId,
}

//ii UriArgValue
impl UriArgValue {
    pub fn new<V: std::any::Any + Clone + 'static>(inner: V) -> Self {
        let type_id = std::any::TypeId::of::<V>();
        let inner = Box::new(inner);
        Self { inner, type_id }
    }

    pub fn downcast_ref<T: std::any::Any + 'static>(&self) -> Option<&T> {
        self.inner.downcast_ref::<T>()
    }

    pub fn downcast_into<T: std::any::Any>(self) -> Result<Box<T>, Self> {
        let type_id = self.type_id;
        self.inner
            .downcast::<T>()
            .map_err(|inner| Self { inner, type_id })
    }

    pub fn type_id(&self) -> std::any::TypeId {
        self.type_id
    }
}
*/
