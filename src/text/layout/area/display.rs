use std::cell::RefCell;
use std::rc::Rc;

use super::super::super::{
    buffer::Buffer,
    document::Style,
    edit::{Area, AreaWrap, ViewState},
};
use super::super::{
    constants::{TEXT_AREA_FRAME_MAX_LOGICAL_LINES, TEXT_AREA_FRAME_MIN_OVERSCAN_LINES},
    content::{buffer_content_area, text_area_estimated_line_height},
    engine::Engine,
    glyph::{glyph_wrap, set_cosmic_buffer_text, text_area_shaping_for_text},
    height::{TextAreaHeightIndex, TextAreaHeightKey},
    shaping_cache::Shaped,
    system,
    text_area::{
        CachedLineDisplay as CachedTextAreaLineDisplay, DisplaySegment as TextAreaDisplaySegment,
        LineDisplay as TextAreaLineDisplay, LineDisplayKey as TextAreaLineDisplayKey,
    },
};
use crate::paint;

impl Engine {
    pub(super) fn text_area_content_height(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: paint::area::Logical,
    ) -> f32 {
        let line_count = source.logical_line_count().max(1);
        let estimated_line_height = text_area_estimated_line_height(style);
        if committed {
            let key = TextAreaHeightKey::new(area_model, style, viewport.width());
            if let Some(index) = self.text_area_height_indices.get_mut(&key) {
                index.sync(source, line_count, estimated_line_height);
                self.diagnostics.text_area_height_index_hits += 1;
                return index.total_height().max(viewport.height().max(0.0));
            }
            self.diagnostics.text_area_height_index_misses += 1;
        }
        (line_count as f32 * estimated_line_height).max(viewport.height().max(0.0))
    }

    pub(in crate::text) fn text_area_line_display(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: paint::area::Logical,
        source_line: usize,
    ) -> TextAreaLineDisplay {
        let key = TextAreaLineDisplayKey::new(
            area_model,
            source,
            style,
            viewport.width().max(0.0),
            source_line,
        );
        let shaped = if let Some(key) = key {
            self.text_area_line_displays
                .shape(&mut self.font_system, key, committed, |font_system, _| {
                    Some(prepare_text_area_line_display(
                        font_system,
                        area_model,
                        source,
                        style,
                        viewport,
                        source_line,
                    ))
                })
                .expect("text area line shaping should always produce a display")
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

        if shaped.cache_hit {
            self.diagnostics.text_area_line_cache_hits += 1;
            #[cfg(test)]
            {
                self.interaction_stats.text_area_frame_cache_hits += 1;
            }
        } else {
            self.diagnostics.text_area_line_cache_misses += 1;
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
        }
        TextAreaLineDisplay::from_cached(source, source_line, shaped.value)
    }

    fn cached_text_area_line_display(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: paint::area::Logical,
        source_line: usize,
    ) -> Option<TextAreaLineDisplay> {
        if !committed {
            return None;
        }
        let key = TextAreaLineDisplayKey::new(
            area_model,
            source,
            style,
            viewport.width().max(0.0),
            source_line,
        )?;
        let cached = self.text_area_line_displays.get(&key)?;
        self.diagnostics.text_area_line_cache_hits += 1;
        #[cfg(test)]
        {
            self.interaction_stats.text_area_frame_cache_hits += 1;
        }
        Some(TextAreaLineDisplay::from_cached(
            source,
            source_line,
            cached,
        ))
    }

    pub(super) fn text_area_display_segments(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: paint::area::Logical,
        state: &ViewState,
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
        let shape_overscan = state.caret_visibility_pending()
            || source.has_non_empty_selection_for_state(area_model.state())
            || !committed;
        for line in source_line_start..source_line_end {
            let visible = line >= first_visible && line < visible_line_end;
            let display =
                if visible || shape_overscan {
                    Some(self.text_area_line_display(
                        area_model, source, committed, style, viewport, line,
                    ))
                } else {
                    self.cached_text_area_line_display(
                        area_model, source, committed, style, viewport, line,
                    )
                };
            let segment_y = y;
            let display_height = display
                .as_ref()
                .map(|display| display.height.max(1.0))
                .unwrap_or_else(|| height_index.line_height(line));
            if let Some(display) = display {
                height_index.update_line(source, line, display_height);
                segments.push(TextAreaDisplaySegment {
                    display,
                    y: segment_y,
                });
            }
            y += display_height;
        }
        self.diagnostics.text_area_visible_logical_lines += visible_lines;
        self.diagnostics.text_area_layout_segments += segments.len();
        self.diagnostics.text_area_overscan_segments +=
            segments.len().saturating_sub(visible_lines);

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
    viewport: paint::area::Logical,
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
    CachedTextAreaLineDisplay {
        buffer: Rc::new(RefCell::new(buffer)),
        height: content.height(),
        width: content.width(),
    }
}
