use std::time::{Duration, Instant};

use crate::geometry::{Rect, area, point};

const TEXT_FIELD_CARET_BLINK_INTERVAL: Duration = Duration::from_millis(500);

use super::{
    Buffer, Caret, CommandResult, TextPosition,
    buffer::{Mark, TextChange},
    edit::{EditHistory, HistoryKind},
    layout::{Engine, TextAreaSurface},
    surface::Area,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    area: area::Logical,
    scroll: point::Logical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Above,
    Below,
    Before,
    After,
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RevealIntent {
    #[default]
    None,
    EnsureCaretVisible,
    CaretForce,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollAnchor {
    mark: Mark,
    offset_y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct ObservedArea<'a> {
    viewport: Rect,
    scroll: point::Logical,
    content_area: area::Logical,
    surfaces: &'a [TextAreaSurface],
}

#[derive(Debug, Clone, Copy, Default)]
pub struct View;

const SELECTION_DRAG_AUTOSCROLL_MIN_STEP: f32 = 2.0;
const SELECTION_DRAG_AUTOSCROLL_MAX_STEP: f32 = 36.0;

impl Viewport {
    pub fn new(area: area::Logical, scroll: point::Logical) -> Self {
        Self {
            area,
            scroll: point::logical(scroll.x().max(0.0), scroll.y().max(0.0)),
        }
    }

    pub fn area(self) -> area::Logical {
        self.area
    }

    pub fn scroll(self) -> point::Logical {
        self.scroll
    }

    pub fn visibility_of_local_caret(self, caret: Caret, margin: f32) -> Visibility {
        let margin = margin.max(0.0);
        if caret.y() + caret.height() < -margin {
            return Visibility::Above;
        }
        if caret.y() > self.area.height() + margin {
            return Visibility::Below;
        }
        if caret.x() < -margin {
            return Visibility::Before;
        }
        if caret.x() > self.area.width() + margin {
            return Visibility::After;
        }
        Visibility::Visible
    }
}

impl Visibility {
    pub fn is_visible(self) -> bool {
        matches!(self, Self::Visible)
    }
}

impl RevealIntent {
    pub fn should_reveal(self) -> bool {
        !matches!(self, Self::None)
    }

    pub fn should_ensure_caret_visible(self) -> bool {
        matches!(self, Self::EnsureCaretVisible)
    }
}

impl ScrollAnchor {
    pub fn new(mark: Mark, offset_y: f32) -> Self {
        Self { mark, offset_y }
    }

    pub fn mark(self) -> Mark {
        self.mark
    }

    pub fn offset_y(self) -> f32 {
        self.offset_y
    }
}

impl<'a> ObservedArea<'a> {
    pub fn new(
        viewport: Rect,
        scroll: point::Logical,
        content_area: area::Logical,
        surfaces: &'a [TextAreaSurface],
    ) -> Self {
        Self {
            viewport,
            scroll,
            content_area,
            surfaces,
        }
    }

    pub fn viewport(self) -> Rect {
        self.viewport
    }

    pub fn scroll(self) -> point::Logical {
        self.scroll
    }

    pub fn content_area(self) -> area::Logical {
        self.content_area
    }

    pub fn surfaces(self) -> &'a [TextAreaSurface] {
        self.surfaces
    }

    pub fn local_position(self, position: point::Logical) -> point::Logical {
        point::logical(
            position.x() - self.viewport.origin.x(),
            position.y() - self.viewport.origin.y(),
        )
    }
}

impl View {
    pub fn selection_drag_autoscroll_offset(
        observed: ObservedArea<'_>,
        position: point::Logical,
    ) -> Option<point::Logical> {
        let viewport = observed.viewport();
        let current = observed.scroll();
        let top = viewport.origin.y();
        let bottom = viewport.origin.y() + viewport.area.height();
        let distance = if position.y() < top {
            position.y() - top
        } else if position.y() > bottom {
            position.y() - bottom
        } else {
            return None;
        };
        let step = distance.abs().clamp(
            SELECTION_DRAG_AUTOSCROLL_MIN_STEP,
            SELECTION_DRAG_AUTOSCROLL_MAX_STEP,
        );
        let next_y = if distance.is_sign_negative() {
            current.y() - step
        } else {
            current.y() + step
        };
        let max_y = (observed.content_area().height() - viewport.area.height()).max(0.0);
        let next = point::logical(current.x(), next_y.clamp(0.0, max_y));

        (next != current).then_some(next)
    }

    pub fn position_at_observed_area(
        text_engine: &mut Engine,
        area_model: &Area,
        state: TextViewState,
        observed: ObservedArea<'_>,
        position: point::Logical,
    ) -> Option<TextPosition> {
        if observed.surfaces().is_empty() {
            return None;
        }
        let local = observed.local_position(position);
        let scroll = observed.scroll();
        text_engine.text_area_position_at_for_observed_surfaces(
            area_model,
            local,
            state,
            scroll.x(),
            observed.surfaces(),
        )
    }

    pub fn scroll_anchor_for_observed_area(
        area_model: &Area,
        observed: ObservedArea<'_>,
    ) -> Option<ScrollAnchor> {
        let viewport_height = observed.viewport().area.height().max(0.0);
        let top_surface = observed
            .surfaces()
            .iter()
            .filter(|surface| {
                let bottom = surface.y() + surface.height().max(1.0);
                bottom > 0.0 && surface.y() < viewport_height
            })
            .min_by(|a, b| a.y().total_cmp(&b.y()))?;
        let mark = area_model
            .buffer()
            .mark_for_position(TextPosition::new(top_surface.source_start()))?;
        Some(ScrollAnchor::new(mark, (-top_surface.y()).max(0.0)))
    }

    pub fn state_after_caret_blink_reset(state: TextViewState, now: Instant) -> TextViewState {
        state.reset_caret_blink_without_scroll(now)
    }

    pub fn state_after_visible_pointer_placement(
        state: TextViewState,
        now: Instant,
    ) -> TextViewState {
        Self::state_after_caret_blink_reset(state, now)
    }

    pub fn state_after_selection_only_change(state: TextViewState, now: Instant) -> TextViewState {
        Self::state_after_caret_blink_reset(state, now)
    }

    pub fn state_after_text_edit(state: TextViewState, now: Instant) -> TextViewState {
        state.ensure_caret_visible(now)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    scroll_x: f32,
    scroll_y: f32,
    caret_epoch: Instant,
    preedit: Option<Preedit>,
    history: EditHistory,
    reveal_intent: RevealIntent,
    preferred_caret_x: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preedit {
    text: String,
    selection: Option<(usize, usize)>,
}

pub type TextViewState = State;

impl State {
    pub fn new(scroll_x: f32) -> Self {
        Self::new_at(scroll_x, Instant::now())
    }

    pub fn new_at(scroll_x: f32, caret_epoch: Instant) -> Self {
        Self {
            scroll_x: scroll_x.max(0.0),
            scroll_y: 0.0,
            caret_epoch,
            preedit: None,
            history: EditHistory::default(),
            reveal_intent: RevealIntent::None,
            preferred_caret_x: None,
        }
    }

    pub fn scroll_x(&self) -> f32 {
        self.scroll_x
    }

    pub fn scroll_y(&self) -> f32 {
        self.scroll_y
    }

    pub fn field_scroll_x(&self) -> f32 {
        self.scroll_x
    }

    pub fn field_scroll_y(&self) -> f32 {
        self.scroll_y
    }

    pub fn with_scroll_x(mut self, scroll_x: f32) -> Self {
        self.scroll_x = scroll_x.max(0.0);
        self
    }

    pub fn with_scroll_y(mut self, scroll_y: f32) -> Self {
        self.scroll_y = scroll_y.max(0.0);
        self
    }

    pub fn with_scroll(mut self, scroll_x: f32, scroll_y: f32) -> Self {
        self.scroll_x = scroll_x.max(0.0);
        self.scroll_y = scroll_y.max(0.0);
        self
    }

    pub fn with_field_scroll_x(self, scroll_x: f32) -> Self {
        self.with_scroll_x(scroll_x)
    }

    pub fn with_field_scroll_y(self, scroll_y: f32) -> Self {
        self.with_scroll_y(scroll_y)
    }

    pub fn with_field_scroll(self, scroll_x: f32, scroll_y: f32) -> Self {
        self.with_scroll(scroll_x, scroll_y)
    }

    pub(crate) fn without_scroll(mut self) -> Self {
        self.scroll_x = 0.0;
        self.scroll_y = 0.0;
        self
    }

    pub(crate) fn same_except_scroll(&self, other: &Self) -> bool {
        let mut left = self.clone();
        let mut right = other.clone();
        left.scroll_x = 0.0;
        left.scroll_y = 0.0;
        right.scroll_x = 0.0;
        right.scroll_y = 0.0;
        left == right
    }

    pub fn reset_caret_blink(mut self, now: Instant) -> Self {
        self.caret_epoch = now;
        self.reveal_intent = RevealIntent::CaretForce;
        self
    }

    pub fn ensure_caret_visible(mut self, now: Instant) -> Self {
        self.caret_epoch = now;
        self.reveal_intent = RevealIntent::EnsureCaretVisible;
        self
    }

    pub(crate) fn reset_caret_blink_without_scroll(mut self, now: Instant) -> Self {
        self.caret_epoch = now;
        self.reveal_intent = RevealIntent::None;
        self
    }

    pub(crate) fn reveal_intent(&self) -> RevealIntent {
        self.reveal_intent
    }

    pub(crate) fn caret_visibility_pending(&self) -> bool {
        self.reveal_intent.should_reveal()
    }

    pub(crate) fn clear_caret_visibility_pending(mut self) -> Self {
        self.reveal_intent = RevealIntent::None;
        self
    }

    pub fn with_preedit(mut self, preedit: Option<Preedit>) -> Self {
        self.preedit = preedit;
        self
    }

    pub fn preedit(&self) -> Option<&Preedit> {
        self.preedit.as_ref()
    }

    pub(crate) fn sync_history(&mut self, buffer: &Buffer) -> bool {
        self.history.sync(buffer.marker())
    }

    pub(crate) fn record_history_at(
        &mut self,
        change: TextChange,
        kind: HistoryKind,
        now: Instant,
    ) {
        self.history.record(change, kind, now);
    }

    pub(crate) fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    #[cfg(test)]
    pub(crate) fn history_undo_len(&self) -> usize {
        self.history.undo_len()
    }

    pub(crate) fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub(crate) fn apply_undo(&mut self, buffer: &mut Buffer) -> CommandResult {
        self.history.undo(buffer)
    }

    pub(crate) fn apply_redo(&mut self, buffer: &mut Buffer) -> CommandResult {
        self.history.redo(buffer)
    }

    pub fn caret_visible(&self, now: Instant) -> bool {
        let elapsed = now.saturating_duration_since(self.caret_epoch);
        let interval = TEXT_FIELD_CARET_BLINK_INTERVAL.as_millis();

        if interval == 0 {
            return true;
        }

        (elapsed.as_millis() / interval) % 2 == 0
    }

    pub fn next_caret_deadline(&self, now: Instant) -> Instant {
        let elapsed = now.saturating_duration_since(self.caret_epoch);
        let interval_ms = TEXT_FIELD_CARET_BLINK_INTERVAL.as_millis();
        let remainder = elapsed.as_millis() % interval_ms;
        let wait_ms = if remainder == 0 {
            interval_ms
        } else {
            interval_ms - remainder
        };

        now.checked_add(Duration::from_millis(wait_ms.min(u64::MAX as u128) as u64))
            .unwrap_or(now)
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl Preedit {
    pub fn new(text: impl Into<String>, selection: Option<(usize, usize)>) -> Self {
        Self {
            text: text.into(),
            selection,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn selection(&self) -> Option<(usize, usize)> {
        self.selection
    }
}
