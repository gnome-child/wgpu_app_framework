use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::animation;
use crate::geometry::{Rect, point};
use crate::widget::menu;
use crate::{action, pointer, text, ui, widget, window};

use super::{command, drag_drop, floating, focus, text_input};

pub use focus::Focus;
#[cfg(test)]
pub(crate) use focus::State as FocusState;

const MULTI_CLICK_MAX_INTERVAL: Duration = Duration::from_millis(500);
const MULTI_CLICK_MAX_DISTANCE: f32 = 4.0;

#[derive(Debug, Default)]
pub struct WindowState {
    pub hovered: Option<ui::Path>,
    pub focus: focus::State,
    pub pressed: Option<ui::Path>,
    pub pressed_source: Option<PressSource>,
    pub modifiers: ui::Modifiers,
    pub command_subject: Option<action::Scope>,
    pub pointer: pointer::Pointer,
    pub floating: floating::State,
    pub open_menu: Option<menu::Id>,
    pub open_submenu: Option<menu::Id>,
    pub command_scope_captures: HashMap<ui::Path, action::Context>,
    pub pointer_capture: Option<pointer::Capture>,
    pub composition: Option<ui::Composition>,
    pub text_input_session: text_input::Session,
    pub drag_drop: drag_drop::State,
    pub text_field_states: HashMap<ui::Path, text::TextFieldState>,
    pub last_text_field_click: Option<TextFieldClick>,
    pub text_selection_drag: Option<ui::Path>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressSource {
    Pointer,
    Keyboard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextFieldClick {
    path: ui::Path,
    position: point::Logical,
    at: Instant,
    count: u8,
}

impl WindowState {
    pub fn hit_test(&self, position: point::Logical) -> Option<ui::Path> {
        self.composition.as_ref().and_then(|composition| {
            let layout = composition.layout();
            layout.hit_test_where(position, |path| {
                composition
                    .interactivity(path)
                    .is_some_and(|interactivity| interactivity.hit_test())
            })
        })
    }

    pub fn scroll_target(&self, position: point::Logical) -> Option<ui::Path> {
        self.composition.as_ref().and_then(|composition| {
            let layout = composition.layout();
            layout.hit_test_where(position, |path| {
                matches!(
                    composition.widget_metrics(path),
                    Some(widget::Metrics::Scroll(_))
                )
            })
        })
    }

    pub fn cursor_for_hovered(&self) -> ui::Cursor {
        let Some(composition) = self.composition.as_ref() else {
            return ui::Cursor::Default;
        };

        if let Some(capture) = self.pointer_capture.as_ref() {
            return composition.cursor(capture.target());
        }

        self.hovered
            .as_ref()
            .map(|path| composition.cursor(path))
            .unwrap_or_default()
    }

    pub fn widget_hit(&self, position: point::Logical) -> Option<widget::Hit> {
        self.composition
            .as_ref()?
            .widget_metrics_iter()
            .filter_map(|(path, metrics)| {
                metrics
                    .hit_test(position)
                    .map(|part| widget::Hit::new(path.clone(), part))
            })
            .max_by_key(|hit| hit.target().ids().len())
    }

    pub fn scroll_metrics(&self, target: &ui::Path) -> Option<widget::scroll::Metrics> {
        match self.composition.as_ref()?.widget_metrics(target)? {
            widget::Metrics::Scroll(metrics) => Some(metrics),
        }
    }

    pub fn start_pointer_capture(
        &mut self,
        hit: &widget::Hit,
        button: pointer::Button,
        position: point::Logical,
    ) -> bool {
        let Some(metrics) = self.scroll_metrics(hit.target()) else {
            return false;
        };

        let Some(grab_offset) = scroll_capture_offset(metrics, hit.part(), position) else {
            return false;
        };

        self.pointer_capture = Some(pointer::Capture::new(
            hit.target().clone(),
            hit.part(),
            button,
            position,
            grab_offset,
        ));
        self.pressed = Some(hit.target().clone());
        self.pressed_source = Some(PressSource::Pointer);
        true
    }

    pub fn pointer_capture_offset(
        &self,
        position: point::Logical,
    ) -> Option<(ui::Path, point::Logical)> {
        let capture = self.pointer_capture.as_ref()?;
        let part = capture.part().scroll()?;
        let metrics = self.scroll_metrics(capture.target())?;
        let offset = metrics.drag_offset(part, position, capture.grab_offset())?;

        Some((capture.target().clone(), offset))
    }

    pub fn clear_pointer_capture(&mut self) -> bool {
        let changed = self.pointer_capture.is_some();
        self.pointer_capture = None;
        changed
    }

    pub fn is_focusable(&self, target: &ui::Path) -> bool {
        self.composition
            .as_ref()
            .and_then(|composition| composition.interactivity(target))
            .is_some_and(|interactivity| interactivity.focusable())
    }

    pub fn is_actionable(&self, target: &ui::Path) -> bool {
        self.composition
            .as_ref()
            .and_then(|composition| composition.interactivity(target))
            .is_some_and(|interactivity| interactivity.actionable())
    }

    pub fn command_subject(&self, target: &ui::Path) -> ui::CommandSubject {
        self.composition
            .as_ref()
            .map_or_else(ui::CommandSubject::default, |composition| {
                composition.command_subject(target)
            })
    }

    pub fn intent(&self, target: &ui::Path) -> Option<ui::Intent> {
        self.composition
            .as_ref()
            .and_then(|composition| composition.intent(target))
    }

    pub fn has_responder(&self, target: &ui::Path) -> bool {
        self.composition
            .as_ref()
            .is_some_and(|composition| composition.has_responder(target))
    }

    pub fn text_field(&self, target: &ui::Path) -> Option<&crate::text::Field> {
        self.composition
            .as_ref()
            .and_then(|composition| composition.text_field(target))
    }

    pub fn is_text_field(&self, target: &ui::Path) -> bool {
        self.text_field(target).is_some()
    }

    pub fn is_selectable_text_field(&self, target: &ui::Path) -> bool {
        self.text_field(target)
            .is_some_and(crate::text::Field::is_selectable)
    }

    pub fn is_editable_text_field(&self, target: &ui::Path) -> bool {
        self.text_field(target)
            .is_some_and(crate::text::Field::is_editable)
    }

    pub fn focused_editable_text_field(&self) -> Option<ui::Path> {
        self.focused_path()
            .filter(|path| self.is_editable_text_field(path))
    }

    pub fn focused_selectable_text_field(&self) -> Option<ui::Path> {
        self.focused_path()
            .filter(|path| self.is_selectable_text_field(path))
    }

    pub fn text_input_enabled(&self) -> bool {
        self.focused_editable_text_field().is_some()
    }

    pub fn focused_text_field_caret_rect(&self, text_engine: &mut text::Engine) -> Option<Rect> {
        let target = self.focused_selectable_text_field()?;
        let state = self
            .text_field_states
            .get(&target)
            .cloned()
            .unwrap_or_default();

        self.composition
            .as_ref()?
            .text_field_caret_rect(&target, state, text_engine)
    }

    pub fn text_field_rect(&self, target: &ui::Path) -> Option<Rect> {
        self.composition
            .as_ref()?
            .layout()
            .find_path(target)
            .map(|frame| frame.rect())
    }

    pub fn set_focused_text_field_preedit(
        &mut self,
        preedit: Option<text::Preedit>,
    ) -> Option<ui::Path> {
        let target = self.focused_editable_text_field()?;
        let current = self
            .text_field_states
            .get(&target)
            .cloned()
            .unwrap_or_default();
        let next = current.clone().with_preedit(preedit);

        if next != current {
            self.text_field_states.insert(target.clone(), next);
        }

        Some(target)
    }

    pub fn clear_text_field_preedits(&mut self) -> bool {
        let mut changed = false;

        for state in self.text_field_states.values_mut() {
            if state.preedit().is_some() {
                *state = state.clone().with_preedit(None);
                changed = true;
            }
        }

        changed
    }

    pub fn text_field_edit_at(
        &mut self,
        target: &ui::Path,
        position: point::Logical,
        text_engine: &mut text::Engine,
    ) -> Option<text::Edit> {
        if !self.is_selectable_text_field(target) {
            return None;
        }

        let kind = self.text_field_click_kind(target, position);
        if kind == text::PointerEditKind::Click
            && let Some((range, selected_text, source_editable)) =
                self.text_drag_source_at(target, position, text_engine)
        {
            self.drag_drop.begin_text(
                target.clone(),
                position,
                range,
                selected_text,
                source_editable,
            );
            self.reset_text_field_caret_blink(target, Instant::now());
            return None;
        }

        let edit = self.composition.as_ref()?.text_field_edit_at(
            target,
            position,
            kind,
            self.text_field_states
                .get(target)
                .cloned()
                .unwrap_or_default(),
            text_engine,
        )?;
        self.text_selection_drag = Some(target.clone());
        self.reset_text_field_caret_blink(target, Instant::now());

        Some(edit)
    }

    pub fn text_field_drag_edit_at(
        &mut self,
        position: point::Logical,
        text_engine: &mut text::Engine,
    ) -> Option<(ui::Path, text::Edit)> {
        let target = self.text_selection_drag.as_ref()?.clone();
        if !self.is_selectable_text_field(&target) {
            return None;
        }
        let edit = self.composition.as_ref()?.text_field_edit_at(
            &target,
            position,
            text::PointerEditKind::Drag,
            self.text_field_states
                .get(&target)
                .cloned()
                .unwrap_or_default(),
            text_engine,
        )?;
        self.reset_text_field_caret_blink(&target, Instant::now());

        Some((target, edit))
    }

    pub fn end_text_selection_drag(&mut self) {
        self.text_selection_drag = None;
    }

    pub fn try_start_text_drag(
        &mut self,
        position: point::Logical,
        text_engine: &mut text::Engine,
    ) -> bool {
        let was_active = self.drag_drop.active_text().is_some();
        let active = self.drag_drop.try_start_text_drag(position);
        let changed = active && !was_active;

        if active {
            return self.update_text_drop_target(position, text_engine) || changed;
        }

        changed
    }

    pub fn update_text_drop_target(
        &mut self,
        position: point::Logical,
        text_engine: &mut text::Engine,
    ) -> bool {
        let Some(source) = self.drag_drop.active_text().cloned() else {
            return self.drag_drop.clear_text_target();
        };

        let Some(target) = self
            .hit_test(position)
            .filter(|target| self.text_field(target).is_some_and(text::Field::allows_cut))
        else {
            return self.drag_drop.clear_text_target();
        };

        let Some(cursor) = self.text_field_cursor_at(&target, position, text_engine) else {
            return self.drag_drop.clear_text_target();
        };

        let operation = drag_drop::text_operation(&source, &target, true, self.modifiers);
        if operation == ui::drag_drop::Operation::None
            || (operation == ui::drag_drop::Operation::Move
                && source.path() == &target
                && source.selected_range().contains(&cursor.index))
        {
            return self.drag_drop.clear_text_target();
        }

        let Some(caret_rect) = self.text_field_caret_rect_at_cursor(&target, cursor, text_engine)
        else {
            return self.drag_drop.clear_text_target();
        };

        self.drag_drop
            .set_text_target(Some(drag_drop::TextTarget::new(
                target, cursor, operation, caret_rect,
            )))
    }

    pub fn finish_text_drop(&mut self) -> Option<ui::Event> {
        let source = self.drag_drop.active_text().cloned()?;
        let target = self.drag_drop.text_target().cloned()?;
        let operation = target.operation();

        if operation == ui::drag_drop::Operation::None {
            self.drag_drop.clear();
            return None;
        }

        let text = source.selected_text().to_owned();
        let source_range = source.selected_range();
        let source_edit = if operation == ui::drag_drop::Operation::Move
            && source.source_editable()
            && source.path() != target.path()
        {
            Some((
                source.path().clone(),
                text::Edit::replace_range(source_range.clone(), ""),
            ))
        } else {
            None
        };
        let target_edit =
            if operation == ui::drag_drop::Operation::Move && source.path() == target.path() {
                text::Edit::move_range(source_range, target.cursor().index)
            } else {
                text::Edit::insert_at(target.cursor().index, text)
            };
        let target_path = target.path().clone();
        let event = ui::Event::TextDropRequested {
            source: source_edit,
            target: target_path.clone(),
            edit: target_edit,
            operation,
        };

        self.set_focus(
            target_path.clone(),
            ui::focus::Reason::Pointer,
            self.focus_visibility_for_activation(&target_path, action::Source::Pointer),
        );
        command::set_subject_from_path(self, &target_path);
        text_input::sync_session(self);
        self.drag_drop.clear();
        Some(event)
    }

    pub fn clear_text_drag_drop(&mut self) -> bool {
        self.drag_drop.clear()
    }

    pub fn text_drop_caret(&self) -> Option<(ui::Path, Rect)> {
        let target = self.drag_drop.text_target()?;

        Some((target.path().clone(), target.caret_rect()))
    }

    fn text_drag_source_at(
        &self,
        target: &ui::Path,
        position: point::Logical,
        text_engine: &mut text::Engine,
    ) -> Option<(std::ops::Range<usize>, String, bool)> {
        let field = self.text_field(target)?;
        if !field.allows_copy() {
            return None;
        }

        let range = field.buffer().selected_range()?;
        let selected_text = field.buffer().selected_text()?;
        let cursor = self.text_field_cursor_at(target, position, text_engine)?;

        (range.start <= cursor.index && cursor.index <= range.end).then_some((
            range,
            selected_text,
            field.allows_cut(),
        ))
    }

    fn text_field_cursor_at(
        &self,
        target: &ui::Path,
        position: point::Logical,
        text_engine: &mut text::Engine,
    ) -> Option<text::Cursor> {
        self.composition.as_ref()?.text_field_cursor_at(
            target,
            position,
            self.text_field_states
                .get(target)
                .cloned()
                .unwrap_or_default(),
            text_engine,
        )
    }

    fn text_field_caret_rect_at_cursor(
        &self,
        target: &ui::Path,
        cursor: text::Cursor,
        text_engine: &mut text::Engine,
    ) -> Option<Rect> {
        self.composition.as_ref()?.text_field_caret_rect_at_cursor(
            target,
            cursor,
            self.text_field_states
                .get(target)
                .cloned()
                .unwrap_or_default(),
            text_engine,
        )
    }

    pub fn sync_text_field_states(&mut self, text_engine: &mut text::Engine) -> bool {
        let Some(composition) = self.composition.as_ref() else {
            let changed = !self.text_field_states.is_empty();
            self.text_field_states.clear();
            self.last_text_field_click = None;
            self.text_selection_drag = None;
            self.drag_drop.clear();
            let session_changed = text_input::sync_session(self);
            return changed || session_changed;
        };

        let mut changed = composition.sync_text_field_states(
            &mut self.text_field_states,
            self.text_input_session.target(),
            text_engine,
        );

        for (path, field) in composition.text_fields() {
            let state = self.text_field_states.entry(path.clone()).or_default();
            changed |= state.sync_history(field.buffer());
        }

        changed
    }

    pub fn reset_text_field_caret_blink(&mut self, target: &ui::Path, now: Instant) -> bool {
        if !self.is_text_field(target) && !self.text_field_states.contains_key(target) {
            return false;
        }

        let current = self
            .text_field_states
            .get(target)
            .cloned()
            .unwrap_or_else(|| text::TextFieldState::new_at(0.0, now));
        let next = current.clone().reset_caret_blink(now);

        if next == current {
            return false;
        }

        self.text_field_states.insert(target.clone(), next);
        true
    }

    pub(crate) fn record_text_field_history(
        &mut self,
        target: &ui::Path,
        change: text::TextChange,
        kind: text::HistoryKind,
    ) -> bool {
        if !self.is_text_field(target) && !self.text_field_states.contains_key(target) {
            return false;
        }

        self.text_field_states
            .entry(target.clone())
            .or_default()
            .record_history(change, kind);
        true
    }

    pub(crate) fn can_apply_text_edit(&self, target: &ui::Path, edit: &text::Edit) -> bool {
        let Some(field) = self.text_field(target) else {
            return false;
        };

        if !field.is_selectable() {
            return false;
        }

        !edit.mutates_text() || field.allows_text_mutation()
    }

    pub(crate) fn apply_text_history_command(
        &mut self,
        target: &ui::Path,
        buffer: &mut text::Buffer,
        command: text::Command,
    ) -> text::CommandResult {
        let Some(state) = self.text_field_states.get_mut(target) else {
            return text::CommandResult {
                unavailable: true,
                ..text::CommandResult::default()
            };
        };

        match command {
            text::Command::Undo => state.apply_undo(buffer),
            text::Command::Redo => state.apply_redo(buffer),
            _ => text::CommandResult {
                unavailable: true,
                ..text::CommandResult::default()
            },
        }
    }

    pub fn animation_schedule(&self, now: Instant) -> animation::Schedule {
        let Some(focus) = self.focus.as_ref() else {
            return animation::Schedule::Idle;
        };
        let Some(field) = self.text_field(&focus.path) else {
            return animation::Schedule::Idle;
        };

        if !field.paints_caret() || field.buffer().has_selection() {
            return animation::Schedule::Idle;
        }

        let state = self
            .text_field_states
            .get(&focus.path)
            .cloned()
            .unwrap_or_default();

        animation::Schedule::At(state.next_caret_deadline(now))
    }

    fn text_field_click_kind(
        &mut self,
        target: &ui::Path,
        position: point::Logical,
    ) -> text::PointerEditKind {
        let now = Instant::now();
        let count = self
            .last_text_field_click
            .as_ref()
            .filter(|click| {
                click.path == *target
                    && now.duration_since(click.at) <= MULTI_CLICK_MAX_INTERVAL
                    && point_distance(click.position, position) <= MULTI_CLICK_MAX_DISTANCE
            })
            .map_or(1, |click| (click.count + 1).min(3));

        self.last_text_field_click = Some(TextFieldClick {
            path: target.clone(),
            position,
            at: now,
            count,
        });

        match count {
            1 => text::PointerEditKind::Click,
            2 => text::PointerEditKind::DoubleClick,
            _ => text::PointerEditKind::TripleClick,
        }
    }

    pub fn set_hovered(&mut self, target: Option<ui::Path>) -> Vec<ui::Event> {
        if self.hovered == target {
            return Vec::new();
        }

        let old = self.hovered.clone();
        self.hovered = target.clone();
        let mut events = Vec::new();

        if let Some(target) = old {
            events.push(ui::Event::PointerLeft { target });
        }

        if let Some(target) = target {
            events.push(ui::Event::PointerEntered { target });
        }

        events
    }

    pub fn pointer_down(
        &mut self,
        position: point::Logical,
        delta: point::Logical,
        target: Option<ui::Path>,
        button: pointer::Button,
    ) -> ui::Event {
        let preserve_focus = button == pointer::Button::Primary
            && target
                .as_ref()
                .is_some_and(|target| matches!(self.intent(target), Some(ui::Intent::OpenMenu(_))));

        if !preserve_focus {
            if let Some(path) = target.as_ref().filter(|target| self.is_focusable(target)) {
                let visibility =
                    self.focus_visibility_for_activation(path, action::Source::Pointer);
                self.set_focus(path.clone(), ui::focus::Reason::Pointer, visibility);
            } else {
                self.clear_focus();
            }
        }
        if let Some(target) = target.as_ref() {
            self.reset_text_field_caret_blink(target, Instant::now());
        }
        if let Some(target) = target.as_ref().filter(|_| !preserve_focus) {
            command::set_subject_from_path(self, target);
        }
        self.pressed = target.clone();
        self.pressed_source = target.as_ref().map(|_| PressSource::Pointer);

        ui::Event::PointerDown {
            position,
            delta,
            target,
            button,
        }
    }

    pub fn focus_visibility_for_activation(
        &self,
        target: &ui::Path,
        source: action::Source,
    ) -> ui::focus::Visibility {
        match source {
            action::Source::Keyboard => ui::focus::Visibility::Visible,
            action::Source::Pointer if self.is_selectable_text_field(target) => {
                ui::focus::Visibility::Visible
            }
            _ => ui::focus::Visibility::Hidden,
        }
    }

    pub fn pointer_up(
        &mut self,
        position: point::Logical,
        delta: point::Logical,
        target: Option<ui::Path>,
        button: pointer::Button,
    ) -> (ui::Event, Option<ui::Path>) {
        let pressed = if self.pressed_source == Some(PressSource::Pointer) {
            self.pressed.take()
        } else {
            None
        };
        if self.pressed_source == Some(PressSource::Pointer) {
            self.pressed_source = None;
        }
        let routed_target = pressed.clone().or(target);
        let invoke = if button == pointer::Button::Primary {
            pressed
        } else {
            None
        }
        .filter(|target| self.is_actionable(target));

        (
            ui::Event::PointerUp {
                position,
                delta,
                target: routed_target,
                button,
            },
            invoke,
        )
    }

    pub fn toggle_menu<T>(
        &mut self,
        id: menu::Id,
        registry: &action::Registry<T>,
        window: window::Id,
        source: action::Source,
    ) -> bool {
        if self.open_menu == Some(id) {
            return self.close_menu();
        }

        let Some(menu) = self
            .composition
            .as_ref()
            .and_then(|composition| composition.menu(id))
        else {
            return false;
        };

        if !self.menu_can_open(menu, registry, window) {
            return false;
        }

        self.floating.close_all();
        self.focus
            .close_transient(focus::TransientScope::ContextMenu, None);
        let command_context = self.command_context(window);
        self.begin_menu_focus_scope(ui::Path::from(widget::MENU_POPUP));
        let focus_policy = match source {
            action::Source::Keyboard | action::Source::Shortcut => {
                ui::floating::FocusPolicy::FocusFirstEnabledRow
            }
            action::Source::Pointer | action::Source::Programmatic => {
                ui::floating::FocusPolicy::PreserveCurrentFocus
            }
        };
        self.floating
            .open_top_menu(id, command_context, source, focus_policy);
        self.sync_open_menu_mirrors();
        true
    }

    pub fn open_submenu<T>(
        &mut self,
        id: menu::Id,
        registry: &action::Registry<T>,
        window: window::Id,
        source: action::Source,
    ) -> bool {
        if self.open_menu.is_none() || self.open_submenu == Some(id) {
            return false;
        }

        let Some(menu) = self
            .composition
            .as_ref()
            .and_then(|composition| composition.menu(id))
        else {
            return false;
        };

        if !self.menu_can_open(menu, registry, window) {
            return false;
        }

        if self.floating.open_menu().is_none()
            && let Some(open_menu) = self.open_menu
        {
            let command_context = self.command_context(window);
            self.floating.open_top_menu(
                open_menu,
                command_context,
                source,
                ui::floating::FocusPolicy::PreserveCurrentFocus,
            );
        }

        self.focus.include_transient_root(
            focus::TransientScope::Submenu,
            ui::Path::from(widget::MENU_SUBMENU_POPUP),
        );
        let command_context = self.command_context(window);
        self.floating.show_submenu(id, command_context, source);
        self.sync_open_menu_mirrors();
        true
    }

    pub fn close_submenu(&mut self) -> bool {
        self.close_submenu_with_focus_visibility(None)
    }

    pub fn close_submenu_with_focus_visibility(
        &mut self,
        visibility: Option<ui::focus::Visibility>,
    ) -> bool {
        let had_floating_menu = self.floating.open_menu().is_some();
        let had_submenu = self.open_submenu.is_some();
        let closed_floating = self.floating.close_submenu();
        let changed = had_submenu || closed_floating;
        if had_floating_menu {
            self.sync_open_menu_mirrors();
        } else {
            self.open_submenu = None;
        }
        let closed = self
            .focus
            .close_transient(focus::TransientScope::Submenu, visibility)
            || changed;
        let session_changed = text_input::sync_session(self);

        closed || session_changed
    }

    pub fn close_menu(&mut self) -> bool {
        self.close_menu_with_focus_visibility(None)
    }

    pub fn close_menu_with_focus_visibility(
        &mut self,
        visibility: Option<ui::focus::Visibility>,
    ) -> bool {
        let had_surface = self.open_menu.is_some()
            || self.open_submenu.is_some()
            || self.floating.has_open_surface();
        let closed_floating = self.floating.close_all();
        let changed = had_surface || closed_floating;
        self.sync_open_menu_mirrors();
        let closed_context_menu = self
            .focus
            .close_transient(focus::TransientScope::ContextMenu, visibility);
        let closed_submenu = self
            .focus
            .close_transient(focus::TransientScope::Submenu, visibility);
        let closed_menu = self
            .focus
            .close_transient(focus::TransientScope::Menu, visibility);

        let session_changed = text_input::sync_session(self);

        changed || closed_context_menu || closed_submenu || closed_menu || session_changed
    }

    pub fn dismiss_menu_for_target(&mut self, target: Option<&ui::Path>) -> bool {
        if self.open_menu.is_none() && !self.floating.has_open_surface() {
            return false;
        }

        if target.is_some_and(|target| self.is_menu_path(target)) {
            return false;
        }

        self.close_menu()
    }

    pub fn is_menu_path(&self, path: &ui::Path) -> bool {
        self.is_dropdown_path(path)
            || path.ids().iter().enumerate().any(|(index, _)| {
                let candidate = ui::Path::new(path.ids()[..=index].to_vec());
                matches!(
                    self.intent(&candidate),
                    Some(ui::Intent::OpenMenu(_) | ui::Intent::OpenSubmenu(_))
                )
            })
    }

    pub fn is_dropdown_path(&self, path: &ui::Path) -> bool {
        path.ids().iter().any(|id| *id == widget::MENU_POPUP)
            || self.is_submenu_popup_path(path)
            || self.is_context_menu_popup_path(path)
    }

    pub fn is_top_menu_popup_path(&self, path: &ui::Path) -> bool {
        path.ids().iter().any(|id| *id == widget::MENU_POPUP) && !self.is_submenu_popup_path(path)
    }

    pub fn is_submenu_popup_path(&self, path: &ui::Path) -> bool {
        path.ids()
            .iter()
            .any(|id| *id == widget::MENU_SUBMENU_POPUP)
    }

    pub fn is_context_menu_popup_path(&self, path: &ui::Path) -> bool {
        path.ids()
            .iter()
            .any(|id| *id == widget::TEXT_CONTEXT_MENU_POPUP)
    }

    pub fn focus_preserves_text_input_session(&self, path: &ui::Path) -> bool {
        self.is_menu_path(path)
            || matches!(
                self.command_subject(path),
                ui::CommandSubject::Current | ui::CommandSubject::Captured
            )
    }

    pub fn focused_path(&self) -> Option<ui::Path> {
        self.focus.path()
    }

    pub fn focus_visibility(&self) -> ui::focus::Visibility {
        self.focus.visibility()
    }

    pub fn set_focus(
        &mut self,
        path: ui::Path,
        reason: ui::focus::Reason,
        visibility: ui::focus::Visibility,
    ) -> bool {
        if !self.is_focusable(&path) {
            return self.clear_focus();
        }

        self.prepare_focus_scope_for_path(&path);
        let focus = Focus::new(path.clone(), reason, visibility);

        let subject_changed = command::set_subject_from_path(self, &path);
        if self.focus.as_ref() == Some(&focus) {
            return subject_changed || text_input::sync_session(self);
        }

        self.focus.set(focus);
        self.reset_text_field_caret_blink(&path, Instant::now());
        text_input::sync_session(self);
        true
    }

    pub fn clear_focus(&mut self) -> bool {
        let changed = self.focus.clear();
        let session_changed = text_input::sync_session(self);

        changed || session_changed
    }

    pub fn clear_stale_focus(&mut self) -> bool {
        let focusable = self
            .focused_path()
            .is_some_and(|path| self.is_focusable(&path));
        let changed = self.focus.clear_stale(|_| focusable);
        let session_changed = text_input::sync_session(self);

        changed || session_changed
    }

    pub fn sync_menu_focus_scopes(&mut self) -> bool {
        let mut changed = false;

        if self.open_menu.is_some()
            && let Some(root) = self.popup_root_path(widget::MENU_POPUP)
        {
            changed |= self.begin_menu_focus_scope(root);
        }

        if self.open_submenu.is_some()
            && let Some(root) = self.popup_root_path(widget::MENU_SUBMENU_POPUP)
        {
            changed |= self.begin_submenu_focus_scope(root);
        }

        if self.floating.context_menu().is_some()
            && let Some(root) = self.popup_root_path(widget::TEXT_CONTEXT_MENU_POPUP)
        {
            changed |= self.begin_context_menu_focus_scope(root);
        }

        changed
    }

    pub fn focus_first_floating_row<T>(
        &mut self,
        registry: &action::Registry<T>,
        window: window::Id,
    ) -> bool {
        if !self.floating.take_keyboard_focus_request() {
            return false;
        }

        let target = self.composition.as_ref().and_then(|composition| {
            composition
                .focus_order()
                .iter()
                .find(|path| {
                    self.is_dropdown_path(path) && self.can_focus_path(registry, window, path)
                })
                .cloned()
        });

        let Some(target) = target else {
            return false;
        };

        self.set_focus(
            target,
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible,
        )
    }

    pub fn open_text_context_menu(
        &mut self,
        window: window::Id,
        target: ui::Path,
        anchor: point::Logical,
        source: action::Source,
    ) -> bool {
        if !self.is_selectable_text_field(&target) {
            return false;
        }

        let mut changed = self.floating.close_all();
        changed |= self
            .focus
            .close_transient(focus::TransientScope::Submenu, None);
        changed |= self
            .focus
            .close_transient(focus::TransientScope::Menu, None);
        changed |= self
            .focus
            .close_transient(focus::TransientScope::ContextMenu, None);
        self.sync_open_menu_mirrors();

        let reason = match source {
            action::Source::Keyboard => ui::focus::Reason::Keyboard,
            action::Source::Pointer | action::Source::Programmatic | action::Source::Shortcut => {
                ui::focus::Reason::Pointer
            }
        };
        let visibility = self.focus_visibility_for_activation(&target, source);
        changed |= self.set_focus(target.clone(), reason, visibility);
        changed |= command::set_subject_from_path(self, &target);
        text_input::sync_session(self);

        changed |= self.floating.open_context_menu(
            target.clone(),
            anchor,
            action::Context::path(window, target),
            source,
        );
        changed |=
            self.begin_context_menu_focus_scope(ui::Path::from(widget::TEXT_CONTEXT_MENU_POPUP));
        self.sync_open_menu_mirrors();

        changed
    }

    pub fn command_context(&self, window: window::Id) -> action::Context {
        command::context(self, window)
    }

    pub fn action_context_for_path(&self, window: window::Id, path: &ui::Path) -> action::Context {
        command::context_for_path(self, window, path)
    }

    pub fn set_command_subject(&mut self, context: action::Context) -> bool {
        command::set_subject(self, context)
    }

    pub fn clear_command_subject(&mut self) -> bool {
        command::clear_subject(self)
    }

    pub fn clear_stale_command_subject(&mut self) -> bool {
        command::clear_stale_subject(self)
    }

    pub fn update_command_scope_captures(&mut self, window: window::Id) {
        command::update_scope_captures(self, window);
    }

    pub fn resolve_request(&self, request: action::Request) -> action::Request {
        command::resolve_request(self, request)
    }

    fn prepare_focus_scope_for_path(&mut self, path: &ui::Path) {
        if self.open_menu.is_some() && self.is_dropdown_path(path) {
            self.begin_menu_focus_scope(path.clone());
        }

        if self.is_submenu_popup_path(path) {
            self.begin_submenu_focus_scope(path.clone());
        }

        if self.is_context_menu_popup_path(path) {
            self.begin_context_menu_focus_scope(path.clone());
        }
    }

    fn begin_menu_focus_scope(&mut self, root: ui::Path) -> bool {
        let restore = text_input::editing_target(self)
            .map(|target| {
                let visibility =
                    self.focus_visibility_for_activation(&target, action::Source::Pointer);
                Focus::new(target, ui::focus::Reason::Programmatic, visibility)
            })
            .or_else(|| self.focus.as_ref().cloned());

        self.focus
            .begin_transient_with_restore(focus::TransientScope::Menu, root, restore)
    }

    fn begin_submenu_focus_scope(&mut self, root: ui::Path) -> bool {
        self.focus
            .begin_transient(focus::TransientScope::Submenu, root)
    }

    fn begin_context_menu_focus_scope(&mut self, root: ui::Path) -> bool {
        let restore = text_input::editing_target(self)
            .map(|target| {
                let visibility =
                    self.focus_visibility_for_activation(&target, action::Source::Pointer);
                Focus::new(target, ui::focus::Reason::Programmatic, visibility)
            })
            .or_else(|| self.focus.as_ref().cloned());

        self.focus
            .begin_transient_with_restore(focus::TransientScope::ContextMenu, root, restore)
    }

    pub fn sync_open_menu_mirrors(&mut self) {
        self.open_menu = self.floating.open_menu();
        self.open_submenu = self.floating.open_submenu();
    }

    fn popup_root_path(&self, id: ui::Id) -> Option<ui::Path> {
        self.composition
            .as_ref()
            .and_then(|composition| path_with_leaf(composition.layout(), id))
    }

    fn can_focus_path<T>(
        &self,
        registry: &action::Registry<T>,
        window: window::Id,
        path: &ui::Path,
    ) -> bool {
        if !self.is_focusable(path) {
            return false;
        }

        let Some(action) = self
            .composition
            .as_ref()
            .and_then(|composition| composition.action(path))
        else {
            return true;
        };

        registry.can_invoke(action, self.action_context_for_path(window, path))
    }
}

fn scroll_capture_offset(
    metrics: widget::scroll::Metrics,
    part: widget::Part,
    position: point::Logical,
) -> Option<point::Logical> {
    match part.scroll()? {
        widget::scroll::Part::VerticalThumb if metrics.max_offset().y() > 0.0 => {
            let thumb = metrics.vertical_thumb()?;
            Some(point::logical(0.0, position.y() - thumb.origin.y()))
        }
        widget::scroll::Part::HorizontalThumb if metrics.max_offset().x() > 0.0 => {
            let thumb = metrics.horizontal_thumb()?;
            Some(point::logical(position.x() - thumb.origin.x(), 0.0))
        }
        _ => None,
    }
}

fn point_distance(a: point::Logical, b: point::Logical) -> f32 {
    let dx = a.x() - b.x();
    let dy = a.y() - b.y();

    (dx.mul_add(dx, dy * dy)).sqrt()
}

fn path_with_leaf(frame: &ui::Frame, id: ui::Id) -> Option<ui::Path> {
    if frame.path().leaf() == Some(id) {
        return Some(frame.path().clone());
    }

    frame
        .children()
        .iter()
        .find_map(|child| path_with_leaf(child, id))
}

pub fn action_request(
    state: &WindowState,
    window: window::Id,
    origin: ui::Path,
    source: action::Source,
) -> Option<action::Request> {
    let action = match state.intent(&origin) {
        Some(ui::Intent::Action(action)) => action,
        Some(ui::Intent::OpenMenu(_) | ui::Intent::OpenSubmenu(_) | ui::Intent::CloseSubmenu) => {
            return None;
        }
        None => state
            .composition
            .as_ref()
            .and_then(|composition| composition.action(&origin))?,
    };
    let context = state.action_context_for_path(window, &origin);

    Some(action::Request::new(action, source, context).with_origin(origin))
}

impl WindowState {
    fn menu_can_open<T>(
        &self,
        menu: &menu::Menu,
        registry: &action::Registry<T>,
        window: window::Id,
    ) -> bool {
        if self.composition.is_none() {
            return false;
        }

        menu.actions().any(|action| {
            let request = action::Request::new(
                action,
                action::Source::Pointer,
                self.command_context(window),
            );
            let request = self.resolve_request(request);

            self.can_execute_menu_action(registry, &request)
        })
    }

    fn can_execute_menu_action<T>(
        &self,
        registry: &action::Registry<T>,
        request: &action::Request,
    ) -> bool {
        let Some(definition) = registry.action(request.action()) else {
            return false;
        };

        if !definition.payload().accepts(request.payload()) {
            return false;
        }

        if registry
            .state(request.action(), request.target().clone())
            .is_busy()
        {
            return false;
        }

        let Some(command) = text_input::command_for_action(request.action()) else {
            return registry.can_execute(request);
        };

        let action::Scope::Path(target) = request.target().scope() else {
            return false;
        };

        let Some(field) = self.text_field(target) else {
            return registry.can_execute(request);
        };

        text_input::can_apply_command(self, target, field, command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Action;
    use crate::geometry::{Rect, area};
    use crate::widget::menu;

    const ROOT: ui::Id = ui::Id::new("root");
    const CHILD: ui::Id = ui::Id::new("child");
    const OUTSIDE: ui::Id = ui::Id::new("outside");
    const CLICK: action::Id = action::Id::new("click");
    const FILE: menu::Id = menu::Id::new("file");
    const PANELS: menu::Id = menu::Id::new("panels");

    fn path(id: ui::Id) -> ui::Path {
        ui::Path::from(id)
    }

    fn single_box(id: ui::Id) -> crate::ui::Frame {
        crate::layout::Frame::<ui::Path>::new(
            ui::Path::root(id),
            Rect::new(point::logical(0.0, 0.0), area::logical(20.0, 20.0)),
            Vec::new(),
        )
    }

    fn composition(
        layout: crate::ui::Frame,
        menus: HashMap<menu::Id, menu::Menu>,
        actions: HashMap<ui::Path, action::Id>,
        command_subjects: HashMap<ui::Path, ui::CommandSubject>,
        intents: HashMap<ui::Path, ui::Intent>,
        responders: HashMap<ui::Path, Vec<action::Id>>,
        interactivity: HashMap<ui::Path, ui::Interactivity>,
        focus_order: Vec<ui::Path>,
    ) -> ui::Composition {
        ui::Composition::for_test(
            layout,
            menus,
            actions,
            command_subjects,
            intents,
            responders,
            Vec::new(),
            interactivity,
            HashMap::new(),
            focus_order,
        )
    }

    fn state_with_composition(
        layout: crate::ui::Frame,
        menus: HashMap<menu::Id, menu::Menu>,
        actions: HashMap<ui::Path, action::Id>,
        command_subjects: HashMap<ui::Path, ui::CommandSubject>,
        intents: HashMap<ui::Path, ui::Intent>,
        responders: HashMap<ui::Path, Vec<action::Id>>,
        interactivity: HashMap<ui::Path, ui::Interactivity>,
        focus_order: Vec<ui::Path>,
    ) -> WindowState {
        WindowState {
            composition: Some(composition(
                layout,
                menus,
                actions,
                command_subjects,
                intents,
                responders,
                interactivity,
                focus_order,
            )),
            ..WindowState::default()
        }
    }

    fn text_field_window_state(field: impl Into<text::Field>, epoch: Instant) -> WindowState {
        let window = window::Id::new(1);
        let mut tree = ui::Tree::new();
        let mut registry = action::Registry::<()>::new();
        let mut text_engine = text::Engine::new();

        tree.set_root(widget::text_field(CHILD, field).with_size(
            crate::layout::Size::Fixed(120.0),
            crate::layout::Size::Fixed(32.0),
        ));

        let composition = tree
            .compose(
                window,
                area::logical(120.0, 32.0),
                &mut registry,
                &[],
                &mut text_engine,
            )
            .expect("text field tree should compose");

        let mut state = WindowState {
            focus: FocusState::focused(Focus::new(
                path(CHILD),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            composition: Some(composition),
            text_field_states: HashMap::from([(
                path(CHILD),
                text::TextFieldState::new_at(0.0, epoch),
            )]),
            ..WindowState::default()
        };
        text_input::sync_session(&mut state);
        state
    }

    fn state_from_tree(root: ui::Node) -> WindowState {
        let window = window::Id::new(1);
        let mut tree = ui::Tree::new();
        let mut registry = action::Registry::<()>::new();
        let mut text_engine = text::Engine::new();

        tree.set_root(root);
        let composition = tree
            .compose(
                window,
                area::logical(200.0, 80.0),
                &mut registry,
                &[],
                &mut text_engine,
            )
            .expect("tree should compose");

        WindowState {
            composition: Some(composition),
            ..WindowState::default()
        }
    }

    #[test]
    fn hover_changes_emit_leave_then_enter() {
        let mut state = WindowState {
            hovered: Some(path(ROOT)),
            ..WindowState::default()
        };

        let events = state.set_hovered(Some(path(CHILD)));

        assert_eq!(
            events,
            vec![
                ui::Event::PointerLeft { target: path(ROOT) },
                ui::Event::PointerEntered {
                    target: path(CHILD)
                }
            ]
        );
    }

    #[test]
    fn cursor_for_hovered_resolves_hovered_node_cursor() {
        let root = ui::Node::container(ROOT, crate::layout::Axis::Vertical)
            .with_child(
                widget::text_field(CHILD, text::Buffer::from_text("Editable")).with_size(
                    crate::layout::Size::Fixed(120.0),
                    crate::layout::Size::Fixed(32.0),
                ),
            )
            .with_child(widget::button(OUTSIDE, CLICK).with_size(
                crate::layout::Size::Fixed(120.0),
                crate::layout::Size::Fixed(32.0),
            ));
        let mut state = state_from_tree(root);

        state.hovered = Some(ui::Path::new([ROOT, CHILD]));
        assert_eq!(state.cursor_for_hovered(), ui::Cursor::Text);

        state.hovered = Some(ui::Path::new([ROOT, OUTSIDE]));
        assert_eq!(state.cursor_for_hovered(), ui::Cursor::Default);
    }

    #[test]
    fn cursor_for_hovered_resolves_default_when_hover_leaves_window() {
        let mut state =
            text_field_window_state(text::Buffer::from_text("Editable"), Instant::now());

        state.hovered = Some(path(CHILD));
        assert_eq!(state.cursor_for_hovered(), ui::Cursor::Text);

        state.hovered = None;
        assert_eq!(state.cursor_for_hovered(), ui::Cursor::Default);
    }

    #[test]
    fn cursor_for_hovered_prefers_pointer_capture_target() {
        let root = ui::Node::container(ROOT, crate::layout::Axis::Vertical)
            .with_child(
                widget::text_field(CHILD, text::Buffer::from_text("Editable")).with_size(
                    crate::layout::Size::Fixed(120.0),
                    crate::layout::Size::Fixed(32.0),
                ),
            )
            .with_child(widget::button(OUTSIDE, CLICK).with_size(
                crate::layout::Size::Fixed(120.0),
                crate::layout::Size::Fixed(32.0),
            ));
        let mut state = state_from_tree(root);

        state.hovered = Some(ui::Path::new([ROOT, OUTSIDE]));
        state.pointer_capture = Some(pointer::Capture::new(
            ui::Path::new([ROOT, CHILD]),
            widget::Part::Scroll(widget::scroll::Part::VerticalThumb),
            pointer::Button::Primary,
            point::logical(0.0, 0.0),
            point::logical(0.0, 0.0),
        ));

        assert_eq!(state.cursor_for_hovered(), ui::Cursor::Text);
    }

    #[test]
    fn pointer_down_updates_focused_element() {
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            Vec::new(),
        );

        let event = state.pointer_down(
            point::logical(1.0, 2.0),
            point::logical(0.0, 0.0),
            Some(path(CHILD)),
            pointer::Button::Primary,
        );

        assert_eq!(state.focused_path(), Some(path(CHILD)));
        assert_eq!(
            state.focus.as_ref().map(|focus| focus.reason),
            Some(ui::focus::Reason::Pointer)
        );
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Hidden);
        assert_eq!(state.pressed, Some(path(CHILD)));
        assert_eq!(
            event,
            ui::Event::PointerDown {
                position: point::logical(1.0, 2.0),
                delta: point::logical(0.0, 0.0),
                target: Some(path(CHILD)),
                button: pointer::Button::Primary
            }
        );
    }

    #[test]
    fn pointer_down_on_editable_text_field_shows_focus_ring() {
        let mut state = text_field_window_state(text::Buffer::from_text("hello"), Instant::now());

        state.pointer_down(
            point::logical(1.0, 2.0),
            point::logical(0.0, 0.0),
            Some(path(CHILD)),
            pointer::Button::Primary,
        );

        assert_eq!(state.focused_path(), Some(path(CHILD)));
        assert_eq!(
            state.focus.as_ref().map(|focus| focus.reason),
            Some(ui::focus::Reason::Pointer)
        );
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Visible);
    }

    #[test]
    fn pointer_down_on_read_only_text_field_shows_focus_ring() {
        let mut state =
            text_field_window_state(text::Field::new("hello").read_only(), Instant::now());

        state.pointer_down(
            point::logical(1.0, 2.0),
            point::logical(0.0, 0.0),
            Some(path(CHILD)),
            pointer::Button::Primary,
        );

        assert_eq!(state.focused_path(), Some(path(CHILD)));
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Visible);
    }

    #[test]
    fn pointer_down_on_disabled_text_field_does_not_focus() {
        let mut state =
            text_field_window_state(text::Field::new("hello").disabled(), Instant::now());

        state.pointer_down(
            point::logical(1.0, 2.0),
            point::logical(0.0, 0.0),
            Some(path(CHILD)),
            pointer::Button::Primary,
        );

        assert_eq!(state.focused_path(), None);
        assert_eq!(state.pressed, Some(path(CHILD)));
    }

    #[test]
    fn passive_pointer_down_does_not_focus_element() {
        let mut state = WindowState::default();

        state.pointer_down(
            point::logical(1.0, 2.0),
            point::logical(0.0, 0.0),
            Some(path(CHILD)),
            pointer::Button::Primary,
        );

        assert_eq!(state.focused_path(), None);
        assert_eq!(state.pressed, Some(path(CHILD)));
    }

    #[test]
    fn focused_text_field_schedules_next_caret_blink() {
        let epoch = Instant::now();
        let state = text_field_window_state(text::Buffer::from_text("hello"), epoch);

        assert_eq!(
            state.animation_schedule(epoch),
            animation::Schedule::At(epoch + Duration::from_millis(500))
        );
    }

    #[test]
    fn unfocused_window_state_has_idle_animation_schedule() {
        let epoch = Instant::now();
        let mut state = text_field_window_state(text::Buffer::from_text("hello"), epoch);
        state.clear_focus();

        assert_eq!(state.animation_schedule(epoch), animation::Schedule::Idle);
    }

    #[test]
    fn selected_text_field_has_idle_animation_schedule() {
        let epoch = Instant::now();
        let mut engine = text::Engine::new();
        let mut buffer = text::Buffer::from_text("hello");
        engine.apply_text_edit(&mut buffer, text::Edit::SelectAll);
        let state = text_field_window_state(buffer, epoch);

        assert_eq!(state.animation_schedule(epoch), animation::Schedule::Idle);
    }

    #[test]
    fn resetting_text_field_caret_blink_moves_next_deadline() {
        let epoch = Instant::now();
        let later = epoch + Duration::from_millis(200);
        let mut state = text_field_window_state(text::Buffer::from_text("hello"), epoch);

        assert!(state.reset_text_field_caret_blink(&path(CHILD), later));

        assert_eq!(
            state.animation_schedule(later),
            animation::Schedule::At(later + Duration::from_millis(500))
        );
    }

    #[test]
    fn programmatic_focus_can_choose_visible_or_hidden_indication() {
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            Vec::new(),
        );

        assert!(state.set_focus(
            path(CHILD),
            ui::focus::Reason::Programmatic,
            ui::focus::Visibility::Visible,
        ));
        assert_eq!(state.focused_path(), Some(path(CHILD)));
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Visible);

        assert!(state.set_focus(
            path(CHILD),
            ui::focus::Reason::Programmatic,
            ui::focus::Visibility::Hidden,
        ));
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Hidden);
    }

    #[test]
    fn stale_focused_paths_are_cleared_when_not_focusable() {
        let mut state = WindowState {
            focus: FocusState::focused(Focus::new(
                path(CHILD),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            ..WindowState::default()
        };

        assert!(state.clear_stale_focus());
        assert_eq!(state.focused_path(), None);
    }

    #[test]
    fn pointer_capture_routes_release_to_pressed_element() {
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            Vec::new(),
        );
        state.pressed = Some(path(CHILD));
        state.pressed_source = Some(PressSource::Pointer);

        let (event, invoke) = state.pointer_up(
            point::logical(50.0, 50.0),
            point::logical(0.0, 0.0),
            Some(path(OUTSIDE)),
            pointer::Button::Primary,
        );

        assert_eq!(
            event,
            ui::Event::PointerUp {
                position: point::logical(50.0, 50.0),
                delta: point::logical(0.0, 0.0),
                target: Some(path(CHILD)),
                button: pointer::Button::Primary
            }
        );
        assert_eq!(invoke, Some(path(CHILD)));
    }

    #[test]
    fn non_primary_release_does_not_invoke_action() {
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            Vec::new(),
        );
        state.pressed = Some(path(CHILD));
        state.pressed_source = Some(PressSource::Pointer);

        let (_, invoke) = state.pointer_up(
            point::logical(1.0, 1.0),
            point::logical(0.0, 0.0),
            Some(path(CHILD)),
            pointer::Button::Secondary,
        );

        assert_eq!(invoke, None);
    }

    #[test]
    fn passive_pressed_element_does_not_invoke_action() {
        let mut state = WindowState {
            pressed: Some(path(CHILD)),
            pressed_source: Some(PressSource::Pointer),
            ..WindowState::default()
        };

        let (_, invoke) = state.pointer_up(
            point::logical(1.0, 1.0),
            point::logical(0.0, 0.0),
            Some(path(CHILD)),
            pointer::Button::Primary,
        );

        assert_eq!(invoke, None);
    }

    #[test]
    fn pointer_release_over_pressed_action_emits_contextual_request() {
        let window = window::Id::new(1);
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::from([(path(CHILD), CLICK)]),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            Vec::new(),
        );
        let mut registry = action::Registry::<()>::new();

        registry.register(Action::new(CLICK, "Click"));
        state.pointer_down(
            point::logical(1.0, 1.0),
            point::logical(0.0, 0.0),
            Some(path(CHILD)),
            pointer::Button::Primary,
        );
        let (_, target) = state.pointer_up(
            point::logical(1.0, 1.0),
            point::logical(0.0, 0.0),
            Some(path(CHILD)),
            pointer::Button::Primary,
        );
        let request = action_request(
            &state,
            window,
            target.expect("release should target pressed element"),
            action::Source::Pointer,
        )
        .filter(|request| registry.can_execute(request));

        assert_eq!(
            request,
            Some(
                action::Request::new(
                    CLICK,
                    action::Source::Pointer,
                    action::Context::path(window, path(CHILD))
                )
                .with_origin(path(CHILD))
            )
        );
    }

    #[test]
    fn disabled_action_bound_node_does_not_invoke() {
        let window = window::Id::new(1);
        let context = action::Context::path(window, path(CHILD));
        let mut registry = action::Registry::<()>::new();
        let state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::from([(path(CHILD), CLICK)]),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            Vec::new(),
        );

        registry.register(Action::new(CLICK, "Click"));
        registry.set_state(CLICK, context, action::State::disabled());

        assert_eq!(
            action_request(&state, window, path(CHILD), action::Source::Pointer)
                .filter(|request| registry.can_execute(request)),
            None
        );
    }

    #[test]
    fn menu_opens_only_when_an_item_can_invoke_after_resolution() {
        let window = window::Id::new(1);
        let menu = menu::Menu::new(FILE, "File")
            .section(menu::Section::new().item(menu::Item::new(action::SELECT_ALL)));
        let mut registry = action::Registry::<()>::new();
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::from([(FILE, menu)]),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), vec![action::SELECT_ALL])]),
            HashMap::new(),
            Vec::new(),
        );
        state.command_subject = Some(action::Scope::Path(path(CHILD)));

        registry.register(Action::new(action::SELECT_ALL, "Select All"));
        registry.set_state(
            action::SELECT_ALL,
            action::Context::window(window),
            action::State::disabled(),
        );

        assert!(!state.toggle_menu(FILE, &registry, window, action::Source::Pointer));
        assert_eq!(state.open_menu, None);

        registry.set_state(
            action::SELECT_ALL,
            action::Context::path(window, path(CHILD)),
            action::State::enabled(),
        );

        assert!(state.toggle_menu(FILE, &registry, window, action::Source::Pointer));
        assert_eq!(state.open_menu, Some(FILE));
    }

    #[test]
    fn menu_toggle_switches_and_closes_current_menu() {
        let window = window::Id::new(1);
        let edit = menu::Id::new("edit");
        let file_menu = menu::Menu::new(FILE, "File")
            .section(menu::Section::new().item(menu::Item::new(CLICK)));
        let edit_menu = menu::Menu::new(edit, "Edit")
            .section(menu::Section::new().item(menu::Item::new(CLICK)));
        let mut registry = action::Registry::<()>::new();
        let mut state = state_with_composition(
            single_box(ROOT),
            HashMap::from([(FILE, file_menu), (edit, edit_menu)]),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            Vec::new(),
        );

        registry.register(Action::new(CLICK, "Click"));

        assert!(state.toggle_menu(FILE, &registry, window, action::Source::Pointer));
        assert_eq!(state.open_menu, Some(FILE));
        state.open_submenu = Some(PANELS);
        assert!(state.toggle_menu(edit, &registry, window, action::Source::Pointer));
        assert_eq!(state.open_menu, Some(edit));
        assert_eq!(state.open_submenu, None);
        assert!(state.toggle_menu(edit, &registry, window, action::Source::Pointer));
        assert_eq!(state.open_menu, None);
    }

    #[test]
    fn submenu_opens_only_when_parent_menu_is_open_and_item_can_invoke() {
        let window = window::Id::new(1);
        let submenu = menu::Menu::new(PANELS, "Panels")
            .section(menu::Section::new().item(menu::Item::new(CLICK)));
        let mut registry = action::Registry::<()>::new();
        let mut state = state_with_composition(
            single_box(ROOT),
            HashMap::from([(PANELS, submenu)]),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            Vec::new(),
        );

        registry.register(Action::new(CLICK, "Click"));

        assert!(!state.open_submenu(PANELS, &registry, window, action::Source::Pointer));
        state.open_menu = Some(FILE);
        assert!(state.open_submenu(PANELS, &registry, window, action::Source::Pointer));
        assert_eq!(state.open_submenu, Some(PANELS));
    }

    #[test]
    fn closing_top_level_menu_also_closes_submenu() {
        let mut state = WindowState {
            open_menu: Some(FILE),
            open_submenu: Some(PANELS),
            ..WindowState::default()
        };

        assert!(state.close_menu());
        assert_eq!(state.open_menu, None);
        assert_eq!(state.open_submenu, None);
    }

    #[test]
    fn outside_pointer_target_dismisses_open_menu() {
        let mut state = WindowState {
            open_menu: Some(FILE),
            ..WindowState::default()
        };

        assert!(state.dismiss_menu_for_target(Some(&path(CHILD))));
        assert_eq!(state.open_menu, None);
    }

    #[test]
    fn menu_pointer_target_does_not_dismiss_open_menu() {
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), ui::Intent::OpenMenu(FILE))]),
            HashMap::new(),
            HashMap::new(),
            Vec::new(),
        );
        state.open_menu = Some(FILE);

        assert!(!state.dismiss_menu_for_target(Some(&path(CHILD))));
        assert_eq!(state.open_menu, Some(FILE));
        assert!(state.is_menu_path(&path(CHILD)));
    }

    #[test]
    fn submenu_popup_target_does_not_dismiss_open_menu() {
        let submenu_row = ui::Path::new([widget::MENU_SUBMENU_POPUP, CHILD]);
        let mut state = WindowState {
            open_menu: Some(FILE),
            open_submenu: Some(PANELS),
            ..WindowState::default()
        };

        assert!(!state.dismiss_menu_for_target(Some(&submenu_row)));
        assert_eq!(state.open_menu, Some(FILE));
        assert_eq!(state.open_submenu, Some(PANELS));
        assert!(state.is_menu_path(&submenu_row));
    }

    #[test]
    fn busy_action_bound_node_does_not_invoke() {
        let window = window::Id::new(1);
        let context = action::Context::path(window, path(CHILD));
        let mut registry = action::Registry::<()>::new();
        let state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::from([(path(CHILD), CLICK)]),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            Vec::new(),
        );

        registry.register(Action::new(CLICK, "Click"));
        registry.set_busy(CLICK, context, true);

        assert_eq!(
            action_request(&state, window, path(CHILD), action::Source::Pointer)
                .filter(|request| registry.can_execute(request)),
            None
        );
    }

    #[test]
    fn command_subject_survives_focus_changes() {
        let window = window::Id::new(1);
        let mut state = state_with_composition(
            single_box(OUTSIDE),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(OUTSIDE), ui::Interactivity::CONTROL)]),
            Vec::new(),
        );
        state.command_subject = Some(action::Scope::Path(path(CHILD)));
        state.focus = FocusState::focused(Focus::new(
            path(ROOT),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible,
        ));

        assert!(state.set_focus(
            path(OUTSIDE),
            ui::focus::Reason::Programmatic,
            ui::focus::Visibility::Hidden
        ));
        assert_eq!(
            state.command_context(window),
            action::Context::path(window, path(CHILD))
        );
    }

    #[test]
    fn command_subject_setters_update_command_subject_behavior() {
        let window = window::Id::new(1);
        let subject = action::Context::path(window, path(CHILD));
        let mut state = WindowState::default();

        assert!(state.set_command_subject(subject.clone()));
        assert_eq!(state.command_context(window), subject);
        assert!(!state.set_command_subject(action::Context::path(window, path(CHILD))));
        assert!(state.clear_command_subject());
        assert_eq!(
            state.command_context(window),
            action::Context::window(window)
        );
    }

    #[test]
    fn transient_focus_does_not_replace_command_subject() {
        let window = window::Id::new(1);
        let mut state = state_with_composition(
            single_box(OUTSIDE),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), vec![CLICK])]),
            HashMap::from([(path(OUTSIDE), ui::Interactivity::CONTROL)]),
            Vec::new(),
        );
        state.command_subject = Some(action::Scope::Path(path(CHILD)));

        assert!(state.set_focus(
            path(OUTSIDE),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible
        ));
        assert_eq!(
            state.command_context(window),
            action::Context::path(window, path(CHILD))
        );
    }

    #[test]
    fn responder_focus_replaces_command_subject() {
        let window = window::Id::new(1);
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(ROOT), vec![CLICK]), (path(CHILD), vec![CLICK])]),
            HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            Vec::new(),
        );
        state.command_subject = Some(action::Scope::Path(path(ROOT)));

        assert!(state.set_focus(
            path(CHILD),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible
        ));
        assert_eq!(
            state.command_context(window),
            action::Context::path(window, path(CHILD))
        );
    }

    #[test]
    fn focused_responder_automatically_becomes_command_subject() {
        let window = window::Id::new(1);
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), vec![CLICK])]),
            HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            Vec::new(),
        );

        assert!(state.set_focus(
            path(CHILD),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible
        ));
        assert_eq!(
            state.command_context(window),
            action::Context::path(window, path(CHILD))
        );
    }

    #[test]
    fn refocusing_same_responder_restores_cleared_command_subject() {
        let window = window::Id::new(1);
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), vec![CLICK])]),
            HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            Vec::new(),
        );
        state.focus = FocusState::focused(Focus::new(
            path(CHILD),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible,
        ));

        assert_eq!(state.command_subject, None);
        assert!(state.set_focus(
            path(CHILD),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible
        ));
        assert_eq!(
            state.command_context(window),
            action::Context::path(window, path(CHILD))
        );
    }

    #[test]
    fn command_subject_falls_back_to_focus_then_window() {
        let window = window::Id::new(1);
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(ROOT), vec![CLICK]), (path(CHILD), vec![CLICK])]),
            HashMap::new(),
            Vec::new(),
        );
        state.hovered = Some(path(ROOT));
        state.focus = FocusState::focused(Focus::new(
            path(CHILD),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible,
        ));

        assert_eq!(
            state.command_context(window),
            action::Context::path(window, path(CHILD))
        );

        state.focus = FocusState::default();
        assert_eq!(
            state.command_context(window),
            action::Context::window(window)
        );
    }

    #[test]
    fn hover_alone_does_not_become_command_subject() {
        let window = window::Id::new(1);
        let mut state = state_with_composition(
            single_box(ROOT),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(ROOT), vec![CLICK])]),
            HashMap::new(),
            Vec::new(),
        );
        state.hovered = Some(path(ROOT));

        assert_eq!(
            state.command_context(window),
            action::Context::window(window)
        );
    }

    #[test]
    fn stale_command_subject_is_cleared_when_path_disappears() {
        let window = window::Id::new(1);
        let mut state = state_with_composition(
            single_box(ROOT),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(ROOT), ui::Interactivity::CONTROL)]),
            Vec::new(),
        );
        state.command_subject = Some(action::Scope::Path(path(CHILD)));

        assert!(state.clear_stale_command_subject());
        assert_eq!(state.command_subject, None);
        assert_eq!(
            state.command_context(window),
            action::Context::window(window)
        );
    }

    #[test]
    fn command_subject_policy_resolves_stored_subject() {
        let window = window::Id::new(1);
        let mut state = state_with_composition(
            single_box(ROOT),
            HashMap::new(),
            HashMap::from([(path(ROOT), CLICK)]),
            HashMap::from([(path(ROOT), ui::CommandSubject::Current)]),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            Vec::new(),
        );
        state.command_subject = Some(action::Scope::Path(path(CHILD)));

        let request = action_request(&state, window, path(ROOT), action::Source::Pointer)
            .expect("command-subject action should produce request");

        assert_eq!(request.origin(), Some(&path(ROOT)));
        assert_eq!(request.payload(), &action::Payload::None);
        assert_eq!(
            request.target(),
            &action::Context::path(window, path(CHILD))
        );
    }

    #[test]
    fn command_subject_policy_resolves_window_without_subject() {
        let window = window::Id::new(1);
        let state = state_with_composition(
            single_box(ROOT),
            HashMap::new(),
            HashMap::from([(path(ROOT), CLICK)]),
            HashMap::from([(path(ROOT), ui::CommandSubject::Current)]),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            Vec::new(),
        );

        let request = action_request(&state, window, path(ROOT), action::Source::Pointer)
            .expect("command-subject action should produce request");

        assert_eq!(request.target(), &action::Context::window(window));
    }

    #[test]
    fn window_subject_policy_resolves_window_context() {
        let window = window::Id::new(1);
        let mut state = state_with_composition(
            single_box(ROOT),
            HashMap::new(),
            HashMap::from([(path(ROOT), CLICK)]),
            HashMap::from([(path(ROOT), ui::CommandSubject::Window)]),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            Vec::new(),
        );
        state.command_subject = Some(action::Scope::Path(path(CHILD)));

        let request = action_request(&state, window, path(ROOT), action::Source::Pointer)
            .expect("window-subject action should produce request");

        assert_eq!(request.origin(), Some(&path(ROOT)));
        assert_eq!(request.target(), &action::Context::window(window));
    }

    #[test]
    fn captured_subject_policy_resolves_nearest_scope_capture() {
        let window = window::Id::new(1);
        let scope = path(ROOT);
        let origin = ui::Path::new([ROOT, CHILD]);
        let subject = path(OUTSIDE);
        let mut state = state_with_composition(
            single_box(ROOT),
            HashMap::new(),
            HashMap::from([(origin.clone(), CLICK)]),
            HashMap::from([(origin.clone(), ui::CommandSubject::Captured)]),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            Vec::new(),
        );
        state
            .command_scope_captures
            .insert(scope, action::Context::path(window, subject.clone()));

        let request = action_request(&state, window, origin, action::Source::Pointer)
            .expect("captured-target action should produce request");

        assert_eq!(request.target(), &action::Context::path(window, subject));
    }

    #[test]
    fn local_responder_inside_scope_becomes_command_subject() {
        let window = window::Id::new(1);
        let local = ui::Path::new([ROOT, CHILD]);
        let button = ui::Path::new([ROOT, OUTSIDE]);
        let mut state = state_with_composition(
            single_box(ROOT),
            HashMap::new(),
            HashMap::from([(button.clone(), CLICK)]),
            HashMap::from([(button.clone(), ui::CommandSubject::Current)]),
            HashMap::new(),
            HashMap::from([(local.clone(), vec![CLICK])]),
            HashMap::from([(local.clone(), ui::Interactivity::CONTROL)]),
            Vec::new(),
        );

        assert!(state.set_focus(
            local.clone(),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible
        ));
        let request = action_request(&state, window, button, action::Source::Pointer)
            .expect("command-subject action should produce request");

        assert_eq!(request.target(), &action::Context::path(window, local));
    }

    #[test]
    fn responder_resolution_picks_nearest_handler() {
        let window = window::Id::new(1);
        let root = path(ROOT);
        let child = ui::Path::new([ROOT, CHILD]);
        let outside = ui::Path::new([ROOT, CHILD, OUTSIDE]);
        let state = state_with_composition(
            single_box(ROOT),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(root.clone(), vec![CLICK]), (child.clone(), vec![CLICK])]),
            HashMap::new(),
            Vec::new(),
        );
        let request = action::Request::new(
            CLICK,
            action::Source::Shortcut,
            action::Context::path(window, outside),
        );

        assert_eq!(
            state.resolve_request(request).target(),
            &action::Context::path(window, child)
        );
    }

    #[test]
    fn responder_resolution_falls_back_to_window_without_handler() {
        let window = window::Id::new(1);
        let state = state_with_composition(
            single_box(ROOT),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(ROOT), vec![action::COPY])]),
            HashMap::new(),
            Vec::new(),
        );
        let request = action::Request::new(
            CLICK,
            action::Source::Shortcut,
            action::Context::path(window, path(ROOT)),
        );

        assert_eq!(
            state.resolve_request(request).target(),
            &action::Context::window(window)
        );
    }

    #[test]
    fn origin_bound_action_resolves_to_itself() {
        let window = window::Id::new(1);
        let state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::from([(path(CHILD), CLICK)]),
            HashMap::from([(path(CHILD), ui::CommandSubject::Origin)]),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            Vec::new(),
        );
        let request = action::Request::new(
            CLICK,
            action::Source::Pointer,
            action::Context::path(window, path(CHILD)),
        )
        .with_origin(path(CHILD));

        assert_eq!(
            state.resolve_request(request).target(),
            &action::Context::path(window, path(CHILD))
        );
    }

    #[test]
    fn disabled_responder_target_blocks_invocation_after_resolution() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(path(CHILD), vec![action::SELECT_ALL])]),
            HashMap::new(),
            Vec::new(),
        );
        let request = action::Request::new(
            action::SELECT_ALL,
            action::Source::Shortcut,
            action::Context::path(window, path(CHILD)),
        );

        registry.register(Action::new(action::SELECT_ALL, "Select All"));
        registry.set_state(
            action::SELECT_ALL,
            action::Context::path(window, path(CHILD)),
            action::State::disabled(),
        );
        let request = state.resolve_request(request);

        assert!(!registry.can_execute(&request));
    }
}
