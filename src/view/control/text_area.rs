use std::time::Instant;

use crate::text;

use super::super::action::Action;
use super::Wrap;
use crate::{interaction, session};

#[derive(Debug, Clone, PartialEq)]
pub struct TextArea {
    buffer: text::Buffer,
    state: text::edit::State,
    wrap: Wrap,
    mode: text::edit::FieldMode,
    focus: Option<session::Focus>,
    focused: bool,
    focus_visible: bool,
    scroll: interaction::ScrollOffset,
    reveal: bool,
    preedit: Option<text::edit::Preedit>,
    caret_epoch: Option<Instant>,
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
            mode: text::edit::FieldMode::Editable,
            focus: None,
            focused: false,
            focus_visible: false,
            scroll: interaction::ScrollOffset::default(),
            reveal: false,
            preedit: None,
            caret_epoch: None,
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

    pub(crate) fn with_mode(mut self, mode: text::edit::FieldMode) -> Self {
        self.mode = mode;
        self
    }

    pub(crate) fn read_only(self) -> Self {
        self.with_mode(text::edit::FieldMode::ReadOnly)
    }

    pub(crate) fn mode(&self) -> text::edit::FieldMode {
        self.mode
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
        match self.mode {
            text::edit::FieldMode::Editable if self.focused => area,
            text::edit::FieldMode::Editable | text::edit::FieldMode::ReadOnly => area.read_only(),
            text::edit::FieldMode::Disabled => area.disabled(),
        }
    }

    pub(crate) fn view_state_at(&self, now: Instant) -> text::edit::ViewState {
        let epoch = self.caret_epoch.unwrap_or(now);
        let state = text::edit::ViewState::new_at(0.0, epoch)
            .with_scroll(self.scroll.x() as f32, self.scroll.y() as f32)
            .with_preedit(self.preedit.clone());

        if self.reveal {
            state.ensure_caret_visible(now)
        } else {
            state
        }
    }

    pub(crate) fn caret_epoch(&self) -> Option<Instant> {
        self.caret_epoch
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

    pub fn focus_visible(&self) -> bool {
        self.focus_visible
    }

    pub fn preedit(&self) -> Option<&text::edit::Preedit> {
        self.preedit.as_ref()
    }

    pub(crate) fn focus_action(&self) -> Option<Action> {
        Action::text_focus(self.focus)
    }

    pub(crate) fn click_action(&self, position: text::buffer::Position) -> Option<Action> {
        self.pointer_action(text::edit::PointerEditKind::Click, position)
    }

    pub(crate) fn pointer_action(
        &self,
        kind: text::edit::PointerEditKind,
        position: text::buffer::Position,
    ) -> Option<Action> {
        Action::text_pointer(self.focus, kind, position)
    }

    pub(crate) fn drag_action(&self, position: text::buffer::Position) -> Action {
        Action::text_drag(position)
    }

    pub(in crate::view) fn project_layout_interaction(
        &mut self,
        interaction: &interaction::Interaction,
        target: Option<&interaction::Target>,
    ) {
        let Some(target) = target else {
            self.scroll = interaction::ScrollOffset::default();
            self.reveal = false;
            self.preedit = None;
            self.caret_epoch = None;
            return;
        };

        self.scroll = interaction.scroll().offset(target);
        self.reveal = interaction.scroll().should_reveal(target);
        self.preedit = interaction.text_input().preedit_for(target).cloned();
        self.caret_epoch = interaction.text_input().caret_epoch_for(target);
    }

    pub(in crate::view) fn project_focus(&mut self, focus: Option<&session::Focus>) {
        self.focused = self
            .focus
            .as_ref()
            .is_some_and(|text_focus| focus.is_some_and(|focus| text_focus.same_target(focus)));
        self.focus_visible = self.focused
            && focus.is_some_and(|focus| focus.shows_focus_indicator() || self.mode.is_editable());
    }
}
