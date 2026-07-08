use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use super::super::super::{
    buffer::Buffer,
    document::Style,
    edit::{Area, AreaWrap, ViewState},
};
use super::super::{
    constants::TEXT_AREA_RENDER_HORIZONTAL_OVERSCAN,
    content::text_area_estimated_line_height,
    engine::Engine,
    glyph::{glyph_wrap, set_cosmic_buffer_text, text_area_shaping_for_text},
    height::{TextAreaHeightIndex, TextAreaHeightKey},
    output::TextAreaSurface,
    system, text_area,
    text_area::{
        CachedRenderBuffer as CachedTextAreaRenderBuffer, RenderAnchor as TextAreaRenderAnchor,
        RenderBufferKey as TextAreaRenderBufferKey,
    },
};
use crate::paint;

impl Engine {
    pub(super) fn text_area_render_surfaces(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: paint::area::Logical,
        state: &ViewState,
    ) -> Vec<TextAreaSurface> {
        let total_started = Instant::now();
        self.diagnostics.text_area_render_surface_calls += 1;

        let anchor_started = Instant::now();
        let Some(anchor) =
            self.text_area_render_anchor(area_model, source, committed, style, viewport, state)
        else {
            self.diagnostics.text_area_render_surface_anchor_us += elapsed_micros(anchor_started);
            self.diagnostics.text_area_render_surface_total_us += elapsed_micros(total_started);
            return Vec::new();
        };
        let anchor_us = elapsed_micros(anchor_started);
        self.diagnostics.text_area_render_surface_anchor_us += anchor_us;

        let font_size = style.size().max(1.0);
        let metrics = glyphon::Metrics::relative(font_size, 1.25);
        let surface_height = anchor.height.max(viewport.height() - anchor.y).max(1.0);
        let surface_width = viewport.width().max(1.0)
            + state.scroll_x().max(0.0)
            + TEXT_AREA_RENDER_HORIZONTAL_OVERSCAN;
        let layout_width = match area_model.wrap() {
            AreaWrap::None => None,
            AreaWrap::WordOrGlyph => Some(viewport.width().max(0.0)),
        };
        let key_width = layout_width.unwrap_or(0.0);
        let key = TextAreaRenderBufferKey::new(
            area_model,
            source,
            style,
            key_width,
            anchor.source_line,
            anchor.source_line_end,
        );
        let source_lines = anchor.source_line_end.saturating_sub(anchor.source_line);
        self.diagnostics.text_area_render_surface_source_lines += source_lines;

        let mut cache_hit = false;
        let mut text_us = 0;
        let mut buffer_us = 0;
        let mut attrs_us = 0;
        let mut size_us = 0;
        let mut shape_us = 0;
        let buffer = if committed
            && let Some(key) = key.as_ref()
            && let Some(cached) = self.text_area_render_buffers.get(key)
        {
            cache_hit = true;
            self.diagnostics.text_area_render_surface_cache_hits += 1;
            cached.buffer.clone()
        } else {
            self.diagnostics.text_area_render_surface_cache_misses += 1;
            let attrs = system::attrs_for_style(style);

            let text_started = Instant::now();
            let text = source.text_for_line_range(anchor.source_line, anchor.source_line_end);
            text_us = elapsed_micros(text_started);
            self.diagnostics.text_area_render_surface_text_us += text_us;

            let buffer_started = Instant::now();
            let mut buffer = glyphon::Buffer::new_empty(metrics);
            buffer_us = elapsed_micros(buffer_started);

            let size_started = Instant::now();
            buffer.set_wrap(&mut self.font_system, glyph_wrap(area_model.wrap()));
            buffer.set_size(&mut self.font_system, layout_width, None);
            size_us = elapsed_micros(size_started);
            self.diagnostics.text_area_render_surface_size_us += size_us;

            let attrs_started = Instant::now();
            let attrs = glyphon::AttrsList::new(&attrs);
            attrs_us = elapsed_micros(attrs_started);
            self.diagnostics.text_area_render_surface_attrs_us += attrs_us;

            let buffer_started = Instant::now();
            let shaping = text_area_shaping_for_text(style, &text);
            set_cosmic_buffer_text(&mut buffer, &text, attrs, shaping);
            buffer_us += elapsed_micros(buffer_started);
            self.diagnostics.text_area_render_surface_buffer_us += buffer_us;

            let shape_started = Instant::now();
            buffer.shape_until_scroll(&mut self.font_system, false);
            shape_us = elapsed_micros(shape_started);
            self.diagnostics.text_area_render_surface_shape_us += shape_us;

            let buffer = Rc::new(RefCell::new(buffer));
            if committed && let Some(key) = key {
                self.text_area_render_buffers.put(
                    key,
                    CachedTextAreaRenderBuffer {
                        buffer: buffer.clone(),
                    },
                );
            }
            buffer
        };
        let metadata_started = Instant::now();
        let (source_start, source_text_len) = {
            let inner = &source.inner;
            let start = inner.document.line_start(anchor.source_line);
            let end = inner
                .document
                .line_start(anchor.source_line_end.min(source.logical_line_count()));
            (start, end.saturating_sub(start))
        };
        let metadata_us = elapsed_micros(metadata_started);
        self.diagnostics.text_area_render_surface_metadata_us += metadata_us;
        self.diagnostics.text_area_render_surface_source_bytes += source_text_len;
        let total_us = elapsed_micros(total_started);
        self.diagnostics.text_area_render_surface_total_us += total_us;

        if std::env::var_os("WGPU_L3_SCROLL_TRACE").is_some() {
            eprintln!(
                concat!(
                    "[wgpu_l3 text-surface] ",
                    "lines={}..{} count={} bytes={} cache_hit={} ",
                    "viewport={:.1}x{:.1} scroll={:.1},{:.1} surface={:.1}x{:.1} ",
                    "anchor={}us text={}us buffer={}us attrs={}us size={}us shape={}us meta={}us total={}us"
                ),
                anchor.source_line,
                anchor.source_line_end,
                source_lines,
                source_text_len,
                cache_hit,
                viewport.width(),
                viewport.height(),
                state.scroll_x(),
                state.scroll_y(),
                surface_width,
                surface_height,
                anchor_us,
                text_us,
                buffer_us,
                attrs_us,
                size_us,
                shape_us,
                metadata_us,
                total_us,
            );
        }

        vec![TextAreaSurface {
            x: -state.scroll_x(),
            y: anchor.y,
            width: surface_width,
            height: surface_height,
            source_line: anchor.source_line,
            source_line_id: source
                .line_layout_identity(anchor.source_line)
                .map(|identity| identity.id),
            source_start,
            source_text_len,
            buffer,
            default_color: style.color(),
        }]
    }

