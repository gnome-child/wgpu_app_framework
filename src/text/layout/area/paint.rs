use crate::geometry::area;
use std::time::Instant;

use super::super::super::{
    Preedit,
    buffer::{Cursor, local_cursor_range_for_source_line},
    document::Style,
    surface::{Area, AreaWrap, PreeditProjection},
    view::ViewState,
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

struct AreaPaintRequest<'a> {
    area_model: &'a Area,
    style: Style,
    viewport: area::Logical,
    state: ViewState,
    preedit: Option<&'a Preedit>,
    now: Instant,
    content_area: Option<area::Logical>,
}

struct SegmentLayoutRequest<'a> {
    area_model: &'a Area,
    style: Style,
    viewport: area::Logical,
    state: &'a ViewState,
    now: Instant,
    projection: &'a PreeditProjection,
    segments: &'a [TextAreaDisplaySegment],
    content_area: Option<area::Logical>,
}

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
        let projection = PreeditProjection::new(area_model.buffer(), area_model.state(), None);
        let content_height = self.text_area_content_height(
            area_model,
            &projection.buffer,
            !projection.has_preedit(),
            style,
            viewport,
        );
        let content_width = match area_model.wrap() {
            AreaWrap::None => {
                self.text_area_logical_width(&projection.buffer, style, viewport.width().max(0.0))
            }
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
        self.text_area_paint_layout_for_area_with_preedit_at(
            area_model, style, viewport, state, None, now,
        )
    }

    pub(crate) fn text_area_paint_layout_for_area_with_preedit_at(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: ViewState,
        preedit: Option<&Preedit>,
        now: Instant,
    ) -> TextAreaPaintLayout {
        self.text_area_layout_for_area_at_with_observation(
            AreaPaintRequest {
                area_model,
                style,
                viewport,
                state,
                preedit,
                now,
                content_area: None,
            },
            TextAreaObservation::Observe,
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
        self.text_area_render_layout_for_area_with_preedit_at(
            area_model,
            style,
            viewport,
            state,
            None,
            now,
            content_area,
        )
    }

    pub(crate) fn text_area_render_layout_for_area_with_preedit_at(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: area::Logical,
        state: ViewState,
        preedit: Option<&Preedit>,
        now: Instant,
        content_area: area::Logical,
    ) -> TextAreaPaintLayout {
        self.text_area_layout_for_area_at_with_observation(
            AreaPaintRequest {
                area_model,
                style,
                viewport,
                state,
                preedit,
                now,
                content_area: Some(content_area),
            },
            TextAreaObservation::RenderOnly,
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
        let projection = PreeditProjection::new(area_model.buffer(), area_model.state(), None);
        let committed = !projection.has_preedit();
        let segments = self.text_area_display_segments(
            area_model,
            &projection.buffer,
            committed,
            style,
            viewport,
            &state,
        );
        let layout = self.text_area_layout_from_segments(SegmentLayoutRequest {
            area_model,
            style,
            viewport,
            state: &state,
            now,
            projection: &projection,
            segments: &segments,
            content_area: Some(content_area),
        });
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
        let projection = PreeditProjection::new(area_model.buffer(), area_model.state(), None);
        let selection = projection.selection_bounds();
        let (preedit_underline, preedit_selection) = projection.highlight_ranges();
        let mut spans = HighlightSpans::default();
        #[cfg(test)]
        let mut combined_stats = HighlightStats::default();
        #[cfg(not(test))]
        let combined_stats = HighlightStats::default();
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
                let source_cursor = projection.cursor();
                let cursor =
                    Cursor::new_with_affinity(0, source_cursor.index, source_cursor.affinity);
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
        request: AreaPaintRequest<'_>,
        observation: TextAreaObservation,
    ) -> TextAreaPaintLayout {
        self.diagnostics.text_area_paint_layout_calls += 1;
        let projection = PreeditProjection::new(
            request.area_model.buffer(),
            request.area_model.state(),
            request.preedit,
        );
        let committed = !projection.has_preedit();

        let observe = observation == TextAreaObservation::Observe;

        let segments = if observe {
            self.text_area_display_segments(
                request.area_model,
                &projection.buffer,
                committed,
                request.style,
                request.viewport,
                &request.state,
            )
        } else {
            self.record_text_area_render_window(
                request.area_model,
                &projection.buffer,
                committed,
                request.style,
                request.viewport,
                &request.state,
            );
            Vec::new()
        };
        let layout = self.text_area_layout_from_segments(SegmentLayoutRequest {
            area_model: request.area_model,
            style: request.style,
            viewport: request.viewport,
            state: &request.state,
            now: request.now,
            projection: &projection,
            segments: &segments,
            content_area: request.content_area,
        });
        self.diagnostics.text_area_interaction_surfaces += segments.len();
        let surfaces = segments
            .iter()
            .map(|segment| {
                text_area::surface_for_segment(
                    segment,
                    request.style,
                    request.viewport,
                    &request.state,
                )
            })
            .collect();
        let render_surfaces = self.text_area_render_surfaces(
            request.area_model,
            &projection.buffer,
            committed,
            request.style,
            request.viewport,
            &request.state,
        );
        TextAreaPaintLayout {
            layout,
            interaction_surfaces: surfaces,
            render_surfaces,
        }
    }

    fn text_area_layout_from_segments(
        &mut self,
        request: SegmentLayoutRequest<'_>,
    ) -> TextFieldLayout {
        let mut spans = HighlightSpans::default();
        #[cfg(test)]
        let mut combined_stats = HighlightStats::default();
        #[cfg(not(test))]
        let combined_stats = HighlightStats::default();
        let selection = request.projection.selection_bounds();
        let (preedit_underline, preedit_selection) = request.projection.highlight_ranges();
        let mut caret = None;
        let mut observed_width: f32 = 0.0;

        for segment in request.segments {
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
                request.state.scroll_x(),
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
                && !request.projection.has_non_empty_selection()
                && request.area_model.paints_caret()
                && request.state.caret_visible(request.now)
                && request.projection.cursor().line == segment.display.source_line
            {
                let source_cursor = request.projection.cursor();
                let cursor =
                    Cursor::new_with_affinity(0, source_cursor.index, source_cursor.affinity);
                caret = cursor_position(&buffer, cursor).map(|(x, y)| Caret {
                    x: x as f32 - request.state.scroll_x(),
                    y: segment.y + y as f32,
                    height: buffer.metrics().line_height,
                });
            }
        }

        self.add_highlight_stats(combined_stats);
        let content_area = request.content_area.unwrap_or_else(|| {
            let content_width = match request.area_model.wrap() {
                AreaWrap::None => self.text_area_logical_width(
                    &request.projection.buffer,
                    request.style,
                    request.viewport.width().max(observed_width),
                ),
                AreaWrap::WordOrGlyph => text_area_content_width(
                    request.area_model.wrap(),
                    request.viewport,
                    observed_width,
                ),
            };
            area::logical(
                content_width,
                self.text_area_content_height(
                    request.area_model,
                    &request.projection.buffer,
                    !request.projection.has_preedit(),
                    request.style,
                    request.viewport,
                ),
            )
        });
        TextFieldLayout {
            selection_spans: spans.selection,
            preedit_underline_spans: spans.preedit_underline,
            preedit_selection_spans: spans.preedit_selection,
            caret,
            scroll_x: request.state.scroll_x(),
            scroll_y: request.state.scroll_y(),
            content_area,
        }
    }

    fn text_area_logical_width(
        &mut self,
        source: &super::super::super::buffer::Buffer,
        style: Style,
        minimum: f32,
    ) -> f32 {
        let key = super::super::width::Key::new(source, style);
        let width = if let Some(width) = self.text_area_widths.get(&key).copied() {
            self.diagnostics.text_area_width_cache_hits += 1;
            width
        } else {
            self.diagnostics.text_area_width_cache_misses += 1;
            self.diagnostics.text_area_width_source_lines += source.logical_line_count();
            self.diagnostics.text_area_width_source_bytes += source.len();
            let started = Instant::now();
            let width = super::super::width::measure(&mut self.font_system, source, style);
            self.diagnostics.text_area_width_measure_us += started.elapsed().as_micros();
            self.text_area_widths.put(key, width);
            width
        };
        width.max(minimum.max(0.0))
    }
}
