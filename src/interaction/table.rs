use std::collections::HashMap;

use crate::table;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Tables {
    widths: HashMap<table::HeaderCell, i32>,
    active_columns: HashMap<crate::interaction::Id, crate::interaction::Id>,
    rejections: HashMap<table::Cell, String>,
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

    pub(crate) fn reject(&mut self, cell: table::Cell, reason: String) -> bool {
        if self.rejections.get(&cell) == Some(&reason) {
            return false;
        }
        self.rejections.insert(cell, reason);
        true
    }

    pub(crate) fn clear_rejection(&mut self, cell: table::Cell) -> bool {
        self.rejections.remove(&cell).is_some()
    }

    pub(crate) fn rejection(&self, cell: table::Cell) -> Option<&str> {
        self.rejections.get(&cell).map(String::as_str)
    }

    pub(crate) fn prune_removed(&mut self, cells: &[table::Cell]) -> bool {
        let before = self.rejections.len();
        self.rejections.retain(|cell, _| !cells.contains(cell));
        before != self.rejections.len()
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
