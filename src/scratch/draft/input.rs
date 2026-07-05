use std::collections::{HashMap, VecDeque};

use crate::text;

use super::{Change, State};
use crate::scratch::interaction::Target;

const DRAFT_LIMIT: usize = 64;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Input {
    target: Option<Target>,
    drafts: HashMap<Target, State>,
    draft_order: VecDeque<Target>,
    preedit: Option<text::edit::Preedit>,
}

impl Input {
    pub fn target(&self) -> Option<&Target> {
        self.target.as_ref()
    }

    pub fn draft(&self) -> Option<&State> {
        self.target
            .as_ref()
            .and_then(|target| self.draft_for(target))
    }

    pub fn draft_for(&self, target: &Target) -> Option<&State> {
        self.drafts.get(target)
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
            if self.target.as_ref() == Some(&target) && self.drafts.contains_key(&target) {
                let changed = self.preedit.is_some();
                self.preedit = None;
                return changed;
            }

            if self.target.as_ref() == Some(&target) {
                let changed = self.preedit.is_some() || self.target.is_some();
                self.preedit = None;
                self.target = None;
                return changed;
            }

            return false;
        }

        let target_changed = self.target.as_ref() != Some(&target);
        let changed = target_changed || self.preedit.as_ref() != Some(&preedit);
        self.target = Some(target);
        self.preedit = Some(preedit);
        changed
    }

    pub(in crate::scratch) fn edit(
        &mut self,
        target: Target,
        base: impl Into<String>,
        edit: text::edit::Edit,
    ) -> Change {
        let base = base.into();
        let target_changed = self.target.as_ref() != Some(&target);
        self.target = Some(target.clone());
        self.touch_draft(&target);

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
            .entry(target)
            .or_insert_with(|| State::new(base));
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

    pub(in crate::scratch) fn seal(&mut self, target: &Target) -> bool {
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
        if !self.drafts.contains_key(target) {
            return None;
        }

        let target_changed = self.target.as_ref() != Some(target);
        self.target = Some(target.clone());
        self.touch_draft(target);

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

        Some(Change::new(
            text,
            text_changed,
            cursor_changed,
            selection_changed,
            target_changed
                || changed_by_operation
                || text_changed
                || cursor_changed
                || selection_changed
                || preedit_cleared,
            false,
        ))
    }

    pub(in crate::scratch) fn clear(&mut self) -> bool {
        let changed = self.target.is_some() || !self.drafts.is_empty() || self.preedit.is_some();
        self.target = None;
        self.drafts.clear();
        self.draft_order.clear();
        self.preedit = None;
        changed
    }

    pub(in crate::scratch) fn clear_preedit(&mut self) -> bool {
        let changed = self.preedit.is_some();
        self.preedit = None;
        if self
            .target
            .as_ref()
            .is_none_or(|target| !self.drafts.contains_key(target))
        {
            self.target = None;
        }

        changed
    }

    pub(in crate::scratch) fn clear_draft(&mut self, target: &Target) -> bool {
        let draft_changed = self.drafts.remove(target).is_some();
        let order_len = self.draft_order.len();
        self.draft_order.retain(|existing| existing != target);
        let order_changed = self.draft_order.len() != order_len;
        let target_changed = if self.target.as_ref() == Some(target) {
            self.target = None;
            self.preedit = None;
            true
        } else {
            false
        };

        draft_changed || order_changed || target_changed
    }

    pub(in crate::scratch) fn deactivate(&mut self, target: &Target) -> bool {
        if self.target.as_ref() != Some(target) {
            return false;
        }

        self.preedit = None;
        self.target = None;
        true
    }

    pub(in crate::scratch) fn clear_unless(&mut self, target: &Target) -> bool {
        if self.target.as_ref() == Some(target) {
            return false;
        }

        let preedit_changed = self.preedit.take().is_some();
        if self.drafts.contains_key(target) {
            self.target = Some(target.clone());
            return true;
        }

        if self
            .target
            .as_ref()
            .is_some_and(|target| !self.drafts.contains_key(target))
        {
            self.target = None;
            return true;
        }

        preedit_changed
    }

    fn touch_draft(&mut self, target: &Target) {
        self.draft_order.retain(|existing| existing != target);
        self.draft_order.push_back(target.clone());

        while self.draft_order.len() > DRAFT_LIMIT {
            let Some(stale) = self.draft_order.pop_front() else {
                break;
            };
            if self.target.as_ref() != Some(&stale) {
                self.drafts.remove(&stale);
            }
        }
    }
}
