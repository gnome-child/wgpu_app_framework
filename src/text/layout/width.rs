use std::collections::{BTreeMap, HashMap};

use lru::LruCache;

use super::super::{
    buffer::{Buffer, ContentVersion, LineEditDelta, LineLayoutIdentity},
    document::{Align, Block, Document, Run, Style},
};
use super::{Measure, constants::TEXT_AREA_WIDTH_CACHE_CAPACITY, key::StyleKey, system};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Key {
    content: ContentVersion,
    style: StyleKey,
}

#[derive(Debug, Clone)]
pub(super) enum Value {
    Observed(f64),
    Indexed(Index),
}

#[derive(Debug, Clone)]
pub(super) struct Index {
    line_count: usize,
    line_widths: HashMap<LineLayoutIdentity, u64>,
    width_counts: BTreeMap<u64, usize>,
}

impl Key {
    pub(super) fn new(buffer: &Buffer, style: Style) -> Self {
        Self {
            content: buffer.content_version(),
            style: StyleKey::new(style),
        }
    }

    pub(super) fn has_style(self, other: Self) -> bool {
        self.style == other.style
    }
}

impl Value {
    pub(super) fn observed(width: f64) -> Self {
        Self::Observed(normalize_width(width))
    }

    pub(super) fn width(&self) -> f64 {
        match self {
            Self::Observed(width) => *width,
            Self::Indexed(index) => index.width(),
        }
    }

    pub(super) fn contains(&self, identity: LineLayoutIdentity) -> bool {
        matches!(self, Self::Indexed(index) if index.line_widths.contains_key(&identity))
    }

    pub(super) fn into_index(self) -> Option<Index> {
        match self {
            Self::Observed(_) => None,
            Self::Indexed(index) => Some(index),
        }
    }
}

impl Index {
    fn new(source: &Buffer, mut widths: Vec<f32>) -> Self {
        let line_count = source.logical_line_count().max(1);
        widths.resize(line_count, 0.0);
        widths.truncate(line_count);
        let mut index = Self {
            line_count,
            line_widths: HashMap::with_capacity(line_count),
            width_counts: BTreeMap::new(),
        };
        for (line, width) in widths.into_iter().enumerate() {
            let identity = source
                .line_layout_identity(line)
                .expect("every logical text line must own layout identity");
            index.insert(identity, f64::from(width));
        }
        index
    }

    pub(super) fn apply_line_edit(
        &mut self,
        source: &Buffer,
        delta: &LineEditDelta,
        width: f64,
    ) -> bool {
        if source.logical_line_count().max(1) != self.line_count
            || delta.line >= self.line_count
            || source.line_layout_identity(delta.line) != Some(delta.after)
        {
            return false;
        }
        let Some(old_width) = self.line_widths.remove(&delta.before) else {
            return false;
        };
        self.remove_width(old_width);
        self.insert(delta.after, width);
        true
    }

    fn width(&self) -> f64 {
        self.width_counts
            .last_key_value()
            .map_or(0.0, |(width, _)| f64::from_bits(*width))
    }

    fn insert(&mut self, identity: LineLayoutIdentity, width: f64) {
        let width = width_bits(width);
        self.line_widths.insert(identity, width);
        *self.width_counts.entry(width).or_insert(0) += 1;
    }

    fn remove_width(&mut self, width: u64) {
        let Some(count) = self.width_counts.get_mut(&width) else {
            debug_assert!(false, "line width multiset must contain every indexed line");
            return;
        };
        *count -= 1;
        if *count == 0 {
            self.width_counts.remove(&width);
        }
    }
}

pub(super) fn measure(
    font_system: &mut glyphon::FontSystem,
    source: &Buffer,
    style: Style,
) -> Value {
    let text = source.text();
    let document = document(&text, style);
    let (_, widths) =
        system::measure_document_with_line_widths(font_system, &document, Measure::unbounded());
    Value::Indexed(Index::new(source, widths))
}

pub(super) fn measure_line(
    font_system: &mut glyphon::FontSystem,
    source: &Buffer,
    style: Style,
    line: usize,
) -> (f64, usize) {
    let text = source.text_for_line_range(line, line.saturating_add(1));
    let source_bytes = text.len();
    let document = document(&text, style);
    let (metrics, _) =
        system::measure_document_with_line_widths(font_system, &document, Measure::unbounded());
    (f64::from(metrics.width()), source_bytes)
}

pub(super) fn cache() -> LruCache<Key, Value> {
    LruCache::new(TEXT_AREA_WIDTH_CACHE_CAPACITY)
}

fn document(text: &str, style: Style) -> Document {
    let mut block = Block::new(Align::Start);
    block.push_run(Run::new(text, style));
    Document::from_block(block)
}

fn normalize_width(width: f64) -> f64 {
    if width.is_finite() {
        width.max(0.0)
    } else {
        0.0
    }
}

fn width_bits(width: f64) -> u64 {
    normalize_width(width).to_bits()
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
