use std::time::Instant;

use super::super::super::{
    buffer::{Cursor, local_cursor_range_for_source_line},
    document::Style,
    edit::{Area, AreaWrap, ViewState, surface::PreeditProjection},
};
use super::super::{
    caret::Caret,
    content::text_area_content_width,
    diagnostics::HighlightStats,
    engine::Engine,
    glyph::cursor_position,
    highlight::{HighlightSpans, spans_for_ranges as highlight_spans_for_ranges},
    output::{TextAreaPaintLayout, TextAreaSurface, TextFieldLayout},
    text_area,
    text_area::{DisplaySegment as TextAreaDisplaySegment, Observation as TextAreaObservation},
};
use crate::paint_geometry::area;

impl Engine {
    pub fn text_area_metrics_layout_for_area_at(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: ViewState,
        _now: Instant,
    ) -> TextFieldLayout {
        self.diagnostics.text_area_metrics_layout_calls += 1;
        let projection = PreeditProjection::new(area_model.buffer(), area_model.state(), &state);
        let content_height = self.text_area_content_height(
            area_model,
            &projection.buffer,
            !projection.has_preedit(),
            style,
            viewport,
        );
        let content_width = match area_model.wrap() {
            AreaWrap::None => self
                .text_area_display_segments(
                    area_model,
                    &projection.buffer,
                    !projection.has_preedit(),
                    style,
                    viewport,
                    &state,
                )
                .iter()
                .map(|segment| segment.display.width)
                .fold(viewport.width().max(0.0), f32::max),
            AreaWrap::WordOrGlyph => viewport.width().max(0.0),
        };
        let content_area = area::logical(content_width, content_height);
        TextFieldLayout {
            selection_spans: Vec::new(),
            preedit_underline_spans: Vec::new(),
            preedit_selection_spans: Vec::new(),
            caret: None,
            scroll_x: state.scroll_x(),
            scroll_y: state.scroll_y(),
            content_area,
        }
    }

    pub fn text_area_paint_layout_for_area_at(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: ViewState,
        now: Instant,
    ) -> TextAreaPaintLayout {
        self.text_area_layout_for_area_at_with_observation(
            area_model,
            style,
            viewport,
            state,
            now,
            TextAreaObservation::Observe,
            None,
        )
    }

    pub fn text_area_render_layout_for_area_at(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: ViewState,
        now: Instant,
        content_area: area::Logical,
    ) -> TextAreaPaintLayout {
        self.text_area_layout_for_area_at_with_observation(
            area_model,
            style,
            viewport,
            state,
            now,
            TextAreaObservation::RenderOnly,
            Some(content_area),
        )
    }

    pub fn text_area_overlay_layout_for_area_at(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: ViewState,
        now: Instant,
        content_area: area::Logical,
    ) -> TextFieldLayout {
        let projection = PreeditProjection::new(area_model.buffer(), area_model.state(), &state);
        let committed = !projection.has_preedit();
        let segments = self.text_area_display_segments(
            area_model,
            &projection.buffer,
            committed,
            style,
            viewport,
            &state,
        );
        let layout = self.text_area_layout_from_segments(
            area_model,
            style,
            viewport,
            &state,
            now,
            &projection,
            &segments,
            Some(content_area),
        );
        self.diagnostics.text_area_interaction_surfaces += segments.len();
        layout
    }

    pub fn text_area_overlay_layout_for_surfaces_at(
        &mut self,
        area_model: &Area,
        state: ViewState,
        now: Instant,
        content_area: area::Logical,
        surfaces: &[TextAreaSurface],
    ) -> TextFieldLayout {
        let projection = PreeditProjection::new(area_model.buffer(), area_model.state(), &state);
        let selection = projection.selection_bounds();
        let (preedit_underline, preedit_selection) = projection.highlight_ranges();
        let mut spans = HighlightSpans::default();
        #[allow(unused_mut)]
        let mut combined_stats = HighlightStats::default();
        let mut caret = None;

        for surface in surfaces {
            let buffer = surface.buffer.borrow();
            let selection = selection.and_then(|range| {
                local_cursor_range_for_source_line(
                    range,
                    surface.source_line,
                    surface.source_text_len,
                )
            });
            let preedit_underline = preedit_underline.and_then(|range| {
                local_cursor_range_for_source_line(
                    range,
                    surface.source_line,
                    surface.source_text_len,
                )
            });
            let preedit_selection = preedit_selection.and_then(|range| {
                local_cursor_range_for_source_line(
                    range,
                    surface.source_line,
                    surface.source_text_len,
                )
            });
            let (line_spans, stats) = highlight_spans_for_ranges(
                &buffer,
                selection,
                preedit_underline,
                preedit_selection,
                surface.y,
                -surface.x,
                0.0,
            );
            spans.selection.extend(line_spans.selection);
            spans.preedit_underline.extend(line_spans.preedit_underline);
            spans.preedit_selection.extend(line_spans.preedit_selection);
            #[cfg(test)]
            combined_stats.add(stats);
            #[cfg(not(test))]
            let _ = stats;

            if caret.is_none()
                && !projection.has_non_empty_selection()
                && area_model.paints_caret()
                && state.caret_visible(now)
                && projection.cursor().line == surface.source_line
            {
                let cursor = Cursor::new(0, projection.cursor().index);
                caret = cursor_position(&buffer, cursor).map(|(x, y)| Caret {
                    x: x as f32 + surface.x,
                    y: surface.y + y as f32,
                    height: buffer.metrics().line_height,
                });
            }
        }

        self.add_highlight_stats(combined_stats);
        TextFieldLayout {
            selection_spans: spans.selection,
            preedit_underline_spans: spans.preedit_underline,
            preedit_selection_spans: spans.preedit_selection,
            caret,
            scroll_x: state.scroll_x(),
            scroll_y: state.scroll_y(),
            content_area,
        }
    }

