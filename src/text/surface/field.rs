use super::super::buffer::Buffer;
use super::super::selection::State;
use super::mode::FieldMode;
use super::projection::obscured_dot_text;

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    buffer: Buffer,
    state: State,
    mode: FieldMode,
    obscuring: Obscuring,
    placeholder: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Obscuring {
    #[default]
    None,
    Dot,
}

impl Field {
    pub fn new(buffer: impl Into<Buffer>) -> Self {
        let buffer = buffer.into();
        let state = buffer.initial_state();
        Self {
            buffer,
            state,
            mode: FieldMode::Editable,
            obscuring: Obscuring::None,
            placeholder: None,
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn mode(&self) -> FieldMode {
        self.mode
    }

    pub fn obscuring(&self) -> Obscuring {
        self.obscuring
    }

    pub fn placeholder(&self) -> Option<&str> {
        self.placeholder.as_deref()
    }

    pub fn with_mode(mut self, mode: FieldMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_state(mut self, state: State) -> Self {
        self.state = state;
        self
    }

    pub fn read_only(self) -> Self {
        self.with_mode(FieldMode::ReadOnly)
    }

    pub fn disabled(self) -> Self {
        self.with_mode(FieldMode::Disabled)
    }

    pub fn with_obscuring(mut self, obscuring: Obscuring) -> Self {
        self.obscuring = obscuring;
        self
    }

    pub fn obscured_dot(self) -> Self {
        self.with_obscuring(Obscuring::Dot)
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn is_editable(&self) -> bool {
        self.mode == FieldMode::Editable
    }

    pub fn is_read_only(&self) -> bool {
        self.mode == FieldMode::ReadOnly
    }

    pub fn is_disabled(&self) -> bool {
        self.mode == FieldMode::Disabled
    }

    pub fn is_selectable(&self) -> bool {
        !self.is_disabled()
    }

    pub fn accepts_text_input(&self) -> bool {
        self.is_editable()
    }

    pub fn paints_caret(&self) -> bool {
        self.is_editable()
    }

    pub fn allows_text_mutation(&self) -> bool {
        self.is_editable()
    }

    pub fn allows_copy(&self) -> bool {
        self.is_selectable() && self.obscuring == Obscuring::None
    }

    pub fn allows_cut(&self) -> bool {
        self.is_editable() && self.obscuring == Obscuring::None
    }

    pub fn presentation_text(&self) -> String {
        match self.obscuring {
            Obscuring::None => self.buffer.text(),
            Obscuring::Dot => obscured_dot_text(&self.buffer.text()),
        }
    }
}

impl From<Buffer> for Field {
    fn from(value: Buffer) -> Self {
        Self::new(value)
    }
}

impl From<String> for Field {
    fn from(value: String) -> Self {
        Self::new(Buffer::from(value))
    }
}

impl From<&str> for Field {
    fn from(value: &str) -> Self {
        Self::new(Buffer::from(value))
    }
}
