use std::collections::HashMap;

use crate::{feedback, table};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Tables {
    widths: HashMap<table::HeaderCell, i32>,
    active_columns: HashMap<crate::interaction::Id, crate::interaction::Id>,
    editing: Option<table::Cell>,
    feedback: HashMap<table::Cell, feedback::Stack>,
}

impl Tables {
    pub(crate) fn width(&self, column: table::HeaderCell) -> Option<i32> {
        self.widths.get(&column).copied()
    }

    pub(crate) fn set_width(&mut self, column: table::HeaderCell, width: i32) -> bool {
        let width = width.max(0);
        if self.width(column) == Some(width) {
            return false;
        }
        self.widths.insert(column, width);
        true
    }

    pub(crate) fn active_column(
        &self,
        table: crate::interaction::Id,
    ) -> Option<crate::interaction::Id> {
        self.active_columns.get(&table).copied()
    }

    pub(crate) fn set_active_column(
        &mut self,
        table: crate::interaction::Id,
        column: crate::interaction::Id,
    ) -> bool {
        if self.active_column(table) == Some(column) {
            return false;
        }
        self.active_columns.insert(table, column);
        true
    }

    pub(crate) fn editing(&self) -> Option<table::Cell> {
        self.editing
    }

    pub(crate) fn begin_edit(&mut self, cell: table::Cell) -> bool {
        if self.editing == Some(cell) {
            return false;
        }
        self.editing = Some(cell);
        true
    }

    pub(crate) fn finish_edit(&mut self, cell: table::Cell) -> bool {
        if self.editing != Some(cell) {
            return false;
        }
        self.editing = None;
        true
    }

    pub(crate) fn reject(&mut self, cell: table::Cell, reason: impl std::fmt::Display) -> bool {
        self.feedback
            .entry(cell)
            .or_default()
            .report(feedback::Severity::Error, reason)
    }

    pub(crate) fn clear_rejection(&mut self, cell: table::Cell) -> bool {
        let changed = self
            .feedback
            .get_mut(&cell)
            .is_some_and(|feedback| feedback.clear(feedback::Severity::Error));
        if self
            .feedback
            .get(&cell)
            .is_some_and(feedback::Stack::is_empty)
        {
            self.feedback.remove(&cell);
        }
        changed
    }

    pub(crate) fn rejection(&self, cell: table::Cell) -> Option<&str> {
        self.feedback
            .get(&cell)?
            .winner()
            .filter(|entry| entry.severity() == feedback::Severity::Error)
            .map(feedback::Entry::text)
    }

    pub(crate) fn feedback(&self, cell: table::Cell) -> Option<(feedback::Severity, &str)> {
        self.feedback
            .get(&cell)
            .and_then(feedback::Stack::winner)
            .map(|entry| (entry.severity(), entry.text()))
    }

    pub(crate) fn prune_removed(&mut self, cells: &[table::Cell]) -> bool {
        let before = self.feedback.len();
        self.feedback.retain(|cell, _| !cells.contains(cell));
        let edit_removed = self.editing.is_some_and(|cell| cells.contains(&cell));
        if edit_removed {
            self.editing = None;
        }
        before != self.feedback.len() || edit_removed
    }

    pub(crate) fn snapshot(
        &self,
    ) -> (
        Vec<(table::HeaderCell, i32)>,
        Vec<(crate::interaction::Id, crate::interaction::Id)>,
    ) {
        let widths = self
            .widths
            .iter()
            .map(|(key, value)| (*key, *value))
            .collect();
        let active_columns = self
            .active_columns
            .iter()
            .map(|(table, column)| (*table, *column))
            .collect();
        (widths, active_columns)
    }

    pub(crate) fn restore(
        &mut self,
        widths: Vec<(table::HeaderCell, i32)>,
        active_columns: Vec<(crate::interaction::Id, crate::interaction::Id)>,
    ) {
        self.widths = widths.into_iter().collect();
        self.active_columns = active_columns.into_iter().collect();
    }
}
