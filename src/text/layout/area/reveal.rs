use std::time::Instant;

use super::super::super::{
    buffer::Cursor,
    document::Style,
    edit::{Area, PreeditProjection, ScrollAnchor, ViewState},
};
use super::super::{
    caret::{Caret, CaretLayout, ensure_visible_from_layout as ensure_caret_visible_from_layout},
    content::text_area_estimated_line_height,
    engine::Engine,
    glyph::cursor_position,
    height::{TextAreaHeightIndex, TextAreaHeightKey},
    output::TextFieldLayout,
    text_area::DisplaySegment as TextAreaDisplaySegment,
};
use crate::paint;

impl Engine {
    pub fn text_area_caret_for_area(
        &mut self,
        area_model: &Area,
        style: Style,
        area: paint::area::Logical,
        state: ViewState,
    ) -> Option<Caret> {
        self.text_area_paint_layout_for_area_at(area_model, style, area, state, Instant::now())
            .into_interaction_parts()
            .0
            .caret()
    }

    pub fn ensure_caret_visible_for_area(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: paint::area::Logical,
        state: ViewState,
        observed_layout: Option<&TextFieldLayout>,
    ) -> ViewState {
        if state.reveal_intent().should_ensure_caret_visible()
            && let Some(layout) = observed_layout
            && let Some(caret_layout) = layout.caret_layout()
            && let Some(next) = ensure_caret_visible_from_layout(
                state.clone(),
                viewport,
                caret_layout,
                Some(layout.content_area()),
            )
        {
            return next;
        }

        let projection = PreeditProjection::new(area_model.buffer(), area_model.state(), &state);
        let source = &projection.buffer;
        let committed = !projection.has_preedit();
        let source_cursor = projection.cursor();

        if state.reveal_intent().should_ensure_caret_visible() {
            let segments = self
                .text_area_display_segments(area_model, source, committed, style, viewport, &state);
            if let Some(caret_layout) =
                text_area_caret_layout_from_segments(area_model, &projection, &state, &segments)
                && let Some(next) =
                    ensure_caret_visible_from_layout(state.clone(), viewport, caret_layout, None)
            {
                return next;
            }
        }

        let line_count = source.logical_line_count().max(1);
        let cursor_line = source_cursor.line.min(line_count.saturating_sub(1));
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

        let display = self.text_area_line_display(
            area_model,
            source,
            committed,
            style,
            viewport,
            cursor_line,
        );
        height_index.update_line(source, cursor_line, display.height.max(1.0));
        let caret_line_top = height_index.line_top(cursor_line);
        let content_height = height_index.total_height().max(viewport.height().max(0.0));
        if committed {
            self.text_area_height_indices.put(height_key, height_index);
        }

        let caret_layout = {
            let buffer = display.buffer.borrow();
            cursor_position(
                &buffer,
                Cursor::new_with_affinity(0, source_cursor.index, source_cursor.affinity),
            )
            .map(|(x, y)| {
                CaretLayout::new(Caret::new(
                    x as f32 - state.scroll_x(),
                    caret_line_top + y as f32 - state.scroll_y(),
                    buffer.metrics().line_height.max(1.0),
                ))
            })
        };

        let content_area =
            paint::area::logical(display.width.max(viewport.width()), content_height);
        if let Some(caret_layout) = caret_layout
            && let Some(next) = ensure_caret_visible_from_layout(
                state.clone(),
                viewport,
                caret_layout,
                Some(content_area),
            )
        {
            return next;
        }

        let max_scroll_x = (content_area.width() - viewport.width()).max(0.0);
        let max_scroll_y = (content_area.height() - viewport.height()).max(0.0);
        let scroll_x = state.scroll_x().clamp(0.0, max_scroll_x);
        let scroll_y = state.scroll_y().clamp(0.0, max_scroll_y);
        state.with_scroll(scroll_x, scroll_y)
    }

    pub fn text_area_scroll_y_for_anchor(
        &mut self,
        area_model: &Area,
        style: Style,
        viewport: paint::area::Logical,
        _state: ViewState,
        anchor: ScrollAnchor,
    ) -> Option<f32> {
        let source = area_model.buffer();
        let anchor_position = source.position_for_mark(anchor.mark())?;
        let line_count = source.logical_line_count().max(1);
        let anchor_line = source
            .cursor_for_text_index(anchor_position.index)
            .line
            .min(line_count.saturating_sub(1));
        let estimated_line_height = text_area_estimated_line_height(style);
        let height_key = TextAreaHeightKey::new(area_model, style, viewport.width());
        let mut height_index = self
            .text_area_height_indices
            .pop(&height_key)
            .unwrap_or_else(|| TextAreaHeightIndex::new(line_count, estimated_line_height));
        height_index.sync(source, line_count, estimated_line_height);

        let display =
            self.text_area_line_display(area_model, source, true, style, viewport, anchor_line);
        height_index.update_line(source, anchor_line, display.height.max(1.0));
        let scroll_y = (height_index.line_top(anchor_line) + anchor.offset_y()).max(0.0);
        self.text_area_height_indices.put(height_key, height_index);

        Some(scroll_y)
    }
}

fn text_area_caret_layout_from_segments(
    area_model: &Area,
    projection: &PreeditProjection,
    state: &ViewState,
    segments: &[TextAreaDisplaySegment],
) -> Option<CaretLayout> {
    if !area_model.paints_caret() {
        return None;
    }

    let source_cursor = projection.cursor();
    for segment in segments {
        if source_cursor.line != segment.display.source_line {
            continue;
        }

        let buffer = segment.display.buffer.borrow();
        let cursor = Cursor::new_with_affinity(
            0,
            source_cursor.index.min(segment.display.source_text_len),
            source_cursor.affinity,
        );
        return cursor_position(&buffer, cursor).map(|(x, y)| {
            CaretLayout::new(Caret::new(
                x as f32 - state.scroll_x(),
                segment.y + y as f32,
                buffer.metrics().line_height,
            ))
        });
    }

    None
}
