use crate::geometry::area;
use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::{
    buffer::Buffer,
    document::Style,
    surface::{Area, AreaWrap},
    view::ViewState,
};
use super::super::{
    constants::{
        TEXT_AREA_FRAME_MAX_LOGICAL_LINES, TEXT_AREA_FRAME_MIN_OVERSCAN_LINES,
        TEXT_AREA_LINE_DISPLAY_CACHE_MAX_RESIDENT_BYTES,
    },
    content::{buffer_content_area, text_area_estimated_line_height},
    engine::Engine,
    glyph::{glyph_wrap, set_cosmic_buffer_text, text_area_shaping_for_text},
    height::{TextAreaHeightIndex, TextAreaHeightKey},
    horizontal::{LineIndex as HorizontalLineIndex, Window as HorizontalWindow, prepared_window},
    shaping_cache::Shaped,
    system,
    text_area::{
        CachedLineDisplay as CachedTextAreaLineDisplay, DisplaySegment as TextAreaDisplaySegment,
        LineDisplay as TextAreaLineDisplay, LineDisplayKey as TextAreaLineDisplayKey,
        LineWindowKey as TextAreaLineWindowKey,
    },
};

impl Engine {
    #[cfg(test)]
    pub(in crate::text) fn text_area_unwindowed_line_display_for_test(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        style: Style,
        viewport: area::Logical,
        source_line: usize,
    ) -> TextAreaLineDisplay {
        let cached = prepare_text_area_line_display(
            &mut self.font_system,
            area_model,
            source,
            style,
            viewport,
            source_line,
        );
        TextAreaLineDisplay::from_cached(
            source,
            source_line,
            cached,
            0.0,
            viewport.width(),
            0.0,
            false,
        )
    }

    pub(super) fn text_area_content_height(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: area::Logical,
    ) -> f32 {
        let line_count = source.logical_line_count().max(1);
        let estimated_line_height = text_area_estimated_line_height(style);
        if committed {
            let key = TextAreaHeightKey::new(area_model, style, viewport.width());
            if let Some(index) = self.text_area_height_indices.get_mut(&key) {
                index.sync(source, line_count, estimated_line_height);
                self.diagnostics.text_area_height_index_hits += 1;
                self.diagnostics.text_area_height_index_queries += 1;
                return index.total_height().max(viewport.height().max(0.0));
            }
            self.diagnostics.text_area_height_index_misses += 1;
        }
        (line_count as f32 * estimated_line_height).max(viewport.height().max(0.0))
    }

    #[cfg(test)]
    pub(in crate::text) fn text_area_line_display(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        _committed: bool,
        style: Style,
        viewport: area::Logical,
        source_line: usize,
    ) -> TextAreaLineDisplay {
        self.text_area_unwindowed_line_display_for_test(
            area_model,
            source,
            style,
            viewport,
            source_line,
        )
    }

    pub(super) fn text_area_line_displays_at(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: area::Logical,
        state: &ViewState,
        source_line: usize,
    ) -> Vec<TextAreaLineDisplay> {
        self.text_area_line_displays_for_demand(
            area_model,
            source,
            committed,
            style,
            viewport,
            state,
            source_line,
            None,
        )
    }

