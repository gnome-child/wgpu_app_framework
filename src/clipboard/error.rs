use std::fmt;

/// A clipboard operation could not be completed by its configured backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Unavailable,
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable => f.write_str("clipboard is unavailable"),
        }
    }
}

impl std::error::Error for Error {}
