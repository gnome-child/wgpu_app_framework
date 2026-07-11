use crate::{table, window as app_window};

use super::Session;

impl Session {
    pub fn table_edit_error(&self, window: app_window::Id, cell: table::Cell) -> Option<&str> {
        self.window(window)?.interaction.tables().rejection(cell)
    }

    pub(crate) fn reject_table_edit(
        &mut self,
        window: app_window::Id,
        cell: table::Cell,
        reason: String,
    ) -> bool {
        self.window_mut(window)
            .is_some_and(|window| window.interaction.tables_mut().reject(cell, reason))
    }

    pub(crate) fn clear_table_edit_error(
        &mut self,
        window: app_window::Id,
        cell: table::Cell,
    ) -> bool {
        self.window_mut(window)
            .is_some_and(|window| window.interaction.tables_mut().clear_rejection(cell))
    }

    pub fn active_table_cell(
        &self,
        window: app_window::Id,
        table: crate::interaction::Id,
    ) -> Option<table::Cell> {
        let row = self.selection(window, table)?.active()?;
        let column = self
            .window(window)?
            .interaction
            .tables()
            .active_column(table)?;
        Some(table::Cell::new(table, row, column))
    }

    pub(crate) fn set_active_table_column(
        &mut self,
        window: app_window::Id,
        table: crate::interaction::Id,
        column: crate::interaction::Id,
    ) -> bool {
        self.window_mut(window).is_some_and(|window| {
            window
                .interaction
                .tables_mut()
                .set_active_column(table, column)
        })
    }

    pub(crate) fn ensure_active_table_column(
        &mut self,
        window: app_window::Id,
        table: crate::interaction::Id,
        column: crate::interaction::Id,
    ) -> bool {
        let Some(window) = self.window_mut(window) else {
            return false;
        };
        if window.interaction.tables().active_column(table).is_some() {
            return false;
        }
        window
            .interaction
            .tables_mut()
            .set_active_column(table, column)
    }

    pub(crate) fn move_active_table_column(
        &mut self,
        window: app_window::Id,
        table: crate::interaction::Id,
        columns: &[crate::interaction::Id],
        delta: isize,
    ) -> bool {
        let Some(window) = self.window_mut(window) else {
            return false;
        };
        let Some(first) = columns.first().copied() else {
            return false;
        };
        let current = window
            .interaction
            .tables()
            .active_column(table)
            .and_then(|active| columns.iter().position(|column| *column == active))
            .unwrap_or(0);
        let next = current
            .saturating_add_signed(delta)
            .min(columns.len().saturating_sub(1));
        window
            .interaction
            .tables_mut()
            .set_active_column(table, columns.get(next).copied().unwrap_or(first))
    }

    pub(crate) fn resize_table_column(
        &mut self,
        window: app_window::Id,
        column: table::HeaderCell,
        width: i32,
    ) -> bool {
        self.window_mut(window)
            .is_some_and(|window| window.interaction.tables_mut().set_width(column, width))
    }
}
