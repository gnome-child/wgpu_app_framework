mod change;
mod state;

pub use change::Change;
pub use state::State;

use crate::text;

use super::interaction::Target;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Input {
    target: Option<Target>,
    draft: Option<State>,
    preedit: Option<text::edit::Preedit>,
}

impl Input {
    pub fn target(&self) -> Option<&Target> {
        self.target.as_ref()
    }

    pub fn draft(&self) -> Option<&State> {
        self.draft.as_ref()
    }

    pub fn draft_for(&self, target: &Target) -> Option<&State> {
        (self.target.as_ref() == Some(target))
            .then_some(self.draft.as_ref())
            .flatten()
    }

    pub fn preedit(&self) -> Option<&text::edit::Preedit> {
        self.preedit.as_ref()
    }

    pub fn preedit_for(&self, target: &Target) -> Option<&text::edit::Preedit> {
        (self.target.as_ref() == Some(target))
            .then_some(self.preedit.as_ref())
            .flatten()
    }

    pub(in crate::scratch) fn set_preedit(
        &mut self,
        target: Target,
        preedit: text::edit::Preedit,
    ) -> bool {
        if preedit.text().is_empty() {
            if self.target.as_ref() == Some(&target) && self.draft.is_some() {
                let changed = self.preedit.is_some();
                self.preedit = None;
                return changed;
            }

            return if self.target.as_ref() == Some(&target) {
                self.clear()
            } else {
                false
            };
        }

        let target_changed = self.target.as_ref() != Some(&target);
        let changed = target_changed || self.preedit.as_ref() != Some(&preedit);
        self.target = Some(target);
        if target_changed {
            self.draft = None;
        }
        self.preedit = Some(preedit);
        changed
    }

    pub(in crate::scratch) fn edit(
        &mut self,
        target: Target,
        base: impl Into<String>,
        edit: text::edit::Edit,
    ) -> Change {
        let target_changed = self.target.as_ref() != Some(&target);
        if target_changed {
            self.target = Some(target);
            self.draft = None;
        }
        let draft = self.draft.get_or_insert_with(|| State::new(base.into()));
        let before_text = draft.text().to_owned();
        let before_cursor = draft.cursor();
        let before_selection = draft.selection();
        let submit = draft.apply(edit);
        let text = draft.text().to_owned();
        let cursor = draft.cursor();
        let selection = draft.selection();
        let text_changed = before_text != text;
        let cursor_changed = before_cursor != cursor;
        let selection_changed = before_selection != selection;
        let preedit_cleared = self.preedit.is_some();

        self.preedit = None;

        Change::new(
            text,
            text_changed,
            cursor_changed,
            selection_changed,
            target_changed
                || text_changed
                || cursor_changed
                || selection_changed
                || preedit_cleared,
            submit,
        )
    }

    pub(in crate::scratch) fn undo(&mut self, target: &Target) -> Option<Change> {
        self.change_existing(target, State::undo)
    }

    pub(in crate::scratch) fn redo(&mut self, target: &Target) -> Option<Change> {
        self.change_existing(target, State::redo)
    }

    fn change_existing(
        &mut self,
        target: &Target,
        change: impl FnOnce(&mut State) -> bool,
    ) -> Option<Change> {
        if self.target.as_ref() != Some(target) {
            return None;
        }

        let draft = self.draft.as_mut()?;
        let before_text = draft.text().to_owned();
        let before_cursor = draft.cursor();
        let before_selection = draft.selection();
        let changed_by_operation = change(draft);
        let text = draft.text().to_owned();
        let cursor = draft.cursor();
        let selection = draft.selection();
        let text_changed = before_text != text;
        let cursor_changed = before_cursor != cursor;
        let selection_changed = before_selection != selection;
        let preedit_cleared = self.preedit.is_some();

        self.preedit = None;

        Some(Change::new(
            text,
            text_changed,
            cursor_changed,
            selection_changed,
            changed_by_operation
                || text_changed
                || cursor_changed
                || selection_changed
                || preedit_cleared,
            false,
        ))
    }

    pub(in crate::scratch) fn clear(&mut self) -> bool {
        let changed = self.target.is_some() || self.draft.is_some() || self.preedit.is_some();
        self.target = None;
        self.draft = None;
        self.preedit = None;
        changed
    }

    pub(in crate::scratch) fn clear_unless(&mut self, target: &Target) -> bool {
        if self.target.as_ref() == Some(target) {
            return false;
        }

        self.clear()
    }
}