    fn text_area_render_anchor(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: paint::area::Logical,
        state: &ViewState,
    ) -> Option<TextAreaRenderAnchor> {
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
        let visible_line = height_index.line_at_y(scroll_y);
        let visible_lines = height_index.visible_line_count(scroll_y, viewport.height());
        let visible_line_end = visible_line.saturating_add(visible_lines).min(line_count);
        let window = text_area::render_line_window(visible_line, visible_line_end, line_count);
        let source_line = window.start;
        let source_line_end = window.end;
        let y = height_index.line_top(source_line) - scroll_y;
        let height =
            (height_index.line_top(source_line_end) - height_index.line_top(source_line)).max(1.0);

        if committed {
            self.text_area_height_indices.put(height_key, height_index);
        }

        Some(TextAreaRenderAnchor {
            source_line,
            source_line_end,
            y,
            height,
        })
    }

    pub(super) fn record_text_area_render_window(
        &mut self,
        area_model: &Area,
        source: &Buffer,
        committed: bool,
        style: Style,
        viewport: paint::area::Logical,
        state: &ViewState,
    ) {
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
        let visible_lines = height_index.visible_line_count(scroll_y, viewport.height());
        self.diagnostics.text_area_visible_logical_lines += visible_lines;

        if committed {
            self.text_area_height_indices.put(height_key, height_index);
        }
    }
}

fn elapsed_micros(start: Instant) -> u128 {
    start.elapsed().as_micros()
}
