use crate::geometry::{area, point};
use std::{cell::RefCell, num::NonZeroUsize, rc::Rc, time::Instant};

use super::super::{
    Preedit,
    buffer::{Buffer, Position},
    document::{Style, TextDirection, Weight},
    selection::State,
    surface::{Field, FieldProjection, PreeditProjection, projected_preedit_for_field},
    view::ViewState,
};
use super::{
    caret::{Caret, CaretLayout, ensure_visible_from_layout},
    content::buffer_content_area,
    engine::Engine,
    glyph::{cosmic_buffer_from_text, cursor_position},
    highlight::spans_for_ranges as highlight_spans_for_ranges,
    key::finite_bits,
    map::TextLayoutMap,
    output::{TextAreaSurface, TextFieldLayout, TextFieldPaintLayout},
    shaping_cache::ShapingCache,
    system,
};

const TEXT_FIELD_SURFACE_CACHE_CAPACITY: NonZeroUsize = NonZeroUsize::new(512).unwrap();

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct FieldSurfaceKey {
    text: String,
    size: u32,
    weight: Weight,
    direction: TextDirection,
    width: u32,
    height: u32,
}

#[derive(Clone)]
pub(super) struct CachedFieldSurface {
    buffer: Rc<RefCell<glyphon::Buffer>>,
    vertical_offset: f32,
}

pub(super) fn surface_cache() -> ShapingCache<FieldSurfaceKey, CachedFieldSurface> {
    ShapingCache::new(TEXT_FIELD_SURFACE_CACHE_CAPACITY)
}

impl FieldSurfaceKey {
    fn new(buffer: &Buffer, style: Style, area: area::Logical) -> Self {
        Self {
            text: buffer.text_for_line_range(0, 1),
            size: finite_bits(style.size().max(1.0)),
            weight: style.weight(),
            direction: style.direction(),
            width: finite_bits(area.width().max(0.0)),
            height: finite_bits(area.height().max(0.0)),
        }
    }
}

impl Engine {
    pub fn text_field_layout_for_field(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: ViewState,
    ) -> TextFieldLayout {
        self.text_field_layout_for_field_at(field, style, area, state, Instant::now())
    }

    fn text_field_layout_at_for_state(
        &mut self,
        buffer: &Buffer,
        edit_state: State,
        style: Style,
        area: area::Logical,
        state: ViewState,
        preedit: Option<&Preedit>,
        now: Instant,
    ) -> TextFieldLayout {
        self.text_field_paint_layout_at_for_state(
            buffer, edit_state, style, area, state, preedit, now,
        )
        .layout
    }

    fn text_field_paint_layout_at_for_state(
        &mut self,
        buffer: &Buffer,
        edit_state: State,
        style: Style,
        area: area::Logical,
        state: ViewState,
        preedit: Option<&Preedit>,
        now: Instant,
    ) -> TextFieldPaintLayout {
        let projection = PreeditProjection::new(buffer, edit_state, preedit);
        let (prepared, vertical_offset) =
            self.prepare_text_field_buffer(&projection.buffer, style, area);
        let prepared_buffer = prepared.borrow();
        let ranges = projection.highlight_ranges();
        let (spans, stats) = highlight_spans_for_ranges(
            &prepared_buffer,
            projection.selection_bounds(),
            ranges.0,
            ranges.1,
            vertical_offset,
            state.field_scroll_x(),
            0.0,
        );
        self.add_highlight_stats(stats);
        let caret = (!projection.has_non_empty_selection() && state.caret_visible(now))
            .then(|| {
                cursor_position(&prepared_buffer, projection.cursor()).map(|(x, y)| Caret {
                    x: x as f32 - state.field_scroll_x(),
                    y: vertical_offset + y as f32,
                    height: prepared_buffer.metrics().line_height,
                })
            })
            .flatten();

        let content_area = buffer_content_area(&prepared_buffer);
        let line_height = prepared_buffer.metrics().line_height;
        drop(prepared_buffer);
        let source_text_len = projection.buffer.text_for_line_range(0, 1).len();
        let surface = (area.width() > 0.0 && area.height() > 0.0).then(|| TextAreaSurface {
            x: -state.field_scroll_x(),
            y: vertical_offset,
            width: content_area.width().max(area.width()) + state.field_scroll_x().max(0.0),
            height: line_height.max(1.0),
            source_line: 0,
            source_line_id: projection
                .buffer
                .line_layout_identity(0)
                .map(|identity| identity.id),
            source_start: 0,
            source_text_len,
            buffer: prepared,
            default_color: style.color(),
        });

        TextFieldPaintLayout {
            layout: TextFieldLayout {
                selection_spans: spans.selection,
                preedit_underline_spans: spans.preedit_underline,
                preedit_selection_spans: spans.preedit_selection,
                caret,
                scroll_x: state.field_scroll_x(),
                scroll_y: 0.0,
                content_area,
            },
            surface,
        }
    }

