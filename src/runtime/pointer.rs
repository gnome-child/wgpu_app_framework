use super::super::{
    command::Error, geometry, input, interaction, layout, pointer, response, session, state, text,
    view, virtual_list, window,
};
use super::Runtime;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PressAdmission {
    Inert,
    SelectionOnly,
    Target,
}

pub(super) struct ResolvedPress {
    hit: Option<layout::Hit>,
    target: Option<interaction::Target>,
    row: Option<VirtualRowGesture>,
    task_focus: Option<session::Focus>,
    admission: PressAdmission,
    intent: Option<interaction::PressIntent>,
    cursor: pointer::Cursor,
    cursor_after_release: pointer::Cursor,
    inside_palette: bool,
    inside_menu: bool,
}

impl ResolvedPress {
    fn hit(&self) -> Option<&layout::Hit> {
        self.hit.as_ref()
    }

    fn target(&self) -> Option<&interaction::Target> {
        self.target.as_ref()
    }

    fn row(&self) -> Option<&VirtualRowGesture> {
        self.row.as_ref()
    }

    fn task_focus(&self) -> Option<session::Focus> {
        self.task_focus.clone()
    }

    fn admission(&self) -> PressAdmission {
        self.admission
    }

    pub(super) fn cursor(&self) -> pointer::Cursor {
        self.cursor
    }

    fn cursor_after_release(&self) -> pointer::Cursor {
        self.cursor_after_release
    }

    fn press_action(&self, target: interaction::Target) -> view::Action {
        view::Action::pointer_press(
            target,
            self.intent
                .expect("a resolved target press must carry its intent"),
            self.cursor_after_release,
        )
    }
}

#[derive(Clone)]
struct VirtualRowGesture {
    model: virtual_list::Model,
    key: virtual_list::Key,
    index: usize,
    cell: Option<crate::table::Cell>,
    was_focal: bool,
}

