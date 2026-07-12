use super::super::{
    error::Error, geometry, input, interaction, layout, pointer, response, session, state, text,
    view, window,
};
use super::Runtime;
impl<M: state::State, E: Send + 'static> Runtime<M, E, view::View> {
    pub fn pointer_move_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
    ) -> std::result::Result<input::Outcome, Error> {
        if self
            .session
            .interaction(window)
            .and_then(|interaction| interaction.pointer().pressed())
            .is_some()
        {
            return self.pointer_drag_at(window, size, point);
        }

        let target = self
            .hit_test(window, size, point)
            .inspect(|hit| self.set_cursor_for_hit(window, Some(hit)))
            .and_then(|hit| hit.target().cloned());
        if target.is_none() {
            self.set_pointer_cursor(window, pointer::Cursor::Default);
        }

        self.handle_view(window, view::Action::pointer_move(target))
    }

    pub fn pointer_down_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
    ) -> std::result::Result<input::Outcome, Error> {
        self.pointer_down_at_with_modifiers(window, size, point, input::Modifiers::default())
    }

    pub fn pointer_down_at_with_modifiers(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
        modifiers: input::Modifiers,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(hit) = self.hit_test(window, size, point) else {
            self.set_pointer_cursor(window, pointer::Cursor::Default);
            return self.clear_pointer_focus(window);
        };
        let table_cell = hit.table_cell();
        let selection_change = self.select_virtual_row_for_hit(window, &hit, point, modifiers);
        let selection_row = selection_change.is_some();
        let selection_changed = selection_change.unwrap_or(false);
        self.set_cursor_for_hit(window, Some(&hit));
        let Some(target) = hit.target().cloned() else {
            if selection_changed {
                self.session
                    .request_invalidation(window, response::Invalidation::Layout);
                return Ok(input::Outcome::handled(false, response::Effect::Layout));
            }
            return self.clear_pointer_focus(window);
        };
        let dismissed_overlays = self.dismiss_overlays_for_hit(window, Some(&hit));

        let click_count =
            self.session
                .classify_click(window, &target, point, std::time::Instant::now());
        let text_click = match click_count {
            interaction::ClickCount::Single => text::edit::PointerEditKind::Click,
            interaction::ClickCount::Double => text::edit::PointerEditKind::DoubleClick,
            interaction::ClickCount::Triple => text::edit::PointerEditKind::TripleClick,
        };

        let action = if target.kind() == interaction::Kind::TableDivider {
            view::Action::pointer_down(target)
        } else if hit.is_chrome() {
            view::Action::pointer_manipulate(target)
        } else if matches!(
            hit.frame().role(),
            view::Role::TextArea | view::Role::TextBox
        ) {
            let pointer_down = text_pointer_down_action(hit.frame(), target.clone());
            hit.text_action_at_with_engine(point, text_click, &mut self.layout)
                .map(|action| {
                    let mut actions = vec![pointer_down.clone(), action];
                    if click_count == interaction::ClickCount::Double
                        && hit.frame().is_table_editable()
                        && let Some(cell) = table_cell
                    {
                        actions.push(view::Action::begin_table_edit(cell));
                    }
                    view::Action::sequence(actions)
                })
                .unwrap_or(pointer_down)
        } else if hit.frame().role() == view::Role::Slider {
            hit.action_at_with_engine(point, &mut self.layout)
                .map(|action| {
                    view::Action::sequence([
                        view::Action::focus(session::Focus::control(&target).pointer()),
                        view::Action::pointer_manipulate(target.clone()),
                        action,
                    ])
                })
                .unwrap_or_else(|| {
                    view::Action::sequence([
                        view::Action::focus(session::Focus::control(&target).pointer()),
                        view::Action::pointer_manipulate(target),
                    ])
                })
        } else if is_pointer_focusable(hit.frame())
            || (selection_row && hit.frame().role() == view::Role::VirtualList)
        {
            view::Action::sequence([
                view::Action::focus(session::Focus::control(&target).pointer()),
                view::Action::pointer_down(target),
            ])
        } else {
            view::Action::pointer_down(target)
        };

        let mut outcome = self.handle_view(window, action)?;
        if selection_changed {
            self.session
                .request_invalidation(window, response::Invalidation::Layout);
            outcome = input::Outcome::handled(
                outcome.changed_state(),
                outcome.effect().clone().then(response::Effect::Layout),
            );
        }
        Ok(self.with_overlay_dismissal(window, outcome, dismissed_overlays))
    }

    fn select_virtual_row_for_hit(
        &mut self,
        window: window::Id,
        hit: &layout::Hit,
        point: geometry::Point,
        modifiers: input::Modifiers,
    ) -> Option<bool> {
        let table_cell = hit.table_cell();
        let composition = self.composition.get(window)?;
        let row = composition.provided_row_for_node(hit.frame().node_id());
        let (list, key, index) = if let Some(row) = row {
            (row.list(), row.key(), row.index())
        } else {
            let list = hit.frame().target()?.element_id()?;
            let model = composition.virtual_list_model(list)?;
            if !model.is_selectable() {
                return None;
            }
            let index = hit.frame().virtual_row_index_at(point)?;
            let key = model.key_at(index)?;
            (list, key, index)
        };
        let model = composition.virtual_list_model(list)?.clone();
        if !model.is_selectable() {
            return None;
        }
        let toggle = modifiers.control() || modifiers.super_key();
        let selected =
            self.session
                .select_virtual_row(window, &model, key, index, modifiers.shift(), toggle);
        let column_changed = table_cell.is_some_and(|cell| {
            self.session
                .set_active_table_column(window, cell.table(), cell.column())
        });
        Some(selected || column_changed)
    }

    fn clear_pointer_focus(
        &mut self,
        window: window::Id,
    ) -> std::result::Result<input::Outcome, Error> {
        let dismissed_palette = self
            .session
            .dismiss_command_palette_for_target(window, None);
        let dismissed_menu = self.session.dismiss_menu_for_target(window, None);
        let mut outcome = self.clear_focus_committing_text_box(window)?;

        if dismissed_menu || dismissed_palette {
            let effect = outcome.effect().clone().then(response::Effect::Rebuild);
            outcome = input::Outcome::handled(outcome.changed_state(), effect);
            self.apply_window_update(window, outcome.changed_state(), outcome.effect());
        }

        Ok(outcome)
    }

    pub fn pointer_up_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
    ) -> std::result::Result<input::Outcome, Error> {
        let hit = self.hit_test(window, size, point);
        self.set_cursor_for_hit(window, hit.as_ref());
        let target = hit.as_ref().and_then(|hit| hit.target().cloned());
        let action = hit.as_ref().and_then(|hit| {
            (!hit.is_chrome()
                && !matches!(
                    hit.frame().role(),
                    view::Role::Slider | view::Role::TextArea | view::Role::TextBox
                ))
            .then(|| hit.action_at(point))
            .flatten()
        });

        self.handle_view(window, view::Action::pointer_up(target, action))
    }

    pub fn pointer_drag_at(
        &mut self,
        window: window::Id,
        _size: geometry::Size,
        point: geometry::Point,
    ) -> std::result::Result<input::Outcome, Error> {
        self.session.cancel_click_sequence(window);
        let Some(layout) = self.presented_layout(window) else {
            return Ok(input::Outcome::ignored());
        };
        let hit = layout.hit_test(point);
        let captured_target = self
            .session
            .interaction(window)
            .and_then(|interaction| {
                interaction
                    .pointer()
                    .capture()
                    .map(|capture| capture.target())
                    .or_else(|| interaction.pointer().pressed())
            })
            .map(interaction::Target::kind);
        if captured_target == Some(interaction::Kind::TextArea) {
            self.set_pointer_cursor(window, pointer::Cursor::Text);
        } else if captured_target == Some(interaction::Kind::TableDivider) {
            self.set_pointer_cursor(window, pointer::Cursor::ResizeHorizontal);
        } else {
            self.set_cursor_for_hit(window, hit.as_ref());
        }
        let hovered = hit.as_ref().and_then(|hit| hit.target().cloned());
        let active = self.session.interaction(window).and_then(|interaction| {
            interaction
                .pointer()
                .capture()
                .map(|capture| capture.target().clone())
                .or_else(|| interaction.pointer().pressed().cloned())
        });

        let Some(target) = active else {
            return self.handle_view(window, view::Action::pointer_move(hovered));
        };

        let dragged = layout.drag_action_for_target(&target, point, &mut self.layout);
        let demoted_text_box_activation = dragged
            .as_ref()
            .is_some_and(|(role, _)| *role == view::Role::TextBox)
            && self.session.set_pointer_press_intent(
                window,
                &target,
                interaction::PressIntent::Manipulate,
            );
        let action = dragged.and_then(|(_, action)| action);

        let outcome =
            self.handle_view(window, view::Action::pointer_drag(hovered, target, action))?;
        if demoted_text_box_activation {
            let effect = outcome.effect().clone().then(response::Effect::Paint);
            return Ok(input::Outcome::handled(outcome.changed_state(), effect));
        }

        Ok(outcome)
    }

    pub fn pointer_left_at(
        &mut self,
        window: window::Id,
    ) -> std::result::Result<input::Outcome, Error> {
        let captured_kind = self
            .session
            .interaction(window)
            .and_then(|interaction| interaction.pointer().capture())
            .map(|capture| capture.target().kind());
        self.set_pointer_cursor(
            window,
            match captured_kind {
                Some(interaction::Kind::TextArea) => pointer::Cursor::Text,
                Some(interaction::Kind::TableDivider) => pointer::Cursor::ResizeHorizontal,
                _ => pointer::Cursor::Default,
            },
        );

        self.handle_view(window, view::Action::pointer_left())
    }

    pub fn scroll_at(
        &mut self,
        window: window::Id,
        _size: geometry::Size,
        point: geometry::Point,
        delta: interaction::ScrollDelta,
    ) -> std::result::Result<input::Outcome, Error> {
        let Some(layout) = self.presented_layout(window) else {
            return Ok(input::Outcome::ignored());
        };
        let viewport_target = layout.scroll_target_at(point, delta);
        let Some(target) = viewport_target
            .or_else(|| layout.hit_test(point).and_then(|hit| hit.target().cloned()))
        else {
            return Ok(input::Outcome::ignored());
        };

        self.handle_view(window, view::Action::scroll(target, delta))
    }

    fn dismiss_overlays_for_hit(&mut self, window: window::Id, hit: Option<&layout::Hit>) -> bool {
        let inside_palette =
            hit.is_some_and(|hit| self.hit_is_command_palette_surface(window, hit));
        let inside_menu = hit.is_some_and(|hit| self.hit_is_menu_surface(window, hit));
        let dismissed_palette = self
            .session
            .dismiss_command_palette_for_surface(window, inside_palette);
        let dismissed_menu = self.session.dismiss_menu_for_surface(window, inside_menu);

        dismissed_palette || dismissed_menu
    }

    fn hit_is_command_palette_surface(&self, window: window::Id, hit: &layout::Hit) -> bool {
        hit.target()
            .is_some_and(interaction::Target::is_command_palette_surface)
            || self.hit_owner_is_descendant_of_element(
                window,
                hit,
                interaction::CommandPalette::panel_id(),
            )
    }

    fn hit_is_menu_surface(&self, window: window::Id, hit: &layout::Hit) -> bool {
        if hit
            .target()
            .is_some_and(interaction::Target::is_menu_surface)
        {
            return true;
        }

        let Some(menu_id) = self
            .session
            .interaction(window)
            .and_then(|interaction| interaction.open_menu())
            .map(interaction::Menu::id)
        else {
            return false;
        };

        self.hit_owner_is_descendant_of_element(window, hit, menu_id)
    }

    fn hit_owner_is_descendant_of_element(
        &self,
        window: window::Id,
        hit: &layout::Hit,
        element_id: interaction::Id,
    ) -> bool {
        self.composition.get(window).is_some_and(|composition| {
            composition.node_is_self_or_descendant_of_element(hit.frame().node_id(), element_id)
        })
    }

    fn with_overlay_dismissal(
        &mut self,
        window: window::Id,
        outcome: input::Outcome,
        dismissed: bool,
    ) -> input::Outcome {
        if !dismissed {
            return outcome;
        }

        self.session
            .request_invalidation(window, response::Invalidation::Rebuild);
        input::Outcome::handled(
            outcome.changed_state(),
            outcome.effect().clone().then(response::Effect::Rebuild),
        )
    }
}

