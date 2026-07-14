use super::super::Runtime;
use crate::{
    command::Error, input, interaction, response, selection, session, state, virtual_list, window,
};

#[derive(Clone, Copy)]
enum CellMove {
    Column(isize),
    Row(selection::Move),
    RowEdge { last: bool, table_edge: bool },
    Linear { reverse: bool },
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::runtime::input) fn handle_table_edit_key(
        &mut self,
        window: window::Id,
        key: input::Key,
        modifiers: input::Modifiers,
    ) -> std::result::Result<Option<input::Outcome>, Error> {
        let Some(cell) = self
            .session
            .interaction(window)
            .and_then(|interaction| interaction.text_input().target())
            .and_then(interaction::Target::table_cell)
        else {
            return Ok(None);
        };
        if !matches!(key, input::Key::Enter | input::Key::Tab) {
            return Ok(None);
        }

        let transition = self.commit_and_deactivate_focused_text_box(window)?;
        if transition
            .as_ref()
            .is_some_and(|transition| !transition.is_accepted())
        {
            return Ok(transition.map(|transition| transition.into_outcome()));
        }
        let committed = transition
            .map(|transition| transition.into_outcome())
            .unwrap_or_else(input::Outcome::ignored);

        let movement = if key == input::Key::Tab {
            CellMove::Linear {
                reverse: modifiers.shift(),
            }
        } else {
            CellMove::Row(if modifiers.shift() {
                selection::Move::Previous
            } else {
                selection::Move::Next
            })
        };
        let moved = self.move_table_cell(window, cell.table(), movement, true);
        if key == input::Key::Tab && !moved {
            let tabbed = self.handle_tab_focus(window, modifiers.shift())?;
            return Ok(Some(merge_outcomes(committed, tabbed)));
        }

        Ok(Some(if moved {
            merge_outcomes(
                committed,
                input::Outcome::handled(false, response::Effect::Rebuild),
            )
        } else {
            committed
        }))
    }

    pub(in crate::runtime::input) fn handle_virtual_selection_key(
        &mut self,
        window: window::Id,
        key: input::Key,
        modifiers: input::Modifiers,
    ) -> Option<input::Outcome> {
        let focus = self.session.focused(window)?;
        if self
            .session
            .interaction(window)
            .and_then(|interaction| interaction.text_input().target())
            .and_then(interaction::Target::table_cell)
            .is_some()
        {
            return None;
        }
        let model = self
            .composition
            .get(window)?
            .selectable_virtual_list_for_focus(focus)?
            .clone();
        let table_columns = self
            .composition
            .get(window)?
            .view()
            .table_columns(model.id());

        if !table_columns.is_empty() && matches!(key, input::Key::Enter | input::Key::F2) {
            let cell = focus
                .table_cell_identity()
                .or_else(|| self.session.active_table_cell(window, model.id()))?;
            let editable = self
                .text_draft_base(window, session::Focus::table_cell(cell))
                .is_some();
            if !editable {
                return None;
            }
            if let Some(index) = model.index_of(cell.row()) {
                self.session
                    .select_virtual_row(window, &model, cell.row(), index, false, false);
            }
            self.session
                .set_active_table_column(window, cell.table(), cell.column());
            let focused = self
                .session
                .focus(window, session::Focus::table_cell(cell).keyboard());
            let focus = session::Focus::table_cell(cell);
            let activated = self
                .text_draft_base(window, focus)
                .is_some_and(|base| self.session.activate_text_draft(window, focus, base));
            if activated || focused {
                self.session
                    .request_invalidation(window, response::effect::Invalidation::Rebuild);
            }
            return Some(input::Outcome::handled(
                false,
                (activated || focused)
                    .then_some(response::Effect::Rebuild)
                    .unwrap_or_default(),
            ));
        }

        if let Some(first) = table_columns.first().copied() {
            self.session
                .ensure_active_table_column(window, model.id(), first);
        }

        let primary = modifiers.control() || modifiers.super_key();
        if !table_columns.is_empty() {
            let movement = match key {
                input::Key::ArrowLeft => Some(CellMove::Column(-1)),
                input::Key::ArrowRight => Some(CellMove::Column(1)),
                input::Key::ArrowUp => Some(CellMove::Row(selection::Move::Previous)),
                input::Key::ArrowDown => Some(CellMove::Row(selection::Move::Next)),
                input::Key::Home => Some(CellMove::RowEdge {
                    last: false,
                    table_edge: primary,
                }),
                input::Key::End => Some(CellMove::RowEdge {
                    last: true,
                    table_edge: primary,
                }),
                input::Key::PageUp => Some(CellMove::Row(selection::Move::PagePrevious(
                    self.table_page(window, &model),
                ))),
                input::Key::PageDown => Some(CellMove::Row(selection::Move::PageNext(
                    self.table_page(window, &model),
                ))),
                input::Key::Tab => Some(CellMove::Linear {
                    reverse: modifiers.shift(),
                }),
                _ => None,
            };
            if let Some(movement) = movement {
                let moved = self.move_table_cell(window, model.id(), movement, false);
                if key == input::Key::Tab && !moved {
                    return Some(self.leave_table(window, model.id(), modifiers.shift()));
                }
                return Some(input::Outcome::handled(
                    false,
                    moved
                        .then_some(response::Effect::Rebuild)
                        .unwrap_or_default(),
                ));
            }
        }

        let changed = if primary
            && !modifiers.alt()
            && key.normalized() == input::Key::Character('a')
        {
            self.session.select_all_virtual_rows(window, &model)
        } else {
            let movement = match key {
                input::Key::ArrowUp => selection::Move::Previous,
                input::Key::ArrowDown => selection::Move::Next,
                input::Key::Home => selection::Move::First,
                input::Key::End => selection::Move::Last,
                input::Key::PageUp => {
                    selection::Move::PagePrevious(self.table_page(window, &model))
                }
                input::Key::PageDown => selection::Move::PageNext(self.table_page(window, &model)),
                _ => return None,
            };
            self.session
                .move_virtual_selection(window, &model, movement, modifiers.shift())
        };

        if changed {
            self.reveal_table_cell(window, model.id());
        }
        Some(input::Outcome::handled(
            false,
            changed
                .then_some(response::Effect::Rebuild)
                .unwrap_or_default(),
        ))
    }

    fn table_page(&self, window: window::Id, model: &virtual_list::Model) -> usize {
        self.presented_layout(window)
            .and_then(|layout| layout.virtual_list_page(model.id(), model.row_height()))
            .unwrap_or(10)
    }

    fn move_table_cell(
        &mut self,
        window: window::Id,
        table: interaction::Id,
        movement: CellMove,
        force_cell_focus: bool,
    ) -> bool {
        let (model, columns) = {
            let composition = match self.composition.get(window) {
                Some(composition) => composition,
                None => return false,
            };
            let model = match composition.view().virtual_list_model(table) {
                Some(model) => model.clone(),
                None => return false,
            };
            (model, composition.view().table_columns(table))
        };
        let Some(first) = columns.first().copied() else {
            return false;
        };
        let current_column = self
            .session
            .active_table_cell(window, table)
            .map(crate::table::Cell::column)
            .unwrap_or(first);
        let current_index = columns
            .iter()
            .position(|column| *column == current_column)
            .unwrap_or(0);

        let changed = match movement {
            CellMove::Column(delta) => self
                .session
                .move_active_table_column(window, table, &columns, delta),
            CellMove::Row(movement) => self
                .session
                .move_virtual_selection(window, &model, movement, false),
            CellMove::RowEdge { last, table_edge } => {
                let column = if last {
                    *columns.last().unwrap_or(&first)
                } else {
                    first
                };
                let column_changed = self.session.set_active_table_column(window, table, column);
                let row_changed = table_edge
                    && self.session.move_virtual_selection(
                        window,
                        &model,
                        if last {
                            selection::Move::Last
                        } else {
                            selection::Move::First
                        },
                        false,
                    );
                column_changed || row_changed
            }
            CellMove::Linear { reverse } => {
                if reverse && current_index > 0 {
                    self.session
                        .set_active_table_column(window, table, columns[current_index - 1])
                } else if !reverse && current_index + 1 < columns.len() {
                    self.session
                        .set_active_table_column(window, table, columns[current_index + 1])
                } else {
                    let row_changed = self.session.move_virtual_selection(
                        window,
                        &model,
                        if reverse {
                            selection::Move::Previous
                        } else {
                            selection::Move::Next
                        },
                        false,
                    );
                    if !row_changed {
                        return false;
                    }
                    self.session.set_active_table_column(
                        window,
                        table,
                        if reverse {
                            *columns.last().unwrap_or(&first)
                        } else {
                            first
                        },
                    );
                    true
                }
            }
        };

        if !changed {
            return false;
        }
        self.reveal_table_cell(window, table);
        let should_focus_cell = force_cell_focus
            || self
                .session
                .focused(window)
                .and_then(session::Focus::table_cell_identity)
                .is_some();
        if should_focus_cell
            && let Some(cell) = self.session.active_table_cell(window, table)
            && let Some(focus) = self
                .composition
                .get(window)
                .and_then(|composition| composition.view().table_cell_focus(cell))
        {
            self.session.focus(window, focus.keyboard());
        }
        true
    }

    fn reveal_table_cell(&mut self, window: window::Id, table: interaction::Id) {
        self.session
            .reveal_active_descendant(window, interaction::Target::scroll(table, "Selected rows"));
        if let Some(target) = self
            .presented_layout(window)
            .and_then(|layout| layout.table_scroll_target(table))
        {
            self.session.reveal_active_descendant(window, target);
        }
        self.session
            .request_invalidation(window, response::effect::Invalidation::Rebuild);
    }

    fn leave_table(
        &mut self,
        window: window::Id,
        table: interaction::Id,
        reverse: bool,
    ) -> input::Outcome {
        let Some(current) = self.session.focused(window) else {
            return input::Outcome::ignored();
        };
        let direction = if reverse {
            crate::view::FocusDirection::Backward
        } else {
            crate::view::FocusDirection::Forward
        };
        let Some(next) = self.composition.get(window).and_then(|composition| {
            composition.next_focus_outside_table(current, direction, table)
        }) else {
            return input::Outcome::ignored();
        };
        let changed = self.session.focus(window, next);
        input::Outcome::handled(
            false,
            changed
                .then_some(response::Effect::Layout)
                .unwrap_or_default(),
        )
    }
}

fn merge_outcomes(left: input::Outcome, right: input::Outcome) -> input::Outcome {
    if !left.is_handled() && !right.is_handled() {
        return input::Outcome::ignored();
    }
    input::Outcome::handled(
        left.changed_state() || right.changed_state(),
        left.effect().clone().then(right.effect().clone()),
    )
}
