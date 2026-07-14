use std::{
    cell::{Cell, RefCell},
    fmt,
    rc::Rc,
    time::Instant,
};

use crate::text as text_engine;

use super::super::{
    geometry::{Point, Rect, Size, area},
    interaction, scene,
    theme::Theme,
    view,
};
use super::Viewport;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Text {
    pub author_text_overflows: usize,
    pub text_area_paint_layout_calls: usize,
    pub text_area_metrics_layout_calls: usize,
    pub text_area_visible_logical_lines: usize,
    pub text_area_shaped_logical_lines: usize,
    pub text_area_layout_segments: usize,
    pub text_area_overscan_segments: usize,
    pub text_area_interaction_surfaces: usize,
    pub highlight_run_scans: usize,
    pub text_area_line_cache_hits: usize,
    pub text_area_line_cache_misses: usize,
    pub text_area_render_surface_calls: usize,
    pub text_area_render_surface_cache_hits: usize,
    pub text_area_render_surface_cache_misses: usize,
    pub text_area_render_surface_source_lines: usize,
    pub text_area_render_surface_source_bytes: usize,
}

#[derive(Clone)]
pub(crate) struct Service {
    inner: Rc<RefCell<text_engine::layout::Engine>>,
    author_text_overflows: Rc<Cell<usize>>,
}

#[derive(Clone)]
pub(crate) struct Area {
    layout: text_engine::layout::TextFieldLayout,
    interaction_surfaces: Vec<text_engine::layout::TextAreaSurface>,
    render_surfaces: Vec<text_engine::layout::TextAreaSurface>,
    viewport: Viewport,
    state: text_engine::view::ViewState,
}

#[derive(Clone)]
pub(crate) struct Field {
    layout: text_engine::layout::TextFieldLayout,
    render_surface: Option<text_engine::layout::TextAreaSurface>,
    state: text_engine::view::ViewState,
    style: text_engine::document::Style,
}

pub(crate) type Selectable = text_engine::layout::OverflowProjection;

