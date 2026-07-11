use std::{collections::HashMap, time::Instant};

use crate::text;

use super::{Change, State};
use crate::interaction::Target;

mod store;

use store::Store;

pub(crate) use store::DEFAULT_DRAFT_LIMIT;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Input {
    target: Option<Target>,
    drafts: Store,
    preedit: Option<text::edit::Preedit>,
    caret_epochs: HashMap<Target, Instant>,
}

impl Input {
    pub fn target(&self) -> Option<&Target> {
        self.target.as_ref()
    }

    pub fn draft_for(&self, target: &Target) -> Option<&State> {
        self.drafts.get(target)
    }

    #[cfg(test)]
    pub fn preedit(&self) -> Option<&text::edit::Preedit> {
        self.preedit.as_ref()
    }

    pub fn preedit_for(&self, target: &Target) -> Option<&text::edit::Preedit> {
        (self.target.as_ref() == Some(target))
            .then_some(self.preedit.as_ref())
            .flatten()
    }

    pub fn caret_epoch_for(&self, target: &Target) -> Option<Instant> {
        self.caret_epochs.get(target).copied()
    }

    pub(crate) fn reset_caret_blink(&mut self, target: Target, now: Instant) -> bool {
        let changed = self.caret_epochs.get(&target).copied() != Some(now);
        self.caret_epochs.insert(target, now);
        changed
    }

    pub(crate) fn set_preedit(&mut self, target: Target, preedit: text::edit::Preedit) -> bool {
        if preedit.text().is_empty() {
            if self.target.as_ref() == Some(&target) && self.drafts.contains(&target) {
                let changed = self.preedit.is_some();
                self.preedit = None;
                let blink_changed = self.reset_caret_blink(target, Instant::now());
                return changed || blink_changed;
            }

            if self.target.as_ref() == Some(&target) {
                let changed = self.preedit.is_some() || self.target.is_some();
                self.preedit = None;
                self.target = None;
                let blink_changed = self.reset_caret_blink(target, Instant::now());
                return changed || blink_changed;
            }

            return false;
        }

        let target_changed = self.target.as_ref() != Some(&target);
        let changed = target_changed || self.preedit.as_ref() != Some(&preedit);
        self.target = Some(target.clone());
        self.preedit = Some(preedit);
        let blink_changed = self.reset_caret_blink(target, Instant::now());
        changed || blink_changed
    }

    pub(crate) fn edit(
        &mut self,
        target: Target,
        base: impl Into<String>,
        edit: text::edit::Edit,
    ) -> Change {
        let base = base.into();
        let target_changed = self.target.as_ref() != Some(&target);
        self.target = Some(target.clone());
        self.drafts.touch(&target, self.target.as_ref());

        if target_changed
            && self
                .drafts
                .get(&target)
                .is_some_and(|draft| draft.base_text() != base)
        {
            self.drafts.insert(target.clone(), State::new(base.clone()));
        }

        let draft = self
            .drafts
            .get_or_insert_with(target.clone(), || State::new(base));
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
        let blink_changed = self.reset_caret_blink(target.clone(), Instant::now());

        Change::new(
            text_changed,
            selection_changed,
            target_changed
                || text_changed
                || cursor_changed
                || selection_changed
                || preedit_cleared
                || blink_changed,
            submit,
        )
    }

    pub(crate) fn undo(&mut self, target: &Target) -> Option<Change> {
        self.change_existing(target, State::undo)
    }

    pub(crate) fn redo(&mut self, target: &Target) -> Option<Change> {
        self.change_existing(target, State::redo)
    }

    pub(crate) fn seal(&mut self, target: &Target) -> bool {
        let Some(draft) = self.drafts.get_mut(target) else {
            return false;
        };

        draft.seal()
    }

    fn change_existing(
        &mut self,
        target: &Target,
        change: impl FnOnce(&mut State) -> bool,
    ) -> Option<Change> {
        if !self.drafts.contains(target) {
            return None;
        }

        let target_changed = self.target.as_ref() != Some(target);
        self.target = Some(target.clone());
        self.drafts.touch(target, self.target.as_ref());

        let draft = self.drafts.get_mut(target)?;
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
        let blink_changed = self.reset_caret_blink(target.clone(), Instant::now());

        Some(Change::new(
            text_changed,
            selection_changed,
            target_changed
                || changed_by_operation
                || text_changed
                || cursor_changed
                || selection_changed
                || preedit_cleared
                || blink_changed,
            false,
        ))
    }

    pub(crate) fn clear(&mut self) -> bool {
        let changed = self.target.is_some() || !self.drafts.is_empty() || self.preedit.is_some();
        self.target = None;
        self.drafts.clear();
        self.preedit = None;
        if changed {
            self.caret_epochs.clear();
        }
        changed
    }

    pub(crate) fn clear_preedit(&mut self) -> bool {
        let changed = self.preedit.is_some();
        self.preedit = None;
        if self
            .target
            .as_ref()
            .is_none_or(|target| !self.drafts.contains(target))
        {
            self.target = None;
        }

        changed
    }

    pub(crate) fn clear_draft(&mut self, target: &Target) -> bool {
        let store_changed = self.drafts.remove(target);
        let caret_changed = self.caret_epochs.remove(target).is_some();
        let target_changed = if self.target.as_ref() == Some(target) {
            self.target = None;
            self.preedit = None;
            true
        } else {
            false
        };

        store_changed || caret_changed || target_changed
    }

    pub(crate) fn deactivate(&mut self, target: &Target) -> bool {
        if self.target.as_ref() != Some(target) {
            return false;
        }

        self.preedit = None;
        self.target = None;
        self.caret_epochs.remove(target);
        true
    }

    pub(crate) fn clear_unless(&mut self, target: &Target) -> bool {
        if self.target.as_ref() == Some(target) {
            return false;
        }

        let preedit_changed = self.preedit.take().is_some();
        if self.drafts.contains(target) {
            self.target = Some(target.clone());
            return true;
        }

        if self
            .target
            .as_ref()
            .is_some_and(|target| !self.drafts.contains(target))
        {
            self.target = None;
            return true;
        }

        preedit_changed
    }

    pub(crate) fn set_draft_limit(&mut self, limit: usize) {
        self.drafts.set_limit(limit, self.target.as_ref());
    }

    pub(crate) fn prune_removed(
        &mut self,
        removed_nodes: &[crate::composition::NodeId],
        removed_elements: &[crate::interaction::Id],
        removed_table_cells: &[crate::table::Cell],
    ) -> bool {
        let store_changed =
            self.drafts
                .prune_removed(removed_nodes, removed_elements, removed_table_cells);
        let before_epochs = self.caret_epochs.len();
        self.caret_epochs.retain(|target, _| {
            !target.matches_removed_identity(removed_nodes, removed_elements, removed_table_cells)
        });
        let active_removed = self.target.as_ref().is_some_and(|target| {
            target.matches_removed_identity(removed_nodes, removed_elements, removed_table_cells)
        });
        if active_removed {
            self.target = None;
            self.preedit = None;
        }

        store_changed || before_epochs != self.caret_epochs.len() || active_removed
    }
}