    pub fn text_field_paint_layout_for_field(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: ViewState,
    ) -> TextFieldPaintLayout {
        self.text_field_paint_layout_for_field_at(field, style, area, state, Instant::now())
    }

    pub fn text_field_paint_layout_for_field_at(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: ViewState,
        now: Instant,
    ) -> TextFieldPaintLayout {
        self.text_field_paint_layout_for_field_with_preedit_at(field, style, area, state, None, now)
    }

    pub(crate) fn text_field_paint_layout_for_field_with_preedit_at(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: ViewState,
        preedit: Option<&Preedit>,
        now: Instant,
    ) -> TextFieldPaintLayout {
        let projection = FieldProjection::new(field);
        let preedit = projected_preedit_for_field(field, preedit);
        let mut layout = self.text_field_paint_layout_at_for_state(
            &projection.buffer,
            projection.edit_state,
            style,
            area,
            state,
            preedit.as_deref(),
            now,
        );
        if !field.paints_caret() {
            layout.layout.caret = None;
        }
        layout
    }

    pub fn text_field_layout_for_field_at(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: ViewState,
        now: Instant,
    ) -> TextFieldLayout {
        self.text_field_layout_for_field_with_preedit_at(field, style, area, state, None, now)
    }

    pub(crate) fn text_field_layout_for_field_with_preedit_at(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: ViewState,
        preedit: Option<&Preedit>,
        now: Instant,
    ) -> TextFieldLayout {
        let projection = FieldProjection::new(field);
        let preedit = projected_preedit_for_field(field, preedit);
        let mut layout = self.text_field_layout_at_for_state(
            &projection.buffer,
            projection.edit_state,
            style,
            area,
            state,
            preedit.as_deref(),
            now,
        );
        if !field.paints_caret() {
            layout.caret = None;
        }
        layout
    }

    fn text_field_position_at_for_state(
        &mut self,
        buffer: &Buffer,
        edit_state: State,
        style: Style,
        area: area::Logical,
        position: point::Logical,
        state: ViewState,
        preedit: Option<&Preedit>,
    ) -> Option<Position> {
        let projection = PreeditProjection::new(buffer, edit_state, preedit);
        let (prepared, vertical_offset) =
            self.prepare_text_field_buffer(&projection.buffer, style, area);
        let prepared = prepared.borrow();
        TextLayoutMap::from_line_starts(projection.buffer.line_start_offsets()).hit_with_observer(
            &prepared,
            position.x() + state.field_scroll_x(),
            position.y() - vertical_offset,
            |_| {},
        )
    }

    pub fn text_field_position_at_for_field(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        position: point::Logical,
        state: ViewState,
    ) -> Option<Position> {
        self.text_field_position_at_for_field_with_preedit(
            field, style, area, position, state, None,
        )
    }

    pub(crate) fn text_field_position_at_for_field_with_preedit(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        position: point::Logical,
        state: ViewState,
        preedit: Option<&Preedit>,
    ) -> Option<Position> {
        let projection = FieldProjection::new(field);
        let preedit = projected_preedit_for_field(field, preedit);
        let display = self.text_field_position_at_for_state(
            &projection.buffer,
            projection.edit_state,
            style,
            area,
            position,
            state,
            preedit.as_deref(),
        )?;
        Some(projection.source_position(display))
    }

