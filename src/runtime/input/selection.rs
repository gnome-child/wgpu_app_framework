use super::super::Runtime;
use crate::{input, interaction, response, selection, state, window};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::runtime::input) fn handle_virtual_selection_key(
        &mut self,
        window: window::Id,
        key: input::Key,
        modifiers: input::Modifiers,
    ) -> Option<input::Outcome> {
        let focus = self.session.focused(window)?;
        if focus.is_text_input() {
            return None;
        }
        let composition = self.composition.get(window)?;
        let model = composition
            .selectable_virtual_list_for_focus(focus)?
            .clone();
        let table_columns = composition.view().table_columns(model.id());

        if !table_columns.is_empty() && key == input::Key::Enter {
            let cell = self.session.active_table_cell(window, model.id())?;
            let cell_focus = crate::session::Focus::table_cell(cell).keyboard();
            let focused = self
                .composition
                .get(window)
                .is_some_and(|composition| composition.view().contains_focus(cell_focus))
                && self.focus(window, cell_focus);
            return Some(input::Outcome::handled(
                false,
                focused
                    .then_some(response::Effect::Layout)
                    .unwrap_or_default(),
            ));
        }

        if !table_columns.is_empty()
            && matches!(key, input::Key::ArrowLeft | input::Key::ArrowRight)
        {
            let delta = if key == input::Key::ArrowLeft { -1 } else { 1 };
            let changed =
                self.session
                    .move_active_table_column(window, model.id(), &table_columns, delta);
            if changed {
                self.session
                    .request_invalidation(window, response::Invalidation::Rebuild);
            }
            return Some(input::Outcome::handled(
                false,
                changed
                    .then_some(response::Effect::Rebuild)
                    .unwrap_or_default(),
            ));
        }

        if let Some(first) = table_columns.first().copied() {
            self.session
                .ensure_active_table_column(window, model.id(), first);
        }

        let primary = modifiers.control() || modifiers.super_key();
        let changed =
            if primary && !modifiers.alt() && key.normalized() == input::Key::Character('a') {
                self.session.select_all_virtual_rows(window, &model)
            } else {
                let page = self
                    .layout_cache
                    .get(&window)
                    .and_then(|cached| {
                        cached
                            .layout
                            .virtual_list_page(model.id(), model.row_height())
                    })
                    .unwrap_or(10);
                let movement = match key {
                    input::Key::ArrowUp => selection::Move::Previous,
                    input::Key::ArrowDown => selection::Move::Next,
                    input::Key::Home => selection::Move::First,
                    input::Key::End => selection::Move::Last,
                    input::Key::PageUp => selection::Move::PagePrevious(page),
                    input::Key::PageDown => selection::Move::PageNext(page),
                    _ => return None,
                };
                self.session
                    .move_virtual_selection(window, &model, movement, modifiers.shift())
            };

        if changed {
            self.session.reveal_active_descendant(
                window,
                interaction::Target::scroll(model.id(), "Selected rows"),
            );
            self.session
                .request_invalidation(window, response::Invalidation::Rebuild);
        }
        Some(input::Outcome::handled(
            false,
            changed
                .then_some(response::Effect::Rebuild)
                .unwrap_or_default(),
        ))
    }
}
