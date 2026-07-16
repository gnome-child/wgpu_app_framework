use lru::LruCache;

use super::super::{
    buffer::Buffer,
    document::{Align, Block, Document, Run, Style},
};
use super::{Measure, constants::TEXT_AREA_WIDTH_CACHE_CAPACITY, key::StyleKey, system};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Key {
    buffer: u64,
    revision: u64,
    style: StyleKey,
}

impl Key {
    pub(super) fn new(buffer: &Buffer, style: Style) -> Self {
        Self {
            buffer: buffer.id(),
            revision: buffer.revision(),
            style: StyleKey::new(style),
        }
    }
}

pub(super) fn measure(font_system: &mut glyphon::FontSystem, source: &Buffer, style: Style) -> f32 {
    let mut block = Block::new(Align::Start);
    block.push_run(Run::new(source.text(), style));
    system::measure_document(
        font_system,
        &Document::from_block(block),
        Measure::unbounded(),
    )
    .width()
}

pub(super) fn cache() -> LruCache<Key, f32> {
    LruCache::new(TEXT_AREA_WIDTH_CACHE_CAPACITY)
}
