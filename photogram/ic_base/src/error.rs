use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read json")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    JsonCtxt(String),
    #[error("{0} {1}")]
    File(String, std::io::Error),
    #[error("{0}")]
    Database(String),
    #[error("{0}")]
    Msg(String),
    #[error("{0}")]
    Image(#[from] image::ImageError),
    #[error("{0}")]
    IOError(#[from] std::io::Error),
}

impl<P: std::fmt::Display> std::convert::From<(P, std::io::Error)> for Error {
    fn from((path, e): (P, std::io::Error)) -> Error {
        Error::File(format!("Error reading file {path}"), e)
    }
}

impl std::convert::From<String> for Error {
    fn from(s: String) -> Error {
        Error::Msg(s)
    }
}
impl std::convert::From<&str> for Error {
    fn from(s: &str) -> Error {
        Error::Msg(s.to_owned())
    }
}