    fn text_area_layout_for_area_at_with_observation(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: ViewState,
        now: Instant,
        observation: TextAreaObservation,
        content_area: Option<area::Logical>,
    ) -> TextAreaPaintLayout {
        self.diagnostics.text_area_paint_layout_calls += 1;
        let projection = PreeditProjection::new(area_model.buffer(), area_model.state(), &state);
        let committed = !projection.has_preedit();

        let observe = observation == TextAreaObservation::Observe;

        let segments = if observe {
            self.text_area_display_segments(
                area_model,
                &projection.buffer,
                committed,
                style,
                viewport,
                &state,
            )
        } else {
            self.record_text_area_render_window(
                area_model,
                &projection.buffer,
                committed,
                style,
                viewport,
                &state,
            );
            Vec::new()
        };
        let layout = self.text_area_layout_from_segments(
            area_model,
            style,
            viewport,
            &state,
            now,
            &projection,
            &segments,
            content_area,
        );
        self.diagnostics.text_area_interaction_surfaces += segments.len();
        let surfaces = segments
            .iter()
            .map(|segment| text_area::surface_for_segment(segment, style, viewport, &state))
            .collect();
        let render_surfaces = self.text_area_render_surfaces(
            area_model,
            &projection.buffer,
            committed,
            style,
            viewport,
            &state,
        );
        TextAreaPaintLayout {
            layout,
            interaction_surfaces: surfaces,
            render_surfaces,
        }
    }

    #[allow(dead_code)]
    fn text_area_layout_for_area_at(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: ViewState,
        now: Instant,
    ) -> TextFieldLayout {
        let projection = PreeditProjection::new(area_model.buffer(), area_model.state(), &state);
        let committed = !projection.has_preedit();
        let segments = self.text_area_display_segments(
            area_model,
            &projection.buffer,
            committed,
            style,
            viewport,
            &state,
        );
        self.text_area_layout_from_segments(
            area_model,
            style,
            viewport,
            &state,
            now,
            &projection,
            &segments,
            None,
        )
    }

    fn text_area_layout_from_segments(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: &ViewState,
        now: Instant,
        projection: &PreeditProjection,
        segments: &[TextAreaDisplaySegment],
        content_area: Option<area::Logical>,
    ) -> TextFieldLayout {
        let mut spans = HighlightSpans::default();
        #[allow(unused_mut)]
        let mut combined_stats = HighlightStats::default();
        let selection = projection.selection_bounds();
        let (preedit_underline, preedit_selection) = projection.highlight_ranges();
        let mut caret = None;
        let mut observed_width: f32 = 0.0;

        for segment in segments {
            observed_width = observed_width.max(segment.display.width);
            let buffer = segment.display.buffer.borrow();
            let selection = selection.and_then(|range| {
                local_cursor_range_for_source_line(
                    range,
                    segment.display.source_line,
                    segment.display.source_text_len,
                )
            });
            let preedit_underline = preedit_underline.and_then(|range| {
                local_cursor_range_for_source_line(
                    range,
                    segment.display.source_line,
                    segment.display.source_text_len,
                )
            });
            let preedit_selection = preedit_selection.and_then(|range| {
                local_cursor_range_for_source_line(
                    range,
                    segment.display.source_line,
                    segment.display.source_text_len,
                )
            });
            let (line_spans, stats) = highlight_spans_for_ranges(
                &buffer,
                selection,
                preedit_underline,
                preedit_selection,
                segment.y,
                state.scroll_x(),
                0.0,
            );
            spans.selection.extend(line_spans.selection);
            spans.preedit_underline.extend(line_spans.preedit_underline);
            spans.preedit_selection.extend(line_spans.preedit_selection);
            #[cfg(test)]
            combined_stats.add(stats);
            #[cfg(not(test))]
            let _ = stats;

            if caret.is_none()
                && !projection.has_non_empty_selection()
                && area_model.paints_caret()
                && state.caret_visible(now)
                && projection.cursor().line == segment.display.source_line
            {
                let cursor = Cursor::new(0, projection.cursor().index);
                caret = cursor_position(&buffer, cursor).map(|(x, y)| Caret {
                    x: x as f32 - state.scroll_x(),
                    y: segment.y + y as f32,
                    height: buffer.metrics().line_height,
                });
            }
        }

        self.add_highlight_stats(combined_stats);
        let content_area = content_area.unwrap_or_else(|| {
            area::logical(
                text_area_content_width(area_model.wrap(), viewport, observed_width),
                self.text_area_content_height(
                    area_model,
                    &projection.buffer,
                    !projection.has_preedit(),
                    style,
                    viewport,
                ),
            )
        });
        TextFieldLayout {
            selection_spans: spans.selection,
            preedit_underline_spans: spans.preedit_underline,
            preedit_selection_spans: spans.preedit_selection,
            caret,
            scroll_x: state.scroll_x(),
            scroll_y: state.scroll_y(),
            content_area,
        }
    }
}
