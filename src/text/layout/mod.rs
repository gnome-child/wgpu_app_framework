use std::time::Instant;

use super::buffer::{Buffer, CursorSelection, Position};
use super::document::{Document, Style};
use super::edit;
use super::edit::{Motion, State, Surface, ViewState};
use crate::geometry::{area as geom_area, point};

mod area;
mod caret;
mod constants;
mod content;
mod diagnostics;
mod engine;
mod field;
mod glyph;
mod height;
mod highlight;
mod key;
mod map;
mod measure_cache;
mod output;
pub(crate) mod system;
mod text_area;

pub use caret::{Caret, CaretLayout};
#[cfg(test)]
pub(super) use constants::{TEXT_AREA_FRAME_MAX_LOGICAL_LINES, TEXT_AREA_FRAME_MIN_OVERSCAN_LINES};
#[cfg(test)]
pub(super) use constants::{
    TEXT_AREA_LINE_DISPLAY_CACHE_CAPACITY, TEXT_AREA_RENDER_GUARD_LINES, TEXT_FIELD_CARET_MARGIN,
    TEXT_LAYOUT_VISUAL_LINE_EPSILON,
};
#[cfg(test)]
pub(super) use content::text_area_estimated_line_height;
pub(crate) use content::{
    AreaScrollKey, stable_text_area_content_area, text_area_scroll_base_content_area,
};
pub use diagnostics::Diagnostics;
pub(crate) use diagnostics::HighlightStats;
#[cfg(test)]
pub(super) use diagnostics::TextInteractionStats;
pub use engine::Engine;
#[cfg(test)]
pub(crate) use glyph::{
    buffer_text_len, clamp_cursor_in_buffer, clamp_selection_in_buffer, cosmic_buffer_from_text,
    cursor_for_text_index, cursor_for_text_index_in_buffer, cursor_position,
    fast_selection_bounds_in_buffer, floor_text_index_in_buffer, has_non_empty_selection_in_buffer,
    line_start_offsets_for_buffer, normalized_range_in_buffer, selection_anchor,
    text_index_for_cursor_in_buffer, text_position_for_motion_in_buffer, text_range_for_cursors,
    word_selection_cursors,
};
use glyph::{cosmic_motion_for_text_motion, glyph_cursor, glyph_selection, text_cursor};
pub use highlight::SelectionSpan;
#[cfg(test)]
pub(super) use map::TextLayoutMap;
#[cfg(test)]
pub(super) use map::VisualLineGroup;
#[cfg(test)]
use measure_cache::MeasureCache;
pub use output::{
    Measure, Metrics, TextAreaPaintLayout, TextAreaSurface, TextFieldLayout, TextFieldPaintLayout,
};

impl Engine {
    fn visual_motion_position(
        &mut self,
        buffer: &Buffer,
        state: State,
        motion: Motion,
    ) -> Option<Position> {
        let cosmic_motion = cosmic_motion_for_text_motion(motion)?;
        #[cfg(test)]
        {
            self.interaction_stats.aggregate_buffer_fallbacks += 1;
        }

        let mut prepared = glyph::cosmic_buffer_from_text(&buffer.text());
        prepared.set_wrap(&mut self.font_system, glyphon::Wrap::None);
        prepared.shape_until_scroll(&mut self.font_system, false);
        let mut editor = glyphon::Editor::new(&mut prepared);
        glyphon::Edit::set_cursor(&mut editor, glyph_cursor(buffer.cursor_for_state(state)));
        glyphon::Edit::set_selection(&mut editor, glyph_selection(CursorSelection::None));
        glyphon::Edit::action(
            &mut editor,
            &mut self.font_system,
            glyphon::Action::Motion(cosmic_motion),
        );
        let cursor = text_cursor(glyphon::Edit::cursor(&editor));
        drop(editor);
        Some(glyph::text_position_for_cursor_in_buffer(&prepared, cursor))
    }

