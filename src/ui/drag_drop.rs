use super::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Move,
    Copy,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    path: Path,
    payload: Payload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    path: Path,
    operation: Operation,
}

impl Source {
    pub fn new(path: Path, payload: Payload) -> Self {
        Self { path, payload }
    }

    pub fn text(path: Path, text: impl Into<String>) -> Self {
        Self::new(path, Payload::Text(text.into()))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }
}

impl Target {
    pub fn new(path: Path, operation: Operation) -> Self {
        Self { path, operation }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn operation(&self) -> Operation {
        self.operation
    }
}
