use crate::{interaction, list, selection::Selection, window as app_window};

use super::Session;

impl Session {
    pub fn selection(&self, window: app_window::Id, list: interaction::Id) -> Option<&Selection> {
        self.window(window)?.interaction.selections().get(list)
    }

    pub(crate) fn reconcile_virtual_selections(
        &mut self,
        window: app_window::Id,
        models: &[list::State],
    ) -> bool {
        self.window_mut(window)
            .is_some_and(|window| window.interaction.selections_mut().reconcile(models))
    }

    pub(crate) fn virtual_selection_snapshot(
        &self,
        window: app_window::Id,
    ) -> Vec<(interaction::Id, Selection)> {
        self.window(window)
            .map(|window| window.interaction.selections().snapshot())
            .unwrap_or_default()
    }

    pub(crate) fn select_virtual_row(
        &mut self,
        window: app_window::Id,
        model: &list::State,
        key: list::Key,
        index: usize,
        extend: bool,
        toggle: bool,
    ) -> bool {
        let Some(window) = self.window_mut(window) else {
            return false;
        };
        let selection = window
            .interaction
            .selections_mut()
            .get_mut_or_insert(model.id());
        model.select_row(selection, key, index, extend, toggle)
    }

    pub(crate) fn select_all_virtual_rows(
        &mut self,
        window: app_window::Id,
        model: &list::State,
    ) -> bool {
        let Some(window) = self.window_mut(window) else {
            return false;
        };
        let selection = window
            .interaction
            .selections_mut()
            .get_mut_or_insert(model.id());
        model.select_all(selection)
    }

    pub(crate) fn move_virtual_selection(
        &mut self,
        window: app_window::Id,
        model: &list::State,
        movement: crate::selection::Move,
        extend: bool,
    ) -> bool {
        let Some(window) = self.window_mut(window) else {
            return false;
        };
        let selection = window
            .interaction
            .selections_mut()
            .get_mut_or_insert(model.id());
        model.move_selection(selection, movement, extend)
    }
}
