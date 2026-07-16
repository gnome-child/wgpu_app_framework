use crate::geometry::{area, point};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

const TEXT_FIELD_CARET_BLINK_INTERVAL: Duration = Duration::from_millis(500);

use super::surface::Area;
use super::{
    buffer::{Mark, Position},
    layout::Caret,
    layout::{Engine, TextAreaSurface},
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

#[derive(Debug, Clone)]
pub(crate) struct ScrollAnchorBand {
    baseline_scroll_y: f32,
    viewport_height: f32,
    samples: Vec<ScrollAnchorSample>,
}

#[derive(Debug, Clone, Copy)]
struct ScrollAnchorSample {
    mark: Mark,
    y: f32,
    height: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct ObservedArea<'a> {
    origin: point::Logical,
    viewport: Viewport,
    content_area: area::Logical,
    interaction_surfaces: &'a [TextAreaSurface],
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

impl ScrollAnchorBand {
    pub(crate) fn observe(
        area_model: &Area,
        baseline_scroll_y: f32,
        viewport_height: f32,
        interaction_surfaces: &[TextAreaSurface],
        render_surfaces: &[TextAreaSurface],
    ) -> Option<Self> {
        let source = area_model.buffer();
        let line_starts = source.line_start_offsets();
        let mut samples = Vec::<ScrollAnchorSample>::new();
        for surface in interaction_surfaces.iter().chain(render_surfaces) {
            let shaped = surface.shaped_buffer();
            let buffer = shaped.borrow();
            let mut rows = BTreeMap::<usize, (f32, f32)>::new();
            for run in buffer.layout_runs() {
                let top = run.line_top;
                let bottom = top + run.line_height.max(1.0);
                rows.entry(run.line_i)
                    .and_modify(|(row_top, row_bottom)| {
                        *row_top = row_top.min(top);
                        *row_bottom = row_bottom.max(bottom);
                    })
                    .or_insert((top, bottom));
            }
            drop(buffer);
            if rows.is_empty() {
                rows.insert(0, (0.0, surface.height().max(1.0)));
            }
            for (local_line, (top, bottom)) in rows {
                let source_line = surface.source_line().saturating_add(local_line);
                let source_start = line_starts
                    .get(source_line)
                    .copied()
                    .unwrap_or(surface.source_start());
                let sample = ScrollAnchorSample {
                    mark: source.mark_for_position(Position::new(source_start)),
                    y: surface.y() + top,
                    height: (bottom - top).max(1.0),
                };
                if !samples.iter().any(|current| {
                    current.mark == sample.mark
                        && current.y.to_bits() == sample.y.to_bits()
                        && current.height.to_bits() == sample.height.to_bits()
                }) {
                    samples.push(sample);
                }
            }
        }
        (!samples.is_empty()).then_some(Self {
            baseline_scroll_y: baseline_scroll_y.max(0.0),
            viewport_height: viewport_height.max(0.0),
            samples,
        })
    }

    pub(crate) fn anchor_at(&self, scroll_y: f32) -> Option<ScrollAnchor> {
        let delta_y = scroll_y.max(0.0) - self.baseline_scroll_y;
        self.samples
            .iter()
            .filter_map(|sample| {
                let y = sample.y - delta_y;
                let bottom = y + sample.height;
                (bottom > 0.0 && y < self.viewport_height).then_some((sample, y))
            })
            .min_by(|(_, left_y), (_, right_y)| left_y.total_cmp(right_y))
            .map(|(sample, y)| ScrollAnchor::new(sample.mark, (-y).max(0.0)))
    }
}

impl<'a> ObservedArea<'a> {
    pub fn new(
        origin: point::Logical,
        viewport: Viewport,
        content_area: area::Logical,
        interaction_surfaces: &'a [TextAreaSurface],
    ) -> Self {
        Self {
            origin,
            viewport,
            content_area,
            interaction_surfaces,
        }
    }

    pub fn viewport(self) -> Viewport {
        self.viewport
    }

    pub fn scroll(self) -> point::Logical {
        self.viewport.scroll()
    }

    pub fn content_area(self) -> area::Logical {
        self.content_area
    }

    pub fn interaction_surfaces(self) -> &'a [TextAreaSurface] {
        self.interaction_surfaces
    }

    pub fn local_position(self, position: point::Logical) -> point::Logical {
        point::logical(
            position.x() - self.origin.x(),
            position.y() - self.origin.y(),
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
        let top = observed.origin.y();
        let bottom = observed.origin.y() + viewport.area().height();
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
        let max_y = (observed.content_area().height() - viewport.area().height()).max(0.0);
        let next = point::logical(current.x(), next_y.clamp(0.0, max_y));

        (next != current).then_some(next)
    }

    pub fn position_at_observed_area(
        text_engine: &mut Engine,
        area_model: &Area,
        state: ViewState,
        observed: ObservedArea<'_>,
        position: point::Logical,
    ) -> Option<Position> {
        if observed.interaction_surfaces().is_empty() {
            return None;
        }
        let local = observed.local_position(position);
        let scroll = observed.scroll();
        text_engine.text_area_position_at_for_observed_surfaces(
            area_model,
            local,
            state,
            scroll.x(),
            observed.interaction_surfaces(),
        )
    }

    pub fn scroll_anchor_for_observed_area(
        area_model: &Area,
        observed: ObservedArea<'_>,
    ) -> Option<ScrollAnchor> {
        ScrollAnchorBand::observe(
            area_model,
            observed.scroll().y(),
            observed.viewport().area().height(),
            observed.interaction_surfaces(),
            &[],
        )
        .and_then(|band| band.anchor_at(observed.scroll().y()))
    }

    pub fn scroll_anchor_for_text_area(
        area_model: &Area,
        observed: ObservedArea<'_>,
        render_surfaces: &[TextAreaSurface],
    ) -> Option<ScrollAnchor> {
        ScrollAnchorBand::observe(
            area_model,
            observed.scroll().y(),
            observed.viewport().area().height(),
            observed.interaction_surfaces(),
            render_surfaces,
        )
        .and_then(|band| band.anchor_at(observed.scroll().y()))
    }

    pub fn state_after_caret_blink_reset(state: ViewState, now: Instant) -> ViewState {
        state.reset_caret_blink_without_scroll(now)
    }

    pub fn state_after_visible_pointer_placement(state: ViewState, now: Instant) -> ViewState {
        Self::state_after_caret_blink_reset(state, now)
    }

    pub fn state_after_selection_only_change(state: ViewState, now: Instant) -> ViewState {
        Self::state_after_caret_blink_reset(state, now)
    }

    pub fn state_after_text_edit(state: ViewState, now: Instant) -> ViewState {
        state.ensure_caret_visible(now)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ViewState {
    scroll_x: f32,
    scroll_y: f32,
    caret_epoch: Instant,
    reveal_intent: RevealIntent,
    preferred_caret_x: Option<f32>,
}

impl ViewState {
    pub fn new(scroll_x: f32) -> Self {
        Self::new_at(scroll_x, Instant::now())
    }

    pub fn new_at(scroll_x: f32, caret_epoch: Instant) -> Self {
        Self {
            scroll_x: scroll_x.max(0.0),
            scroll_y: 0.0,
            caret_epoch,
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

    pub fn caret_visible(&self, now: Instant) -> bool {
        let elapsed = now.saturating_duration_since(self.caret_epoch);
        let interval = TEXT_FIELD_CARET_BLINK_INTERVAL.as_millis();

        if interval == 0 {
            return true;
        }

        (elapsed.as_millis() / interval).is_multiple_of(2)
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

impl Default for ViewState {
    fn default() -> Self {
        Self::new(0.0)
    }
}
