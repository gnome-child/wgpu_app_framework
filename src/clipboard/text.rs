use crate::text as text_engine;

use super::{Clipboard, Payload, Representations};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text(String);

impl Text {
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl Payload for Text {
    fn write(&self, out: &mut Representations) {
        out.set_text(self.0.clone());
    }

    fn read(source: &Representations) -> Option<Self> {
        source.non_empty_text().map(Self)
    }
}

impl text_engine::edit::Clipboard for Clipboard {
    fn read_text(&mut self) -> text_engine::edit::ClipboardResult<Option<String>> {
        self.text()
            .map_err(|_| text_engine::edit::ClipboardError::Unavailable)
    }

    fn write_text(&mut self, text: &str) -> text_engine::edit::ClipboardResult<()> {
        self.put(&Text::new(text))
            .map_err(|_| text_engine::edit::ClipboardError::Unavailable)
    }
}

impl From<String> for Text {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for Text {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}
