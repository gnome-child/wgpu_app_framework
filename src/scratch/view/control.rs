use std::time::Instant;

use crate::text;

use super::super::{interaction, session};
use super::Action;

#[derive(Debug, Clone, PartialEq)]
pub enum Control {
    Button(Button),
    Checkbox(Checkbox),
    Radio(Radio),
    Slider(Slider),
    TextBox(TextBox),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Button {
    label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkbox {
    label: String,
    checked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Radio {
    label: String,
    selected: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Slider {
    label: String,
    value: f64,
    start: f64,
    end: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBox {
    text: String,
    placeholder: Option<String>,
    focus: Option<session::Focus>,
    focused: bool,
    cursor: Option<usize>,
    preedit: Option<text::Preedit>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextArea {
    buffer: text::Buffer,
    edit_state: text::edit::State,
    wrap: Wrap,
    focus: Option<session::Focus>,
    focused: bool,
    scroll: interaction::ScrollOffset,
    reveal: bool,
    preedit: Option<text::Preedit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wrap {
    None,
    Word,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

impl Checkbox {
    pub fn new(label: impl Into<String>, checked: bool) -> Self {
        Self {
            label: label.into(),
            checked,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn checked(&self) -> bool {
        self.checked
    }

    pub(super) fn display_label(&self) -> String {
        let marker = if self.checked { "[x]" } else { "[ ]" };
        format!("{marker} {}", self.label)
    }
}

impl Radio {
    pub fn new(label: impl Into<String>, selected: bool) -> Self {
        Self {
            label: label.into(),
            selected,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn selected(&self) -> bool {
        self.selected
    }

    pub(super) fn display_label(&self) -> String {
        let marker = if self.selected { "(o)" } else { "( )" };
        format!("{marker} {}", self.label)
    }
}

impl Slider {
    pub fn new(label: impl Into<String>, value: f64, start: f64, end: f64) -> Self {
        let (start, end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        let value = value.clamp(start, end);

        Self {
            label: label.into(),
            value,
            start,
            end,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn start(&self) -> f64 {
        self.start
    }

    pub fn end(&self) -> f64 {
        self.end
    }

    pub fn value_at_fraction(&self, fraction: f64) -> f64 {
        let fraction = if fraction.is_finite() {
            fraction.clamp(0.0, 1.0)
        } else {
            0.0
        };

        self.start + (self.end - self.start) * fraction
    }

    pub(super) fn display_label(&self) -> String {
        format!(
            "{}: {:.2} ({:.2}..{:.2})",
            self.label, self.value, self.start, self.end
        )
    }
}

impl TextBox {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            placeholder: None,
            focus: None,
            focused: false,
            cursor: None,
            preedit: None,
        }
    }

    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn with_focus(mut self, focus: session::Focus) -> Self {
        self.focus = Some(focus);
        self
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn placeholder(&self) -> Option<&str> {
        self.placeholder.as_deref()
    }

    pub fn display_text(&self) -> &str {
        if self.text.is_empty() {
            self.placeholder.as_deref().unwrap_or_default()
        } else {
            &self.text
        }
    }

    pub fn focus(&self) -> Option<session::Focus> {
        self.focus
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn cursor(&self) -> Option<usize> {
        self.cursor
    }

    pub fn preedit(&self) -> Option<&text::Preedit> {
        self.preedit.as_ref()
    }

    pub fn focus_action(&self) -> Option<Action> {
        self.focus.map(Action::focus)
    }

    pub fn click_action(&self, position: text::TextPosition) -> Option<Action> {
        Some(Action::sequence([
            self.focus_action()?,
            Action::text_edit(text::edit::Edit::pointer(
                text::PointerEditKind::Click,
                position,
            )),
        ]))
    }

    pub(super) fn project_interaction(&mut self, interaction: &interaction::Interaction) {
        let Some(focus) = self.focus else {
            self.preedit = None;
            return;
        };

        let target = interaction::Target::text_area(focus);
        if let Some(draft) = interaction.text_input().draft_for(&target) {
            self.text = draft.text().to_owned();
            self.cursor = Some(draft.cursor());
        } else {
            self.cursor = None;
        }
        self.preedit = interaction.text_input().preedit_for(&target).cloned();
    }

    pub(super) fn project_focus(&mut self, focus: Option<session::Focus>) {
        self.focused = self.focus.is_some() && self.focus == focus;
    }
}

impl TextArea {
    pub fn new(text: impl Into<String>) -> Self {
        let buffer = text::Buffer::from_multiline_text(text);
        let edit_state = buffer.edit_state();
        Self::from_buffer(buffer, edit_state)
    }

    pub fn from_buffer(buffer: text::Buffer, edit_state: text::edit::State) -> Self {
        Self {
            buffer,
            edit_state,
            wrap: Wrap::Word,
            focus: None,
            focused: false,
            scroll: interaction::ScrollOffset::default(),
            reveal: false,
            preedit: None,
        }
    }

    pub fn with_wrap(mut self, wrap: Wrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn with_focus(mut self, focus: session::Focus) -> Self {
        self.focus = Some(focus);
        self
    }

    pub fn text(&self) -> String {
        self.buffer.text()
    }

    pub fn buffer(&self) -> &text::Buffer {
        &self.buffer
    }

    pub fn edit_state(&self) -> text::edit::State {
        self.edit_state
    }

    pub fn area_model(&self) -> text::Area {
        let wrap = match self.wrap {
            Wrap::None => text::AreaWrap::None,
            Wrap::Word => text::AreaWrap::WordOrGlyph,
        };
        let area =
            text::Area::new(self.buffer.clone().with_edit_state(self.edit_state)).with_wrap(wrap);
        if self.focused { area } else { area.read_only() }
    }

    pub fn view_state(&self) -> text::TextViewState {
        let state = text::TextViewState::default()
            .with_scroll(self.scroll.x() as f32, self.scroll.y() as f32)
            .with_preedit(self.preedit.clone());

        if self.reveal {
            state.ensure_caret_visible(Instant::now())
        } else {
            state
        }
    }

    pub fn wrap(&self) -> Wrap {
        self.wrap
    }

    pub fn focus(&self) -> Option<session::Focus> {
        self.focus
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn preedit(&self) -> Option<&text::Preedit> {
        self.preedit.as_ref()
    }

    pub fn focus_action(&self) -> Option<Action> {
        self.focus.map(Action::focus)
    }

    pub fn click_action(&self, position: text::TextPosition) -> Option<Action> {
        Some(Action::sequence([
            self.focus_action()?,
            Action::text_edit(text::edit::Edit::pointer(
                text::PointerEditKind::Click,
                position,
            )),
        ]))
    }

    pub fn drag_action(&self, position: text::TextPosition) -> Action {
        Action::text_edit(text::edit::Edit::pointer(
            text::PointerEditKind::Drag,
            position,
        ))
    }

    pub(super) fn project_interaction(
        &mut self,
        interaction: &interaction::Interaction,
        target: Option<&interaction::Target>,
    ) {
        let Some(target) = target else {
            self.scroll = interaction::ScrollOffset::default();
            self.reveal = false;
            self.preedit = None;
            return;
        };

        self.scroll = interaction.scroll().offset(target);
        self.reveal = interaction.scroll().should_reveal(target);
        self.preedit = interaction.text_input().preedit_for(target).cloned();
    }

    pub(super) fn project_focus(&mut self, focus: Option<session::Focus>) {
        self.focused = self.focus.is_some() && self.focus == focus;
    }
}
