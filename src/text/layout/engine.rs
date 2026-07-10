use lru::LruCache;

use super::{
    constants::MEASURE_CACHE_CAPACITY,
    diagnostics::Diagnostics,
    field::{self, CachedFieldSurface, FieldSurfaceKey},
    height::{self, TextAreaHeightIndex, TextAreaHeightKey},
    measure_cache::MeasureCache,
    shaping_cache::ShapingCache,
    system, text_area,
    text_area::{
        CachedLineDisplay as CachedTextAreaLineDisplay,
        CachedRenderBuffer as CachedTextAreaRenderBuffer, LineDisplayKey as TextAreaLineDisplayKey,
        RenderBufferKey as TextAreaRenderBufferKey,
    },
};

#[cfg(test)]
use super::diagnostics::{HighlightStats, TextInteractionStats};

pub struct Engine {
    pub(super) font_system: glyphon::FontSystem,
    pub(super) cache: MeasureCache,
    pub(in crate::text) text_area_line_displays:
        ShapingCache<TextAreaLineDisplayKey, CachedTextAreaLineDisplay>,
    pub(super) text_area_render_buffers:
        LruCache<TextAreaRenderBufferKey, CachedTextAreaRenderBuffer>,
    pub(super) text_field_surfaces: ShapingCache<FieldSurfaceKey, CachedFieldSurface>,
    pub(super) text_area_height_indices: LruCache<TextAreaHeightKey, TextAreaHeightIndex>,
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
            text_area_render_buffers: text_area::render_buffer_cache(),
            text_field_surfaces: field::surface_cache(),
            text_area_height_indices: height::cache(),
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
