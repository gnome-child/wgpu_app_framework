use super::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    Text(String),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Operation {
    #[default]
    None,
    Move,
    Copy,
    Link,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Operations {
    copy: bool,
    move_: bool,
    link: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Boundary {
    #[default]
    Internal,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropResult {
    Rejected,
    Completed { operation: Operation },
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

impl Operations {
    pub const NONE: Self = Self {
        copy: false,
        move_: false,
        link: false,
    };
    pub const COPY: Self = Self {
        copy: true,
        move_: false,
        link: false,
    };
    pub const MOVE: Self = Self {
        copy: false,
        move_: true,
        link: false,
    };
    pub const LINK: Self = Self {
        copy: false,
        move_: false,
        link: true,
    };
    pub const COPY_MOVE: Self = Self {
        copy: true,
        move_: true,
        link: false,
    };

    pub const fn contains(self, operation: Operation) -> bool {
        match operation {
            Operation::None => false,
            Operation::Copy => self.copy,
            Operation::Move => self.move_,
            Operation::Link => self.link,
        }
    }

    pub const fn is_empty(self) -> bool {
        !self.copy && !self.move_ && !self.link
    }

    pub const fn intersection(self, other: Self) -> Self {
        Self {
            copy: self.copy && other.copy,
            move_: self.move_ && other.move_,
            link: self.link && other.link,
        }
    }
}
