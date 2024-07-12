use serde_json::error::Category as JsonErrorCategory;
use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("message length exceeded {} bytes", crate::bus::MAX_MESSAGE_SIZE)]
    MessageLengthExceeded,
    #[error("read zero bytes from device bus")]
    ReadZero,
    #[error("I/O error: {0}")]
    Io(io::Error),
    #[error("JSON error: {0}")]
    Json(serde_json::Error),
    #[error("HLAPI error: {0}")]
    Api(Box<str>),
}

impl Error {
    fn from_io_error(e: io::Error) -> Self {
        if e.kind() == io::ErrorKind::WriteZero {
            Self::MessageLengthExceeded
        } else {
            Self::Io(e)
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        match value.classify() {
            JsonErrorCategory::Io => Self::from_io_error(value.into()),
            _ => Self::Json(value),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<Box<str>> for Error {
    fn from(value: Box<str>) -> Self {
        Self::Api(value)
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::from(value.into_boxed_str())
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Api(value.into())
    }
}