impl Service {
    pub(super) fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(text_engine::layout::Engine::new())),
            author_text_overflows: Rc::new(Cell::new(0)),
        }
    }

    pub(super) fn caret_map(&self) -> Rc<RefCell<dyn text_engine::selection::CaretMap>> {
        self.inner.clone()
    }

    pub(super) fn resolve_overflow(
        &self,
        label: &str,
        width: i32,
        style: super::super::theme::TypeStyle,
        overflow: text_engine::Overflow,
    ) -> String {
        self.inner.borrow_mut().resolve_overflow(
            label,
            style.document_style(text_engine::Color::BLACK),
            width.max(0) as f32,
            overflow,
        )
    }

    pub(super) fn resolve_selectable(
        &self,
        source: &str,
        width: i32,
        style: super::super::theme::TypeStyle,
        wrap: view::Wrap,
        overflow: text_engine::Overflow,
    ) -> Selectable {
        let style = style.document_style(text_engine::Color::BLACK);
        let width = width.max(0) as f32;
        match wrap {
            view::Wrap::None => self
                .inner
                .borrow_mut()
                .resolve_single_line_overflow_projection(source, style, width, overflow),
            view::Wrap::Word => self
                .inner
                .borrow_mut()
                .resolve_overflow_projection(source, style, width, overflow),
        }
    }

    pub(super) fn diagnose_author_overflow(
        &self,
        label: &str,
        width: i32,
        height: i32,
        style: super::super::theme::TypeStyle,
    ) {
        let measured = self.label_size_for_width_with_style(label, width, style);
        if measured.width() > width.max(0) || measured.height() > height.max(0) {
            self.author_text_overflows
                .set(self.author_text_overflows.get().saturating_add(1));
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
        text_area: &view::TextArea,
        width: i32,
        theme: &Theme,
    ) -> Size {
        let measure = match text_area.wrap() {
            view::Wrap::None => text_engine::layout::Measure::unbounded(),
            view::Wrap::Word => measure_for_width(width),
        };
        let metrics = self.inner.borrow_mut().measure(
            &document(text_area.buffer().text(), theme.typography().interface()),
            measure,
        );

        Size::new(
            metrics.width().ceil().max(0.0) as i32,
            metrics.height().ceil().max(0.0) as i32,
        )
    }

    pub(super) fn take_diagnostics(&self) -> Text {
        let mut engine = self.inner.borrow_mut();
        let mut diagnostics = Text::default();
        diagnostics.add_text_layout(engine.diagnostics());
        engine.reset_diagnostics();
        diagnostics.author_text_overflows = self.author_text_overflows.replace(0);
        diagnostics
    }

    pub(super) fn text_area_layout(
        &self,
        text_area: &view::TextArea,
        rect: Rect,
        theme: &Theme,
        color: scene::Color,
        now: Instant,
    ) -> Area {
        let area_model = text_area.area_model();
        let style = field_style(theme, color);
        let logical_viewport =
            text_engine::layout::surface_area(rect.width() as f32, rect.height() as f32);
        let layout_now = text_area.caret_epoch().unwrap_or(now);
        let mut state = text_area.view_state_at(layout_now);
        let preedit = text_area.preedit();
        let paint_layout = {
            let mut engine = self.inner.borrow_mut();
            if state.caret_visibility_pending() {
                state = engine.ensure_caret_visible_for_area_with_preedit(
                    &area_model,
                    style,
                    logical_viewport,
                    state,
                    preedit,
                    None,
                );
            }
            let mut paint_layout = engine.text_area_paint_layout_for_area_with_preedit_at(
                &area_model,
                style,
                logical_viewport,
                state.clone(),
                preedit,
                layout_now,
            );
            let clamped_state =
                clamp_text_area_scroll_state(&state, paint_layout.layout(), logical_viewport);
            if clamped_state.scroll_x() != state.scroll_x()
                || clamped_state.scroll_y() != state.scroll_y()
            {
                state = clamped_state;
                paint_layout = engine.text_area_paint_layout_for_area_with_preedit_at(
                    &area_model,
                    style,
                    logical_viewport,
                    state.clone(),
                    preedit,
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
        text_area: &view::TextArea,
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
            .text_area_position_at_for_observed_surfaces_with_preedit(
                &area_model,
                local,
                layout.state.clone(),
                text_area.preedit(),
                layout.state.scroll_x(),
                layout.interaction_surfaces(),
            )
    }

    pub(super) fn text_field_layout(
        &self,
        text_box: &view::TextBox,
        rect: Rect,
        theme: &Theme,
        now: Instant,
    ) -> Field {
        let field = field_model(text_box);
        let style = field_style(theme, theme.text_input().foreground);
        let viewport = text_engine::layout::surface_area(rect.width() as f32, rect.height() as f32);
        let epoch = text_box.caret_epoch().unwrap_or(now);
        let mut state = text_engine::view::ViewState::new_at(0.0, epoch);
        let preedit = text_box.preedit();
        let mut engine = self.inner.borrow_mut();

        if text_box.cursor().is_some() {
            state = engine.ensure_caret_visible_for_field_with_preedit(
                &field, style, viewport, state, preedit,
            );
        }

        let paint_layout = engine.text_field_paint_layout_for_field_with_preedit_at(
            &field,
            style,
            viewport,
            state.clone(),
            preedit,
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
        text_box: &view::TextBox,
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

        self.inner
            .borrow_mut()
            .text_field_position_at_for_field_with_preedit(
                &field,
                style,
                viewport,
                local,
                layout.state.clone(),
                text_box.preedit(),
            )
    }
}

impl Text {
    pub(crate) fn add(&mut self, diagnostics: Self) {
        self.author_text_overflows += diagnostics.author_text_overflows;
        self.text_area_paint_layout_calls += diagnostics.text_area_paint_layout_calls;
        self.text_area_metrics_layout_calls += diagnostics.text_area_metrics_layout_calls;
        self.text_area_visible_logical_lines += diagnostics.text_area_visible_logical_lines;
        self.text_area_shaped_logical_lines += diagnostics.text_area_shaped_logical_lines;
        self.text_area_layout_segments += diagnostics.text_area_layout_segments;
        self.text_area_overscan_segments += diagnostics.text_area_overscan_segments;
        self.text_area_interaction_surfaces += diagnostics.text_area_interaction_surfaces;
        self.highlight_run_scans += diagnostics.highlight_run_scans;
        self.text_area_line_cache_hits += diagnostics.text_area_line_cache_hits;
        self.text_area_line_cache_misses += diagnostics.text_area_line_cache_misses;
        self.text_area_render_surface_calls += diagnostics.text_area_render_surface_calls;
        self.text_area_render_surface_cache_hits += diagnostics.text_area_render_surface_cache_hits;
        self.text_area_render_surface_cache_misses +=
            diagnostics.text_area_render_surface_cache_misses;
        self.text_area_render_surface_source_lines +=
            diagnostics.text_area_render_surface_source_lines;
        self.text_area_render_surface_source_bytes +=
            diagnostics.text_area_render_surface_source_bytes;
    }

    fn add_text_layout(&mut self, diagnostics: text_engine::layout::Diagnostics) {
        self.text_area_paint_layout_calls += diagnostics.text_area_paint_layout_calls;
        self.text_area_metrics_layout_calls += diagnostics.text_area_metrics_layout_calls;
        self.text_area_visible_logical_lines += diagnostics.text_area_visible_logical_lines;
        self.text_area_shaped_logical_lines += diagnostics.text_area_shaped_logical_lines;
        self.text_area_layout_segments += diagnostics.text_area_layout_segments;
        self.text_area_overscan_segments += diagnostics.text_area_overscan_segments;
        self.text_area_interaction_surfaces += diagnostics.text_area_interaction_surfaces;
        self.highlight_run_scans += diagnostics.highlight_run_scans;
        self.text_area_line_cache_hits += diagnostics.text_area_line_cache_hits;
        self.text_area_line_cache_misses += diagnostics.text_area_line_cache_misses;
        self.text_area_render_surface_calls += diagnostics.text_area_render_surface_calls;
        self.text_area_render_surface_cache_hits += diagnostics.text_area_render_surface_cache_hits;
        self.text_area_render_surface_cache_misses +=
            diagnostics.text_area_render_surface_cache_misses;
        self.text_area_render_surface_source_lines +=
            diagnostics.text_area_render_surface_source_lines;
        self.text_area_render_surface_source_bytes +=
            diagnostics.text_area_render_surface_source_bytes;
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

impl text_engine::selection::CaretMap for Service {
    fn position_for_motion(
        &mut self,
        buffer: &text_engine::Buffer,
        state: text_engine::selection::State,
        motion: text_engine::selection::Motion,
    ) -> Option<text_engine::buffer::Position> {
        <text_engine::layout::Engine as text_engine::selection::CaretMap>::position_for_motion(
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

fn field_model(text_box: &view::TextBox) -> text_engine::surface::Field {
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
    let state = text_engine::selection::State::new(cursor, selection);
    text_engine::surface::Field::new(buffer)
        .with_state(state)
        .with_mode(text_box.mode())
}

fn field_style(theme: &Theme, color: scene::Color) -> text_engine::document::Style {
    theme
        .typography()
        .interface()
        .document_style(text_color_from_scene(color))
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
        crate::color::byte_to_unit(r),
        crate::color::byte_to_unit(g),
        crate::color::byte_to_unit(b),
        crate::color::byte_to_unit(a),
    )
}

fn scroll_offset_for_text_state(state: &text_engine::view::ViewState) -> interaction::ScrollOffset {
    interaction::ScrollOffset::new(
        scroll_component(state.scroll_x()),
        scroll_component(state.scroll_y()),
    )
}

fn viewport_for_text_area(
    rect: Rect,
    layout: &text_engine::layout::TextFieldLayout,
    state: &text_engine::view::ViewState,
) -> Viewport {
    let content_area = layout.content_area();
    let content = Size::new(
        content_area.width().ceil().max(0.0) as i32,
        content_area.height().ceil().max(0.0) as i32,
    );

    Viewport::new(rect, content, scroll_offset_for_text_state(state))
}

fn clamp_text_area_scroll_state(
    state: &text_engine::view::ViewState,
    layout: &text_engine::layout::TextFieldLayout,
    viewport: area::Logical,
) -> text_engine::view::ViewState {
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
