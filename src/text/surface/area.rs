use super::super::buffer::{Buffer, normalize_for_buffer};
use super::super::{selection::State, view::ViewState};
use super::mode::FieldMode;
use super::projection::{composed_presentation_text, preedit_replacement_range};

#[derive(Debug, Clone, PartialEq)]
pub struct Area {
    buffer: Buffer,
    state: State,
    mode: FieldMode,
    placeholder: Option<String>,
    wrap: AreaWrap,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum AreaWrap {
    None,
    #[default]
    WordOrGlyph,
}

impl Area {
    pub fn new(buffer: impl Into<Buffer>) -> Self {
        let mut buffer = buffer.into();
        if !buffer.is_multiline() {
            buffer.promote_to_multiline();
        }
        let state = buffer.initial_state();

        Self {
            buffer,
            state,
            mode: FieldMode::Editable,
            placeholder: None,
            wrap: AreaWrap::default(),
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

    pub fn placeholder(&self) -> Option<&str> {
        self.placeholder.as_deref()
    }

    pub fn wrap(&self) -> AreaWrap {
        self.wrap
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

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn with_wrap(mut self, wrap: AreaWrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn no_wrap(self) -> Self {
        self.with_wrap(AreaWrap::None)
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
        self.is_selectable()
    }

    pub fn allows_cut(&self) -> bool {
        self.is_editable()
    }

    pub fn presentation_text(&self) -> String {
        self.buffer.text()
    }

    pub fn presentation_text_for_state(&self, state: &ViewState) -> String {
        let source = self.presentation_text();
        let Some(preedit) = state.preedit() else {
            return source;
        };

        let range = preedit_replacement_range(&self.buffer, self.state, &source);
        let preedit_text = normalize_for_buffer(&self.buffer, preedit.text());
        composed_presentation_text(&source, range, &preedit_text)
    }
}

impl From<Buffer> for Area {
    fn from(value: Buffer) -> Self {
        Self::new(value)
    }
}

impl From<String> for Area {
    fn from(value: String) -> Self {
        Self::new(Buffer::from_multiline_text(value))
    }
}

impl From<&str> for Area {
    fn from(value: &str) -> Self {
        Self::new(Buffer::from_multiline_text(value))
    }
}
