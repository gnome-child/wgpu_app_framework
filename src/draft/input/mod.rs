use std::{collections::HashMap, time::Instant};

use crate::{feedback, text};

use super::{Change, State};
use crate::interaction::Target;

mod store;

use store::Store;

pub(crate) use store::DEFAULT_DRAFT_LIMIT;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Input {
    target: Option<Target>,
    drafts: Store,
    preedit: Option<text::Preedit>,
    caret_epochs: HashMap<Target, Instant>,
}

impl Input {
    pub fn target(&self) -> Option<&Target> {
        self.target.as_ref()
    }

    pub fn draft_for(&self, target: &Target) -> Option<&State> {
        self.drafts.get(target).map(store::Entry::draft)
    }

    pub fn feedback_for(&self, target: &Target) -> Option<(feedback::Severity, &str)> {
        self.drafts.get(target)?.feedback().winner()
    }

    pub(crate) fn report_feedback(
        &mut self,
        target: &Target,
        severity: feedback::Severity,
        text: String,
    ) -> bool {
        self.drafts
            .get_mut(target)
            .is_some_and(|entry| entry.feedback_mut().report_text(severity, text))
    }

    pub(crate) fn activate(&mut self, target: Target, base: impl Into<String>) -> bool {
        let base = base.into();
        let target_changed = self.target.as_ref() != Some(&target);
        self.target = Some(target.clone());
        self.drafts.touch(&target, self.target.as_ref());

        let draft_changed = if !self.drafts.contains(&target) {
            self.drafts.insert(target.clone(), State::new(base));
            true
        } else {
            false
        };
        let preedit_changed = self.preedit.take().is_some();
        let blink_changed = self.reset_caret_blink(target, Instant::now());

        target_changed || draft_changed || preedit_changed || blink_changed
    }

    #[cfg(test)]
    pub fn preedit(&self) -> Option<&text::Preedit> {
        self.preedit.as_ref()
    }

