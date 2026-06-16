#[derive(Debug, Clone, PartialEq)]
pub enum Payload {
    None,
    Text(String),
    Bool(bool),
    Number(f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PayloadKind {
    None,
    Text,
    Bool,
    Number,
}

impl Payload {
    pub fn kind(&self) -> PayloadKind {
        match self {
            Self::None => PayloadKind::None,
            Self::Text(_) => PayloadKind::Text,
            Self::Bool(_) => PayloadKind::Bool,
            Self::Number(_) => PayloadKind::Number,
        }
    }
}

impl Default for Payload {
    fn default() -> Self {
        Self::None
    }
}

impl PayloadKind {
    pub fn accepts(self, payload: &Payload) -> bool {
        self == payload.kind()
    }
}
