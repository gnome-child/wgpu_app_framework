use crate::text;

use super::super::action::Action;
use crate::{interaction, session};
use std::ops::Range;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBox {
    text: String,
    placeholder: Option<String>,
    focus: Option<session::Focus>,
    focused: bool,
    focus_visible: bool,
    cursor: Option<usize>,
    selection: Option<Range<usize>>,
    preedit: Option<text::edit::Preedit>,
    caret_epoch: Option<Instant>,
}

impl TextBox {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            placeholder: None,
            focus: None,
            focused: false,
            focus_visible: false,
            cursor: None,
            selection: None,
            preedit: None,
            caret_epoch: None,
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

    pub fn focus_visible(&self) -> bool {
        self.focus_visible
    }

    pub fn cursor(&self) -> Option<usize> {
        self.cursor
    }

    pub fn selection(&self) -> Option<Range<usize>> {
        self.selection.clone()
    }

    pub fn preedit(&self) -> Option<&text::edit::Preedit> {
        self.preedit.as_ref()
    }

    pub(crate) fn caret_epoch(&self) -> Option<Instant> {
        self.caret_epoch
    }

    pub(crate) fn focus_action(&self) -> Option<Action> {
        Action::text_focus(self.focus)
    }

    pub(crate) fn click_action(&self, position: text::buffer::Position) -> Option<Action> {
        Action::text_click(self.focus, position)
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
            self.preedit = None;
            self.caret_epoch = None;
            return;
        };

        let target = interaction::Target::text_area(focus);
        let active = interaction.text_input().target() == Some(&target);
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

    pub(in crate::view) fn project_focus(&mut self, focus: Option<&session::Focus>) {
        self.focused = self
            .focus
            .as_ref()
            .is_some_and(|text_focus| focus.is_some_and(|focus| text_focus.same_target(focus)));
        self.focus_visible =
            self.focused && focus.is_some_and(|focus| focus.shows_focus_indicator());
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
