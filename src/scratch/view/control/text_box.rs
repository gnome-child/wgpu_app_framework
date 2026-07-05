use crate::text;

use super::super::action::Action;
use crate::scratch::{interaction, session};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextBox {
    text: String,
    placeholder: Option<String>,
    focus: Option<session::Focus>,
    focused: bool,
    cursor: Option<usize>,
    preedit: Option<text::edit::Preedit>,
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

    pub(in crate::scratch::view) fn project_interaction(
        &mut self,
        interaction: &interaction::Interaction,
    ) {
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

    pub(in crate::scratch::view) fn project_focus(&mut self, focus: Option<session::Focus>) {
        self.focused = self.focus.is_some() && self.focus == focus;
    }
}
