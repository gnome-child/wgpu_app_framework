use super::super::super::buffer::{Buffer, normalize_for_buffer};
use super::super::super::unicode::{display_index, source_grapheme_boundaries};
use super::super::{State, ViewState};
use super::mode::FieldMode;
use super::projection::{composed_presentation_text, obscured_dot_text, preedit_replacement_range};

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

    pub(super) fn set_state(&mut self, state: State) {
        self.state = state;
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

    pub fn presentation_text_for_state(&self, state: &ViewState) -> String {
        let source = self.presentation_text();
        let Some(preedit) = state.preedit() else {
            return source;
        };

        match self.obscuring {
            Obscuring::None => {
                let range = preedit_replacement_range(&self.buffer, self.state, &source);
                let preedit_text = normalize_for_buffer(&self.buffer, preedit.text());
                composed_presentation_text(&source, range, &preedit_text)
            }
            Obscuring::Dot => {
                let source_text = self.buffer.text();
                let source_boundaries = source_grapheme_boundaries(&source_text);
                let range = if let Some(range) = self.buffer.selected_range_for_state(self.state) {
                    display_index(&source_boundaries, range.start)
                        ..display_index(&source_boundaries, range.end)
                } else {
                    let index = self
                        .buffer
                        .text_index_for_cursor(self.buffer.cursor_for_state(self.state));
                    let index = display_index(&source_boundaries, index);
                    index..index
                };
                let preedit_text =
                    obscured_dot_text(&normalize_for_buffer(&self.buffer, preedit.text()));
                composed_presentation_text(&source, range, &preedit_text)
            }
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
