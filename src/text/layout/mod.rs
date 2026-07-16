use crate::geometry::point;
use std::time::Instant;

use super::buffer::{Buffer, CursorSelection, Position};
use super::document::{Document, Style};
use super::{
    Surface,
    selection::{self, Motion, State},
    view::ViewState,
};

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
mod horizontal;
mod inline;
mod key;
mod map;
mod measure_cache;
mod output;
mod overflow;
mod shaping_cache;
mod system;
mod text_area;
mod width;

pub use caret::{Caret, CaretLayout};
#[cfg(test)]
pub(super) use constants::{TEXT_AREA_FRAME_MAX_LOGICAL_LINES, TEXT_AREA_FRAME_MIN_OVERSCAN_LINES};
#[cfg(test)]
pub(super) use constants::{
    TEXT_AREA_LINE_DISPLAY_CACHE_CAPACITY, TEXT_FIELD_CARET_MARGIN, TEXT_LAYOUT_VISUAL_LINE_EPSILON,
};
#[cfg(test)]
pub(super) use content::text_area_estimated_line_height;
pub use diagnostics::Diagnostics;
pub(crate) use diagnostics::HighlightStats;
#[cfg(test)]
pub(super) use diagnostics::TextInteractionStats;
pub use engine::Engine;
#[cfg(test)]
pub(super) use glyph::clamp_cursor_in_buffer;
use glyph::{cosmic_motion_for_text_motion, glyph_cursor, glyph_selection, text_cursor};
pub use highlight::SelectionSpan;
pub(crate) use inline::{InlineCache, InlineStats};
#[cfg(test)]
pub(super) use map::TextLayoutMap;
#[cfg(test)]
pub(super) use map::VisualLineGroup;
#[cfg(test)]
use measure_cache::MeasureCache;
pub(crate) use output::ShapedBuffer;
pub use output::{
    Measure, Metrics, TextAreaPaintLayout, TextAreaSurface, TextFieldLayout, TextFieldPaintLayout,
};
pub(crate) use overflow::OverflowProjection;
#[cfg(test)]
pub(super) use system::align as glyphon_align;
#[cfg(test)]
pub(crate) use system::font_system as glyphon_font_system;
#[cfg(test)]
pub(crate) use system::{
    color as glyphon_color, measure_document as measure_document_with_glyphon,
};

pub(crate) fn surface_area(width: f32, height: f32) -> crate::geometry::area::Logical {
    crate::geometry::area::logical(width, height)
}

pub(crate) fn surface_point(x: f32, y: f32) -> point::Logical {
    point::logical(x, y)
}

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
    #[cfg(test)]
    pub(crate) fn invalidate_text_area_surfaces_for(&mut self, _buffer: &Buffer) {
        // Display cache keys use per-line layout identity. Retaining entries keeps
        // unrelated lines warm after edits while stale line revisions age out via LRU.
    }
    pub fn text_layout_for_surface_at(
        &mut self,
        surface: &Surface,
        style: Style,
        area: crate::geometry::area::Logical,
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
        area: crate::geometry::area::Logical,
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
        area: crate::geometry::area::Logical,
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
    pub(super) fn reset_for_test(&mut self) {
        self.cache = MeasureCache::new(constants::MEASURE_CACHE_CAPACITY);
        self.text_area_line_displays = text_area::line_display_cache();
        self.text_area_horizontal_indices = text_area::horizontal_index_cache();
        self.text_area_render_buffers = text_area::render_buffer_cache();
        self.text_field_surfaces = field::surface_cache();
        self.text_area_height_indices = height::cache();
        self.text_area_widths = width::cache();
        self.diagnostics = Diagnostics::default();
        self.highlight_stats = HighlightStats::default();
        self.interaction_stats = TextInteractionStats::default();
        self.uncached_measure_count = 0;
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

impl selection::CaretMap for Engine {
    fn position_for_motion(
        &mut self,
        buffer: &Buffer,
        state: State,
        motion: Motion,
    ) -> Option<Position> {
        self.visual_motion_position(buffer, state, motion)
    }
}
