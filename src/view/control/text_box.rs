use crate::text;

use super::super::Hint;
use super::super::action::Action;
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
    focused: bool,
    focus_visible: bool,
    cursor: Option<usize>,
    selection: Option<Range<usize>>,
    preedit: Option<text::Preedit>,
    caret_epoch: Option<Instant>,
    indicator_hint: Option<Hint>,
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
            focused: false,
            focus_visible: false,
            cursor: None,
            selection: None,
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
        self.focused
    }

    pub(crate) fn is_active(&self) -> bool {
        self.active
    }

    pub(crate) fn projects_inactive_display(&self) -> bool {
        self.inactive_display && !self.active
    }

    pub fn focus_visible(&self) -> bool {
        self.focus_visible
    }

    pub fn cursor(&self) -> Option<usize> {
        self.cursor
    }

    pub fn selection(&self) -> Option<Range<usize>> {
        self.selection.clone()
    }

    pub fn preedit(&self) -> Option<&text::Preedit> {
        self.preedit.as_ref()
    }

    pub(crate) fn caret_epoch(&self) -> Option<Instant> {
        self.caret_epoch
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
        Some(interaction::Target::text_area(self.focus?))
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

        let target = interaction::Target::text_area(focus);
        self.project_input_feedback(interaction);
        let active = interaction.text_input().target() == Some(&target);
        self.active = active;
        if let Some(draft) = interaction.text_input().draft_for(&target)
            && (active || !bound || draft.text() == self.text)
        {
            self.text = draft.text().to_owned();
            self.cursor = Some(draft.cursor());
            self.selection = draft.selection();
        } else {
            self.cursor = None;
            self.selection = None;
        }
        self.preedit = interaction.text_input().preedit_for(&target).cloned();
        self.caret_epoch = interaction.text_input().caret_epoch_for(&target);
    }

    pub(in crate::view) fn project_input_feedback(
        &mut self,
        interaction: &interaction::Interaction,
    ) {
        self.indicator_hint = self.focus.and_then(|focus| {
            interaction
                .text_input()
                .feedback_for(&interaction::Target::text_area(focus))
                .map(|(severity, text)| Hint::from_feedback(severity, text.to_owned()))
        });
    }

    pub(in crate::view) fn project_focus(&mut self, focus: Option<&session::Focus>) {
        self.focused = self
            .focus
            .as_ref()
            .is_some_and(|text_focus| focus.is_some_and(|focus| text_focus.same_target(focus)));
        self.focus_visible = self.focused
            && focus.is_some_and(|focus| {
                focus.shows_focus_indicator()
                    || (self.mode.is_editable() && (!self.inactive_display || self.active))
            });
        if self.focused && self.cursor.is_none() {
            self.cursor = Some(self.text.len());
        }
        if !self.focused {
            self.cursor = None;
            self.selection = None;
            self.preedit = None;
            self.caret_epoch = None;
        }
    }
}
