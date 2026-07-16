use std::time::Instant;

use crate::text;

use super::super::{action::Action, focus};
use super::Wrap;
use crate::{interaction, session};

#[derive(Debug, Clone, PartialEq)]
pub struct TextArea {
    buffer: text::Buffer,
    state: text::selection::State,
    wrap: Wrap,
    mode: text::surface::FieldMode,
    focus: Option<session::Focus>,
    focus_presentation: focus::Presentation,
    scroll: interaction::ScrollOffset,
    reveal: bool,
    preedit: Option<text::Preedit>,
    caret_epoch: Option<Instant>,
}

impl TextArea {
    pub fn new(text: impl Into<String>) -> Self {
        let buffer = text::Buffer::from_multiline_text(text);
        let state = buffer.initial_state();
        Self::from_buffer(buffer, state)
    }

    pub fn from_buffer(buffer: text::Buffer, state: text::selection::State) -> Self {
        Self {
            buffer,
            state,
            wrap: Wrap::Word,
            mode: text::surface::FieldMode::Editable,
            focus: None,
            focus_presentation: focus::Presentation::default(),
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

    pub(crate) fn with_mode(mut self, mode: text::surface::FieldMode) -> Self {
        self.mode = mode;
        self
    }

    pub(crate) fn read_only(self) -> Self {
        self.with_mode(text::surface::FieldMode::ReadOnly)
    }

    pub(crate) fn with_resolved_presentation(
        mut self,
        buffer: text::Buffer,
        state: text::selection::State,
    ) -> Self {
        self.buffer = buffer;
        self.state = state;
        self.scroll = interaction::ScrollOffset::default();
        self.reveal = false;
        self.preedit = None;
        self.caret_epoch = None;
        self
    }

    pub(crate) fn mode(&self) -> text::surface::FieldMode {
        self.mode
    }

    pub fn buffer(&self) -> &text::Buffer {
        &self.buffer
    }

    pub fn state(&self) -> text::selection::State {
        self.state
    }

    pub fn area_model(&self) -> text::surface::Area {
        let wrap = match self.wrap {
            Wrap::None => text::surface::AreaWrap::None,
            Wrap::Word => text::surface::AreaWrap::WordOrGlyph,
        };
        let area = text::surface::Area::new(self.buffer.clone())
            .with_state(self.state)
            .with_wrap(wrap);
        match self.mode {
            text::surface::FieldMode::Editable if self.is_focused() => area,
            text::surface::FieldMode::Editable | text::surface::FieldMode::ReadOnly => {
                area.read_only()
            }
            text::surface::FieldMode::Disabled => area.disabled(),
        }
    }

    pub(crate) fn view_state_at(&self, now: Instant) -> text::view::ViewState {
        let epoch = self.caret_epoch.unwrap_or(now);
        let state = text::view::ViewState::new_at(0.0, epoch)
            .with_integral_scroll(self.scroll.x(), self.scroll.y());

        if self.reveal {
            state.ensure_caret_visible(now)
        } else {
            state
        }
    }

    pub(crate) fn caret_epoch(&self) -> Option<Instant> {
        self.caret_epoch
    }

    pub(in crate::view) fn same_scene_state(&self, other: &Self) -> bool {
        self.buffer == other.buffer
            && self.state == other.state
            && self.wrap == other.wrap
            && self.mode == other.mode
            && self.focus == other.focus
            && self.focus_presentation == other.focus_presentation
            && self.preedit == other.preedit
    }

    pub(crate) fn scroll_reveal_requested(&self) -> bool {
        self.reveal
    }

    pub fn wrap(&self) -> Wrap {
        self.wrap
    }

    pub fn focus(&self) -> Option<session::Focus> {
        self.focus
    }

    pub fn is_focused(&self) -> bool {
        self.focus_presentation.is_focused()
    }

    pub fn focus_visible(&self) -> bool {
        self.focus_presentation.is_visible()
    }

    pub(in crate::view) fn focus_presentation(&self) -> focus::Presentation {
        self.focus_presentation
    }

    pub fn preedit(&self) -> Option<&text::Preedit> {
        self.preedit.as_ref()
    }

    pub(crate) fn focus_action(&self) -> Option<Action> {
        Action::text_focus(self.focus)
    }

    pub(crate) fn click_action(&self, position: text::buffer::Position) -> Option<Action> {
        self.pointer_action(text::selection::PointerKind::Click, position)
    }

    pub(crate) fn pointer_action(
        &self,
        kind: text::selection::PointerKind,
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

        let active_table_target = target.table_cell().is_some()
            && interaction
                .text_input()
                .target()
                .is_some_and(|active| active == target);
        let projects_session = target.table_cell().is_none() || active_table_target;
        self.scroll = if !projects_session {
            interaction::ScrollOffset::default()
        } else {
            interaction.scroll().resident_offset(target)
        };
        self.reveal = projects_session && interaction.scroll().should_reveal(target);
        self.preedit = projects_session
            .then(|| interaction.text_input().preedit_for(target).cloned())
            .flatten();
        self.caret_epoch = projects_session
            .then(|| interaction.text_input().caret_epoch_for(target))
            .flatten();
        if active_table_target && let Some(draft) = interaction.text_input().draft_for(target) {
            if draft.text() != self.buffer.text() {
                self.buffer = text::Buffer::from_multiline_text(draft.text());
            }
            let cursor = self
                .buffer
                .mark_for_position(text::buffer::Position::new(draft.cursor()));
            let selection = draft.selection().map(|selection| text::buffer::MarkRange {
                start: self
                    .buffer
                    .mark_for_position(text::buffer::Position::new(selection.start)),
                end: self
                    .buffer
                    .mark_for_position(text::buffer::Position::new(selection.end)),
            });
            self.state = text::selection::State::new(cursor, selection);
        } else if target.table_cell().is_some() {
            self.state = self.buffer.initial_state();
        }
    }

    pub(in crate::view) fn project_focus(&mut self, focus: Option<&session::Focus>) {
        let focused = self
            .focus
            .as_ref()
            .is_some_and(|text_focus| focus.is_some_and(|focus| text_focus.same_target(focus)));
        self.focus_presentation = if focused {
            focus::Presentation::focused(
                focus.is_some_and(|focus| focus.shows_focus_indicator() || self.mode.is_editable()),
            )
        } else {
            focus::Presentation::default()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn scroll_reveal_and_blink_epoch_are_not_scene_content_state() {
        let first = TextArea::new("one\ntwo");
        let mut second = first.clone();
        second.scroll = interaction::ScrollOffset::new(31, 47);
        second.reveal = true;
        second.caret_epoch = Some(Instant::now() + Duration::from_secs(1));

        assert!(first.same_scene_state(&second));
        assert_ne!(first, second, "full model equality remains diagnostic");
    }

    #[test]
    fn projected_text_scroll_preserves_integral_values_past_f32_precision() {
        for value in [16_777_215, 16_777_216, 16_777_217, 24_000_001] {
            let mut area = TextArea::new("precision");
            area.scroll = interaction::ScrollOffset::new(value, value);
            let state = area.view_state_at(Instant::now());
            assert_eq!(state.exact_scroll_x(), f64::from(value));
            assert_eq!(state.exact_scroll_y(), f64::from(value));
        }
    }
}
