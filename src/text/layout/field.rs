use std::time::Instant;

use super::super::{
    buffer::{Buffer, Position},
    document::Style,
    edit::{
        Field, State, ViewState,
        surface::{FieldProjection, PreeditProjection, projected_state_for_field},
    },
};
use super::{
    caret::{Caret, CaretLayout, ensure_visible_from_layout},
    content::buffer_content_area,
    engine::Engine,
    glyph::{cosmic_buffer_from_text, cursor_position},
    highlight::spans_for_ranges as highlight_spans_for_ranges,
    map::TextLayoutMap,
    output::TextFieldLayout,
    system,
};
use crate::geometry::{area, point};

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
        now: Instant,
    ) -> TextFieldLayout {
        let projection = PreeditProjection::new(buffer, edit_state, &state);
        let (prepared, vertical_offset) =
            self.prepare_text_field_buffer(&projection.buffer, style, area);
        let ranges = projection.highlight_ranges();
        let (spans, stats) = highlight_spans_for_ranges(
            &prepared,
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
                cursor_position(&prepared, projection.cursor()).map(|(x, y)| Caret {
                    x: x as f32 - state.field_scroll_x(),
                    y: vertical_offset + y as f32,
                    height: prepared.metrics().line_height,
                })
            })
            .flatten();
        TextFieldLayout {
            selection_spans: spans.selection,
            preedit_underline_spans: spans.preedit_underline,
            preedit_selection_spans: spans.preedit_selection,
            caret,
            scroll_x: state.field_scroll_x(),
            scroll_y: 0.0,
            content_area: buffer_content_area(&prepared),
        }
    }

    pub fn text_field_layout_for_field_at(
        &mut self,
        field: &Field,
        style: Style,
        area: area::Logical,
        state: ViewState,
        now: Instant,
    ) -> TextFieldLayout {
        let projection = FieldProjection::new(field);
        let state = projected_state_for_field(field, state);
        let mut layout = self.text_field_layout_at_for_state(
            &projection.buffer,
            projection.edit_state,
            style,
            area,
            state,
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
    ) -> Option<Position> {
        let projection = PreeditProjection::new(buffer, edit_state, &state);
        let (prepared, vertical_offset) =
            self.prepare_text_field_buffer(&projection.buffer, style, area);
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
        let projection = FieldProjection::new(field);
        let state = projected_state_for_field(field, state);
        let display = self.text_field_position_at_for_state(
            &projection.buffer,
            projection.edit_state,
            style,
            area,
            position,
            state,
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
    ) -> ViewState {
        let projection = PreeditProjection::new(buffer, edit_state, &state);
        let (prepared, vertical_offset) =
            self.prepare_text_field_buffer(&projection.buffer, style, area);
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
        let projection = FieldProjection::new(field);
        self.ensure_caret_visible_for_state(
            &projection.buffer,
            projection.edit_state,
            style,
            area,
            projected_state_for_field(field, state),
        )
    }

    pub(in crate::text) fn prepare_text_field_buffer(
        &mut self,
        buffer: &Buffer,
        style: Style,
        area: area::Logical,
    ) -> (glyphon::Buffer, f32) {
        let font_size = style.size().max(1.0);
        let line_height = font_size * 1.25;
        let buffer_height = area.height().max(0.0).min(line_height);
        let vertical_offset = (area.height().max(0.0) - buffer_height).max(0.0) * 0.5;
        let attrs = system::attrs_for_style(style);
        let mut prepared = cosmic_buffer_from_text(&buffer.text_for_line_range(0, 1));
        for line in &mut prepared.lines {
            line.set_attrs_list(glyphon::AttrsList::new(&attrs));
        }
        prepared.set_wrap(&mut self.font_system, glyphon::Wrap::None);
        prepared.set_metrics_and_size(
            &mut self.font_system,
            glyphon::Metrics::relative(font_size, 1.25),
            Some(area.width().max(0.0)),
            Some(buffer_height),
        );
        prepared.shape_until_scroll(&mut self.font_system, false);
        (prepared, vertical_offset)
    }
}