impl VirtualRowGesture {
    fn permits_participation(&self, modifiers: input::Modifiers) -> bool {
        self.was_focal && !modifiers.shift() && !modifiers.control() && !modifiers.super_key()
    }
}

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(super) fn resolve_press(
        &self,
        window: window::Id,
        point: geometry::Point,
        modifiers: input::Modifiers,
        hit: Option<layout::Hit>,
    ) -> ResolvedPress {
        let target = hit.as_ref().and_then(layout::Hit::target).cloned();
        let row = hit
            .as_ref()
            .and_then(|hit| self.virtual_row_gesture_for_hit(window, hit, point));
        let admission = if target.is_none() {
            PressAdmission::Inert
        } else if row
            .as_ref()
            .is_some_and(|row| !row.permits_participation(modifiers))
        {
            PressAdmission::SelectionOnly
        } else {
            PressAdmission::Target
        };
        let task_focus = hit.as_ref().and_then(|hit| {
            let target = target.as_ref()?;
            if target.kind() == interaction::Kind::TableDivider || hit.is_chrome() {
                None
            } else if matches!(
                hit.frame().role(),
                view::Role::TextArea | view::Role::TextBox
            ) {
                hit.frame().text_task_focus().map(session::Focus::pointer)
            } else if hit.frame().role() == view::Role::Slider
                || is_pointer_focusable(hit.frame())
                || (row.is_some() && hit.frame().role() == view::Role::VirtualList)
            {
                Some(session::Focus::control(target).pointer())
            } else {
                None
            }
        });
        let intent = hit.as_ref().and_then(|hit| {
            target.as_ref()?;
            Some(
                if hit.is_chrome() || hit.frame().role() == view::Role::Slider {
                    interaction::PressIntent::Manipulate
                } else if matches!(
                    hit.frame().role(),
                    view::Role::TextArea | view::Role::TextBox
                ) && hit.frame().is_focused()
                {
                    interaction::PressIntent::Manipulate
                } else {
                    interaction::PressIntent::Activate
                },
            )
        });
        let hit_cursor = hit
            .as_ref()
            .map(|hit| {
                if !hit.is_chrome()
                    && hit
                        .target()
                        .is_none_or(|target| target.kind() != interaction::Kind::Indicator)
                    && hit.frame().is_enabled()
                    && hit.frame().text_is_selectable()
                    && matches!(
                        hit.frame().role(),
                        view::Role::TextArea | view::Role::TextBox
                    )
                {
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
        let cursor_after_release = if admission == PressAdmission::Target {
            hit_cursor
        } else {
            pointer::Cursor::Default
        };
        let cursor = self
            .session
            .interaction(window)
            .and_then(|interaction| interaction.pointer().capture())
            .map(interaction::Capture::cursor)
            .unwrap_or(cursor_after_release);
        let inside_palette = hit
            .as_ref()
            .is_some_and(|hit| self.hit_is_command_palette_surface(window, hit));
        let inside_menu = hit
            .as_ref()
            .is_some_and(|hit| self.hit_is_menu_surface(window, hit));

        ResolvedPress {
            hit,
            target,
            row,
            task_focus,
            admission,
            intent,
            cursor,
            cursor_after_release,
            inside_palette,
            inside_menu,
        }
    }

    fn virtual_row_gesture_for_hit(
        &self,
        window: window::Id,
        hit: &layout::Hit,
        point: geometry::Point,
    ) -> Option<VirtualRowGesture> {
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
        let was_focal = self
            .session
            .selection(window, model.id())
            .is_some_and(|selection| selection.active() == Some(key));
        Some(VirtualRowGesture {
            model,
            key,
            index,
            cell: table_cell,
            was_focal,
        })
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
}

impl<M: state::State, E: Send + 'static> Runtime<M, E, view::View> {
    pub fn pointer_move_at(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
    ) -> std::result::Result<input::Outcome, Error> {
        self.pointer_move_on_surface(window, size, point, crate::popup::Surface::Parent)
    }

    pub(crate) fn pointer_move_on_surface(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
        surface: crate::popup::Surface,
    ) -> std::result::Result<input::Outcome, Error> {
        self.session
            .set_pointer_position(window, Some(point), surface);
        if self
            .session
            .interaction(window)
            .and_then(|interaction| interaction.pointer().pressed())
            .is_some()
        {
            return self.pointer_drag_on_surface(window, size, point, surface);
        }

        let hit = self.hit_test_on_surface(window, size, point, surface);
        let modifiers = self
            .session
            .interaction(window)
            .map(|interaction| interaction.pointer().modifiers())
            .unwrap_or_default();
        let resolved = self.resolve_press(window, point, modifiers, hit);
        self.set_pointer_cursor(window, resolved.cursor());
        let target = resolved.target().cloned();

        self.handle_view(window, view::Action::pointer_move(target))
    }

    pub(crate) fn pointer_modifiers_changed(
        &mut self,
        window: window::Id,
        modifiers: input::Modifiers,
    ) -> std::result::Result<input::Outcome, Error> {
        let changed = self.session.set_pointer_modifiers(window, modifiers);
        let Some((point, surface)) = self.session.interaction(window).and_then(|interaction| {
            interaction
                .pointer()
                .position()
                .map(|point| (point, interaction.pointer().surface()))
        }) else {
            return Ok(if changed {
                input::Outcome::handled(false, response::Effect::None)
            } else {
                input::Outcome::ignored()
            });
        };
        let hit = self
            .presented_layout(window)
            .and_then(|layout| layout.hit_test_on_surface(point, surface));
        let resolved = self.resolve_press(window, point, modifiers, hit);
        let cursor_changed = self.set_pointer_cursor(window, resolved.cursor());

        Ok(if changed || cursor_changed {
            input::Outcome::handled(false, response::Effect::None)
        } else {
            input::Outcome::ignored()
        })
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
        self.pointer_down_on_surface(
            window,
            size,
            point,
            modifiers,
            crate::popup::Surface::Parent,
        )
    }

    pub(crate) fn pointer_down_on_surface(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
        modifiers: input::Modifiers,
        surface: crate::popup::Surface,
    ) -> std::result::Result<input::Outcome, Error> {
        self.session
            .set_pointer_position(window, Some(point), surface);
        self.session.set_pointer_modifiers(window, modifiers);
        let hit = self.hit_test_on_surface(window, size, point, surface);
        let resolved = self.resolve_press(window, point, modifiers, hit);
        self.set_pointer_cursor(window, resolved.cursor());
        let Some(hit) = resolved.hit() else {
            return self.clear_pointer_focus(window);
        };
        let selection = resolved.row();
        let selection_row = selection.is_some();
        let Some(target) = resolved.target().cloned() else {
            let transition = self.attempt_clear_focus_transition(window)?;
            if !transition.is_accepted() {
                self.session.cancel_click_sequence(window);
                return Ok(transition.into_outcome());
            }
            let selection_changed = selection
                .as_ref()
                .map(|selection| self.apply_virtual_row_gesture(window, selection, modifiers))
                .unwrap_or(false);
            if selection_changed {
                self.session
                    .request_invalidation(window, response::Invalidation::Layout);
                return Ok(
                    transition.then(input::Outcome::handled(false, response::Effect::Layout))
                );
            }
            return Ok(transition.into_outcome());
        };

        if target.kind() == interaction::Kind::Indicator {
            self.session.cancel_click_sequence(window);
            let dismissed_overlays = self.dismiss_overlays_for_press(window, &resolved);
            return Ok(self.with_overlay_dismissal(
                window,
                input::Outcome::handled(false, response::Effect::None),
                dismissed_overlays,
            ));
        }

        let transition = resolved
            .task_focus()
            .map(|focus| self.attempt_focus_transition(window, focus))
            .transpose()?;
        if transition
            .as_ref()
            .is_some_and(|transition| !transition.is_accepted())
        {
            self.session.cancel_click_sequence(window);
            return Ok(transition
                .expect("rejected transition is present")
                .into_outcome());
        }

        let selection_changed = selection
            .as_ref()
            .map(|selection| self.apply_virtual_row_gesture(window, selection, modifiers))
            .unwrap_or(false);
        let dismissed_overlays = self.dismiss_overlays_for_press(window, &resolved);
        if resolved.admission() == PressAdmission::SelectionOnly {
            self.session.cancel_click_sequence(window);
            let mut outcome = transition
                .map(|transition| transition.into_outcome())
                .unwrap_or_else(input::Outcome::ignored);
            if selection_changed {
                self.session
                    .request_invalidation(window, response::Invalidation::Layout);
            }
            let effect = outcome.effect().clone().then(response::Effect::Layout);
            outcome = input::Outcome::handled(outcome.changed_state(), effect);
            return Ok(self.with_overlay_dismissal(window, outcome, dismissed_overlays));
        }

        let click_count =
            self.session
                .classify_click(window, &target, point, std::time::Instant::now());
        let text_click = match click_count {
            interaction::ClickCount::Single => text::selection::PointerKind::Click,
            interaction::ClickCount::Double => text::selection::PointerKind::DoubleClick,
            interaction::ClickCount::Triple => text::selection::PointerKind::TripleClick,
        };

        let action = if target.kind() == interaction::Kind::TableDivider {
            resolved.press_action(target)
        } else if hit.is_chrome() {
            resolved.press_action(target)
        } else if matches!(
            hit.frame().role(),
            view::Role::TextArea | view::Role::TextBox
        ) {
            let pointer_down = resolved.press_action(target.clone());
            hit.text_action_at_with_engine(point, text_click, &mut self.layout)
                .map(|action| view::Action::sequence([pointer_down.clone(), action]))
                .unwrap_or(pointer_down)
        } else if hit.frame().role() == view::Role::Slider {
            hit.action_at_with_engine(point, &mut self.layout)
                .map(|action| {
                    view::Action::sequence([
                        view::Action::focus(session::Focus::control(&target).pointer()),
                        resolved.press_action(target.clone()),
                        action,
                    ])
                })
                .unwrap_or_else(|| {
                    view::Action::sequence([
                        view::Action::focus(session::Focus::control(&target).pointer()),
                        resolved.press_action(target),
                    ])
                })
        } else if is_pointer_focusable(hit.frame())
            || (selection_row && hit.frame().role() == view::Role::VirtualList)
        {
            view::Action::sequence([
                view::Action::focus(session::Focus::control(&target).pointer()),
                resolved.press_action(target),
            ])
        } else {
            resolved.press_action(target)
        };

        let mut outcome = self.handle_view(window, action)?;
        if let Some(transition) = transition {
            outcome = transition.then(outcome);
        }
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

    fn apply_virtual_row_gesture(
        &mut self,
        window: window::Id,
        gesture: &VirtualRowGesture,
        modifiers: input::Modifiers,
    ) -> bool {
        let toggle = modifiers.control() || modifiers.super_key();
        let selected = self.session.select_virtual_row(
            window,
            &gesture.model,
            gesture.key,
            gesture.index,
            modifiers.shift(),
            toggle,
        );
        let column_changed = gesture.cell.is_some_and(|cell| {
            self.session
                .set_active_table_column(window, cell.table(), cell.column())
        });
        selected || column_changed
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
        self.pointer_up_on_surface(window, size, point, crate::popup::Surface::Parent)
    }

    pub(crate) fn pointer_up_on_surface(
        &mut self,
        window: window::Id,
        size: geometry::Size,
        point: geometry::Point,
        surface: crate::popup::Surface,
    ) -> std::result::Result<input::Outcome, Error> {
        self.session
            .set_pointer_position(window, Some(point), surface);
        let hit = self.hit_test_on_surface(window, size, point, surface);
        let modifiers = self
            .session
            .interaction(window)
            .map(|interaction| interaction.pointer().modifiers())
            .unwrap_or_default();
        let resolved = self.resolve_press(window, point, modifiers, hit);
        self.set_pointer_cursor(window, resolved.cursor_after_release());
        let target = resolved.target().cloned();
        let action = resolved.hit().and_then(|hit| {
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
        self.pointer_drag_on_surface(window, _size, point, crate::popup::Surface::Parent)
    }

    fn pointer_drag_on_surface(
        &mut self,
        window: window::Id,
        _size: geometry::Size,
        point: geometry::Point,
        surface: crate::popup::Surface,
    ) -> std::result::Result<input::Outcome, Error> {
        self.session
            .set_pointer_position(window, Some(point), surface);
        self.session.cancel_click_sequence(window);
        let Some(layout) = self.presented_layout(window) else {
            return Ok(input::Outcome::ignored());
        };
        let modifiers = self
            .session
            .interaction(window)
            .map(|interaction| interaction.pointer().modifiers())
            .unwrap_or_default();
        let resolved = self.resolve_press(
            window,
            point,
            modifiers,
            layout.hit_test_on_surface(point, surface),
        );
        self.set_pointer_cursor(window, resolved.cursor());
        let hovered = resolved.target().cloned();
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
        let point = self
            .session
            .interaction(window)
            .and_then(|interaction| interaction.pointer().position())
            .unwrap_or_else(|| geometry::Point::new(0, 0));
        let modifiers = self
            .session
            .interaction(window)
            .map(|interaction| interaction.pointer().modifiers())
            .unwrap_or_default();
        let resolved = self.resolve_press(window, point, modifiers, None);
        self.set_pointer_cursor(window, resolved.cursor());

        self.handle_view(window, view::Action::pointer_left())
    }

    pub fn scroll_at(
        &mut self,
        window: window::Id,
        _size: geometry::Size,
        point: geometry::Point,
        delta: interaction::ScrollDelta,
    ) -> std::result::Result<input::Outcome, Error> {
        self.scroll_on_surface(window, _size, point, delta, crate::popup::Surface::Parent)
    }

    pub(crate) fn scroll_on_surface(
        &mut self,
        window: window::Id,
        _size: geometry::Size,
        point: geometry::Point,
        delta: interaction::ScrollDelta,
        surface: crate::popup::Surface,
    ) -> std::result::Result<input::Outcome, Error> {
        self.session
            .set_pointer_position(window, Some(point), surface);
        let Some(layout) = self.presented_layout(window) else {
            return Ok(input::Outcome::ignored());
        };
        let viewport_target = layout.scroll_target_at_surface(point, delta, surface);
        let Some(target) = viewport_target.or_else(|| {
            layout
                .hit_test_on_surface(point, surface)
                .and_then(|hit| hit.target().cloned())
        }) else {
            return Ok(input::Outcome::ignored());
        };

        self.handle_view(window, view::Action::scroll(target, delta))
    }

    fn dismiss_overlays_for_press(&mut self, window: window::Id, resolved: &ResolvedPress) -> bool {
        let dismissed_palette = self
            .session
            .dismiss_command_palette_for_surface(window, resolved.inside_palette);
        let dismissed_menu = self
            .session
            .dismiss_menu_for_surface(window, resolved.inside_menu);

        dismissed_palette || dismissed_menu
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
    fn set_pointer_cursor(&mut self, window: window::Id, cursor: pointer::Cursor) -> bool {
        self.session.set_cursor(window, cursor)
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
