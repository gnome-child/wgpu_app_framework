use lru::LruCache;
use std::rc::Rc;

use super::{
    constants::MEASURE_CACHE_CAPACITY,
    diagnostics::Diagnostics,
    field::{self, CachedFieldSurface, FieldSurfaceKey},
    height::{self, TextAreaHeightIndex, TextAreaHeightKey},
    measure_cache::MeasureCache,
    shaping_cache::ShapingCache,
    system, text_area,
    text_area::{
        CachedLineDisplay as CachedTextAreaLineDisplay, LineDisplayKey as TextAreaLineDisplayKey,
        LineWindowKey as TextAreaLineWindowKey,
    },
    width,
};

#[cfg(test)]
use super::diagnostics::{HighlightStats, TextInteractionStats};

pub struct Engine {
    pub(super) font_system: glyphon::FontSystem,
    pub(super) cache: MeasureCache,
    pub(in crate::text) text_area_line_displays:
        ShapingCache<TextAreaLineWindowKey, CachedTextAreaLineDisplay>,
    pub(super) text_area_horizontal_indices:
        LruCache<TextAreaLineDisplayKey, Rc<super::horizontal::LineIndex>>,
    pub(super) text_area_horizontal_index_resident_bytes: usize,
    pub(super) text_field_surfaces: ShapingCache<FieldSurfaceKey, CachedFieldSurface>,
    pub(super) text_area_height_indices: LruCache<TextAreaHeightKey, TextAreaHeightIndex>,
    pub(super) text_area_widths: LruCache<width::Key, width::Value>,
    pub(super) diagnostics: Diagnostics,
    #[cfg(test)]
    pub(super) highlight_stats: HighlightStats,
    #[cfg(test)]
    pub(super) interaction_stats: TextInteractionStats,
    #[cfg(test)]
    pub(super) uncached_measure_count: usize,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            font_system: system::font_system(),
            cache: MeasureCache::new(MEASURE_CACHE_CAPACITY),
            text_area_line_displays: text_area::line_display_cache(),
            text_area_horizontal_indices: text_area::horizontal_index_cache(),
            text_area_horizontal_index_resident_bytes: 0,
            text_field_surfaces: field::surface_cache(),
            text_area_height_indices: height::cache(),
            text_area_widths: width::cache(),
            diagnostics: Diagnostics::default(),
            #[cfg(test)]
            highlight_stats: HighlightStats::default(),
            #[cfg(test)]
            interaction_stats: TextInteractionStats::default(),
            #[cfg(test)]
            uncached_measure_count: 0,
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
