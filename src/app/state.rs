use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::time::{Duration, Instant};

use crate::animation;
use crate::geometry::{Rect, point};
use crate::widget::menu;
use crate::{command, pointer, text, ui, widget, window};

use super::{
    command as command_subject, drag_drop, floating, focus, frame, paint_cache, scroll,
    text as app_text, text_input,
};

pub use focus::Focus;
#[cfg(test)]
pub(crate) use focus::State as FocusState;

const MULTI_CLICK_MAX_INTERVAL: Duration = Duration::from_millis(350);
const MULTI_CLICK_MAX_DISTANCE: f32 = 4.0;
const TEXT_DRAG_THRESHOLD: f32 = 4.0;
const SCROLL_IDLE_REFINEMENT_DELAY: Duration = Duration::from_millis(80);

#[derive(Debug, Default)]
pub struct WindowState {
    pub hovered: Option<ui::Path>,
    pub focus: focus::State,
    pub pressed: Option<ui::Path>,
    pub pressed_source: Option<PressSource>,
    pub modifiers: ui::Modifiers,
    pub command_subject: Option<command::call::Scope>,
    pub pointer: pointer::Pointer,
    pub floating: floating::State,
    pub open_menu: Option<menu::Id>,
    pub open_submenu: Option<menu::Id>,
    pub command_scope_captures: HashMap<ui::Path, command::call::Context>,
    pub pointer_capture: Option<pointer::Capture>,
    pub composition: Option<ui::Composition>,
    pub(crate) paint_cache: Option<paint_cache::RetainedPaint>,
    pub scroll: scroll::Driver,
    pub text_input_session: text_input::Session,
    pub drag_drop: drag_drop::State,
    pub text: app_text::Driver,
    pub last_text_field_click: Option<TextFieldClick>,
    pub text_pointer_gesture: Option<TextPointerGesture>,
    pub(crate) last_scroll_input_at: Option<Instant>,
    pub(crate) scroll_commit_targets: HashSet<ui::Path>,
    pub(crate) async_scroll_targets: HashSet<ui::Path>,
    pub(crate) frame: frame::State,
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
    count: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextPointerGesture {
    SelectionDrag(ui::Path),
    DragCandidate(TextDragCandidate),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextDragCandidate {
    path: ui::Path,
    start_position: point::Logical,
    selected_range: Range<usize>,
    selected_text: String,
    source_editable: bool,
    click_edit: text::edit::Edit,
}

impl TextDragCandidate {
    fn new(
        path: ui::Path,
        start_position: point::Logical,
        selected_range: Range<usize>,
        selected_text: String,
        source_editable: bool,
        click_edit: text::edit::Edit,
    ) -> Self {
        Self {
            path,
            start_position,
            selected_range,
            selected_text,
            source_editable,
            click_edit,
        }
    }

    fn crosses_drag_threshold(&self, position: point::Logical) -> bool {
        point_distance(self.start_position, position) >= TEXT_DRAG_THRESHOLD
    }
}

impl WindowState {
    pub(crate) fn invalidate_frame(&mut self, kind: frame::RedrawKind, now: Instant) {
        self.frame.invalidate(kind, now);
    }

    pub(crate) fn frame_diagnostics(&self) -> frame::Diagnostics {
        self.frame.diagnostics()
    }

    pub(crate) fn take_scroll_input_latency(&mut self, now: Instant) -> Option<Duration> {
        self.last_scroll_input_at
            .take()
            .map(|input| now.saturating_duration_since(input))
    }

    pub(crate) fn retain_paint(
        &mut self,
        scene: crate::paint::Scene,
        scroll_ranges: ui::ScrollPaintRecords,
    ) -> Vec<crate::paint::LayerUpdate> {
        let retained = paint_cache::RetainedPaint::new(scene, scroll_ranges);
        let updates = retained.layer_updates();
        self.scroll
            .set_retained_layers(retained.retained_scroll_layers());
        self.paint_cache = Some(retained);
        updates
    }

    pub(crate) fn clear_paint_cache(&mut self) {
        self.paint_cache = None;
    }

    pub(crate) fn paint_cache(&self) -> Option<&paint_cache::RetainedPaint> {
        self.paint_cache.as_ref()
    }

    pub(crate) fn paint_cache_mut(&mut self) -> Option<&mut paint_cache::RetainedPaint> {
        self.paint_cache.as_mut()
    }

    pub(crate) fn committed_scroll_targets(&self) -> Vec<ui::Path> {
        self.scroll_commit_targets.iter().cloned().collect()
    }

    pub(crate) fn clear_committed_scroll_targets(&mut self) {
        self.scroll_commit_targets.clear();
    }

    pub(crate) fn clear_async_scroll_targets(&mut self) {
        self.async_scroll_targets.clear();
    }

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

    pub fn scroll_target(
        &self,
        position: point::Logical,
        text_engine: &mut text::layout::Engine,
    ) -> Option<ui::Path> {
        let layout = self.composition.as_ref()?.layout();

        if !self.scroll.is_empty()
            && let Some(target) = self.scroll.scroll_target_in_frame(layout, position)
        {
            return Some(target);
        }

        scroll_target_in_frame(self, layout, position, text_engine)
    }

    pub fn cursor_for_hovered(&self) -> ui::Cursor {
        let Some(composition) = self.composition.as_ref() else {
            return ui::Cursor::Default;
        };

        if self.pointer_capture_is_scrollbar() {
            return ui::Cursor::Default;
        }

        if let Some(capture) = self.pointer_capture.as_ref() {
            return composition.cursor(capture.target());
        }

        self.hovered
            .as_ref()
            .map(|path| composition.cursor(path))
            .unwrap_or_default()
    }

    pub fn cursor_for_pointer(&self, text_engine: &mut text::layout::Engine) -> ui::Cursor {
        if self.pointer_capture_is_scrollbar() {
            return ui::Cursor::Default;
        }

        if let Some(position) = self.pointer.position()
            && self
                .widget_hit(position, text_engine)
                .is_some_and(|hit| hit.part().scroll().is_some())
        {
            return ui::Cursor::Default;
        }

        self.cursor_for_hovered()
    }

    pub fn widget_hit(
        &self,
        position: point::Logical,
        text_engine: &mut text::layout::Engine,
    ) -> Option<widget::Hit> {
        if !self.scroll.is_empty()
            && let Some(hit) = self.scroll.widget_hit(position)
        {
            return Some(hit);
        }

        let composition = self.composition.as_ref()?;
        let mut hit = composition
            .widget_metrics_iter()
            .filter_map(|(path, metrics)| {
                metrics
                    .hit_test(position)
                    .map(|part| widget::Hit::new(path.clone(), part))
            })
            .max_by_key(|hit| hit.target().ids().len());

        for (path, surface) in composition.text_surfaces() {
            if !surface.is_area() {
                continue;
            }

            let Some(metrics) = self.text_area_scroll_metrics(path, text_engine) else {
                continue;
            };
            let Some(part) = metrics.hit_test(position).map(widget::Part::Scroll) else {
                continue;
            };
            let candidate = widget::Hit::new(path.clone(), part);
            if hit
                .as_ref()
                .is_none_or(|hit| candidate.target().ids().len() > hit.target().ids().len())
            {
                hit = Some(candidate);
            }
        }

        hit
    }

    pub fn scroll_metrics(&self, target: &ui::Path) -> Option<widget::scroll::Metrics> {
        match self.composition.as_ref()?.widget_metrics(target)? {
            widget::Metrics::Scroll(metrics) => Some(metrics),
        }
    }

    pub fn scroll_metrics_for(
        &self,
        target: &ui::Path,
        text_engine: &mut text::layout::Engine,
    ) -> Option<widget::scroll::Metrics> {
        self.scroll
            .metrics(target)
            .or_else(|| self.scroll_metrics(target))
            .or_else(|| self.text_area_scroll_metrics(target, text_engine))
    }

    pub fn text_area_scroll_metrics(
        &self,
        target: &ui::Path,
        text_engine: &mut text::layout::Engine,
    ) -> Option<widget::scroll::Metrics> {
        if let Some(metrics) = self.scroll.metrics(target) {
            return Some(metrics);
        }

        let state = self.text_state_for_layout(target);

        self.composition
            .as_ref()?
            .text_area_scroll_metrics(target, state, text_engine)
    }

    pub(crate) fn text_state_for_layout(&self, target: &ui::Path) -> text::view::TextViewState {
        let state = self.text.get_cloned_or_default(target);
        if self
            .text_surface(target)
            .is_some_and(text::Surface::is_area)
            && let Some(offset) = self.scroll.visual_offset(target)
        {
            return state.with_scroll(offset.x(), offset.y());
        }
        state
    }

    fn text_state_for_storage(
        &self,
        target: &ui::Path,
        state: text::view::TextViewState,
    ) -> text::view::TextViewState {
        if self
            .text_surface(target)
            .is_some_and(text::Surface::is_area)
        {
            state.without_scroll()
        } else {
            state
        }
    }

    fn store_text_state(
        &mut self,
        target: &ui::Path,
        state: text::view::TextViewState,
    ) -> Option<text::view::TextViewState> {
        let state = self.text_state_for_storage(target, state);
        self.text.insert(target.clone(), state)
    }

    pub fn start_pointer_capture(
        &mut self,
        hit: &widget::Hit,
        button: pointer::Button,
        position: point::Logical,
        text_engine: &mut text::layout::Engine,
    ) -> bool {
        let Some(metrics) = self.scroll_metrics_for(hit.target(), text_engine) else {
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
        text_engine: &mut text::layout::Engine,
    ) -> Option<(ui::Path, point::Logical)> {
        let capture = self.pointer_capture.as_ref()?;
        let part = capture.part().scroll()?;
        let metrics = self.scroll_metrics_for(capture.target(), text_engine)?;
        let offset = metrics.drag_offset(part, position, capture.grab_offset())?;

        Some((capture.target().clone(), offset))
    }

    pub fn clear_pointer_capture(&mut self) -> bool {
        let changed = self.pointer_capture.is_some();
        self.pointer_capture = None;
        changed
    }

    pub fn pointer_capture_is_scrollbar(&self) -> bool {
        self.pointer_capture
            .as_ref()
            .is_some_and(|capture| capture.part().scroll().is_some())
    }

    pub fn pointer_capture_is_scroll_thumb(&self) -> bool {
        self.pointer_capture
            .as_ref()
            .and_then(|capture| capture.part().scroll())
            .is_some_and(|part| {
                matches!(
                    part,
                    widget::scroll::Part::VerticalThumb | widget::scroll::Part::HorizontalThumb
                )
            })
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

    pub fn text_surface(&self, target: &ui::Path) -> Option<&crate::text::Surface> {
        self.composition
            .as_ref()
            .and_then(|composition| composition.text_surface(target))
    }

    pub fn is_text_field(&self, target: &ui::Path) -> bool {
        self.text_surface(target).is_some()
    }

    pub fn is_selectable_text_field(&self, target: &ui::Path) -> bool {
        self.text_surface(target)
            .is_some_and(crate::text::Surface::is_selectable)
    }

    pub fn is_editable_text_field(&self, target: &ui::Path) -> bool {
        self.text_surface(target)
            .is_some_and(crate::text::Surface::is_editable)
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

    pub fn focused_text_field_caret_rect(
        &self,
        text_engine: &mut text::layout::Engine,
    ) -> Option<Rect> {
        let target = self.focused_selectable_text_field()?;
        let state = self.text_state_for_layout(&target);

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
        let current = self.text.get_cloned_or_default(&target);
        let next = current.clone().with_preedit(preedit);

        if next != current {
            self.store_text_state(&target, next);
        }

        Some(target)
    }

    pub fn clear_text_field_preedits(&mut self) -> bool {
        let mut changed = false;

        for state in self.text.values_mut() {
            if state.preedit().is_some() {
                *state = state.clone().with_preedit(None);
                changed = true;
            }
        }

        changed || !self.scroll_commit_targets.is_empty()
    }

    pub fn text_field_edit_at(
        &mut self,
        target: &ui::Path,
        position: point::Logical,
        text_engine: &mut text::layout::Engine,
    ) -> Option<text::edit::Edit> {
        if !self.is_selectable_text_field(target) {
            return None;
        }

        let kind = self.text_field_click_kind(target, position);
        let text_position = self.text_field_position_at(target, position, text_engine)?;
        let edit = text::edit::Edit::pointer(kind.clone(), text_position);

        if kind == text::edit::PointerEditKind::Click
            && let Some((range, selected_text, source_editable)) =
                self.text_drag_source_at(target, position, text_engine)
        {
            self.text_pointer_gesture =
                Some(TextPointerGesture::DragCandidate(TextDragCandidate::new(
                    target.clone(),
                    position,
                    range,
                    selected_text,
                    source_editable,
                    edit,
                )));
            self.reset_text_field_caret_blink(target, Instant::now());
            return None;
        }

        self.text_pointer_gesture = Some(TextPointerGesture::SelectionDrag(target.clone()));
        self.reset_text_field_caret_blink(target, Instant::now());

        Some(edit)
    }

    pub fn text_field_drag_edit_at(
        &mut self,
        position: point::Logical,
        text_engine: &mut text::layout::Engine,
    ) -> Option<(ui::Path, text::edit::Edit)> {
        let target = match self.text_pointer_gesture.as_ref()? {
            TextPointerGesture::SelectionDrag(target) => target.clone(),
            TextPointerGesture::DragCandidate(_) => return None,
        };
        if !self.is_selectable_text_field(&target) {
            return None;
        }
        let text_position = self.text_field_position_at(&target, position, text_engine)?;
        let edit = text::edit::Edit::pointer(text::edit::PointerEditKind::Drag, text_position);
        self.reset_text_field_caret_blink(&target, Instant::now());

        Some((target, edit))
    }

    pub fn finish_text_pointer_gesture(&mut self) -> Option<(ui::Path, text::edit::Edit)> {
        let Some(TextPointerGesture::DragCandidate(candidate)) = self.text_pointer_gesture.take()
        else {
            self.text_pointer_gesture = None;
            return None;
        };

        self.reset_text_field_caret_blink(&candidate.path, Instant::now());

        Some((candidate.path, candidate.click_edit))
    }

    pub fn cancel_text_pointer_gesture(&mut self) -> bool {
        let changed = self.text_pointer_gesture.is_some();
        self.text_pointer_gesture = None;
        changed
    }

    pub fn update_text_drag(
        &mut self,
        position: point::Logical,
        text_engine: &mut text::layout::Engine,
    ) -> bool {
        if self.drag_drop.active_text().is_some() {
            return self.update_text_drop_target(position, text_engine);
        }

        let Some(TextPointerGesture::DragCandidate(candidate)) = self.text_pointer_gesture.as_ref()
        else {
            return false;
        };

        if !candidate.crosses_drag_threshold(position) {
            return false;
        }

        self.last_text_field_click = None;

        let candidate = match self.text_pointer_gesture.take() {
            Some(TextPointerGesture::DragCandidate(candidate)) => candidate,
            _ => return false,
        };
        let was_active = self.drag_drop.active_text().is_some();
        self.drag_drop.start_text(
            candidate.path,
            candidate.selected_range,
            candidate.selected_text,
            candidate.source_editable,
        );
        let changed = !was_active;

        self.update_text_drop_target(position, text_engine) || changed
    }

    pub fn update_text_drop_target(
        &mut self,
        position: point::Logical,
        text_engine: &mut text::layout::Engine,
    ) -> bool {
        let Some(source) = self.drag_drop.active_text().cloned() else {
            return self.drag_drop.clear_text_target();
        };

        let Some(target) = self.hit_test(position).filter(|target| {
            self.text_surface(target)
                .is_some_and(text::Surface::allows_cut)
        }) else {
            return self.drag_drop.clear_text_target();
        };

        let Some(text_position) = self.text_field_position_at(&target, position, text_engine)
        else {
            return self.drag_drop.clear_text_target();
        };
        let insert_index = text_position.index;

        let target_operations = ui::drag_drop::Operations::COPY_MOVE;
        let operation = self
            .drag_drop
            .operation_for_target(target_operations, self.modifiers);
        if operation == ui::drag_drop::Operation::None
            || (operation == ui::drag_drop::Operation::Move
                && source.path() == &target
                && source.selected_range().contains(&insert_index))
        {
            return self.drag_drop.clear_text_target();
        }

        let Some(caret_rect) =
            self.text_field_caret_rect_at_position(&target, text_position, text_engine)
        else {
            return self.drag_drop.clear_text_target();
        };

        self.drag_drop.set_text_target(
            Some(drag_drop::TextTarget::new(
                target,
                text_position,
                insert_index,
                caret_rect,
            )),
            target_operations,
            self.modifiers,
        )
    }

    pub fn finish_text_drop(&mut self) -> Option<ui::Event> {
        let source = self.drag_drop.active_text().cloned()?;
        let target = self.drag_drop.text_target().cloned()?;
        let operation = self.drag_drop.resolved_operation();

        if operation == ui::drag_drop::Operation::None {
            self.drag_drop.reject();
            return None;
        }

        let text = source.selected_text().to_owned();
        let source_range = source.selected_range();
        let source_edit = if operation == ui::drag_drop::Operation::Move
            && source.can_move()
            && source.path() != target.path()
        {
            Some((
                source.path().clone(),
                text::edit::Edit::replace_range(source_range.clone(), ""),
            ))
        } else {
            None
        };
        let target_edit =
            if operation == ui::drag_drop::Operation::Move && source.path() == target.path() {
                text::edit::Edit::move_range(source_range, target.insert_index())
            } else {
                text::edit::Edit::insert_at(target.insert_index(), text)
            };
        let target_path = target.path().clone();
        let event = ui::Event::TextDropRequested {
            source_cleanup: source_edit,
            target: target_path.clone(),
            edit: target_edit,
            operation,
        };

        self.set_focus(
            target_path.clone(),
            ui::focus::Reason::Pointer,
            self.focus_visibility_for_activation(&target_path, command::call::Source::Pointer),
        );
        command_subject::set_subject_from_path(self, &target_path);
        text_input::sync_session(self);
        self.drag_drop.complete();
        Some(event)
    }

    pub fn clear_text_drag_drop(&mut self) -> bool {
        self.drag_drop.clear()
    }

    pub fn text_drop_caret(&self) -> Option<(ui::Path, Rect)> {
        let target = self.drag_drop.text_target()?;

        Some((target.path().clone(), target.caret_rect()))
    }

    #[cfg(test)]
    pub fn queue_text_area_scroll_by(
        &mut self,
        target: &ui::Path,
        delta: scroll::WheelDelta,
        text_engine: &mut text::layout::Engine,
    ) -> bool {
        self.queue_text_area_scroll_by_at(target, delta, text_engine, Instant::now())
    }

    pub fn queue_text_area_scroll_by_at(
        &mut self,
        target: &ui::Path,
        delta: scroll::WheelDelta,
        text_engine: &mut text::layout::Engine,
        now: Instant,
    ) -> bool {
        if !self
            .text_surface(target)
            .is_some_and(text::Surface::is_area)
        {
            return false;
        }
        self.ensure_scroll_projection_for(target, text_engine);
        let Some(metrics) = self.scroll.metrics(target) else {
            return false;
        };
        let smooth = self.scroll.wheel_delta_smooths(target, delta);
        let base_metrics = if smooth {
            self.scroll
                .target_offset(target)
                .map_or(metrics, |offset| metrics.with_offset(offset))
        } else {
            metrics
        };
        let delta = if smooth {
            self.scroll
                .wheel_impulse_delta_pixels(base_metrics, delta, self.modifiers.shift())
        } else {
            self.scroll
                .wheel_delta_pixels(base_metrics, delta, self.modifiers.shift())
        };
        let offset = base_metrics.wheel_offset(delta);

        self.queue_text_area_scroll_to_resolved(target, offset, smooth, Some(now))
    }

    pub fn mark_scroll_input(&mut self, now: Instant) {
        self.last_scroll_input_at = Some(now);
    }

    pub fn queue_text_area_scroll_to(
        &mut self,
        target: &ui::Path,
        offset: point::Logical,
        text_engine: &mut text::layout::Engine,
    ) -> bool {
        if !self
            .text_surface(target)
            .is_some_and(text::Surface::is_area)
        {
            return false;
        }
        self.ensure_scroll_projection_for(target, text_engine);
        let Some(metrics) = self.scroll.metrics(target) else {
            return false;
        };
        let offset = metrics.clamp_offset(offset);

        self.queue_text_area_scroll_to_resolved(target, offset, false, None)
    }

    fn queue_text_area_scroll_to_resolved(
        &mut self,
        target: &ui::Path,
        offset: point::Logical,
        smooth: bool,
        now: Option<Instant>,
    ) -> bool {
        let Some(metrics) = self.scroll.metrics(target) else {
            return false;
        };
        let target_offset = metrics.clamp_offset(offset);
        let current_offset = if smooth {
            self.scroll
                .target_offset(target)
                .unwrap_or_else(|| metrics.offset())
        } else {
            metrics.offset()
        };
        if current_offset == target_offset && (!smooth || metrics.offset() == target_offset) {
            return false;
        }

        if smooth {
            self.scroll.queue_wheel_offset_at(
                target,
                target_offset,
                now.unwrap_or_else(Instant::now),
            )
        } else {
            self.scroll.queue_offset(target, target_offset)
        }
    }

    #[cfg(test)]
    pub fn commit_pending_scroll_offsets(&mut self) -> bool {
        let pending = self.scroll.drain_pending_offsets();
        if pending.is_empty() {
            return false;
        }

        let mut changed = false;
        let mut committed = false;
        for (target, _) in pending {
            self.scroll_commit_targets.insert(target.clone());
            committed = true;
            if !self
                .text_surface(&target)
                .is_some_and(text::Surface::is_area)
            {
                continue;
            }

            let current = self.text.get_cloned_or_default(&target);
            let next = current.clone().clear_caret_visibility_pending();

            if next != current {
                self.store_text_state(&target, next);
                changed = true;
            }
            self.async_scroll_targets.remove(&target);
        }

        committed || changed
    }

    pub fn commit_pending_visual_scroll_offsets(&mut self, frame: animation::Frame) -> bool {
        self.scroll.advance_smooth_wheel_scrolls(frame);
        let pending = self.scroll.drain_pending_offsets();
        if pending.is_empty() {
            return false;
        }

        for (target, _) in pending {
            self.scroll_commit_targets.insert(target.clone());
            if self
                .text_surface(&target)
                .is_some_and(text::Surface::is_area)
            {
                self.async_scroll_targets.insert(target);
            }
        }
        self.scroll.publish_pending_scroll_diagnostics();
        true
    }

    pub fn smooth_scroll_active(&self) -> bool {
        self.scroll.has_active_smooth_wheel_scrolls()
    }

    pub fn reconcile_async_scroll_target(
        &mut self,
        target: &ui::Path,
        text_engine: &mut text::layout::Engine,
        now: Instant,
    ) -> bool {
        if !self.async_scroll_targets.remove(target) {
            return false;
        }

        if self
            .text_surface(target)
            .is_some_and(text::Surface::is_area)
            && self.scroll.metrics(target).is_some()
        {
            let current = self.text.get_cloned_or_default(target);
            let next = current.clone().clear_caret_visibility_pending();
            if next != current {
                self.store_text_state(target, next);
            }
        }

        self.scroll_commit_targets.remove(target);
        if let Some(composition) = self.composition.as_ref() {
            self.scroll.sync_filtered(
                composition,
                self.text.states(),
                text_engine,
                now,
                Some(&HashSet::from([target.clone()])),
            );
        }
        self.scroll.record_async_scroll_reconcile();
        false
    }

    pub fn reconcile_async_scroll_targets(
        &mut self,
        text_engine: &mut text::layout::Engine,
        now: Instant,
    ) -> bool {
        let targets = self
            .async_scroll_targets
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let mut changed = false;
        for target in targets {
            changed |= self.reconcile_async_scroll_target(&target, text_engine, now);
        }
        changed
    }

    pub fn text_selection_drag_autoscroll_offset(
        &self,
        position: point::Logical,
    ) -> Option<(ui::Path, point::Logical)> {
        let target = match self.text_pointer_gesture.as_ref()? {
            TextPointerGesture::SelectionDrag(target) => target.clone(),
            TextPointerGesture::DragCandidate(_) => return None,
        };
        if !self
            .text_surface(&target)
            .is_some_and(text::Surface::is_area)
        {
            return None;
        }

        let projection = self.scroll.text_area(&target)?;
        let offset =
            text::View::selection_drag_autoscroll_offset(projection.observed_area(), position)?;

        Some((target, offset))
    }

    pub fn text_selection_drag_autoscroll_at(
        &mut self,
        position: point::Logical,
        text_engine: &mut text::layout::Engine,
    ) -> Option<(ui::Path, point::Logical)> {
        let (target, offset) = self.text_selection_drag_autoscroll_offset(position)?;
        if !self.queue_text_area_scroll_to(&target, offset, text_engine) {
            return None;
        }
        self.observe_scroll_projection_for(&target, text_engine);

        Some((target, offset))
    }

    pub(crate) fn text_area_scroll_anchor(&self, target: &ui::Path) -> Option<text::ScrollAnchor> {
        let area_model = self.text_surface(target)?.as_area()?;
        let projection = self.scroll.text_area(target)?;
        projection.scroll_anchor(area_model)
    }

    pub(crate) fn ensure_text_caret_visible_after_edit(
        &mut self,
        target: &ui::Path,
        now: Instant,
        text_engine: &mut text::layout::Engine,
        scroll_anchor: Option<text::ScrollAnchor>,
    ) -> bool {
        self.reconcile_async_scroll_target(target, text_engine, now);
        if !self.is_text_field(target) && !self.text.contains(target) {
            return false;
        }

        let current = self
            .text
            .get(target)
            .cloned()
            .unwrap_or_else(|| text::view::TextViewState::new_at(0.0, now));
        let is_area = self
            .text_surface(target)
            .is_some_and(text::Surface::is_area);
        let current_offset = self
            .scroll
            .visual_offset(target)
            .unwrap_or_else(|| point::logical(current.scroll_x(), current.scroll_y()));
        let mut next = if is_area {
            current
                .clone()
                .with_scroll(current_offset.x(), current_offset.y())
        } else {
            current.clone()
        };
        let mut offset = None;

        if let Some(anchor) = scroll_anchor
            && let Some(composition) = self.composition.as_ref()
            && let Some(scroll_y) =
                composition.text_area_scroll_y_for_anchor(target, next.clone(), anchor, text_engine)
        {
            next = next.with_scroll_y(scroll_y);
        }

        next = text::View::state_after_text_edit(next, now);

        if let Some(composition) = self.composition.as_ref()
            && let Some(ensured) =
                composition.ensure_caret_visible_for_text_surface(target, next.clone(), text_engine)
        {
            next = ensured.clear_caret_visibility_pending();
            offset = Some(point::logical(next.scroll_x(), next.scroll_y()));
        }

        let stored_next = self.text_state_for_storage(target, next.clone());

        if stored_next == current && offset.is_none() {
            return false;
        }

        self.store_text_state(target, stored_next);
        if let Some(offset) = offset {
            self.scroll.update_offset(target, offset);
        }
        true
    }
    pub fn sync_scroll_projections(
        &mut self,
        text_engine: &mut text::layout::Engine,
        now: Instant,
    ) {
        if let Some(composition) = self.composition.as_ref() {
            let targets = self.scroll_projection_targets();
            self.scroll.sync_filtered(
                composition,
                self.text.states(),
                text_engine,
                now,
                Some(&targets),
            );
            self.scroll_commit_targets.clear();
        } else {
            self.scroll.clear();
            self.scroll_commit_targets.clear();
            self.async_scroll_targets.clear();
        }
    }

    fn ensure_scroll_projection_for(
        &mut self,
        target: &ui::Path,
        text_engine: &mut text::layout::Engine,
    ) {
        if self.scroll.metrics(target).is_some()
            && (!self
                .text_surface(target)
                .is_some_and(text::Surface::is_area)
                || self.scroll.text_area(target).is_some())
        {
            return;
        }
        let Some(composition) = self.composition.as_ref() else {
            return;
        };
        let targets = HashSet::from([target.clone()]);
        self.scroll.sync_filtered(
            composition,
            self.text.states(),
            text_engine,
            Instant::now(),
            Some(&targets),
        );
    }

    fn observe_scroll_projection_for(
        &mut self,
        target: &ui::Path,
        text_engine: &mut text::layout::Engine,
    ) {
        let Some(composition) = self.composition.as_ref() else {
            return;
        };
        self.scroll.observe_text_area(
            composition,
            self.text.states(),
            target,
            text_engine,
            Instant::now(),
        );
    }

    pub fn refine_idle_scroll_models(
        &mut self,
        text_engine: &mut text::layout::Engine,
        now: Instant,
    ) -> bool {
        if !self.can_refine_idle_scroll_models(now) {
            if self.scroll_input_is_active(now) {
                self.scroll.record_idle_refinement_suppressed_by_scroll();
            }
            return false;
        }

        let Some(composition) = self.composition.as_ref() else {
            return false;
        };

        self.scroll.refine_idle_text_area_models(
            composition,
            self.text.states(),
            text_engine,
            now,
            1,
        )
    }

    fn can_refine_idle_scroll_models(&self, now: Instant) -> bool {
        self.pointer_capture.is_none()
            && self.text_pointer_gesture.is_none()
            && self.drag_drop.active_text().is_none()
            && self.drag_drop.text_target().is_none()
            && !self.scroll_input_is_active(now)
    }

    fn scroll_input_is_active(&self, now: Instant) -> bool {
        self.last_scroll_input_at
            .is_some_and(|last| now.saturating_duration_since(last) < SCROLL_IDLE_REFINEMENT_DELAY)
    }

    fn scroll_projection_targets(&self) -> HashSet<ui::Path> {
        let mut targets = HashSet::new();

        if let Some(path) = self.hovered.as_ref() {
            targets.insert(path.clone());
        }
        if let Some(path) = self.focused_path() {
            targets.insert(path);
        }
        if let Some(path) = text_input::editing_target(self) {
            targets.insert(path);
        }
        if let Some(capture) = self.pointer_capture.as_ref() {
            targets.insert(capture.target().clone());
        }
        if let Some(source) = self.drag_drop.active_text() {
            targets.insert(source.path().clone());
        }
        if let Some(target) = self.drag_drop.text_target() {
            targets.insert(target.path().clone());
        }
        if let Some(TextPointerGesture::SelectionDrag(target)) = self.text_pointer_gesture.as_ref()
        {
            targets.insert(target.clone());
        }
        targets.extend(self.scroll_commit_targets.iter().cloned());
        let scroll_driver_text_targets = self
            .scroll
            .metric_paths()
            .filter(|path| self.text_surface(path).is_some_and(text::Surface::is_area))
            .cloned()
            .collect::<Vec<_>>();
        targets.extend(scroll_driver_text_targets);

        targets.retain(|path| {
            self.text_surface(path).is_some_and(text::Surface::is_area)
                || self.scroll.metrics(path).is_some()
        });
        targets
    }

    fn text_drag_source_at(
        &mut self,
        target: &ui::Path,
        position: point::Logical,
        text_engine: &mut text::layout::Engine,
    ) -> Option<(std::ops::Range<usize>, String, bool)> {
        let surface = self.text_surface(target)?;
        if !surface.allows_copy() {
            return None;
        }

        let range = surface.buffer().selected_range()?;
        let selected_text = surface.buffer().selected_text()?;
        let source_editable = surface.allows_cut();
        let text_position = self.text_field_position_at(target, position, text_engine)?;
        let cursor_index = text_position.index;

        (range.start <= cursor_index && cursor_index <= range.end).then_some((
            range.as_range(),
            selected_text,
            source_editable,
        ))
    }

    fn text_field_position_at(
        &mut self,
        target: &ui::Path,
        position: point::Logical,
        text_engine: &mut text::layout::Engine,
    ) -> Option<text::TextPosition> {
        self.reconcile_async_scroll_target(target, text_engine, Instant::now());
        let state = self.text_state_for_layout(target);

        if let Some(surface) = self.text_surface(target)
            && let Some(area_model) = surface.as_area()
            && let Some(projection) = self.scroll.text_area(target)
        {
            if let Some(position) = text::View::position_at_observed_area(
                text_engine,
                area_model,
                state.clone(),
                projection.observed_area(),
                position,
            ) {
                return Some(position);
            }
        }

        self.composition
            .as_ref()?
            .text_field_position_at(target, position, state, text_engine)
    }

    fn text_field_caret_rect_at_position(
        &self,
        target: &ui::Path,
        position: text::TextPosition,
        text_engine: &mut text::layout::Engine,
    ) -> Option<Rect> {
        self.composition
            .as_ref()?
            .text_field_caret_rect_at_position(
                target,
                position,
                self.text_state_for_layout(target),
                text_engine,
            )
    }

    pub fn sync_text_field_states(&mut self, text_engine: &mut text::layout::Engine) -> bool {
        let Some(composition) = self.composition.as_ref() else {
            let changed = !self.text.is_empty();
            self.text.clear();
            self.scroll.clear();
            self.async_scroll_targets.clear();
            self.last_text_field_click = None;
            self.text_pointer_gesture = None;
            self.drag_drop.clear();
            let session_changed = text_input::sync_session(self);
            return changed || session_changed;
        };

        let mut changed = composition.sync_text_field_states(
            self.text.states_mut(),
            self.text_input_session.target(),
            text_engine,
        );

        for (path, surface) in composition.text_surfaces() {
            let state = self.text.entry(path.clone()).or_default();
            changed |= state.sync_history(surface.buffer());
            if surface.is_area() {
                let next = state.clone().without_scroll();
                if next != *state {
                    *state = next;
                    changed = true;
                }
            }
        }
        self.async_scroll_targets.retain(|path| {
            composition
                .text_surface(path)
                .is_some_and(|surface| surface.is_area())
        });

        changed
    }

    pub fn reset_text_field_caret_blink(&mut self, target: &ui::Path, now: Instant) -> bool {
        if !self.is_text_field(target) && !self.text.contains(target) {
            return false;
        }

        let current = self
            .text
            .get(target)
            .cloned()
            .unwrap_or_else(|| text::view::TextViewState::new_at(0.0, now));
        let next = text::View::state_after_caret_blink_reset(current.clone(), now);

        if next == current {
            return false;
        }

        self.store_text_state(target, next);
        true
    }

    pub(crate) fn reset_text_field_caret_blink_without_scroll(
        &mut self,
        target: &ui::Path,
        now: Instant,
    ) -> bool {
        if !self.is_text_field(target) && !self.text.contains(target) {
            return false;
        }

        let current = self
            .text
            .get(target)
            .cloned()
            .unwrap_or_else(|| text::view::TextViewState::new_at(0.0, now));
        let next = text::View::state_after_selection_only_change(current.clone(), now);

        if next == current {
            return false;
        }

        self.store_text_state(target, next);
        true
    }

    pub(crate) fn record_text_field_history(
        &mut self,
        target: &ui::Path,
        change: text::buffer::TextChange,
        kind: text::edit::HistoryKind,
        now: Instant,
    ) -> bool {
        if !self.is_text_field(target) && !self.text.contains(target) {
            return false;
        }

        self.text
            .entry(target.clone())
            .or_default()
            .record_history_at(change, kind, now);
        true
    }

    pub(crate) fn can_apply_text_edit(&self, target: &ui::Path, edit: &text::edit::Edit) -> bool {
        let Some(surface) = self.text_surface(target) else {
            return false;
        };

        if !surface.is_selectable() {
            return false;
        }

        !edit.mutates_text() || surface.allows_text_mutation()
    }

    pub(crate) fn apply_text_history_command(
        &mut self,
        target: &ui::Path,
        buffer: &mut text::Buffer,
        command: text::edit::Command,
    ) -> text::edit::CommandResult {
        let Some(state) = self.text.get_mut(target) else {
            return text::edit::CommandResult {
                unavailable: true,
                ..text::edit::CommandResult::default()
            };
        };

        match command {
            text::edit::Command::Undo => state.apply_undo(buffer),
            text::edit::Command::Redo => state.apply_redo(buffer),
            _ => text::edit::CommandResult {
                unavailable: true,
                ..text::edit::CommandResult::default()
            },
        }
    }

    pub fn animation_schedule(&self, now: Instant) -> animation::Schedule {
        if self.text_selection_drag_autoscroll_active() {
            return animation::Schedule::NextFrame;
        }

        if self.smooth_scroll_active() {
            return animation::Schedule::NextFrame;
        }

        let Some(focus) = self.focus.as_ref() else {
            return animation::Schedule::Idle;
        };
        let Some(surface) = self.text_surface(&focus.path) else {
            return animation::Schedule::Idle;
        };

        if !surface.paints_caret() || surface.buffer().has_selection() {
            return animation::Schedule::Idle;
        }

        let state = self.text.get(&focus.path).cloned().unwrap_or_default();

        animation::Schedule::At(state.next_caret_deadline(now))
    }

    pub(crate) fn text_selection_drag_autoscroll_active(&self) -> bool {
        self.pointer
            .position()
            .and_then(|position| self.text_selection_drag_autoscroll_offset(position))
            .is_some()
    }

    fn text_field_click_kind(
        &mut self,
        target: &ui::Path,
        position: point::Logical,
    ) -> text::edit::PointerEditKind {
        let now = Instant::now();
        let count = self
            .last_text_field_click
            .as_ref()
            .filter(|click| {
                click.path == *target
                    && now.duration_since(click.at) <= MULTI_CLICK_MAX_INTERVAL
                    && point_distance(click.position, position) <= MULTI_CLICK_MAX_DISTANCE
            })
            .map_or(1, |click| click.count.saturating_add(1));

        self.last_text_field_click = Some(TextFieldClick {
            path: target.clone(),
            position,
            at: now,
            count,
        });

        match count {
            1 => text::edit::PointerEditKind::Click,
            2 => text::edit::PointerEditKind::DoubleClick,
            3 => text::edit::PointerEditKind::TripleClick,
            _ if count % 2 == 0 => text::edit::PointerEditKind::DoubleClick,
            _ => text::edit::PointerEditKind::TripleClick,
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
                    self.focus_visibility_for_activation(path, command::call::Source::Pointer);
                self.set_focus(path.clone(), ui::focus::Reason::Pointer, visibility);
            } else {
                self.clear_focus();
            }
        }
        if let Some(target) = target.as_ref() {
            self.reset_text_field_caret_blink(target, Instant::now());
        }
        if let Some(target) = target.as_ref().filter(|_| !preserve_focus) {
            command_subject::set_subject_from_path(self, target);
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
        source: command::call::Source,
    ) -> ui::focus::Visibility {
        match source {
            command::call::Source::Keyboard => ui::focus::Visibility::Visible,
            command::call::Source::Pointer if self.is_selectable_text_field(target) => {
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

    pub fn toggle_menu(
        &mut self,
        id: menu::Id,
        registry: &command::Registry,
        window: window::Id,
        source: command::call::Source,
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
            command::call::Source::Keyboard | command::call::Source::Shortcut => {
                ui::floating::FocusPolicy::FocusFirstEnabledRow
            }
            command::call::Source::Pointer | command::call::Source::Programmatic => {
                ui::floating::FocusPolicy::PreserveCurrentFocus
            }
        };
        self.floating
            .open_top_menu(id, command_context, source, focus_policy);
        self.sync_open_menu_mirrors();
        true
    }

    pub fn open_submenu(
        &mut self,
        id: menu::Id,
        registry: &command::Registry,
        window: window::Id,
        source: command::call::Source,
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

        let subject_changed = command_subject::set_subject_from_path(self, &path);
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

    pub fn focus_first_floating_row(
        &mut self,
        registry: &command::Registry,
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

    pub(crate) fn sync_menu_title_states(
        &mut self,
        registry: &command::Registry,
        window: window::Id,
    ) -> bool {
        let Some(composition) = self.composition.as_ref() else {
            return false;
        };

        let menu_titles = composition
            .intents()
            .iter()
            .filter_map(|(path, intent)| match intent {
                ui::Intent::OpenMenu(menu) => composition
                    .menu(*menu)
                    .map(|menu| (path.clone(), menu.clone())),
                ui::Intent::Command(_) | ui::Intent::OpenSubmenu(_) | ui::Intent::CloseSubmenu => {
                    None
                }
            })
            .collect::<Vec<_>>();

        let path_states = menu_titles
            .into_iter()
            .map(|(path, menu)| {
                let state = if self.menu_can_open(&menu, registry, window) {
                    command::State::available()
                } else {
                    command::State::unavailable()
                };

                (path, state)
            })
            .collect::<HashMap<_, _>>();

        self.composition
            .as_mut()
            .is_some_and(|composition| composition.set_path_states(path_states))
    }

    pub fn open_text_context_menu(
        &mut self,
        window: window::Id,
        target: ui::Path,
        anchor: point::Logical,
        source: command::call::Source,
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
            command::call::Source::Keyboard => ui::focus::Reason::Keyboard,
            command::call::Source::Pointer
            | command::call::Source::Programmatic
            | command::call::Source::Shortcut => ui::focus::Reason::Pointer,
        };
        let visibility = self.focus_visibility_for_activation(&target, source);
        changed |= self.set_focus(target.clone(), reason, visibility);
        changed |= command_subject::set_subject_from_path(self, &target);
        text_input::sync_session(self);

        changed |= self.floating.open_context_menu(
            target.clone(),
            anchor,
            command::call::Context::path(window, target),
            source,
        );
        changed |=
            self.begin_context_menu_focus_scope(ui::Path::from(widget::TEXT_CONTEXT_MENU_POPUP));
        self.sync_open_menu_mirrors();

        changed
    }

    pub fn command_context(&self, window: window::Id) -> command::call::Context {
        command_subject::context(self, window)
    }

    pub fn command_context_for_path(
        &self,
        window: window::Id,
        path: &ui::Path,
    ) -> command::call::Context {
        command_subject::context_for_path(self, window, path)
    }

    pub fn set_command_subject(&mut self, context: command::call::Context) -> bool {
        command_subject::set_subject(self, context)
    }

    pub fn clear_command_subject(&mut self) -> bool {
        command_subject::clear_subject(self)
    }

    pub fn clear_stale_command_subject(&mut self) -> bool {
        command_subject::clear_stale_subject(self)
    }

    pub fn update_command_scope_captures(&mut self, window: window::Id) {
        command_subject::update_scope_captures(self, window);
    }

    pub fn resolve_request(
        &self,
        registry: &command::Registry,
        request: command::call::Raw,
    ) -> command::call::Raw {
        command_subject::resolve_request(self, registry, request)
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
                    self.focus_visibility_for_activation(&target, command::call::Source::Pointer);
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
                    self.focus_visibility_for_activation(&target, command::call::Source::Pointer);
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

    fn can_focus_path(
        &self,
        registry: &command::Registry,
        window: window::Id,
        path: &ui::Path,
    ) -> bool {
        if !self.is_focusable(path) {
            return false;
        }

        let Some(route) = self
            .composition
            .as_ref()
            .and_then(|composition| composition.command(path))
        else {
            return true;
        };

        registry.can_invoke_key(route.command(), self.command_context_for_path(window, path))
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

fn scroll_target_in_frame(
    state: &WindowState,
    frame: &ui::Frame,
    position: point::Logical,
    text_engine: &mut text::layout::Engine,
) -> Option<ui::Path> {
    if !rect_contains(frame.rect(), position) {
        return None;
    }

    for child in frame.children().iter().rev() {
        if let Some(target) = scroll_target_in_frame(state, child, position, text_engine) {
            return Some(target);
        }
    }

    state
        .scroll_metrics_for(frame.path(), text_engine)
        .is_some_and(|metrics| metrics.max_offset().x() > 0.0 || metrics.max_offset().y() > 0.0)
        .then(|| frame.path().clone())
}

fn rect_contains(rect: Rect, position: point::Logical) -> bool {
    let x = position.x();
    let y = position.y();
    let left = rect.origin.x();
    let top = rect.origin.y();
    let right = left + rect.area.width();
    let bottom = top + rect.area.height();

    x >= left && x < right && y >= top && y < bottom
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

pub fn command_request(
    state: &WindowState,
    window: window::Id,
    origin: ui::Path,
    source: command::call::Source,
) -> Option<command::call::Raw> {
    let route = match state.intent(&origin) {
        Some(ui::Intent::Command(route)) => route,
        Some(ui::Intent::OpenMenu(_) | ui::Intent::OpenSubmenu(_) | ui::Intent::CloseSubmenu) => {
            return None;
        }
        None => state
            .composition
            .as_ref()
            .and_then(|composition| composition.command(&origin))?,
    };
    let context = state.command_context_for_path(window, &origin);

    Some(command::call::Raw::from_route(route, source, context).with_origin(origin))
}

impl WindowState {
    fn menu_can_open(
        &self,
        menu: &menu::Menu,
        registry: &command::Registry,
        window: window::Id,
    ) -> bool {
        if self.composition.is_none() {
            return false;
        }

        menu.commands().any(|route| {
            let request = command::call::Raw::from_route(
                route,
                command::call::Source::Pointer,
                self.command_context(window),
            );
            let request = self.resolve_request(registry, request);

            self.can_execute_menu_command(registry, &request)
        })
    }

    fn can_execute_menu_command(
        &self,
        registry: &command::Registry,
        request: &command::call::Raw,
    ) -> bool {
        if registry.command_key(request.command()).is_none() {
            return false;
        };

        if registry
            .state_key(request.command(), request.context().clone())
            .is_running()
        {
            return false;
        }

        let Some(command) = text_input::text_command_for(request.command()) else {
            return registry.can_execute(request);
        };

        let command::call::Scope::Path(target) = request.context().scope() else {
            return false;
        };

        let Some(surface) = self.text_surface(target) else {
            return registry.can_execute(request);
        };

        text_input::can_apply_command(self, target, surface, command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;
    use crate::command;
    use crate::geometry::{Rect, area};
    use crate::widget::menu;

    const ROOT: ui::Id = ui::Id::new("root");
    const CHILD: ui::Id = ui::Id::new("child");
    const OUTSIDE: ui::Id = ui::Id::new("outside");
    struct Click;

    impl Command for Click {
        type Args = ();
        type Output = ();

        const NAME: &'static str = "click";
        const DISPLAY: &'static str = "Click";
    }

    const CLICK: command::Key = command::Key::of::<Click>();
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

    fn register_text_command<C>(registry: &mut command::Registry, display: &'static str)
    where
        C: crate::text::command::EditCommand,
    {
        registry.commands(|commands| {
            crate::text::command::define::<C>(commands, |command| command.with_display(display));
        });
    }

    fn text_route<C>() -> command::binding::Route
    where
        C: crate::text::command::EditCommand,
    {
        command::binding::Route::new(
            command::Key::of::<C>(),
            crate::text::command::text_target_kind(),
        )
    }

    fn composition(
        layout: crate::ui::Frame,
        menus: HashMap<menu::Id, menu::Menu>,
        commands: HashMap<ui::Path, command::Key>,
        command_subjects: HashMap<ui::Path, ui::CommandSubject>,
        intents: HashMap<ui::Path, ui::Intent>,
        responders: HashMap<ui::Path, Vec<command::Key>>,
        interactivity: HashMap<ui::Path, ui::Interactivity>,
        focus_order: Vec<ui::Path>,
    ) -> ui::Composition {
        ui::Composition::for_test(
            layout,
            menus,
            commands,
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
        commands: HashMap<ui::Path, command::Key>,
        command_subjects: HashMap<ui::Path, ui::CommandSubject>,
        intents: HashMap<ui::Path, ui::Intent>,
        responders: HashMap<ui::Path, Vec<command::Key>>,
        interactivity: HashMap<ui::Path, ui::Interactivity>,
        focus_order: Vec<ui::Path>,
    ) -> WindowState {
        WindowState {
            composition: Some(composition(
                layout,
                menus,
                commands,
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
        let mut registry = command::Registry::new();
        let mut text_engine = text::layout::Engine::new();

        tree.set_root(widget::text_field(field).key(CHILD).with_size(
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
            text: HashMap::from([(path(CHILD), text::view::TextViewState::new_at(0.0, epoch))])
                .into(),
            ..WindowState::default()
        };
        text_input::sync_session(&mut state);
        state
    }

    fn state_from_tree(root: ui::Node) -> WindowState {
        let window = window::Id::new(1);
        let mut tree = ui::Tree::new();
        let mut registry = command::Registry::new();
        let mut text_engine = text::layout::Engine::new();

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

    fn line_start(lines: &[String], line: usize) -> usize {
        lines.iter().take(line).map(|line| line.len() + 1).sum()
    }

    fn text_area_lines(count: usize) -> Vec<String> {
        (0..count)
            .map(|line| format!("line {line:02} abcdefghijklmnopqrstuvwxyz"))
            .collect()
    }

    fn text_area_window_state(
        buffer: text::Buffer,
        scroll_y: f32,
    ) -> (WindowState, text::layout::Engine, ui::Path) {
        text_area_window_state_for_area(text::Area::new(buffer), scroll_y)
    }

    fn text_area_window_state_for_area(
        area_model: text::Area,
        scroll_y: f32,
    ) -> (WindowState, text::layout::Engine, ui::Path) {
        let window = window::Id::new(1);
        let mut tree = ui::Tree::new();
        let mut registry = command::Registry::new();
        let mut text_engine = text::layout::Engine::new();
        let root = ui::Node::container(crate::layout::Axis::Vertical)
            .key(ROOT)
            .with_size(crate::layout::Size::Fill, crate::layout::Size::Fill)
            .with_child(widget::text_area(area_model).key(CHILD).with_size(
                crate::layout::Size::Fixed(140.0),
                crate::layout::Size::Fixed(60.0),
            ));
        tree.set_root(root);
        let composition = tree
            .compose(
                window,
                area::logical(180.0, 90.0),
                &mut registry,
                &[],
                &mut text_engine,
            )
            .expect("text area tree should compose");
        let path = ui::Path::from(ROOT).child(CHILD);
        let mut state = WindowState {
            composition: Some(composition),
            text: HashMap::from([(
                path.clone(),
                text::view::TextViewState::default().with_scroll_y(scroll_y),
            )])
            .into(),
            ..WindowState::default()
        };
        sync_text_area_projection(&mut state, &mut text_engine);

        (state, text_engine, path)
    }

    fn sync_text_area_projection(state: &mut WindowState, text_engine: &mut text::layout::Engine) {
        state.scroll.sync(
            state
                .composition
                .as_ref()
                .expect("composition should exist"),
            state.text.states(),
            text_engine,
            Instant::now(),
        );
    }

    fn first_visible_text_area_surface(
        state: &WindowState,
        path: &ui::Path,
    ) -> text::layout::TextAreaSurface {
        let projection = state
            .scroll
            .text_area(path)
            .expect("text area projection should exist");
        let viewport_height = projection.metrics().viewport().area.height();
        projection
            .interaction_surfaces()
            .iter()
            .chain(projection.render_surfaces())
            .filter(|surface| {
                surface.y() + surface.height().max(1.0) > 0.0 && surface.y() < viewport_height
            })
            .min_by(|a, b| a.y().total_cmp(&b.y()))
            .cloned()
            .expect("projection should include a visible text surface")
    }

    fn click_in_surface(
        state: &WindowState,
        path: &ui::Path,
        surface: &text::layout::TextAreaSurface,
    ) -> point::Logical {
        let viewport = state
            .scroll
            .text_area(path)
            .expect("text area projection should exist")
            .metrics()
            .viewport();
        point::logical(
            viewport.origin.x() + surface.x() + 4.0,
            viewport.origin.y() + surface.y() + surface.height().max(1.0) * 0.5,
        )
    }

    fn text_area_scroll_offsets(state: &WindowState, path: &ui::Path) -> (f32, f32) {
        let state_y = state
            .text
            .states()
            .get(path)
            .expect("text area state should exist")
            .scroll_y();
        let projection_y = state
            .scroll
            .text_area(path)
            .expect("text area projection should exist")
            .metrics()
            .offset()
            .y();
        (state_y, projection_y)
    }

    fn assert_pointer_edit_hits_surface(
        edit: text::edit::Edit,
        surface: &text::layout::TextAreaSurface,
    ) {
        let text::edit::Edit::Pointer { position, .. } = edit else {
            panic!("expected pointer edit");
        };
        let line_range =
            surface.source_start()..=surface.source_start() + surface.source_text_len();
        assert!(
            line_range.contains(&position.index),
            "hit index {} should land inside painted surface range {:?}",
            position.index,
            line_range
        );
    }

    #[test]
    fn text_area_click_uses_painted_projection_after_line_delete_above() {
        let lines = text_area_lines(80);
        let text = lines.join("\n");
        let mut buffer = text::Buffer::from_multiline_text(text);
        let (mut state, mut text_engine, path) = text_area_window_state(buffer.clone(), 260.0);
        let delete_len = line_start(&lines, 6);
        let mut editor = text::edit::Editor::new();
        let result = editor.apply_text_edit_with_result(
            &mut buffer,
            text::edit::Edit::replace_range(0..delete_len, ""),
        );
        assert!(result.text_changed);
        text_engine.invalidate_text_area_for_edit(&buffer, &result.impacts);
        sync_text_area_projection(&mut state, &mut text_engine);

        let surface = first_visible_text_area_surface(&state, &path);
        let click = click_in_surface(&state, &path, &surface);
        let before_scroll = text_area_scroll_offsets(&state, &path).1;
        text_engine.reset_diagnostics();
        let edit = state
            .text_field_edit_at(&path, click, &mut text_engine)
            .expect("painted text-area click should resolve");

        assert_pointer_edit_hits_surface(edit, &surface);
        let (stored_scroll, driver_scroll) = text_area_scroll_offsets(&state, &path);
        assert_eq!(stored_scroll, 0.0);
        assert_eq!(driver_scroll, before_scroll);
        assert_eq!(text_engine.diagnostics().text_area_paint_layout_calls, 1);
    }

    #[test]
    fn text_area_click_uses_painted_projection_after_line_insert_above() {
        let lines = text_area_lines(80);
        let text = lines.join("\n");
        let mut buffer = text::Buffer::from_multiline_text(text);
        let (mut state, mut text_engine, path) = text_area_window_state(buffer.clone(), 260.0);
        let inserted = (0..5)
            .map(|line| format!("inserted {line:02}"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        let mut editor = text::edit::Editor::new();
        let result = editor.apply_text_edit_with_result(
            &mut buffer,
            text::edit::Edit::replace_range(0..0, inserted),
        );
        assert!(result.text_changed);
        text_engine.invalidate_text_area_for_edit(&buffer, &result.impacts);
        sync_text_area_projection(&mut state, &mut text_engine);

        let surface = first_visible_text_area_surface(&state, &path);
        let click = click_in_surface(&state, &path, &surface);
        let before_scroll = text_area_scroll_offsets(&state, &path).1;
        text_engine.reset_diagnostics();
        let edit = state
            .text_field_edit_at(&path, click, &mut text_engine)
            .expect("painted text-area click should resolve");

        assert_pointer_edit_hits_surface(edit, &surface);
        let (stored_scroll, driver_scroll) = text_area_scroll_offsets(&state, &path);
        assert_eq!(stored_scroll, 0.0);
        assert_eq!(driver_scroll, before_scroll);
        assert_eq!(text_engine.diagnostics().text_area_paint_layout_calls, 1);
    }

    #[test]
    fn text_area_state_storage_strips_scroll_and_layout_uses_scroll_driver() {
        let buffer = text::Buffer::from_multiline_text(text_area_lines(20).join("\n"));
        let (mut state, mut text_engine, path) = text_area_window_state(buffer, 160.0);
        let driver_offset = state
            .scroll
            .metrics(&path)
            .expect("text area projection should exist")
            .offset();

        state.store_text_state(
            &path,
            text::view::TextViewState::default().with_scroll_y(40.0),
        );

        let stored = state.text.get(&path).expect("stored text state");
        assert_eq!(stored.scroll_y(), 0.0);
        assert_eq!(
            state.text_state_for_layout(&path).scroll_y(),
            driver_offset.y()
        );

        assert!(
            state
                .text_field_edit_at(&path, point::logical(4.0, 4.0), &mut text_engine)
                .is_some()
        );
    }

    #[test]
    fn selection_drag_above_text_area_autoscrolls_up_from_painted_projection() {
        let lines = text_area_lines(80);
        let buffer = text::Buffer::from_multiline_text(lines.join("\n"));
        let (mut state, mut text_engine, path) = text_area_window_state(buffer, 260.0);
        let viewport = state
            .scroll
            .text_area(&path)
            .expect("text area projection should exist")
            .metrics()
            .viewport();
        let before_offset = state
            .scroll
            .text_area(&path)
            .expect("text area projection should exist")
            .metrics()
            .offset()
            .y();
        let position = point::logical(viewport.origin.x() + 10.0, viewport.origin.y() - 24.0);
        state.text_pointer_gesture = Some(TextPointerGesture::SelectionDrag(path.clone()));

        let (target, offset) = state
            .text_selection_drag_autoscroll_at(position, &mut text_engine)
            .expect("drag above a scrolled text area should autoscroll upward");
        let projection = state
            .scroll
            .text_area(&path)
            .expect("autoscroll should refresh the painted projection");

        assert_eq!(target, path);
        assert!(
            offset.y() < before_offset,
            "offset should move upward from {before_offset} to {}",
            offset.y()
        );
        assert_eq!(projection.metrics().offset(), offset);
        assert!(
            !projection.interaction_surfaces().is_empty(),
            "autoscroll should leave an observed text-area layout for drag hit testing"
        );

        let (_, edit) = state
            .text_field_drag_edit_at(position, &mut text_engine)
            .expect("drag after autoscroll should still resolve against the refreshed projection");
        assert!(matches!(
            edit,
            text::edit::Edit::Pointer {
                kind: text::edit::PointerEditKind::Drag,
                ..
            }
        ));
    }

    #[test]
    fn selection_drag_autoscroll_schedules_frames_only_when_scroll_can_move() {
        let lines = text_area_lines(80);
        let buffer = text::Buffer::from_multiline_text(lines.join("\n"));
        let (mut state, _text_engine, path) = text_area_window_state(buffer, 260.0);
        let viewport = state
            .scroll
            .text_area(&path)
            .expect("text area projection should exist")
            .metrics()
            .viewport();
        let position = point::logical(viewport.origin.x() + 10.0, viewport.origin.y() - 24.0);
        state.text_pointer_gesture = Some(TextPointerGesture::SelectionDrag(path.clone()));
        state
            .pointer
            .handle_event(pointer::Event::Moved { position });

        assert_eq!(
            state.animation_schedule(Instant::now()),
            animation::Schedule::NextFrame
        );

        state.text.insert(
            path.clone(),
            text::view::TextViewState::default().with_scroll_y(0.0),
        );
        state.scroll.update_offset(&path, point::logical(0.0, 0.0));

        assert_eq!(
            state.animation_schedule(Instant::now()),
            animation::Schedule::Idle
        );
    }

    #[test]
    fn active_smooth_wheel_scroll_schedules_frames() {
        let lines = text_area_lines(80);
        let buffer = text::Buffer::from_multiline_text(lines.join("\n"));
        let (mut state, _text_engine, path) = text_area_window_state(buffer, 260.0);
        let now = Instant::now();
        let metrics = state
            .scroll
            .metrics(&path)
            .expect("text area metrics should exist");
        let target = point::logical(metrics.offset().x(), metrics.offset().y() + 120.0);

        assert!(state.scroll.queue_wheel_offset_at(&path, target, now));

        assert!(state.smooth_scroll_active());
        assert_eq!(
            state.animation_schedule(now),
            animation::Schedule::NextFrame
        );
    }

    #[test]
    fn async_text_scroll_reconcile_preserves_smooth_wheel_target() {
        let lines = text_area_lines(80);
        let buffer = text::Buffer::from_multiline_text(lines.join("\n"));
        let (mut state, mut text_engine, path) = text_area_window_state(buffer, 260.0);
        let now = Instant::now();
        let metrics = state
            .scroll
            .metrics(&path)
            .expect("text area metrics should exist");
        let target = point::logical(metrics.offset().x(), metrics.offset().y() + 320.0);

        assert!(state.scroll.queue_wheel_offset_at(&path, target, now));
        assert!(
            state.commit_pending_visual_scroll_offsets(animation::Frame::new(
                now + Duration::from_millis(16),
                Some(now),
            ))
        );
        assert!(state.smooth_scroll_active());

        state.reconcile_async_scroll_target(
            &path,
            &mut text_engine,
            now + Duration::from_millis(16),
        );

        assert!(state.smooth_scroll_active());
        assert_eq!(
            state
                .scroll
                .target_offset(&path)
                .expect("smooth wheel target should survive reconcile"),
            target
        );
    }

    #[test]
    fn text_area_scroll_anchor_preserves_surviving_top_line_after_delete_above() {
        let lines = text_area_lines(80);
        let text = lines.join("\n");
        let mut buffer = text::Buffer::from_multiline_text(text);
        let initial_scroll_y = 500.0;
        let (mut state, mut text_engine, path) = text_area_window_state_for_area(
            text::Area::new(buffer.clone()).no_wrap(),
            initial_scroll_y,
        );
        let style = crate::theme::Theme::default_dark()
            .text()
            .style(text::document::Role::Control);
        let line_height = glyphon::Metrics::relative(style.size().max(1.0), 1.25)
            .line_height
            .max(1.0);
        let before_source_line = (initial_scroll_y / line_height).floor() as usize;
        let before_y = before_source_line as f32 * line_height - initial_scroll_y;
        let before_text = buffer.text_for_line_range(before_source_line, before_source_line + 1);
        let anchor = state
            .text_area_scroll_anchor(&path)
            .expect("visible text area should capture a scroll anchor");
        let delete_lines = before_source_line.saturating_sub(1).min(6);
        assert!(
            delete_lines > 0,
            "test must scroll below the first logical line"
        );
        let delete_len = line_start(&lines, delete_lines);
        let mut editor = text::edit::Editor::new();
        let result = editor.apply_text_edit_with_result(
            &mut buffer,
            text::edit::Edit::replace_range(0..delete_len, ""),
        );
        assert!(result.text_changed);
        text_engine.invalidate_text_area_for_edit(&buffer, &result.impacts);

        let current = state.text.get(&path).cloned().unwrap_or_default();
        let scroll_y = state
            .composition
            .as_ref()
            .and_then(|composition| {
                composition.text_area_scroll_y_for_anchor(&path, current, anchor, &mut text_engine)
            })
            .expect("surviving anchor line should resolve");
        let after_source_line = before_source_line - delete_lines;
        assert_eq!(
            buffer.text_for_line_range(after_source_line, after_source_line + 1),
            before_text,
            "fixture should delete lines above the anchored row"
        );
        let expected_scroll_y = (after_source_line as f32 * line_height - before_y).max(0.0);
        assert!(
            (scroll_y - expected_scroll_y).abs() <= 1.0,
            "anchor resolved scroll {} but expected {} for source line {} after deleting {} lines",
            scroll_y,
            expected_scroll_y,
            after_source_line,
            delete_lines
        );
        state
            .scroll
            .update_offset(&path, point::logical(0.0, scroll_y));
        state.text.insert(
            path.clone(),
            text::view::TextViewState::default()
                .with_scroll_y(scroll_y)
                .ensure_caret_visible(Instant::now()),
        );
        sync_text_area_projection(&mut state, &mut text_engine);

        let final_scroll_y = state
            .scroll
            .text_area(&path)
            .expect("text area projection should exist")
            .metrics()
            .offset()
            .y();
        assert!(
            (final_scroll_y - expected_scroll_y).abs() <= 1.0,
            "projection scroll {} should keep anchored line at y {} with expected scroll {}",
            final_scroll_y,
            before_y,
            expected_scroll_y
        );
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
        let root = ui::Node::container(crate::layout::Axis::Vertical)
            .key(ROOT)
            .with_child(
                widget::text_field(text::Buffer::from_text("Editable"))
                    .key(CHILD)
                    .with_size(
                        crate::layout::Size::Fixed(120.0),
                        crate::layout::Size::Fixed(32.0),
                    ),
            )
            .with_child(widget::button_key(OUTSIDE, CLICK).with_size(
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
    fn cursor_for_hovered_uses_default_for_scrollbar_capture() {
        let root = ui::Node::container(crate::layout::Axis::Vertical)
            .key(ROOT)
            .with_child(
                widget::text_field(text::Buffer::from_text("Editable"))
                    .key(CHILD)
                    .with_size(
                        crate::layout::Size::Fixed(120.0),
                        crate::layout::Size::Fixed(32.0),
                    ),
            )
            .with_child(widget::button_key(OUTSIDE, CLICK).with_size(
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

        assert_eq!(state.cursor_for_hovered(), ui::Cursor::Default);
    }

    #[test]
    fn cursor_for_pointer_uses_text_cursor_only_over_text_area_content() {
        let buffer = text::Buffer::from_multiline_text(&text_area_lines(40).join("\n"));
        let (mut state, mut text_engine, path) = text_area_window_state(buffer, 0.0);

        state.hovered = Some(path.clone());
        state.pointer.handle_event(pointer::Event::Moved {
            position: point::logical(8.0, 8.0),
        });
        assert_eq!(
            state.cursor_for_pointer(&mut text_engine),
            ui::Cursor::Text,
            "editable text content should keep the I-beam cursor"
        );

        let thumb = state
            .scroll
            .text_area(&path)
            .and_then(|projection| projection.metrics().vertical_thumb())
            .expect("text area should have a visible vertical scrollbar thumb");
        state.pointer.handle_event(pointer::Event::Moved {
            position: point::logical(thumb.origin.x() + 1.0, thumb.origin.y() + 1.0),
        });
        assert_eq!(
            state.cursor_for_pointer(&mut text_engine),
            ui::Cursor::Default,
            "text-area scrollbar chrome should use the default cursor"
        );

        state.pointer_capture = Some(pointer::Capture::new(
            path,
            widget::Part::Scroll(widget::scroll::Part::VerticalThumb),
            pointer::Button::Primary,
            point::logical(135.0, 8.0),
            point::logical(0.0, 4.0),
        ));
        state.hovered = None;
        assert_eq!(
            state.cursor_for_pointer(&mut text_engine),
            ui::Cursor::Default
        );
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
        let mut editor = text::edit::Editor::new();
        let mut buffer = text::Buffer::from_text("hello");
        editor.apply_text_edit(&mut buffer, text::edit::Edit::SelectAll);
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
    fn non_primary_release_does_not_invoke_command() {
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
    fn passive_pressed_element_does_not_invoke_command() {
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
    fn pointer_release_over_pressed_command_emits_contextual_request() {
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
        let mut registry = command::Registry::new();

        registry.register(command::definition::Definition::for_command::<
            Click,
            command::TestTarget,
        >());
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
        let request = command_request(
            &state,
            window,
            target.expect("release should target pressed element"),
            command::call::Source::Pointer,
        )
        .filter(|request| registry.can_execute(request));

        assert_eq!(
            request,
            Some(
                command::call::Raw::from_key(
                    CLICK,
                    command::call::Source::Pointer,
                    command::call::Context::path(window, path(CHILD))
                )
                .with_origin(path(CHILD))
            )
        );
    }

    #[test]
    fn disabled_command_bound_node_does_not_invoke() {
        let window = window::Id::new(1);
        let context = command::call::Context::path(window, path(CHILD));
        let mut registry = command::Registry::new();
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

        registry.register(command::definition::Definition::for_command::<
            Click,
            command::TestTarget,
        >());
        registry.set_state_key(CLICK, context, command::State::unavailable());

        assert_eq!(
            command_request(&state, window, path(CHILD), command::call::Source::Pointer)
                .filter(|request| registry.can_execute(request)),
            None
        );
    }

    #[test]
    fn menu_opens_only_when_an_item_can_invoke_after_resolution() {
        let window = window::Id::new(1);
        let menu = menu::Menu::new("File").key(FILE).section(
            menu::Section::new().item(menu::Item::text::<crate::text::command::SelectAll>()),
        );
        let mut registry = command::Registry::new();
        let mut state = state_with_composition(
            single_box(CHILD),
            HashMap::from([(FILE, menu)]),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(
                path(CHILD),
                vec![command::Key::of::<crate::text::command::SelectAll>()],
            )]),
            HashMap::new(),
            Vec::new(),
        );
        state.command_subject = Some(command::call::Scope::Path(path(CHILD)));

        register_text_command::<crate::text::command::SelectAll>(&mut registry, "Select All");
        registry.set_state_key(
            command::Key::of::<crate::text::command::SelectAll>(),
            command::call::Context::window(window),
            command::State::unavailable(),
        );

        assert!(!state.toggle_menu(FILE, &registry, window, command::call::Source::Pointer));
        assert_eq!(state.open_menu, None);

        registry.set_state_key(
            command::Key::of::<crate::text::command::SelectAll>(),
            command::call::Context::path(window, path(CHILD)),
            command::State::available(),
        );

        assert!(state.toggle_menu(FILE, &registry, window, command::call::Source::Pointer));
        assert_eq!(state.open_menu, Some(FILE));
    }

    #[test]
    fn menu_toggle_switches_and_closes_current_menu() {
        let window = window::Id::new(1);
        let edit = menu::Id::new("edit");
        let file_menu = menu::Menu::new("File")
            .key(FILE)
            .section(menu::Section::new().item(menu::Item::key(CLICK)));
        let edit_menu = menu::Menu::new("Edit")
            .key(edit)
            .section(menu::Section::new().item(menu::Item::key(CLICK)));
        let mut registry = command::Registry::new();
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

        registry.register(command::definition::Definition::for_command::<
            Click,
            command::TestTarget,
        >());

        assert!(state.toggle_menu(FILE, &registry, window, command::call::Source::Pointer));
        assert_eq!(state.open_menu, Some(FILE));
        state.open_submenu = Some(PANELS);
        assert!(state.toggle_menu(edit, &registry, window, command::call::Source::Pointer));
        assert_eq!(state.open_menu, Some(edit));
        assert_eq!(state.open_submenu, None);
        assert!(state.toggle_menu(edit, &registry, window, command::call::Source::Pointer));
        assert_eq!(state.open_menu, None);
    }

    #[test]
    fn submenu_opens_only_when_parent_menu_is_open_and_item_can_invoke() {
        let window = window::Id::new(1);
        let submenu = menu::Menu::new("Panels")
            .key(PANELS)
            .section(menu::Section::new().item(menu::Item::key(CLICK)));
        let mut registry = command::Registry::new();
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

        registry.register(command::definition::Definition::for_command::<
            Click,
            command::TestTarget,
        >());

        assert!(!state.open_submenu(PANELS, &registry, window, command::call::Source::Pointer));
        state.open_menu = Some(FILE);
        assert!(state.open_submenu(PANELS, &registry, window, command::call::Source::Pointer));
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
    fn busy_command_bound_node_does_not_invoke() {
        let window = window::Id::new(1);
        let context = command::call::Context::path(window, path(CHILD));
        let mut registry = command::Registry::new();
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

        registry.register(command::definition::Definition::for_command::<
            Click,
            command::TestTarget,
        >());
        registry.set_running_key(CLICK, context, true);

        assert_eq!(
            command_request(&state, window, path(CHILD), command::call::Source::Pointer)
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
        state.command_subject = Some(command::call::Scope::Path(path(CHILD)));
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
            command::call::Context::path(window, path(CHILD))
        );
    }

    #[test]
    fn command_subject_setters_update_command_subject_behavior() {
        let window = window::Id::new(1);
        let subject = command::call::Context::path(window, path(CHILD));
        let mut state = WindowState::default();

        assert!(state.set_command_subject(subject.clone()));
        assert_eq!(state.command_context(window), subject);
        assert!(!state.set_command_subject(command::call::Context::path(window, path(CHILD))));
        assert!(state.clear_command_subject());
        assert_eq!(
            state.command_context(window),
            command::call::Context::window(window)
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
        state.command_subject = Some(command::call::Scope::Path(path(CHILD)));

        assert!(state.set_focus(
            path(OUTSIDE),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible
        ));
        assert_eq!(
            state.command_context(window),
            command::call::Context::path(window, path(CHILD))
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
        state.command_subject = Some(command::call::Scope::Path(path(ROOT)));

        assert!(state.set_focus(
            path(CHILD),
            ui::focus::Reason::Keyboard,
            ui::focus::Visibility::Visible
        ));
        assert_eq!(
            state.command_context(window),
            command::call::Context::path(window, path(CHILD))
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
            command::call::Context::path(window, path(CHILD))
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
            command::call::Context::path(window, path(CHILD))
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
            command::call::Context::path(window, path(CHILD))
        );

        state.focus = FocusState::default();
        assert_eq!(
            state.command_context(window),
            command::call::Context::window(window)
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
            command::call::Context::window(window)
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
        state.command_subject = Some(command::call::Scope::Path(path(CHILD)));

        assert!(state.clear_stale_command_subject());
        assert_eq!(state.command_subject, None);
        assert_eq!(
            state.command_context(window),
            command::call::Context::window(window)
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
        state.command_subject = Some(command::call::Scope::Path(path(CHILD)));

        let request = command_request(&state, window, path(ROOT), command::call::Source::Pointer)
            .expect("command-subject command should produce request");

        assert_eq!(request.origin(), Some(&path(ROOT)));
        assert_eq!(request.args(), &command::args::Raw::None);
        assert_eq!(
            request.context(),
            &command::call::Context::path(window, path(CHILD))
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

        let request = command_request(&state, window, path(ROOT), command::call::Source::Pointer)
            .expect("command-subject command should produce request");

        assert_eq!(request.context(), &command::call::Context::window(window));
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
        state.command_subject = Some(command::call::Scope::Path(path(CHILD)));

        let request = command_request(&state, window, path(ROOT), command::call::Source::Pointer)
            .expect("window-subject command should produce request");

        assert_eq!(request.origin(), Some(&path(ROOT)));
        assert_eq!(request.context(), &command::call::Context::window(window));
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
            .insert(scope, command::call::Context::path(window, subject.clone()));

        let request = command_request(&state, window, origin, command::call::Source::Pointer)
            .expect("captured-target command should produce request");

        assert_eq!(
            request.context(),
            &command::call::Context::path(window, subject)
        );
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
        let request = command_request(&state, window, button, command::call::Source::Pointer)
            .expect("command-subject command should produce request");

        assert_eq!(
            request.context(),
            &command::call::Context::path(window, local)
        );
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
        let request = command::call::Raw::from_key(
            CLICK,
            command::call::Source::Shortcut,
            command::call::Context::path(window, outside),
        );
        let registry = command::Registry::new();

        assert_eq!(
            state.resolve_request(&registry, request).context(),
            &command::call::Context::path(window, child)
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
            HashMap::from([(
                path(ROOT),
                vec![command::Key::of::<crate::text::command::Copy>()],
            )]),
            HashMap::new(),
            Vec::new(),
        );
        let request = command::call::Raw::from_key(
            CLICK,
            command::call::Source::Shortcut,
            command::call::Context::path(window, path(ROOT)),
        );
        let registry = command::Registry::new();

        assert_eq!(
            state.resolve_request(&registry, request).context(),
            &command::call::Context::window(window)
        );
    }

    #[test]
    fn origin_bound_command_resolves_to_itself() {
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
        let request = command::call::Raw::from_key(
            CLICK,
            command::call::Source::Pointer,
            command::call::Context::path(window, path(CHILD)),
        )
        .with_origin(path(CHILD));
        let registry = command::Registry::new();

        assert_eq!(
            state.resolve_request(&registry, request).context(),
            &command::call::Context::path(window, path(CHILD))
        );
    }

    #[test]
    fn disabled_responder_target_blocks_invocation_after_resolution() {
        let window = window::Id::new(1);
        let mut registry = command::Registry::new();
        let state = state_with_composition(
            single_box(CHILD),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(
                path(CHILD),
                vec![command::Key::of::<crate::text::command::SelectAll>()],
            )]),
            HashMap::new(),
            Vec::new(),
        );
        let request = command::call::Raw::from_route(
            text_route::<crate::text::command::SelectAll>(),
            command::call::Source::Shortcut,
            command::call::Context::path(window, path(CHILD)),
        );

        register_text_command::<crate::text::command::SelectAll>(&mut registry, "Select All");
        registry.set_state_key(
            command::Key::of::<crate::text::command::SelectAll>(),
            command::call::Context::path(window, path(CHILD)),
            command::State::unavailable(),
        );
        let request = state.resolve_request(&registry, request);

        assert!(!registry.can_execute(&request));
    }
}
