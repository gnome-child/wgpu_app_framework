use std::time::Instant;

use crate::text;

use super::super::action::Action;
use super::Wrap;
use crate::scratch::{interaction, session};

#[derive(Debug, Clone, PartialEq)]
pub struct TextArea {
    buffer: text::Buffer,
    state: text::edit::State,
    wrap: Wrap,
    focus: Option<session::Focus>,
    focused: bool,
    scroll: interaction::ScrollOffset,
    reveal: bool,
    preedit: Option<text::edit::Preedit>,
}

impl TextArea {
    pub fn new(text: impl Into<String>) -> Self {
        let buffer = text::Buffer::from_multiline_text(text);
        let state = buffer.initial_state();
        Self::from_buffer(buffer, state)
    }

    pub fn from_buffer(buffer: text::Buffer, state: text::edit::State) -> Self {
        Self {
            buffer,
            state,
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

    pub fn buffer(&self) -> &text::Buffer {
        &self.buffer
    }

    pub fn state(&self) -> text::edit::State {
        self.state
    }

    pub fn area_model(&self) -> text::edit::Area {
        let wrap = match self.wrap {
            Wrap::None => text::edit::AreaWrap::None,
            Wrap::Word => text::edit::AreaWrap::WordOrGlyph,
        };
        let area = text::edit::Area::new(self.buffer.clone())
            .with_state(self.state)
            .with_wrap(wrap);
        if self.focused { area } else { area.read_only() }
    }

    pub fn view_state(&self) -> text::edit::ViewState {
        let state = text::edit::ViewState::default()
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

    pub fn preedit(&self) -> Option<&text::edit::Preedit> {
        self.preedit.as_ref()
    }

    pub fn focus_action(&self) -> Option<Action> {
        self.focus.map(Action::focus)
    }

    pub fn click_action(&self, position: text::buffer::Position) -> Option<Action> {
        Some(Action::sequence([
            self.focus_action()?,
            Action::text_edit(text::edit::Edit::pointer(
                text::edit::PointerEditKind::Click,
                position,
            )),
        ]))
    }

    pub fn drag_action(&self, position: text::buffer::Position) -> Action {
        Action::text_edit(text::edit::Edit::pointer(
            text::edit::PointerEditKind::Drag,
            position,
        ))
    }

    pub(in crate::scratch::view) fn project_interaction(
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

    pub(in crate::scratch::view) fn project_focus(&mut self, focus: Option<session::Focus>) {
        self.focused = self.focus.is_some() && self.focus == focus;
    }
}
