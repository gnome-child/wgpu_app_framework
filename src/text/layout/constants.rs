use std::num::NonZeroUsize;

pub(super) const MEASURE_CACHE_CAPACITY: usize = 2048;
pub(in crate::text) const TEXT_AREA_LINE_DISPLAY_CACHE_CAPACITY: NonZeroUsize =
    NonZeroUsize::new(2048).unwrap();
pub(super) const TEXT_AREA_LINE_DISPLAY_CACHE_MAX_RESIDENT_BYTES: usize = 16 * 1024 * 1024;
pub(super) const TEXT_AREA_RENDER_BUFFER_CACHE_CAPACITY: NonZeroUsize =
    NonZeroUsize::new(32).unwrap();
pub(super) const TEXT_AREA_HORIZONTAL_INDEX_CACHE_CAPACITY: NonZeroUsize =
    NonZeroUsize::new(64).unwrap();
pub(super) const TEXT_AREA_HEIGHT_INDEX_CACHE_CAPACITY: NonZeroUsize =
    NonZeroUsize::new(128).unwrap();
pub(super) const TEXT_AREA_WIDTH_CACHE_CAPACITY: NonZeroUsize = NonZeroUsize::new(64).unwrap();
pub(super) const TEXT_AREA_HEIGHT_INDEX_BLOCK_LINES: usize = 128;
pub(in crate::text) const TEXT_AREA_FRAME_MIN_OVERSCAN_LINES: usize = 16;
pub(in crate::text) const TEXT_AREA_RENDER_GUARD_LINES: usize = 12;
pub(super) const TEXT_AREA_RENDER_MAX_WINDOW_LINES: usize = 128;
pub(super) const TEXT_AREA_RENDER_HORIZONTAL_OVERSCAN: f32 = 256.0;
pub(in crate::text) const TEXT_AREA_FRAME_MAX_LOGICAL_LINES: usize = 256;
pub(in crate::text) const TEXT_LAYOUT_VISUAL_LINE_EPSILON: f32 = 0.5;
pub(in crate::text) const TEXT_FIELD_CARET_MARGIN: f32 = 5.0;
