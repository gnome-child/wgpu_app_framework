use std::path::PathBuf;

pub const MAX_STRING_ARG_BYTES: usize = 64 * 1024;

pub trait Args: Send + 'static {
    fn validate(&self) -> Result<(), Error> {
        Ok(())
    }

    fn size_hint(&self) -> usize {
        0
    }

    fn from_raw(raw: Raw) -> Result<Self, Error>
    where
        Self: Sized;

    fn into_raw(self) -> Raw;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidRaw(&'static str),
    TooLarge { max: usize, actual: usize },
    Invalid(&'static str),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum Raw {
    #[default]
    None,
    Text(String),
    Bool(bool),
    Number(f64),
    Path(PathBuf),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Kind {
    #[default]
    None,
    Text,
    Bool,
    Number,
    Path,
}

impl Raw {
    pub fn kind(&self) -> Kind {
        match self {
            Self::None => Kind::None,
            Self::Text(_) => Kind::Text,
            Self::Bool(_) => Kind::Bool,
            Self::Number(_) => Kind::Number,
            Self::Path(_) => Kind::Path,
        }
    }
}

impl Kind {
    pub fn accepts(self, raw: &Raw) -> bool {
        self == raw.kind()
    }
}

impl Args for () {
    fn from_raw(raw: Raw) -> Result<Self, Error> {
        match raw {
            Raw::None => Ok(()),
            _ => Err(Error::InvalidRaw("expected no args")),
        }
    }

    fn into_raw(self) -> Raw {
        Raw::None
    }
}

impl Args for bool {
    fn from_raw(raw: Raw) -> Result<Self, Error> {
        match raw {
            Raw::Bool(value) => Ok(value),
            _ => Err(Error::InvalidRaw("expected bool args")),
        }
    }

    fn into_raw(self) -> Raw {
        Raw::Bool(self)
    }
}

impl Args for f64 {
    fn from_raw(raw: Raw) -> Result<Self, Error> {
        match raw {
            Raw::Number(value) => Ok(value),
            _ => Err(Error::InvalidRaw("expected number args")),
        }
    }

    fn into_raw(self) -> Raw {
        Raw::Number(self)
    }
}

impl Args for String {
    fn from_raw(raw: Raw) -> Result<Self, Error> {
        let value = match raw {
            Raw::Text(value) => value,
            _ => return Err(Error::InvalidRaw("expected text args")),
        };

        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> Result<(), Error> {
        if self.len() > MAX_STRING_ARG_BYTES {
            return Err(Error::TooLarge {
                max: MAX_STRING_ARG_BYTES,
                actual: self.len(),
            });
        }

        Ok(())
    }

    fn size_hint(&self) -> usize {
        self.len()
    }

    fn into_raw(self) -> Raw {
        Raw::Text(self)
    }
}

impl Args for PathBuf {
    fn from_raw(raw: Raw) -> Result<Self, Error> {
        let value = match raw {
            Raw::Path(value) => value,
            _ => return Err(Error::InvalidRaw("expected path args")),
        };

        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> Result<(), Error> {
        if self.as_os_str().is_empty() {
            return Err(Error::Invalid("path cannot be empty"));
        }

        Ok(())
    }

    fn into_raw(self) -> Raw {
        Raw::Path(self)
    }
}
