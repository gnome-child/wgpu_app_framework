use std::{cell::RefCell, fmt, rc::Rc, time::Instant};

use crate::text as text_engine;

use super::super::{
    diagnostics,
    geometry::{Point, Rect, Size},
    interaction, scene,
    theme::Theme,
    view,
};
use super::Viewport;

#[derive(Clone)]
pub(crate) struct Service {
    inner: Rc<RefCell<text_engine::layout::Engine>>,
}

#[derive(Clone)]
pub(crate) struct Area {
    layout: text_engine::layout::TextFieldLayout,
    interaction_surfaces: Vec<text_engine::layout::TextAreaSurface>,
    render_surfaces: Vec<text_engine::layout::TextAreaSurface>,
    viewport: Viewport,
    state: text_engine::edit::ViewState,
}

#[derive(Clone)]
pub(crate) struct Field {
    layout: text_engine::layout::TextFieldLayout,
    render_surface: Option<text_engine::layout::TextAreaSurface>,
    state: text_engine::edit::ViewState,
    style: text_engine::document::Style,
}

impl Service {
    pub(super) fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(text_engine::layout::Engine::new())),
        }
    }

    pub(super) fn label_width_with_style(
        &self,
        label: &str,
        style: super::super::theme::TypeStyle,
    ) -> i32 {
        let metrics = self.inner.borrow_mut().measure(
            &document(label, style),
            text_engine::layout::Measure::unbounded(),
        );

        metrics.width().ceil().max(0.0) as i32
    }

    pub(super) fn label_size_for_width_with_style(
        &self,
        label: &str,
        width: i32,
        style: super::super::theme::TypeStyle,
    ) -> Size {
        let metrics = self
            .inner
            .borrow_mut()
            .measure(&document(label, style), measure_for_width(width));

        Size::new(
            metrics.width().ceil().max(0.0) as i32,
            metrics.height().ceil().max(0.0) as i32,
        )
    }

    pub(super) fn text_area_size_for_width(
        &self,
        text_area: &view::control::TextArea,
        width: i32,
        theme: &Theme,
    ) -> Size {
        let measure = match text_area.wrap() {
            view::control::Wrap::None => text_engine::layout::Measure::unbounded(),
            view::control::Wrap::Word => measure_for_width(width),
        };
        let metrics = self.inner.borrow_mut().measure(
            &document(text_area.buffer().text(), theme.typography().body()),
            measure,
        );

        Size::new(
            metrics.width().ceil().max(0.0) as i32,
            metrics.height().ceil().max(0.0) as i32,
        )
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
        theme: &Theme,
        now: Instant,
    ) -> Area {
        let area_model = text_area.area_model();
        let style = field_style(theme);
        let logical_viewport =
            text_engine::layout::surface_area(rect.width() as f32, rect.height() as f32);
        let layout_now = text_area.caret_epoch().unwrap_or(now);
        let mut state = text_area.view_state_at(layout_now);
        let paint_layout = {
            let mut engine = self.inner.borrow_mut();
            if state.caret_visibility_pending() {
                state = engine.ensure_caret_visible_for_area(
                    &area_model,
                    style,
                    logical_viewport,
                    state,
                    None,
                );
            }
            let mut paint_layout = engine.text_area_paint_layout_for_area_at(
                &area_model,
                style,
                logical_viewport,
                state.clone(),
                layout_now,
            );
            let clamped_state =
                clamp_text_area_scroll_state(&state, paint_layout.layout(), logical_viewport);
            if clamped_state.scroll_x() != state.scroll_x()
                || clamped_state.scroll_y() != state.scroll_y()
            {
                state = clamped_state;
                paint_layout = engine.text_area_paint_layout_for_area_at(
                    &area_model,
                    style,
                    logical_viewport,
                    state.clone(),
                    layout_now,
                );
            }
            paint_layout
        };
        let viewport = viewport_for_text_area(rect, paint_layout.layout(), &state);
        let (layout, interaction_surfaces, render_surfaces) = paint_layout.into_projection_parts();

        Area {
            layout,
            interaction_surfaces,
            render_surfaces,
            viewport,
            state,
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
        let local = text_engine::layout::surface_point(
            position.x().saturating_sub(rect.x()) as f32,
            position.y().saturating_sub(rect.y()) as f32,
        );

        self.inner
            .borrow_mut()
            .text_area_position_at_for_observed_surfaces(
                &area_model,
                local,
                layout.state.clone(),
                layout.state.scroll_x(),
                layout.interaction_surfaces(),
            )
    }

    pub(super) fn text_field_layout(
        &self,
        text_box: &view::control::TextBox,
        rect: Rect,
        theme: &Theme,
        now: Instant,
    ) -> Field {
        let field = field_model(text_box);
        let style = field_style(theme);
        let viewport = text_engine::layout::surface_area(rect.width() as f32, rect.height() as f32);
        let epoch = text_box.caret_epoch().unwrap_or(now);
        let mut state = text_engine::edit::ViewState::new_at(0.0, epoch)
            .with_preedit(text_box.preedit().cloned());
        let mut engine = self.inner.borrow_mut();

        if text_box.cursor().is_some() {
            state = engine.ensure_caret_visible_for_field(&field, style, viewport, state);
        }

        let paint_layout = engine.text_field_paint_layout_for_field_at(
            &field,
            style,
            viewport,
            state.clone(),
            epoch,
        );
        let (layout, render_surface) = paint_layout.into_parts();

        Field {
            layout,
            render_surface,
            state,
            style,
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
        let style = layout.style;
        let viewport = text_engine::layout::surface_area(rect.width() as f32, rect.height() as f32);
        let local = text_engine::layout::surface_point(
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

fn document(
    label: impl Into<String>,
    style: super::super::theme::TypeStyle,
) -> text_engine::document::Document {
    let mut block = text_engine::document::Block::new(text_engine::document::Align::Start);
    block.push_run(text_engine::document::Run::new(
        label.into(),
        style.document_style(text_engine::Color::rgb(0.0, 0.0, 0.0)),
    ));
    text_engine::document::Document::from_block(block)
}

impl fmt::Debug for Service {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("layout::TextService")
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
    pub(crate) fn layout(&self) -> &text_engine::layout::TextFieldLayout {
        &self.layout
    }

    pub(crate) fn interaction_surfaces(&self) -> &[text_engine::layout::TextAreaSurface] {
        &self.interaction_surfaces
    }

    pub(crate) fn render_surfaces(&self) -> &[text_engine::layout::TextAreaSurface] {
        &self.render_surfaces
    }

    pub(crate) fn viewport(&self) -> Viewport {
        self.viewport
    }
}

impl Field {
    pub(crate) fn layout(&self) -> &text_engine::layout::TextFieldLayout {
        &self.layout
    }

    pub(crate) fn render_surface(&self) -> Option<&text_engine::layout::TextAreaSurface> {
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
        Some(text_engine::buffer::MarkRange {
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

fn field_style(theme: &Theme) -> text_engine::document::Style {
    theme
        .typography()
        .interface()
        .document_style(text_color_from_scene(theme.text_input().foreground))
}

fn measure_for_width(width: i32) -> text_engine::layout::Measure {
    text_engine::layout::Measure::bounded(text_engine::layout::surface_area(
        width.max(0) as f32,
        f32::MAX,
    ))
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

fn viewport_for_text_area(
    rect: Rect,
    layout: &text_engine::layout::TextFieldLayout,
    state: &text_engine::edit::ViewState,
) -> Viewport {
    let content_area = layout.content_area();
    let content = Size::new(
        content_area.width().ceil().max(0.0) as i32,
        content_area.height().ceil().max(0.0) as i32,
    );

    Viewport::new(rect, content, scroll_offset_for_text_state(state))
}

fn clamp_text_area_scroll_state(
    state: &text_engine::edit::ViewState,
    layout: &text_engine::layout::TextFieldLayout,
    viewport: text_engine::layout::SurfaceArea,
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
