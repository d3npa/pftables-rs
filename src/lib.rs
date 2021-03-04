pub mod bridge;
pub use bridge::*;
use std::{fmt, io};
use std::error::Error;

#[derive(Debug)]
pub enum PfError {
    ConversionError,
    UnknownAddressFamily,
    Unimplemented,
    IoctlError(io::Error),
    Other(String),
}

impl fmt::Display for PfError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use PfError::*;
        match self {
            Other(message) => {
                write!(f, "{}", message)
            },
            _ => {
                write!(f, "{:?}", self)
            }
        }
    }
}

impl Error for PfError {}

