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
        let before = self
            .draft_for(&target)
            .cloned()
            .unwrap_or_else(|| State::new(base.into()));
        let mut draft = before.clone();
        let submit = draft.apply(edit);
        let text_changed = before.text() != draft.text();
        let cursor_changed = before.cursor() != draft.cursor();
        let target_changed = self.target.as_ref() != Some(&target);
        let preedit_cleared = self.preedit.is_some();

        self.target = Some(target);
        self.draft = Some(draft.clone());
        self.preedit = None;

        Change::new(
            draft,
            text_changed,
            cursor_changed,
            target_changed || text_changed || cursor_changed || preedit_cleared,
            submit,
        )
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
