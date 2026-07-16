use crate::geometry::area;
use std::time::Instant;

use super::super::super::{buffer::Buffer, document::Style, surface::Area, view::ViewState};
use super::super::{
    constants::TEXT_AREA_RENDER_MAX_WINDOW_LINES,
    content::text_area_estimated_line_height,
    engine::Engine,
    height::{TextAreaHeightIndex, TextAreaHeightKey},
    output::TextAreaSurface,
    text_area::{self, LineDisplay as TextAreaLineDisplay, RenderLineWindow},
};

const TEXT_AREA_RENDER_REFINEMENT_PASSES: usize = 4;

struct PreparedLine {
    displays: Vec<TextAreaLineDisplay>,
    height: f32,
}

impl Engine {
    pub(super) fn text_area_render_surfaces(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: area::Logical,
        state: &ViewState,
    ) -> Vec<TextAreaSurface> {
        let total_started = Instant::now();
        self.diagnostics.text_area_render_surface_calls += 1;

        let line_count = source.logical_line_count().max(1);
        let estimated_line_height = text_area_estimated_line_height(style);
        let height_key = TextAreaHeightKey::new(area_model, style, viewport.width());
        let mut height_index = if committed {
            self.text_area_height_indices
                .pop(&height_key)
                .unwrap_or_else(|| TextAreaHeightIndex::new(line_count, estimated_line_height))
        } else {
            TextAreaHeightIndex::new(line_count, estimated_line_height)
        };
        height_index.sync(source, line_count, estimated_line_height);

        let scroll_y = state.scroll_y().max(0.0);
        // The runway is small and bounded. One contiguous allocation measured
        // faster than a hash table and avoids one B-tree allocation per line.
        let mut prepared =
            Vec::<(usize, PreparedLine)>::with_capacity(TEXT_AREA_RENDER_MAX_WINDOW_LINES);
        let mut final_window =
            render_window(&height_index, scroll_y, viewport.height(), line_count);

        for _ in 0..TEXT_AREA_RENDER_REFINEMENT_PASSES {
            let window = final_window;
            self.prepare_text_area_render_lines(
                area_model,
                source,
                committed,
                style,
                viewport,
                state,
                window,
                &mut height_index,
                &mut prepared,
            );
            final_window = render_window(&height_index, scroll_y, viewport.height(), line_count);
            if final_window == window {
                break;
            }
        }
        self.prepare_text_area_render_lines(
            area_model,
            source,
            committed,
            style,
            viewport,
            state,
            final_window,
            &mut height_index,
            &mut prepared,
        );

        self.diagnostics.text_area_height_index_queries +=
            final_window.end.saturating_sub(final_window.start) + 2;
        let window_y = height_index.line_top(final_window.start) - scroll_y;
        let window_bottom = height_index.line_top(final_window.end) - scroll_y;
        let resident_bottom = window_bottom.max(viewport.height());
        let resident_height = (resident_bottom - window_y).max(1.0);
        let mut surfaces = Vec::new();
        let mut resident_width = 0.0_f32;
        let mut window_origin_x = 0.0_f64;

        for source_line in final_window.start..final_window.end {
            let Some(index) = prepared.iter().position(|(line, _)| *line == source_line) else {
                continue;
            };
            let (_, line) = prepared.swap_remove(index);
            let y = height_index.line_top(source_line) - scroll_y;
            let height = if source_line + 1 == final_window.end {
                line.height.max(resident_bottom - y)
            } else {
                line.height
            };
            for display in line.displays {
                resident_width = resident_width.max(display.surface_width);
                window_origin_x =
                    window_origin_x.max(f64::from(display.surface_x) + state.exact_scroll_x());
                surfaces.push(TextAreaSurface {
                    x: display.surface_x,
                    y,
                    text_x: display.text_x,
                    width: display.surface_width,
                    height,
                    source_line,
                    source_line_id: display.source_line_id,
                    source_start: display.source_start,
                    source_line_byte_start: display.source_line_byte_start,
                    source_text_len: display.source_text_len,
                    buffer: display.buffer,
                    default_color: style.color(),
                });
            }
        }

        self.diagnostics.text_area_render_window_origin_x_max = self
            .diagnostics
            .text_area_render_window_origin_x_max
            .max(ceil_f64_to_usize(window_origin_x));
        self.diagnostics.text_area_render_window_origin_y_max = self
            .diagnostics
            .text_area_render_window_origin_y_max
            .max(ceil_to_usize(scroll_y));
        self.diagnostics.text_area_render_window_width_max = self
            .diagnostics
            .text_area_render_window_width_max
            .max(ceil_to_usize(resident_width));
        self.diagnostics.text_area_render_window_height_max = self
            .diagnostics
            .text_area_render_window_height_max
            .max(ceil_to_usize(resident_height));
        self.diagnostics.text_area_render_window_area_max = self
            .diagnostics
            .text_area_render_window_area_max
            .max(ceil_to_usize(resident_width).saturating_mul(ceil_to_usize(resident_height)));

        if committed {
            self.text_area_height_indices.put(height_key, height_index);
        }
        self.diagnostics.text_area_render_surface_total_us += elapsed_micros(total_started);
        surfaces
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_text_area_render_lines(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: area::Logical,
        state: &ViewState,
        window: RenderLineWindow,
        height_index: &mut TextAreaHeightIndex,
        prepared: &mut Vec<(usize, PreparedLine)>,
    ) {
        for source_line in window.start..window.end {
            if prepared.iter().any(|(line, _)| *line == source_line) {
                continue;
            }
            let displays = self.text_area_line_displays_at(
                area_model,
                source,
                committed,
                style,
                viewport,
                state,
                source_line,
            );
            let height = displays
                .iter()
                .map(|display| display.height.max(1.0))
                .max_by(f32::total_cmp)
                .unwrap_or_else(|| text_area_estimated_line_height(style));
            let delta = height_index.update_line(source, source_line, height);
            if delta.abs() > f32::EPSILON {
                self.diagnostics.text_area_height_index_updates += 1;
                self.diagnostics.text_area_height_index_refined_pixels +=
                    delta.abs().ceil() as usize;
            }
            for display in &displays {
                if display.cache_hit {
                    self.diagnostics.text_area_render_surface_cache_hits += 1;
                } else {
                    self.diagnostics.text_area_render_surface_cache_misses += 1;
                    self.diagnostics.text_area_render_surface_source_lines += 1;
                    self.diagnostics.text_area_render_surface_source_bytes +=
                        display.source_text_len;
                }
                self.diagnostics.text_area_render_surface_line_reuses += 1;
            }
            prepared.push((source_line, PreparedLine { displays, height }));
        }
    }
}

fn render_window(
    height_index: &TextAreaHeightIndex,
    scroll_y: f32,
    viewport_height: f32,
    line_count: usize,
) -> RenderLineWindow {
    let visible_line = height_index.line_at_y(scroll_y);
    let visible_lines = height_index.visible_line_count(scroll_y, viewport_height);
    let visible_line_end = visible_line.saturating_add(visible_lines).min(line_count);
    text_area::render_line_window(visible_line, visible_line_end, line_count)
}

fn elapsed_micros(start: Instant) -> u128 {
    start.elapsed().as_micros()
}

fn ceil_to_usize(value: f32) -> usize {
    value.ceil().max(0.0).min(usize::MAX as f32) as usize
}

fn ceil_f64_to_usize(value: f64) -> usize {
    value.ceil().max(0.0).min(usize::MAX as f64) as usize
}
