use std::{collections::HashMap, time::Instant};

use crate::{feedback, text};

use super::{Change, State};
use crate::interaction::Target;

mod store;

use store::Store;

pub(crate) use store::DEFAULT_DRAFT_LIMIT;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Input {
    active: Option<Active>,
    drafts: Store,
    caret_epochs: HashMap<Target, Instant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Active {
    target: Target,
    preedit: Option<text::Preedit>,
}

impl Input {
    pub fn target(&self) -> Option<&Target> {
        self.active.as_ref().map(|active| &active.target)
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
        let (target_changed, preedit_changed) = self.set_active_target(target.clone());
        self.drafts.touch(&target, Some(&target));

        let draft_changed = if !self.drafts.contains(&target) {
            self.drafts.insert(target.clone(), State::new(base));
            true
        } else {
            false
        };
        let blink_changed = self.reset_caret_blink(target, Instant::now());

        target_changed || draft_changed || preedit_changed || blink_changed
    }

    #[cfg(test)]
    pub fn preedit(&self) -> Option<&text::Preedit> {
        self.active
            .as_ref()
            .and_then(|active| active.preedit.as_ref())
    }

    pub fn preedit_for(&self, target: &Target) -> Option<&text::Preedit> {
        self.active
            .as_ref()
            .filter(|active| &active.target == target)
            .and_then(|active| active.preedit.as_ref())
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
            if self.target() == Some(&target) && self.drafts.contains(&target) {
                let changed = self
                    .active
                    .as_mut()
                    .and_then(|active| active.preedit.take())
                    .is_some();
                let blink_changed = self.reset_caret_blink(target, Instant::now());
                return changed || blink_changed;
            }

            if self.target() == Some(&target) {
                self.active = None;
                self.reset_caret_blink(target, Instant::now());
                return true;
            }

            return false;
        }

        let target_changed = self.target() != Some(&target);
        let changed = target_changed || self.preedit_for(&target) != Some(&preedit);
        self.active = Some(Active {
            target: target.clone(),
            preedit: Some(preedit),
        });
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
        let (
            target_changed,
            preedit_cleared,
            text_changed,
            cursor_changed,
            selection_changed,
            submit,
        ) = {
            let (target_changed, preedit_cleared, entry) =
                self.prepare_draft(target.clone(), base.into());
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
                preedit_cleared,
                text_changed,
                cursor_changed,
                selection_changed,
                submit,
            )
        };

        self.finish_change(
            target,
            target_changed,
            preedit_cleared,
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
        let (target_changed, preedit_cleared, cursor_changed, selection_changed) = {
            let (target_changed, preedit_cleared, entry) =
                self.prepare_draft(target.clone(), base.into());
            let draft = entry.draft_mut();
            let before_cursor = draft.cursor();
            let before_selection = draft.selection();
            draft.select(operation);
            (
                target_changed,
                preedit_cleared,
                before_cursor != draft.cursor(),
                before_selection != draft.selection(),
            )
        };

        self.finish_change(
            target,
            target_changed,
            preedit_cleared,
            false,
            cursor_changed,
            selection_changed,
            false,
            false,
        )
    }

    fn prepare_draft(&mut self, target: Target, base: String) -> (bool, bool, &mut store::Entry) {
        let (target_changed, preedit_cleared) = self.set_active_target(target.clone());
        self.drafts.touch(&target, Some(&target));

        if target_changed
            && self
                .drafts
                .get(&target)
                .is_some_and(|entry| entry.draft().base_text() != base)
        {
            self.drafts.insert(target.clone(), State::new(base.clone()));
        }

        let entry = self.drafts.get_or_insert_with(target, || State::new(base));
        (target_changed, preedit_cleared, entry)
    }

    fn finish_change(
        &mut self,
        target: Target,
        target_changed: bool,
        preedit_cleared: bool,
        text_changed: bool,
        cursor_changed: bool,
        selection_changed: bool,
        operation_changed: bool,
        submit: bool,
    ) -> Change {
        let blink_changed = self.reset_caret_blink(target, Instant::now());

        Change::new(
            text_changed,
            selection_changed,
            target_changed
                || cursor_changed
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

        let (target_changed, preedit_cleared) = self.set_active_target(target.clone());
        self.drafts.touch(target, Some(target));

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
            preedit_cleared,
            text_changed,
            cursor_changed,
            selection_changed,
            changed_by_operation,
            false,
        ))
    }

    pub(crate) fn clear(&mut self) -> bool {
        let changed = self.active.is_some() || !self.drafts.is_empty();
        self.active = None;
        self.drafts.clear();
        if changed {
            self.caret_epochs.clear();
        }
        changed
    }

    pub(crate) fn clear_preedit(&mut self) -> bool {
        let changed = self
            .active
            .as_mut()
            .and_then(|active| active.preedit.take())
            .is_some();
        if self
            .target()
            .is_none_or(|target| !self.drafts.contains(target))
        {
            self.active = None;
        }

        changed
    }

    pub(crate) fn clear_draft(&mut self, target: &Target) -> bool {
        let store_changed = self.drafts.remove(target);
        let caret_changed = self.caret_epochs.remove(target).is_some();
        let target_changed = if self.target() == Some(target) {
            self.active = None;
            true
        } else {
            false
        };

        store_changed || caret_changed || target_changed
    }

    pub(crate) fn deactivate(&mut self, target: &Target) -> bool {
        if self.target() != Some(target) {
            return false;
        }

        self.active = None;
        self.caret_epochs.remove(target);
        true
    }

    pub(crate) fn deactivate_unless(&mut self, target: &Target) -> bool {
        if self.target() == Some(target) {
            return false;
        }

        self.active.take().is_some()
    }

    pub(crate) fn set_draft_limit(&mut self, limit: usize) {
        let active = self.target().cloned();
        self.drafts.set_limit(limit, active.as_ref());
    }

    pub(crate) fn prune_removed(
        &mut self,
        removed_nodes: &[crate::composition::tree::NodeId],
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
        let active_removed = self.target().is_some_and(|target| {
            target.matches_removed_identity(removed_nodes, removed_elements, removed_table_cells)
        });
        if active_removed {
            self.active = None;
        }

        store_changed || before_epochs != self.caret_epochs.len() || active_removed
    }

    fn set_active_target(&mut self, target: Target) -> (bool, bool) {
        let target_changed = self.target() != Some(&target);
        let preedit_cleared = self
            .active
            .as_ref()
            .is_some_and(|active| active.preedit.is_some());
        self.active = Some(Active {
            target,
            preedit: None,
        });
        (target_changed, preedit_cleared)
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
    fn preedit_lifetime_is_nested_under_the_active_target() {
        let target = Target::text_area_id("preedit-target");
        let mut input = Input::default();

        input.set_preedit(target.clone(), text::Preedit::new("compose", Some((0, 7))));
        assert_eq!(input.target(), Some(&target));
        assert!(input.preedit_for(&target).is_some());

        assert!(input.clear_preedit());
        assert_eq!(
            input.target(),
            None,
            "preedit-only activation retires as one unit"
        );

        input.activate(target.clone(), "base");
        input.set_preedit(target.clone(), text::Preedit::new("compose", Some((0, 7))));
        assert!(input.clear_preedit());
        assert_eq!(
            input.target(),
            Some(&target),
            "the retained draft keeps its target active"
        );
        assert_eq!(input.preedit_for(&target), None);
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