    pub(super) fn text_area_line_displays_for_source_byte(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: area::Logical,
        state: &ViewState,
        source_line: usize,
        source_byte: usize,
    ) -> Vec<TextAreaLineDisplay> {
        self.text_area_line_displays_for_demand(
            area_model,
            source,
            committed,
            style,
            viewport,
            state,
            source_line,
            Some(source_byte),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn text_area_line_displays_for_demand(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: area::Logical,
        state: &ViewState,
        source_line: usize,
        source_byte: Option<usize>,
    ) -> Vec<TextAreaLineDisplay> {
        let line_key = TextAreaLineDisplayKey::new(
            area_model,
            source,
            style,
            viewport.width().max(0.0),
            source_line,
        );
        let scroll_x = state.exact_scroll_x().max(0.0);
        let (surface_x, surface_width) = prepared_window(viewport.width(), scroll_x);
        let horizontal = line_key
            .as_ref()
            .and_then(|key| self.text_area_horizontal_indices.get(key).cloned());
        self.diagnostics
            .text_area_horizontal_index_resident_bytes_max = self
            .diagnostics
            .text_area_horizontal_index_resident_bytes_max
            .max(self.text_area_horizontal_index_resident_bytes);
        let horizontal_windows = horizontal.as_ref().map(|index| {
            source_byte.map_or_else(
                || index.windows_for_x(surface_x, surface_width),
                |source_byte| index.windows_for_source_byte(source_byte),
            )
        });
        let source_text_len = source.inner.document.line_text_len(source_line);
        if horizontal.is_none() {
            let full_key = line_key
                .clone()
                .map(|key| TextAreaLineWindowKey::new(key, 0, source_text_len));
            let shaped = if let Some(key) = full_key.as_ref() {
                self.text_area_line_displays.shape_required(
                    &mut self.font_system,
                    key.clone(),
                    committed,
                    |font_system, _| {
                        prepare_text_area_line_display(
                            font_system,
                            area_model,
                            source,
                            style,
                            viewport,
                            source_line,
                        )
                    },
                )
            } else {
                Shaped {
                    value: prepare_text_area_line_display(
                        &mut self.font_system,
                        area_model,
                        source,
                        style,
                        viewport,
                        source_line,
                    ),
                    cache_hit: false,
                }
            };
            let text = source.text_for_line_range(source_line, source_line + 1);
            let full_glyphs = shaped.value.glyph_count;
            if area_model.wrap() == AreaWrap::None
                && !shaped.cache_hit
                && let Some(line_key) = line_key
                && let Some(index) = {
                    let buffer = shaped.value.buffer.borrow();
                    HorizontalLineIndex::from_ascii_buffer(&text, &buffer)
                }
                && let Some((index, exact_band_shapes)) = index.refine_exact_bands(&text, |band| {
                    prepare_text_area_exact_horizontal_band(&mut self.font_system, style, band)
                })
                && index.width() > f64::from(surface_width)
            {
                let visual_runs = shaped.value.buffer.borrow().layout_runs().count();
                self.diagnostics.text_area_line_shape_calls += 1;
                self.diagnostics.text_area_shaped_logical_lines += 1;
                self.diagnostics.text_area_shaped_visual_lines += visual_runs;
                #[cfg(test)]
                {
                    self.interaction_stats.text_area_frame_cache_misses += 1;
                    self.interaction_stats.text_area_frame_shape_calls += 1;
                    self.interaction_stats.text_area_frame_shaped_logical_lines += 1;
                    self.interaction_stats.text_area_frame_shaped_visual_lines += visual_runs;
                }
                self.diagnostics.text_area_horizontal_index_builds += 1;
                self.diagnostics.text_area_horizontal_index_source_bytes += text.len();
                self.diagnostics.text_area_horizontal_index_glyphs += full_glyphs;
                self.diagnostics.text_area_horizontal_index_checkpoints += index.checkpoint_count();
                self.diagnostics.text_area_horizontal_exact_band_shapes += exact_band_shapes;
                self.diagnostics
                    .text_area_horizontal_exact_band_source_bytes +=
                    usize::from(exact_band_shapes > 0).saturating_mul(text.len());
                let index_resident_bytes = index.resident_bytes();
                if let Some(full_key) = full_key.as_ref() {
                    self.text_area_line_displays.remove(full_key);
                }
                self.text_area_horizontal_indices
                    .put(line_key, Rc::new(index));
                self.text_area_horizontal_index_resident_bytes = self
                    .text_area_horizontal_indices
                    .iter()
                    .map(|(_, index)| index.resident_bytes())
                    .sum::<usize>();
                self.diagnostics
                    .text_area_horizontal_index_resident_bytes_max = self
                    .diagnostics
                    .text_area_horizontal_index_resident_bytes_max
                    .max(index_resident_bytes.max(self.text_area_horizontal_index_resident_bytes));
                self.font_system.shape_run_cache = Default::default();
                return self.text_area_line_displays_for_demand(
                    area_model,
                    source,
                    committed,
                    style,
                    viewport,
                    state,
                    source_line,
                    source_byte,
                );
            }
            self.record_text_area_line_shape(&shaped, false);
            let display = TextAreaLineDisplay::from_cached(
                source,
                source_line,
                shaped.value,
                surface_x,
                surface_width,
                scroll_x,
                shaped.cache_hit,
            );
            self.trim_text_area_line_cache();
            return vec![display];
        }

        let index = horizontal.expect("horizontal windows require an index");
        let windows = horizontal_windows.unwrap_or_default();
        let mut displays = Vec::with_capacity(windows.len());
        let mut any_miss = false;
        for window in windows {
            let key = line_key
                .clone()
                .map(|key| TextAreaLineWindowKey::new(key, window.source.start, window.source.end));
            let shaped = if let Some(key) = key {
                self.text_area_line_displays.shape_required(
                    &mut self.font_system,
                    key,
                    committed,
                    |font_system, _| {
                        prepare_text_area_line_window(
                            font_system,
                            area_model,
                            source,
                            style,
                            source_line,
                            &index,
                            &window,
                        )
                    },
                )
            } else {
                Shaped {
                    value: prepare_text_area_line_window(
                        &mut self.font_system,
                        area_model,
                        source,
                        style,
                        source_line,
                        &index,
                        &window,
                    ),
                    cache_hit: false,
                }
            };
            any_miss |= !shaped.cache_hit;
            self.record_text_area_line_shape(&shaped, true);
            displays.push(TextAreaLineDisplay::from_cached(
                source,
                source_line,
                shaped.value,
                surface_x,
                surface_width,
                scroll_x,
                shaped.cache_hit,
            ));
        }
        if any_miss {
            self.diagnostics.text_area_shaped_logical_lines += 1;
            #[cfg(test)]
            {
                self.interaction_stats.text_area_frame_shaped_logical_lines += 1;
            }
            self.font_system.shape_run_cache.trim(2);
        }
        let resident_source_bytes = displays
            .iter()
            .map(|display| display.source_text_len)
            .sum::<usize>();
        let resident_glyphs = displays
            .iter()
            .map(|display| buffer_glyph_count(&display.buffer.borrow()))
            .sum::<usize>();
        let resident_bytes = displays
            .iter()
            .map(|display| {
                buffer_resident_bytes(
                    display.source_text_len,
                    buffer_glyph_count(&display.buffer.borrow()),
                )
            })
            .sum::<usize>();
        self.diagnostics
            .text_area_horizontal_resident_source_bytes_max = self
            .diagnostics
            .text_area_horizontal_resident_source_bytes_max
            .max(resident_source_bytes);
        self.diagnostics.text_area_horizontal_resident_glyphs_max = self
            .diagnostics
            .text_area_horizontal_resident_glyphs_max
            .max(resident_glyphs);
        self.diagnostics.text_area_horizontal_resident_bytes_max = self
            .diagnostics
            .text_area_horizontal_resident_bytes_max
            .max(resident_bytes);
        self.trim_text_area_line_cache();
        displays
    }

    fn record_text_area_line_shape(
        &mut self,
        shaped: &Shaped<CachedTextAreaLineDisplay>,
        horizontal: bool,
    ) {
        if shaped.cache_hit {
            self.diagnostics.text_area_line_cache_hits += 1;
            #[cfg(test)]
            {
                self.interaction_stats.text_area_frame_cache_hits += 1;
            }
            return;
        }
        self.diagnostics.text_area_line_cache_misses += 1;
        let visual_runs = shaped.value.buffer.borrow().layout_runs().count();
        self.diagnostics.text_area_line_shape_calls += 1;
        self.diagnostics.text_area_shaped_visual_lines += visual_runs;
        #[cfg(test)]
        {
            self.interaction_stats.text_area_frame_cache_misses += 1;
            self.interaction_stats.text_area_frame_shape_calls += 1;
            self.interaction_stats.text_area_frame_shaped_visual_lines += visual_runs;
        }
        if horizontal {
            self.diagnostics.text_area_horizontal_window_shapes += 1;
            self.diagnostics.text_area_horizontal_window_source_bytes +=
                shaped.value.source_text_len;
        }
    }

    fn trim_text_area_line_cache(&mut self) {
        self.text_area_line_displays
            .trim_to_weight(TEXT_AREA_LINE_DISPLAY_CACHE_MAX_RESIDENT_BYTES, |display| {
                display.resident_bytes
            });
        let resident = self
            .text_area_line_displays
            .total_by(|display| display.resident_bytes);
        self.diagnostics.text_area_line_cache_resident_bytes_max = self
            .diagnostics
            .text_area_line_cache_resident_bytes_max
            .max(resident);
    }

    fn cached_text_area_line_displays(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: area::Logical,
        state: &ViewState,
        source_line: usize,
    ) -> Option<Vec<TextAreaLineDisplay>> {
        if !committed {
            return None;
        }
        let line_key = TextAreaLineDisplayKey::new(
            area_model,
            source,
            style,
            viewport.width().max(0.0),
            source_line,
        )?;
        let scroll_x = state.exact_scroll_x().max(0.0);
        let (surface_x, surface_width) = prepared_window(viewport.width(), scroll_x);
        let horizontal = self.text_area_horizontal_indices.get(&line_key).cloned();
        let windows = horizontal
            .as_ref()
            .map(|index| index.windows_for_x(surface_x, surface_width));
        let source_text_len = source.inner.document.line_text_len(source_line);
        let windows = windows.unwrap_or_else(|| {
            vec![HorizontalWindow {
                source: 0..source_text_len,
                x: 0.0,
                width: 0.0,
            }]
        });
        let mut displays = Vec::with_capacity(windows.len());
        for window in windows {
            let key = TextAreaLineWindowKey::new(
                line_key.clone(),
                window.source.start,
                window.source.end,
            );
            let cached = self.text_area_line_displays.get(&key)?;
            self.diagnostics.text_area_line_cache_hits += 1;
            #[cfg(test)]
            {
                self.interaction_stats.text_area_frame_cache_hits += 1;
            }
            displays.push(TextAreaLineDisplay::from_cached(
                source,
                source_line,
                cached,
                surface_x,
                surface_width,
                scroll_x,
                true,
            ));
        }
        Some(displays)
    }

    pub(super) fn text_area_display_segments(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: area::Logical,
        state: &ViewState,
        pinned_line: Option<usize>,
    ) -> Vec<TextAreaDisplaySegment> {
        let estimated_line_height = text_area_estimated_line_height(style);
        let line_count = source.logical_line_count().max(1);
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
        self.diagnostics.text_area_height_index_queries += 2;
        let first_visible = height_index.line_at_y(scroll_y);
        let visible_lines = height_index.visible_line_count(scroll_y, viewport.height());
        let visible_line_end = first_visible
            .saturating_add(visible_lines)
            .saturating_add(1)
            .min(line_count);
        let overscan = TEXT_AREA_FRAME_MIN_OVERSCAN_LINES;
        let source_line_start = first_visible
            .saturating_sub(overscan)
            .min(line_count.saturating_sub(1));
        let source_line_end = first_visible
            .saturating_add(visible_lines)
            .saturating_add(overscan)
            .saturating_add(1)
            .min(line_count)
            .min(source_line_start.saturating_add(TEXT_AREA_FRAME_MAX_LOGICAL_LINES))
            .max(source_line_start + 1)
            .min(line_count.max(1));

        let mut y = height_index.line_top(source_line_start) - scroll_y;
        let mut segments = Vec::with_capacity(source_line_end.saturating_sub(source_line_start));
        let mut realized_lines = 0_usize;
        let shape_overscan = state.caret_visibility_pending()
            || source.has_non_empty_selection_for_state(area_model.state())
            || !committed;
        for line in source_line_start..source_line_end {
            let visible = line >= first_visible && line < visible_line_end;
            let displays = if visible || shape_overscan {
                Some(self.text_area_line_displays_at(
                    area_model, source, committed, style, viewport, state, line,
                ))
            } else {
                self.cached_text_area_line_displays(
                    area_model, source, committed, style, viewport, state, line,
                )
            };
            let segment_y = y;
            let display_height = displays
                .as_ref()
                .and_then(|displays| {
                    displays
                        .iter()
                        .map(|display| display.height.max(1.0))
                        .max_by(f32::total_cmp)
                })
                .unwrap_or_else(|| height_index.line_height(line));
            if let Some(displays) = displays {
                realized_lines += 1;
                let delta = height_index.update_line(source, line, display_height);
                if delta.abs() > f32::EPSILON {
                    self.diagnostics.text_area_height_index_updates += 1;
                    self.diagnostics.text_area_height_index_refined_pixels +=
                        delta.abs().ceil() as usize;
                }
                segments.extend(displays.into_iter().map(|display| TextAreaDisplaySegment {
                    display,
                    y: segment_y,
                }));
            }
            y += display_height;
        }
        if let Some(line) = pinned_line
            .map(|line| line.min(line_count.saturating_sub(1)))
            .filter(|line| *line < source_line_start || *line >= source_line_end)
        {
            let pinned_y = height_index.line_top(line) - scroll_y;
            let displays = self.text_area_line_displays_at(
                area_model, source, committed, style, viewport, state, line,
            );
            let display_height = displays
                .iter()
                .map(|display| display.height.max(1.0))
                .max_by(f32::total_cmp)
                .unwrap_or(estimated_line_height);
            let delta = height_index.update_line(source, line, display_height);
            if delta.abs() > f32::EPSILON {
                self.diagnostics.text_area_height_index_updates += 1;
                self.diagnostics.text_area_height_index_refined_pixels +=
                    delta.abs().ceil() as usize;
            }
            realized_lines += 1;
            segments.extend(displays.into_iter().map(|display| TextAreaDisplaySegment {
                display,
                y: pinned_y,
            }));
        }
        self.diagnostics.text_area_visible_logical_lines += visible_lines;
        self.diagnostics.text_area_layout_segments += segments.len();
        self.diagnostics.text_area_overscan_segments +=
            realized_lines.saturating_sub(visible_lines);

        if committed {
            self.text_area_height_indices.put(height_key, height_index);
        }

        segments
    }
}

fn prepare_text_area_line_display(
    font_system: &mut glyphon::FontSystem,
    area_model: &Area,
    source: &Buffer,
    style: Style,
    viewport: area::Logical,
    source_line: usize,
) -> CachedTextAreaLineDisplay {
    let font_size = style.size().max(1.0);
    let metrics = glyphon::Metrics::relative(font_size, 1.25);
    let attrs = system::attrs_for_style(style);
    let text = source.text_for_line_range(source_line, source_line + 1);
    let mut buffer = glyphon::Buffer::new_empty(metrics);
    buffer.set_wrap(font_system, glyph_wrap(area_model.wrap()));
    buffer.set_size(
        font_system,
        match area_model.wrap() {
            AreaWrap::None => None,
            AreaWrap::WordOrGlyph => Some(viewport.width().max(0.0)),
        },
        None,
    );
    let shaping = text_area_shaping_for_text(style, &text);
    set_cosmic_buffer_text(&mut buffer, &text, glyphon::AttrsList::new(&attrs), shaping);
    buffer.shape_until_scroll(font_system, false);
    let content = buffer_content_area(&buffer);
    let glyph_count = buffer_glyph_count(&buffer);
    CachedTextAreaLineDisplay {
        buffer: Rc::new(RefCell::new(buffer)),
        height: content.height(),
        logical_width: f64::from(content.width()),
        source_byte_start: 0,
        source_text_len: text.len(),
        source_x: 0.0,
        glyph_count,
        resident_bytes: buffer_resident_bytes(text.len(), glyph_count),
    }
}

fn prepare_text_area_line_window(
    font_system: &mut glyphon::FontSystem,
    area_model: &Area,
    source: &Buffer,
    style: Style,
    source_line: usize,
    index: &HorizontalLineIndex,
    window: &HorizontalWindow,
) -> CachedTextAreaLineDisplay {
    debug_assert_eq!(area_model.wrap(), AreaWrap::None);
    let font_size = style.size().max(1.0);
    let metrics = glyphon::Metrics::relative(font_size, 1.25);
    let attrs = system::attrs_for_style(style);
    let line_start = source.inner.document.line_start(source_line);
    let text = source
        .inner
        .document
        .text_for_range(line_start + window.source.start..line_start + window.source.end);
    let mut buffer = glyphon::Buffer::new_empty(metrics);
    buffer.set_wrap(font_system, glyphon::Wrap::None);
    buffer.set_size(font_system, None, None);
    let shaping = text_area_shaping_for_text(style, &text);
    set_cosmic_buffer_text(&mut buffer, &text, glyphon::AttrsList::new(&attrs), shaping);
    buffer.shape_until_scroll(font_system, false);
    let content = buffer_content_area(&buffer);
    let glyph_count = buffer_glyph_count(&buffer);
    CachedTextAreaLineDisplay {
        buffer: Rc::new(RefCell::new(buffer)),
        height: index.height().max(content.height()),
        logical_width: index.width(),
        source_byte_start: window.source.start,
        source_text_len: text.len(),
        source_x: window.x,
        glyph_count,
        resident_bytes: buffer_resident_bytes(text.len(), glyph_count),
    }
}

fn prepare_text_area_exact_horizontal_band(
    font_system: &mut glyphon::FontSystem,
    style: Style,
    text: &str,
) -> Option<HorizontalLineIndex> {
    let metrics = glyphon::Metrics::relative(style.size().max(1.0), 1.25);
    let attrs = system::attrs_for_style(style);
    let mut buffer = glyphon::Buffer::new_empty(metrics);
    buffer.set_wrap(font_system, glyphon::Wrap::None);
    buffer.set_size(font_system, None, None);
    let shaping = text_area_shaping_for_text(style, text);
    set_cosmic_buffer_text(&mut buffer, text, glyphon::AttrsList::new(&attrs), shaping);
    buffer.shape_until_scroll(font_system, false);
    HorizontalLineIndex::from_ascii_fragment_buffer(text, &buffer)
}

fn buffer_glyph_count(buffer: &glyphon::Buffer) -> usize {
    buffer.layout_runs().map(|run| run.glyphs.len()).sum()
}

fn buffer_resident_bytes(source_bytes: usize, glyph_count: usize) -> usize {
    source_bytes.saturating_add(glyph_count.saturating_mul(
        std::mem::size_of::<glyphon::cosmic_text::LayoutGlyph>()
            + std::mem::size_of::<glyphon::cosmic_text::ShapeGlyph>(),
    ))
}