    pub fn text_field_caret_for_field(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: ViewState,
    ) -> Option<Caret> {
        if !field.paints_caret() {
            None
        } else {
            self.text_field_layout_for_field_at(field, style, area, state, Instant::now())
                .caret()
        }
    }

    fn ensure_caret_visible_for_state(
        &mut self,
        buffer: &Buffer,
        edit_state: State,
        style: Style,
        area: area::Logical,
        state: ViewState,
        preedit: Option<&Preedit>,
    ) -> ViewState {
        let projection = PreeditProjection::new(buffer, edit_state, preedit);
        let (prepared, vertical_offset) =
            self.prepare_text_field_buffer(&projection.buffer, style, area);
        let prepared = prepared.borrow();
        let content_area = buffer_content_area(&prepared);
        let max_scroll_x = (content_area.width() - area.width().max(0.0)).max(0.0);
        let Some((caret_x, caret_y)) = cursor_position(&prepared, projection.cursor()) else {
            return state
                .clone()
                .with_field_scroll_x(state.field_scroll_x().clamp(0.0, max_scroll_x));
        };
        let caret_layout = CaretLayout::new(Caret::new(
            caret_x as f32 - state.field_scroll_x(),
            vertical_offset + caret_y as f32,
            prepared.metrics().line_height,
        ));
        ensure_visible_from_layout(state.clone(), area, caret_layout, Some(content_area))
            .unwrap_or(state)
    }

    pub fn ensure_caret_visible_for_field(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: ViewState,
    ) -> ViewState {
        self.ensure_caret_visible_for_field_with_preedit(field, style, area, state, None)
    }

    pub(crate) fn ensure_caret_visible_for_field_with_preedit(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: ViewState,
        preedit: Option<&Preedit>,
    ) -> ViewState {
        let projection = FieldProjection::new(field);
        let preedit = projected_preedit_for_field(field, preedit);
        self.ensure_caret_visible_for_state(
            &projection.buffer,
            projection.edit_state,
            style,
            area,
            state,
            preedit.as_deref(),
        )
    }

    pub(in crate::text) fn prepare_text_field_buffer(
        &mut self,
        buffer: &Buffer,
        style: Style,
        area: area::Logical,
    ) -> (Rc<RefCell<glyphon::Buffer>>, f32) {
        let key = FieldSurfaceKey::new(buffer, style, area);
        let shaped = self.text_field_surfaces.shape_required(
            &mut self.font_system,
            key,
            true,
            prepare_cached_field_surface,
        );
        (shaped.value.buffer, shaped.value.vertical_offset)
    }
}

fn prepare_cached_field_surface(
    font_system: &mut glyphon::FontSystem,
    key: &FieldSurfaceKey,
) -> CachedFieldSurface {
    let font_size = f32::from_bits(key.size).max(1.0);
    let line_height = font_size * 1.25;
    let area_width = f32::from_bits(key.width).max(0.0);
    let area_height = f32::from_bits(key.height).max(0.0);
    let buffer_height = area_height.min(line_height);
    let vertical_offset = (area_height - buffer_height).max(0.0) * 0.5;
    let style = Style::default()
        .with_size(font_size)
        .with_weight(key.weight)
        .with_direction(key.direction);
    let attrs = system::attrs_for_style(style);
    let mut prepared = cosmic_buffer_from_text(&key.text);
    for line in &mut prepared.lines {
        line.set_attrs_list(glyphon::AttrsList::new(&attrs));
    }
    prepared.set_wrap(font_system, glyphon::Wrap::None);
    prepared.set_metrics_and_size(
        font_system,
        glyphon::Metrics::relative(font_size, 1.25),
        Some(area_width),
        Some(buffer_height),
    );
    prepared.shape_until_scroll(font_system, false);
    CachedFieldSurface {
        buffer: Rc::new(RefCell::new(prepared)),
        vertical_offset,
    }
}