impl<M: state::State, E: Send + 'static> Runtime<M, E, view::View> {
    fn set_cursor_for_hit(&mut self, window: window::Id, hit: Option<&layout::Hit>) {
        let cursor = hit
            .map(|hit| {
                if hit_promises_text_edit(hit) {
                    pointer::Cursor::Text
                } else if hit
                    .target()
                    .is_some_and(|target| target.kind() == interaction::Kind::TableDivider)
                {
                    pointer::Cursor::ResizeHorizontal
                } else {
                    pointer::Cursor::Default
                }
            })
            .unwrap_or(pointer::Cursor::Default);
        self.set_pointer_cursor(window, cursor);
    }

    fn set_pointer_cursor(&mut self, window: window::Id, cursor: pointer::Cursor) {
        self.session.set_cursor(window, cursor);
    }
}

fn hit_promises_text_edit(hit: &layout::Hit) -> bool {
    !hit.is_chrome()
        && hit.frame().is_enabled()
        && matches!(
            hit.frame().role(),
            view::Role::TextArea | view::Role::TextBox
        )
}

fn text_pointer_down_action(frame: &layout::Frame, target: interaction::Target) -> view::Action {
    if frame.is_focused() {
        view::Action::pointer_manipulate(target)
    } else {
        view::Action::pointer_down(target)
    }
}

fn is_pointer_focusable(frame: &layout::Frame) -> bool {
    if frame.is_menu_row() {
        return false;
    }

    frame.is_enabled()
        && matches!(
            frame.role(),
            view::Role::Binding | view::Role::Button | view::Role::Checkbox | view::Role::Radio
        )
}
