mod projection;

pub(crate) use projection::{FieldProjection, PreeditProjection, projected_state_for_field};
use projection::{composed_presentation_text, obscured_dot_text, preedit_replacement_range};

use crate::command;

use super::buffer::Buffer;
use super::buffer::normalize_for_buffer;
use super::command as text_command;
use super::unicode::{display_index, source_grapheme_boundaries};
use super::view::TextViewState;

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    buffer: Buffer,
    mode: FieldMode,
    obscuring: Obscuring,
    placeholder: Option<String>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Area {
    buffer: Buffer,
    mode: FieldMode,
    placeholder: Option<String>,
    wrap: AreaWrap,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Surface {
    Field(Field),
    Area(Area),
}
impl text_command::TextTarget for Surface {}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum FieldMode {
    #[default]
    Editable,
    ReadOnly,
    Disabled,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum Obscuring {
    #[default]
    None,
    Dot,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum AreaWrap {
    None,
    #[default]
    WordOrGlyph,
}
impl Field {
    pub fn new(buffer: impl Into<Buffer>) -> Self {
        Self {
            buffer: buffer.into(),
            mode: FieldMode::Editable,
            obscuring: Obscuring::None,
            placeholder: None,
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
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

    pub fn presentation_text_for_state(&self, state: &TextViewState) -> String {
        let source = self.presentation_text();
        let Some(preedit) = state.preedit() else {
            return source;
        };

        match self.obscuring {
            Obscuring::None => {
                let range = preedit_replacement_range(&self.buffer, &source);
                let preedit_text = normalize_for_buffer(&self.buffer, preedit.text());
                composed_presentation_text(&source, range, &preedit_text)
            }
            Obscuring::Dot => {
                let source_text = self.buffer.text();
                let source_boundaries = source_grapheme_boundaries(&source_text);
                let range = if let Some(range) = self.buffer.selected_range() {
                    display_index(&source_boundaries, range.start)
                        ..display_index(&source_boundaries, range.end)
                } else {
                    let index = self.buffer.text_index_for_cursor(self.buffer.cursor());
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

impl Area {
    pub fn new(buffer: impl Into<Buffer>) -> Self {
        let buffer = buffer.into();
        if !buffer.is_multiline() {
            buffer.promote_to_multiline();
        }

        Self {
            buffer,
            mode: FieldMode::Editable,
            placeholder: None,
            wrap: AreaWrap::default(),
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
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

    pub fn presentation_text_for_state(&self, state: &TextViewState) -> String {
        let source = self.presentation_text();
        let Some(preedit) = state.preedit() else {
            return source;
        };

        let range = preedit_replacement_range(&self.buffer, &source);
        let preedit_text = normalize_for_buffer(&self.buffer, preedit.text());
        composed_presentation_text(&source, range, &preedit_text)
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

impl Surface {
    pub fn buffer(&self) -> &Buffer {
        match self {
            Self::Field(field) => field.buffer(),
            Self::Area(area) => area.buffer(),
        }
    }

    pub fn is_field(&self) -> bool {
        matches!(self, Self::Field(_))
    }

    pub fn is_area(&self) -> bool {
        matches!(self, Self::Area(_))
    }

    pub fn as_field(&self) -> Option<&Field> {
        match self {
            Self::Field(field) => Some(field),
            Self::Area(_) => None,
        }
    }

    pub fn as_area(&self) -> Option<&Area> {
        match self {
            Self::Field(_) => None,
            Self::Area(area) => Some(area),
        }
    }

    pub fn placeholder(&self) -> Option<&str> {
        match self {
            Self::Field(field) => field.placeholder(),
            Self::Area(area) => area.placeholder(),
        }
    }

    pub fn is_editable(&self) -> bool {
        match self {
            Self::Field(field) => field.is_editable(),
            Self::Area(area) => area.is_editable(),
        }
    }

    pub fn is_read_only(&self) -> bool {
        match self {
            Self::Field(field) => field.is_read_only(),
            Self::Area(area) => area.is_read_only(),
        }
    }

    pub fn is_disabled(&self) -> bool {
        match self {
            Self::Field(field) => field.is_disabled(),
            Self::Area(area) => area.is_disabled(),
        }
    }

    pub fn is_selectable(&self) -> bool {
        match self {
            Self::Field(field) => field.is_selectable(),
            Self::Area(area) => area.is_selectable(),
        }
    }

    pub fn accepts_text_input(&self) -> bool {
        match self {
            Self::Field(field) => field.accepts_text_input(),
            Self::Area(area) => area.accepts_text_input(),
        }
    }

    pub fn paints_caret(&self) -> bool {
        match self {
            Self::Field(field) => field.paints_caret(),
            Self::Area(area) => area.paints_caret(),
        }
    }

    pub fn allows_text_mutation(&self) -> bool {
        match self {
            Self::Field(field) => field.allows_text_mutation(),
            Self::Area(area) => area.allows_text_mutation(),
        }
    }

    pub fn allows_copy(&self) -> bool {
        match self {
            Self::Field(field) => field.allows_copy(),
            Self::Area(area) => area.allows_copy(),
        }
    }

    pub fn allows_cut(&self) -> bool {
        match self {
            Self::Field(field) => field.allows_cut(),
            Self::Area(area) => area.allows_cut(),
        }
    }

    pub fn presentation_text(&self) -> String {
        match self {
            Self::Field(field) => field.presentation_text(),
            Self::Area(area) => area.presentation_text(),
        }
    }

    pub fn presentation_text_for_state(&self, state: &TextViewState) -> String {
        match self {
            Self::Field(field) => field.presentation_text_for_state(state),
            Self::Area(area) => area.presentation_text_for_state(state),
        }
    }
}

impl command::binding::Responder for Surface {
    fn bind_targets(&self, targets: &mut Vec<command::target::Kind>) {
        if self.is_selectable() {
            targets.push(text_command::text_target_kind());
        }
    }

    fn bind_commands(&self, bindings: &mut Vec<command::binding::Binding>) {
        if self.is_editable() {
            bindings.extend([
                command::binding::Binding::of::<text_command::SelectAll>(),
                command::binding::Binding::of::<text_command::Copy>(),
                command::binding::Binding::of::<text_command::Cut>(),
                command::binding::Binding::of::<text_command::Paste>(),
                command::binding::Binding::of::<text_command::Undo>(),
                command::binding::Binding::of::<text_command::Redo>(),
                command::binding::Binding::of::<text_command::InsertText>(),
            ]);
        } else if self.is_read_only() {
            bindings.extend([
                command::binding::Binding::of::<text_command::SelectAll>(),
                command::binding::Binding::of::<text_command::Copy>(),
            ]);
        }
    }
}

impl From<Field> for Surface {
    fn from(value: Field) -> Self {
        Self::Field(value)
    }
}

impl From<Area> for Surface {
    fn from(value: Area) -> Self {
        Self::Area(value)
    }
}

impl From<AreaWrap> for glyphon::Wrap {
    fn from(value: AreaWrap) -> Self {
        match value {
            AreaWrap::None => glyphon::Wrap::None,
            AreaWrap::WordOrGlyph => glyphon::Wrap::WordOrGlyph,
        }
    }
}
