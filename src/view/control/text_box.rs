use crate::text;

use super::super::{Hint, action::Action, focus};
use super::{TextArea, Wrap};
use crate::{interaction, session};
use std::ops::Range;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBox {
    text: String,
    placeholder: Option<String>,
    input: text::Input,
    mode: text::surface::FieldMode,
    focus: Option<session::Focus>,
    active: bool,
    inactive_display: bool,
    inactive_display_buffer: Option<text::Buffer>,
    focus_presentation: focus::Presentation,
    caret: Option<Caret>,
    preedit: Option<text::Preedit>,
    caret_epoch: Option<Instant>,
    indicator_hint: Option<Hint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Caret {
    cursor: usize,
    selection: Option<Range<usize>>,
}

impl TextBox {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            placeholder: None,
            input: text::Input::unrestricted(),
            mode: text::surface::FieldMode::Editable,
            focus: None,
            active: false,
            inactive_display: false,
            inactive_display_buffer: None,
            focus_presentation: focus::Presentation::default(),
            caret: None,
            preedit: None,
            caret_epoch: None,
            indicator_hint: None,
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

    pub fn with_input(mut self, input: text::Input) -> Self {
        self.input = input;
        self
    }

    pub(crate) fn with_inactive_display(mut self) -> Self {
        self.inactive_display = true;
        self.inactive_display_buffer = Some(text::Buffer::from_multiline_text(
            self.display_text().to_owned(),
        ));
        self
    }

    pub(crate) fn mode(&self) -> text::surface::FieldMode {
        self.mode
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn placeholder(&self) -> Option<&str> {
        self.placeholder.as_deref()
    }

    pub fn input(&self) -> text::Input {
        self.input
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
        self.focus_presentation.is_focused()
    }

    pub(crate) fn is_active(&self) -> bool {
        self.active
    }

    pub(crate) fn projects_inactive_display(&self) -> bool {
        self.inactive_display && !self.active
    }

    pub(crate) fn inactive_display_text_area(&self, wrap: Wrap) -> TextArea {
        let buffer = self
            .inactive_display_buffer
            .as_ref()
            .filter(|buffer| buffer.to_plain_text() == self.display_text())
            .cloned()
            .unwrap_or_else(|| text::Buffer::from_multiline_text(self.display_text().to_owned()));
        let state = buffer.initial_state();
        let mut area = TextArea::from_buffer(buffer, state)
            .with_wrap(wrap)
            .read_only();
        if let Some(focus) = self.focus {
            area = area.with_focus(focus);
        }
        area
    }

    pub(in crate::view) fn reuse_inactive_display_buffer_from(&mut self, previous: &Self) -> bool {
        if !self.projects_inactive_display()
            || !previous.projects_inactive_display()
            || self.display_text() != previous.display_text()
        {
            return false;
        }
        let Some(buffer) = previous.inactive_display_buffer.as_ref() else {
            return false;
        };
        self.inactive_display_buffer = Some(buffer.clone());
        true
    }

    pub fn focus_visible(&self) -> bool {
        self.focus_presentation.is_visible()
    }

    pub(in crate::view) fn focus_presentation(&self) -> focus::Presentation {
        self.focus_presentation
    }

    pub fn cursor(&self) -> Option<usize> {
        self.caret.as_ref().map(|caret| caret.cursor)
    }

    pub fn selection(&self) -> Option<Range<usize>> {
        self.caret
            .as_ref()
            .and_then(|caret| caret.selection.clone())
    }

    pub fn preedit(&self) -> Option<&text::Preedit> {
        self.preedit.as_ref()
    }

    pub(crate) fn caret_epoch(&self) -> Option<Instant> {
        self.caret_epoch
    }

    pub(in crate::view) fn same_scene_state(&self, other: &Self) -> bool {
        self.text == other.text
            && self.placeholder == other.placeholder
            && self.input == other.input
            && self.mode == other.mode
            && self.focus == other.focus
            && self.active == other.active
            && self.inactive_display == other.inactive_display
            && self.focus_presentation == other.focus_presentation
            && self.caret == other.caret
            && self.preedit == other.preedit
            && self.indicator_hint == other.indicator_hint
    }

    pub(crate) fn indicator_hint(&self) -> Option<&Hint> {
        self.indicator_hint.as_ref()
    }

    pub(crate) fn indicator_target(&self) -> Option<interaction::Target> {
        let hint = self.indicator_hint.as_ref()?;
        let owner = self.input_target()?;
        Some(interaction::Target::indicator(
            &owner,
            self.error_message().unwrap_or_else(|| hint.description()),
        ))
    }

    pub(crate) fn input_target(&self) -> Option<interaction::Target> {
        self.focus?.text_target()
    }

    pub(crate) fn is_invalid(&self) -> bool {
        self.indicator_hint
            .as_ref()
            .is_some_and(|hint| hint.tone() == super::super::Tone::Error)
    }

    pub(crate) fn error_message(&self) -> Option<&str> {
        self.is_invalid()
            .then(|| self.indicator_hint.as_ref().map(Hint::description))
            .flatten()
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
        bound: bool,
    ) {
        let Some(focus) = self.focus else {
            self.active = false;
            self.preedit = None;
            self.caret_epoch = None;
            self.indicator_hint = None;
            return;
        };

        let Some(target) = focus.text_target() else {
            self.active = false;
            self.caret = None;
            self.preedit = None;
            self.caret_epoch = None;
            self.indicator_hint = None;
            return;
        };
        self.project_input_feedback(interaction);
        let active = interaction.text_input().target() == Some(&target);
        self.active = active;
        if let Some(draft) = interaction.text_input().draft_for(&target)
            && (active || !bound || draft.text() == self.text)
        {
            self.text = draft.text().to_owned();
            self.caret = Some(Caret {
                cursor: draft.cursor(),
                selection: draft.selection(),
            });
        } else {
            self.caret = None;
        }
        self.preedit = interaction.text_input().preedit_for(&target).cloned();
        self.caret_epoch = interaction.text_input().caret_epoch_for(&target);
    }

    pub(in crate::view) fn project_input_feedback(
        &mut self,
        interaction: &interaction::Interaction,
    ) {
        self.indicator_hint = self.focus.and_then(|focus| {
            let target = focus.text_target()?;
            interaction
                .text_input()
                .feedback_for(&target)
                .map(|(severity, text)| Hint::from_feedback(severity, text.to_owned()))
        });
    }

    pub(in crate::view) fn project_focus(&mut self, focus: Option<&session::Focus>) {
        let focused = self
            .focus
            .as_ref()
            .is_some_and(|text_focus| focus.is_some_and(|focus| text_focus.same_target(focus)));
        self.focus_presentation = if focused {
            focus::Presentation::focused(focus.is_some_and(|focus| {
                focus.shows_focus_indicator()
                    || (self.mode.is_editable() && (!self.inactive_display || self.active))
            }))
        } else {
            focus::Presentation::default()
        };
        if self.is_focused() && self.caret.is_none() {
            self.caret = Some(Caret {
                cursor: self.text.len(),
                selection: None,
            });
        }
        if !self.is_focused() {
            self.caret = None;
            self.preedit = None;
            self.caret_epoch = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn blink_epoch_is_not_scene_content_state() {
        let first = TextBox::new("query");
        let mut second = first.clone();
        second.caret_epoch = Some(Instant::now() + Duration::from_secs(1));

        assert!(first.same_scene_state(&second));
        assert_ne!(first, second, "full model equality remains diagnostic");
    }

    #[test]
    fn inactive_table_display_reuses_only_equal_text_line_identity() {
        let previous = TextBox::new("42").with_inactive_display();
        let previous_line = previous
            .inactive_display_buffer
            .as_ref()
            .and_then(|buffer| buffer.line_layout_identity(0))
            .expect("previous inactive display line identity");
        let mut current = TextBox::new("42").with_inactive_display();
        assert!(current.reuse_inactive_display_buffer_from(&previous));
        assert_eq!(
            current
                .inactive_display_buffer
                .as_ref()
                .and_then(|buffer| buffer.line_layout_identity(0)),
            Some(previous_line)
        );

        let mut changed = TextBox::new("43").with_inactive_display();
        assert!(!changed.reuse_inactive_display_buffer_from(&previous));
        assert_ne!(
            changed
                .inactive_display_buffer
                .as_ref()
                .and_then(|buffer| buffer.line_layout_identity(0)),
            Some(previous_line)
        );
    }
}
