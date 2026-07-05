use std::{cell::RefCell, fmt, rc::Rc, time::Instant};

use crate::{
    geometry::{area, point},
    text as text_engine,
};

use super::super::{
    diagnostics,
    geometry::{Point, Rect},
    interaction, scene,
    theme::Theme,
    view,
};

#[derive(Clone)]
pub(in crate::scratch) struct Service {
    inner: Rc<RefCell<text_engine::layout::Engine>>,
}

#[derive(Clone)]
pub struct Area {
    layout: text_engine::layout::TextFieldLayout,
    interaction_surfaces: Vec<text_engine::layout::TextAreaSurface>,
    render_surfaces: Vec<text_engine::layout::TextAreaSurface>,
    resolved_scroll: Option<interaction::ScrollOffset>,
}

#[derive(Clone)]
pub struct Field {
    layout: text_engine::layout::TextFieldLayout,
    render_surface: Option<text_engine::layout::TextAreaSurface>,
    state: text_engine::edit::ViewState,
}

impl Service {
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

        metrics.width().ceil().max(0.0) as i32
    }

    pub(super) fn take_diagnostics(&self) -> diagnostics::Text {
        let mut engine = self.inner.borrow_mut();
        let mut diagnostics = diagnostics::Text::default();
        diagnostics.add_text_layout(engine.diagnostics());
        engine.reset_diagnostics();
        diagnostics
    }

    pub(super) fn text_area_layout(&self, text_area: &view::control::TextArea, rect: Rect) -> Area {
        let area_model = text_area.area_model();
        let style = field_style();
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

        Area {
            layout,
            interaction_surfaces,
            render_surfaces,
            resolved_scroll,
        }
    }

    pub(super) fn text_area_position_at(
        &self,
        text_area: &view::control::TextArea,
        layout: &Area,
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

    pub(super) fn text_field_layout(&self, text_box: &view::control::TextBox, rect: Rect) -> Field {
        let field = field_model(text_box);
        let style = field_style();
        let viewport = area::logical(rect.width() as f32, rect.height() as f32);
        let now = Instant::now();
        let mut state =
            text_engine::edit::ViewState::default().with_preedit(text_box.preedit().cloned());
        let mut engine = self.inner.borrow_mut();

        if text_box.cursor().is_some() {
            state = state.ensure_caret_visible(now);
            state = engine.ensure_caret_visible_for_field(&field, style, viewport, state);
        }

        let paint_layout = engine.text_field_paint_layout_for_field_at(
            &field,
            style,
            viewport,
            state.clone(),
            now,
        );
        let (layout, render_surface) = paint_layout.into_parts();

        Field {
            layout,
            render_surface,
            state,
        }
    }

    pub(super) fn text_field_position_at(
        &self,
        text_box: &view::control::TextBox,
        layout: &Field,
        rect: Rect,
        position: Point,
    ) -> Option<text_engine::buffer::Position> {
        let field = field_model(text_box);
        let style = field_style();
        let viewport = area::logical(rect.width() as f32, rect.height() as f32);
        let local = point::logical(
            position.x().saturating_sub(rect.x()) as f32,
            position.y().saturating_sub(rect.y()) as f32,
        );

        self.inner.borrow_mut().text_field_position_at_for_field(
            &field,
            style,
            viewport,
            local,
            layout.state.clone(),
        )
    }
}

impl fmt::Debug for Service {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("layout::text::Service")
            .finish_non_exhaustive()
    }
}

impl text_engine::edit::CaretMap for Service {
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

impl Area {
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

impl Field {
    pub fn layout(&self) -> &text_engine::layout::TextFieldLayout {
        &self.layout
    }

    pub fn render_surface(&self) -> Option<&text_engine::layout::TextAreaSurface> {
        self.render_surface.as_ref()
    }
}

fn field_model(text_box: &view::control::TextBox) -> text_engine::edit::Field {
    let buffer = text_engine::Buffer::from_text(text_box.text());
    let cursor = text_box.cursor().unwrap_or_else(|| text_box.text().len());
    let cursor = buffer
        .mark_for_position(text_engine::buffer::Position::new(cursor))
        .unwrap_or_else(|| {
            buffer
                .mark_for_position(text_engine::buffer::Position::new(buffer.len()))
                .expect("text buffers always contain a valid end position")
        });
    let selection = text_box.selection().and_then(|selection| {
        Some(text_engine::buffer::mark::Range {
            start: buffer.mark_for_position(text_engine::buffer::Position::new(selection.start))?,
            end: buffer.mark_for_position(text_engine::buffer::Position::new(selection.end))?,
        })
    });
    let state = text_engine::edit::State::new(cursor, selection);
    let field = text_engine::edit::Field::new(buffer).with_state(state);

    if text_box.cursor().is_some() {
        field
    } else {
        field.read_only()
    }
}

fn field_style() -> text_engine::document::Style {
    text_engine::document::Style::default()
        .with_color(text_color_from_scene(Theme::default().text().inverse))
}

fn text_color_from_scene(color: scene::Color) -> text_engine::Color {
    let (r, g, b, a) = color.channels();

    text_engine::Color::rgba(
        linear_channel(r),
        linear_channel(g),
        linear_channel(b),
        alpha_channel(a),
    )
}

fn linear_channel(channel: u8) -> f32 {
    let value = alpha_channel(channel);

    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn alpha_channel(channel: u8) -> f32 {
    channel as f32 / 255.0
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
