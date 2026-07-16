use lru::LruCache;

use super::super::{
    buffer::{Buffer, ContentVersion},
    document::{Align, Block, Document, Run, Style},
};
use super::{Measure, constants::TEXT_AREA_WIDTH_CACHE_CAPACITY, key::StyleKey, system};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Key {
    content: ContentVersion,
    style: StyleKey,
}

impl Key {
    pub(super) fn new(buffer: &Buffer, style: Style) -> Self {
        Self {
            content: buffer.content_version(),
            style: StyleKey::new(style),
        }
    }
}

pub(super) fn measure(font_system: &mut glyphon::FontSystem, source: &Buffer, style: Style) -> f64 {
    let mut block = Block::new(Align::Start);
    block.push_run(Run::new(source.text(), style));
    system::measure_document(
        font_system,
        &Document::from_block(block),
        Measure::unbounded(),
    )
    .width()
    .into()
}

pub(super) fn cache() -> LruCache<Key, f64> {
    LruCache::new(TEXT_AREA_WIDTH_CACHE_CAPACITY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_width_key_survives_clones_but_not_independent_content() {
        let source = Buffer::from_multiline_text("alpha\nbeta");
        let clone = source.clone();
        let independent = Buffer::from_multiline_text("a much wider independent document");
        let style = Style::default();

        assert_ne!(source.id(), clone.id());
        assert_eq!(Key::new(&source, style), Key::new(&clone, style));
        assert_ne!(Key::new(&source, style), Key::new(&independent, style));
    }
}
