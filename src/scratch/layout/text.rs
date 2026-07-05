use std::{cell::RefCell, fmt, rc::Rc, time::Instant};

use crate::{
    geometry::{area, point},
    text as text_engine,
};

use super::super::{
    diagnostics,
    geometry::{Point, Rect},
    interaction, view,
};

const LABEL_PADDING: i32 = 24;

#[derive(Clone)]
pub(in crate::scratch) struct TextService {
    inner: Rc<RefCell<text_engine::layout::Engine>>,
}

#[derive(Clone)]
pub struct TextAreaLayout {
    layout: text_engine::layout::TextFieldLayout,
    interaction_surfaces: Vec<text_engine::layout::TextAreaSurface>,
    render_surfaces: Vec<text_engine::layout::TextAreaSurface>,
    resolved_scroll: Option<interaction::ScrollOffset>,
}

#[derive(Clone)]
pub(super) struct TextHitMap {
    boundaries: Vec<TextBoundary>,
}

#[derive(Clone)]
struct TextBoundary {
    index: usize,
    x: i32,
}

impl TextService {
    pub(super) fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(text_engine::layout::Engine::new())),
        }
    }

    pub(super) fn label_width(&self, label: &str) -> i32 {
        let metrics = self.inner.borrow_mut().measure(
            &text_engine::document::Document::plain(label),
            text_engine::layout::Measure::unbounded(),
        );

        metrics.width().ceil().max(0.0) as i32 + LABEL_PADDING
    }

    fn text_width(&self, text: &str) -> i32 {
        let metrics = self.inner.borrow_mut().measure(
            &text_engine::document::Document::plain(text),
            text_engine::layout::Measure::unbounded(),
        );

        metrics.width().ceil().max(0.0) as i32
    }

    pub(super) fn take_diagnostics(&self) -> diagnostics::Text {
        let mut engine = self.inner.borrow_mut();
        let mut diagnostics = diagnostics::Text::default();
        diagnostics.add_text_layout(engine.diagnostics());
        engine.reset_diagnostics();
        diagnostics
    }

    pub(super) fn text_area_layout(
        &self,
        text_area: &view::control::TextArea,
        rect: Rect,
    ) -> TextAreaLayout {
        let area_model = text_area.area_model();
        let style = text_engine::document::Style::default()
            .with_color(text_engine::Color::rgb(0.10, 0.11, 0.13));
        let viewport = area::logical(rect.width() as f32, rect.height() as f32);
        let now = Instant::now();
        let mut state = text_area.view_state();
        let paint_layout = {
            let mut engine = self.inner.borrow_mut();
            if state.caret_visibility_pending() {
                state =
                    engine.ensure_caret_visible_for_area(&area_model, style, viewport, state, None);
            }
            let mut paint_layout = engine.text_area_paint_layout_for_area_at(
                &area_model,
                style,
                viewport,
                state.clone(),
                now,
            );
            let clamped_state =
                clamp_text_area_scroll_state(&state, paint_layout.layout(), viewport);
            if clamped_state.scroll_x() != state.scroll_x()
                || clamped_state.scroll_y() != state.scroll_y()
            {
                state = clamped_state;
                paint_layout = engine.text_area_paint_layout_for_area_at(
                    &area_model,
                    style,
                    viewport,
                    state.clone(),
                    now,
                );
            }
            paint_layout
        };
        let resolved_scroll = Some(scroll_offset_for_text_state(&state));
        let (layout, interaction_surfaces, render_surfaces) = paint_layout.into_projection_parts();

        TextAreaLayout {
            layout,
            interaction_surfaces,
            render_surfaces,
            resolved_scroll,
        }
    }

    pub(super) fn text_area_position_at(
        &self,
        text_area: &view::control::TextArea,
        layout: &TextAreaLayout,
        rect: Rect,
        position: Point,
    ) -> Option<text_engine::buffer::Position> {
        let area_model = text_area.area_model();
        let local = point::logical(
            position.x().saturating_sub(rect.x()) as f32,
            position.y().saturating_sub(rect.y()) as f32,
        );

        self.inner
            .borrow_mut()
            .text_area_position_at_for_observed_surfaces(
                &area_model,
                local,
                text_area.view_state(),
                text_area.view_state().scroll_x(),
                layout.interaction_surfaces(),
            )
    }
}

impl fmt::Debug for TextService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextService").finish_non_exhaustive()
    }
}

impl text_engine::edit::CaretMap for TextService {
    fn position_for_motion(
        &mut self,
        buffer: &text_engine::Buffer,
        state: text_engine::edit::State,
        motion: text_engine::edit::Motion,
    ) -> Option<text_engine::buffer::Position> {
        <text_engine::layout::Engine as text_engine::edit::CaretMap>::position_for_motion(
            &mut *self.inner.borrow_mut(),
            buffer,
            state,
            motion,
        )
    }
}

impl TextAreaLayout {
    pub fn layout(&self) -> &text_engine::layout::TextFieldLayout {
        &self.layout
    }

    pub fn interaction_surfaces(&self) -> &[text_engine::layout::TextAreaSurface] {
        &self.interaction_surfaces
    }

    pub fn render_surfaces(&self) -> &[text_engine::layout::TextAreaSurface] {
        &self.render_surfaces
    }

    pub fn resolved_scroll(&self) -> Option<interaction::ScrollOffset> {
        self.resolved_scroll
    }
}

impl TextHitMap {
    pub(super) fn new(text: &str, service: &TextService) -> Self {
        let mut boundaries = Vec::with_capacity(text.chars().count() + 1);
        boundaries.push(TextBoundary { index: 0, x: 0 });

        for (index, _) in text.char_indices().skip(1) {
            boundaries.push(TextBoundary {
                index,
                x: service.text_width(&text[..index]),
            });
        }

        boundaries.push(TextBoundary {
            index: text.len(),
            x: service.text_width(text),
        });

        Self { boundaries }
    }

    pub(super) fn position_at_x(&self, x: i32) -> text_engine::buffer::Position {
        text_engine::buffer::Position::new(self.index_at_x(x))
    }

    fn index_at_x(&self, x: i32) -> usize {
        let Some(first) = self.boundaries.first() else {
            return 0;
        };
        if x <= first.x {
            return first.index;
        }

        for pair in self.boundaries.windows(2) {
            let left = &pair[0];
            let right = &pair[1];
            let midpoint = left.x.saturating_add(right.x.saturating_sub(left.x) / 2);
            if x < midpoint {
                return left.index;
            }
        }

        self.boundaries
            .last()
            .map(|boundary| boundary.index)
            .unwrap_or(0)
    }
}

fn scroll_offset_for_text_state(state: &text_engine::edit::ViewState) -> interaction::ScrollOffset {
    interaction::ScrollOffset::new(
        scroll_component(state.scroll_x()),
        scroll_component(state.scroll_y()),
    )
}

fn clamp_text_area_scroll_state(
    state: &text_engine::edit::ViewState,
    layout: &text_engine::layout::TextFieldLayout,
    viewport: area::Logical,
) -> text_engine::edit::ViewState {
    let content_area = layout.content_area();
    let max_scroll_x = (content_area.width() - viewport.width()).max(0.0);
    let max_scroll_y = (content_area.height() - viewport.height()).max(0.0);

    state.clone().with_scroll(
        state.scroll_x().clamp(0.0, max_scroll_x),
        state.scroll_y().clamp(0.0, max_scroll_y),
    )
}

fn scroll_component(value: f32) -> i32 {
    value.round().clamp(0.0, i32::MAX as f32) as i32
}