    pub fn preedit_for(&self, target: &Target) -> Option<&text::Preedit> {
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

    pub(crate) fn set_preedit(&mut self, target: Target, preedit: text::Preedit) -> bool {
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
        edit: text::Edit,
        input: text::Input,
    ) -> Change {
        let (target_changed, text_changed, cursor_changed, selection_changed, submit) = {
            let (target_changed, entry) = self.prepare_draft(target.clone(), base.into());
            let draft = entry.draft_mut();
            let before_text = draft.text().to_owned();
            let before_cursor = draft.cursor();
            let before_selection = draft.selection();
            let submit = draft.apply(edit, input);
            let text_changed = before_text != draft.text();
            let cursor_changed = before_cursor != draft.cursor();
            let selection_changed = before_selection != draft.selection();

            if text_changed {
                entry.feedback_mut().clear(feedback::Severity::Error);
            }

            (
                target_changed,
                text_changed,
                cursor_changed,
                selection_changed,
                submit,
            )
        };

        self.finish_change(
            target,
            target_changed,
            text_changed,
            cursor_changed,
            selection_changed,
            false,
            submit,
        )
    }

    pub(crate) fn select(
        &mut self,
        target: Target,
        base: impl Into<String>,
        operation: text::selection::Operation,
    ) -> Change {
        let (target_changed, cursor_changed, selection_changed) = {
            let (target_changed, entry) = self.prepare_draft(target.clone(), base.into());
            let draft = entry.draft_mut();
            let before_cursor = draft.cursor();
            let before_selection = draft.selection();
            draft.select(operation);
            (
                target_changed,
                before_cursor != draft.cursor(),
                before_selection != draft.selection(),
            )
        };

        self.finish_change(
            target,
            target_changed,
            false,
            cursor_changed,
            selection_changed,
            false,
            false,
        )
    }

    fn prepare_draft(&mut self, target: Target, base: String) -> (bool, &mut store::Entry) {
        let target_changed = self.target.as_ref() != Some(&target);
        self.target = Some(target.clone());
        self.drafts.touch(&target, self.target.as_ref());

        if target_changed
            && self
                .drafts
                .get(&target)
                .is_some_and(|entry| entry.draft().base_text() != base)
        {
            self.drafts.insert(target.clone(), State::new(base.clone()));
        }

        let entry = self.drafts.get_or_insert_with(target, || State::new(base));
        (target_changed, entry)
    }

    fn finish_change(
        &mut self,
        target: Target,
        target_changed: bool,
        text_changed: bool,
        cursor_changed: bool,
        selection_changed: bool,
        operation_changed: bool,
        submit: bool,
    ) -> Change {
        let preedit_cleared = self.preedit.take().is_some();
        let blink_changed = self.reset_caret_blink(target, Instant::now());

        Change::new(
            text_changed,
            selection_changed,
            target_changed
                || text_changed
                || cursor_changed
                || selection_changed
                || operation_changed
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
        let Some(entry) = self.drafts.get_mut(target) else {
            return false;
        };

        let draft_changed = entry.draft_mut().seal();
        let feedback_changed = entry.feedback_mut().clear_all();
        draft_changed || feedback_changed
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

        let (text_changed, cursor_changed, selection_changed, changed_by_operation) = {
            let entry = self.drafts.get_mut(target)?;
            let draft = entry.draft_mut();
            let before_text = draft.text().to_owned();
            let before_cursor = draft.cursor();
            let before_selection = draft.selection();
            let changed_by_operation = change(draft);
            let text_changed = before_text != draft.text();
            let cursor_changed = before_cursor != draft.cursor();
            let selection_changed = before_selection != draft.selection();

            if text_changed {
                entry.feedback_mut().clear(feedback::Severity::Error);
            }

            (
                text_changed,
                cursor_changed,
                selection_changed,
                changed_by_operation,
            )
        };

        Some(self.finish_change(
            target.clone(),
            target_changed,
            text_changed,
            cursor_changed,
            selection_changed,
            changed_by_operation,
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

    pub(crate) fn deactivate_unless(&mut self, target: &Target) -> bool {
        if self.target.as_ref() == Some(target) {
            return false;
        }

        let preedit_changed = self.preedit.take().is_some();
        if self.target.take().is_some() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_clears_preedit_without_entering_mutation_history() {
        let target = Target::text_area_id("selection-draft");
        let mut input = Input::default();

        input.activate(target.clone(), "alpha");
        input.set_preedit(target.clone(), text::Preedit::new("compose", Some((0, 7))));

        let change = input.select(
            target.clone(),
            "alpha",
            text::selection::Operation::SelectAll,
        );
        let draft = input.draft_for(&target).expect("selection keeps the draft");

        assert!(!change.text_changed());
        assert!(change.selection_changed());
        assert!(change.changed());
        assert_eq!(input.preedit(), None);
        assert_eq!(draft.text(), "alpha");
        assert_eq!(draft.selection(), Some(0..5));
        assert!(!draft.can_undo(), "selection is not mutation history");
    }

    #[test]
    fn rejection_cannot_outlive_its_draft() {
        let first = Target::text_area_id("first-draft");
        let second = Target::text_area_id("second-draft");
        let removed = Target::text_area_id("removed-draft");
        let mut input = Input::default();

        input.activate(first.clone(), "old");
        assert!(input.report_feedback(
            &first,
            feedback::Severity::Warning,
            "still local".to_owned(),
        ));
        assert!(input.report_feedback(
            &first,
            feedback::Severity::Error,
            "invalid value".to_owned(),
        ));
        assert_eq!(
            input.feedback_for(&first),
            Some((feedback::Severity::Error, "invalid value")),
            "an error must win without destroying lower-severity retained feedback"
        );

        input.edit(
            first.clone(),
            "old",
            text::Edit::replace_range(0..3, "new"),
            text::Input::unrestricted(),
        );
        assert_eq!(
            input.feedback_for(&first),
            Some((feedback::Severity::Warning, "still local")),
            "editing the rejected text clears its error while preserving unrelated feedback"
        );
        assert!(input.seal(&first));
        assert_eq!(
            input.feedback_for(&first),
            None,
            "a successful commit clears every fact owned by the sealed draft"
        );

        assert!(input.report_feedback(
            &first,
            feedback::Severity::Error,
            "invalid again".to_owned(),
        ));
        assert!(input.clear_draft(&first));
        assert_eq!(input.feedback_for(&first), None);

        input.activate(first.clone(), "first");
        assert!(input.report_feedback(&first, feedback::Severity::Error, "evict me".to_owned(),));
        assert!(input.deactivate(&first));
        input.set_draft_limit(1);
        input.activate(second, "second");
        assert_eq!(input.draft_for(&first), None);
        assert_eq!(input.feedback_for(&first), None);

        input.activate(removed.clone(), "removed");
        assert!(input.report_feedback(&removed, feedback::Severity::Error, "prune me".to_owned(),));
        assert!(input.prune_removed(&[], &[crate::interaction::Id::new("removed-draft")], &[]));
        assert_eq!(input.draft_for(&removed), None);
        assert_eq!(input.feedback_for(&removed), None);

        let removed_cell = crate::table::Cell::new(
            crate::interaction::Id::new("removed-table"),
            crate::virtual_list::Key::new(7),
            crate::interaction::Id::new("removed-column"),
        );
        let removed_cell_target = Target::table_cell_editor(removed_cell);
        input.activate(removed_cell_target.clone(), "cell");
        assert!(input.report_feedback(
            &removed_cell_target,
            feedback::Severity::Error,
            "remove the row".to_owned(),
        ));
        assert!(input.prune_removed(&[], &[], &[removed_cell]));
        assert_eq!(input.draft_for(&removed_cell_target), None);
        assert_eq!(input.feedback_for(&removed_cell_target), None);

        let destroyed = Target::text_area_id("destroyed-draft");
        input.activate(destroyed.clone(), "destroyed");
        assert!(input.report_feedback(
            &destroyed,
            feedback::Severity::Error,
            "destroy me".to_owned(),
        ));
        assert!(input.clear());
        assert_eq!(input.feedback_for(&destroyed), None);
    }
}