    pub fn measure(&mut self, document: &Document, measure: Measure) -> Metrics {
        if document.is_empty() {
            return Metrics::empty();
        }
        if let Some(metrics) = self.cache.get(document, measure) {
            return metrics;
        }
        #[cfg(test)]
        {
            self.uncached_measure_count += 1;
        }
        let metrics = system::measure_document(&mut self.font_system, document, measure);
        self.cache.insert(document, measure, metrics);
        metrics
    }
    fn invalidate_text_area_height_indices_for_impacts(
        &mut self,
        _buffer: &Buffer,
        _impacts: &[edit::transaction::Impact],
    ) {
        // Height indices are keyed by presentation style and reconcile by
        // per-line identity during sync. Text edits change line revisions and
        // line counts, so the next sync drops stale measured entries without a
        // buffer-id scoped invalidation pass.
    }

    pub(crate) fn invalidate_text_area_for_edit(
        &mut self,
        buffer: &Buffer,
        impacts: &[edit::transaction::Impact],
    ) {
        self.invalidate_text_area_height_indices_for_impacts(buffer, impacts);
        self.invalidate_text_area_surfaces_for(buffer);
    }

    pub(crate) fn invalidate_text_area_surfaces_for(&mut self, _buffer: &Buffer) {
        // Display cache keys use per-line layout identity. Retaining entries keeps
        // unrelated lines warm after edits while stale line revisions age out via LRU.
    }
    pub fn text_layout_for_surface_at(
        &mut self,
        surface: &Surface,
        style: Style,
        area: geom_area::Logical,
        state: ViewState,
        now: Instant,
    ) -> TextFieldLayout {
        match surface {
            Surface::Field(field) => {
                self.text_field_layout_for_field_at(field, style, area, state, now)
            }
            Surface::Area(area_model) => {
                self.text_area_paint_layout_for_area_at(area_model, style, area, state, now)
                    .into_interaction_parts()
                    .0
            }
        }
    }
    pub fn text_position_at_for_surface(
        &mut self,
        surface: &Surface,
        style: Style,
        area: geom_area::Logical,
        position: point::Logical,
        state: ViewState,
    ) -> Option<Position> {
        match surface {
            Surface::Field(field) => {
                self.text_field_position_at_for_field(field, style, area, position, state)
            }
            Surface::Area(area_model) => {
                self.text_area_position_at_for_area(area_model, style, area, position, state)
            }
        }
    }

    pub fn ensure_caret_visible_for_surface(
        &mut self,
        surface: &Surface,
        style: Style,
        area: geom_area::Logical,
        state: ViewState,
    ) -> ViewState {
        match surface {
            Surface::Field(field) => self.ensure_caret_visible_for_field(field, style, area, state),
            Surface::Area(area_model) => {
                self.ensure_caret_visible_for_area(area_model, style, area, state, None)
            }
        }
    }

    #[cfg(test)]
    pub fn uncached_measure_count(&self) -> usize {
        self.uncached_measure_count
    }
    #[cfg(test)]
    pub fn cache_len(&self) -> usize {
        self.cache.len()
    }
    #[cfg(test)]
    pub(super) fn with_cache_capacity(capacity: usize) -> Self {
        Self {
            cache: MeasureCache::new(capacity),
            ..Self::new()
        }
    }
    #[cfg(test)]
    pub(super) fn reset_highlight_stats(&mut self) {
        self.highlight_stats = HighlightStats::default();
    }
    #[cfg(test)]
    pub(super) fn highlight_stats(&self) -> HighlightStats {
        self.highlight_stats
    }
    #[cfg(test)]
    pub(super) fn reset_interaction_stats(&mut self) {
        self.interaction_stats = TextInteractionStats::default();
    }
    #[cfg(test)]
    pub(super) fn interaction_stats(&self) -> TextInteractionStats {
        self.interaction_stats
    }
    pub fn diagnostics(&self) -> Diagnostics {
        self.diagnostics
    }
    pub fn reset_diagnostics(&mut self) {
        self.diagnostics = Diagnostics::default();
        #[cfg(test)]
        {
            self.highlight_stats = HighlightStats::default();
            self.interaction_stats = TextInteractionStats::default();
        }
    }
    fn add_highlight_stats(&mut self, stats: HighlightStats) {
        self.diagnostics.add_highlight_stats(stats);
        #[cfg(test)]
        self.highlight_stats.add(stats);
        #[cfg(not(test))]
        let _ = stats;
    }
}

impl edit::CaretMap for Engine {
    fn position_for_motion(
        &mut self,
        buffer: &Buffer,
        state: State,
        motion: Motion,
    ) -> Option<Position> {
        self.visual_motion_position(buffer, state, motion)
    }
}
