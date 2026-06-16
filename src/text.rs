use crate::geometry::area;
use crate::{paint, text_system};
use std::collections::{HashMap, VecDeque};

const MEASURE_CACHE_CAPACITY: usize = 2048;

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    runs: Vec<Run>,
    align: Align,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    text: String,
    style: Style,
}

pub struct Engine {
    font_system: glyphon::FontSystem,
    cache: MeasureCache,
    #[cfg(test)]
    uncached_measure_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measure {
    max: Option<area::Logical>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    area: area::Logical,
    line_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    size: f32,
    color: paint::Color,
    weight: Weight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weight {
    Normal,
    Medium,
    Bold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Align {
    Start,
    Center,
    End,
}

#[derive(Debug)]
struct MeasureCache {
    entries: HashMap<MeasureKey, Metrics>,
    order: VecDeque<MeasureKey>,
    capacity: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MeasureKey {
    blocks: Vec<BlockKey>,
    max: Option<BoundsKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BlockKey {
    align: Align,
    runs: Vec<RunKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RunKey {
    text: String,
    size: u32,
    weight: Weight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BoundsKey {
    width: u32,
    height: u32,
}

impl Document {
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            blocks: vec![Block::plain(text)],
        }
    }

    pub fn from_block(block: Block) -> Self {
        Self {
            blocks: vec![block],
        }
    }

    pub fn push_block(&mut self, block: Block) {
        self.blocks.push(block);
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn with_color(mut self, color: paint::Color) -> Self {
        for block in &mut self.blocks {
            for run in &mut block.runs {
                run.style = run.style.with_color(color);
            }
        }

        self
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.iter().all(Block::is_empty)
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for Document {
    fn from(value: String) -> Self {
        Self::plain(value)
    }
}

impl From<&str> for Document {
    fn from(value: &str) -> Self {
        Self::plain(value)
    }
}

impl Engine {
    pub fn new() -> Self {
        Self {
            font_system: text_system::font_system(),
            cache: MeasureCache::new(MEASURE_CACHE_CAPACITY),
            #[cfg(test)]
            uncached_measure_count: 0,
        }
    }

    pub fn measure(&mut self, document: &Document, measure: Measure) -> Metrics {
        if document.is_empty() {
            return Metrics::empty();
        }

        let key = MeasureKey::new(document, measure);
        if let Some(metrics) = self.cache.get(&key) {
            return metrics;
        }

        let metrics = self.measure_uncached(document, measure);
        self.cache.insert(key, metrics);
        metrics
    }

    fn measure_uncached(&mut self, document: &Document, measure: Measure) -> Metrics {
        #[cfg(test)]
        {
            self.uncached_measure_count += 1;
        }

        text_system::measure_document(&mut self.font_system, document, measure)
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
    fn with_cache_capacity(capacity: usize) -> Self {
        Self {
            font_system: text_system::font_system(),
            cache: MeasureCache::new(capacity),
            uncached_measure_count: 0,
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Measure {
    pub fn unbounded() -> Self {
        Self { max: None }
    }

    pub fn bounded(max: area::Logical) -> Self {
        Self {
            max: Some(area::logical(max.width().max(0.0), max.height().max(0.0))),
        }
    }

    pub fn max(self) -> Option<area::Logical> {
        self.max
    }
}

impl Metrics {
    pub fn new(area: area::Logical, line_count: usize) -> Self {
        Self { area, line_count }
    }

    pub fn empty() -> Self {
        Self::new(area::logical(0.0, 0.0), 0)
    }

    pub fn area(self) -> area::Logical {
        self.area
    }

    pub fn width(self) -> f32 {
        self.area.width()
    }

    pub fn height(self) -> f32 {
        self.area.height()
    }

    pub fn line_count(self) -> usize {
        self.line_count
    }
}

impl Block {
    pub fn new(align: Align) -> Self {
        Self {
            runs: Vec::new(),
            align,
        }
    }

    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            runs: vec![Run::new(text, Style::default())],
            align: Align::Start,
        }
    }

    pub fn push_run(&mut self, run: Run) {
        self.runs.push(run);
    }

    pub fn runs(&self) -> &[Run] {
        &self.runs
    }

    pub fn align(&self) -> Align {
        self.align
    }

    pub fn set_align(&mut self, align: Align) {
        self.align = align;
    }

    pub fn with_align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.runs.iter().all(Run::is_empty)
    }
}

impl Run {
    pub fn new(text: impl Into<String>, style: Style) -> Self {
        Self {
            text: text.into(),
            style,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn style(&self) -> Style {
        self.style
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

impl Style {
    pub fn size(self) -> f32 {
        self.size
    }

    pub fn color(self) -> paint::Color {
        self.color
    }

    pub fn weight(self) -> Weight {
        self.weight
    }

    pub fn with_color(mut self, color: paint::Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn with_weight(mut self, weight: Weight) -> Self {
        self.weight = weight;
        self
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            size: 16.0,
            color: paint::Color::rgb(0.92, 0.94, 0.98),
            weight: Weight::Normal,
        }
    }
}

impl MeasureCache {
    fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            capacity,
        }
    }

    fn get(&self, key: &MeasureKey) -> Option<Metrics> {
        self.entries.get(key).copied()
    }

    fn insert(&mut self, key: MeasureKey, metrics: Metrics) {
        if self.capacity == 0 {
            return;
        }

        if let Some(entry) = self.entries.get_mut(&key) {
            *entry = metrics;
            return;
        }

        while self.entries.len() >= self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                self.entries.remove(&oldest);
            } else {
                break;
            }
        }

        self.order.push_back(key.clone());
        self.entries.insert(key, metrics);
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

impl MeasureKey {
    fn new(document: &Document, measure: Measure) -> Self {
        Self {
            blocks: document
                .blocks()
                .iter()
                .filter(|block| !block.is_empty())
                .map(BlockKey::new)
                .collect(),
            max: measure.max().map(BoundsKey::new),
        }
    }
}

impl BlockKey {
    fn new(block: &Block) -> Self {
        Self {
            align: block.align(),
            runs: block.runs().iter().map(RunKey::new).collect(),
        }
    }
}

impl RunKey {
    fn new(run: &Run) -> Self {
        let style = run.style();

        Self {
            text: run.text().to_owned(),
            size: finite_bits(style.size().max(1.0)),
            weight: style.weight(),
        }
    }
}

impl BoundsKey {
    fn new(bounds: area::Logical) -> Self {
        Self {
            width: finite_bits(bounds.width().max(0.0)),
            height: finite_bits(bounds.height().max(0.0)),
        }
    }
}

fn finite_bits(value: f32) -> u32 {
    if value.is_finite() {
        value.to_bits()
    } else if value.is_sign_negative() {
        0.0_f32.to_bits()
    } else {
        f32::INFINITY.to_bits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_stores_block_run_and_style_data() {
        let style = Style::default()
            .with_size(18.0)
            .with_color(paint::Color::RED)
            .with_weight(Weight::Bold);
        let mut block = Block::new(Align::Center);
        block.push_run(Run::new("Label", style));
        let document = Document::from_block(block);

        assert_eq!(document.blocks().len(), 1);
        assert_eq!(document.blocks()[0].align(), Align::Center);
        assert_eq!(document.blocks()[0].runs()[0].text(), "Label");
        assert_eq!(document.blocks()[0].runs()[0].style(), style);
    }

    #[test]
    fn empty_document_is_empty() {
        assert!(Document::new().is_empty());
        assert!(Document::plain("").is_empty());
        assert!(!Document::plain("x").is_empty());
    }

    #[test]
    fn document_color_can_be_overridden() {
        let document = Document::plain("Label").with_color(paint::Color::BLACK);

        assert_eq!(
            document.blocks()[0].runs()[0].style().color(),
            paint::Color::BLACK
        );
    }

    #[test]
    fn engine_returns_non_zero_metrics_for_non_empty_text() {
        let mut engine = Engine::new();
        let metrics = engine.measure(&Document::plain("Label"), Measure::unbounded());

        assert!(metrics.width() > 0.0);
        assert!(metrics.height() > 0.0);
        assert_eq!(metrics.line_count(), 1);
    }

    #[test]
    fn longer_text_measures_wider_than_shorter_text() {
        let mut engine = Engine::new();
        let short = engine.measure(&Document::plain("Run"), Measure::unbounded());
        let long = engine.measure(&Document::plain("Run workspace task"), Measure::unbounded());

        assert!(long.width() > short.width());
        assert!(long.height() >= short.height());
    }

    #[test]
    fn larger_font_measures_taller_than_smaller_font() {
        let mut engine = Engine::new();
        let small = Document::from_block({
            let mut block = Block::new(Align::Start);
            block.push_run(Run::new("Label", Style::default().with_size(10.0)));
            block
        });
        let large = Document::from_block({
            let mut block = Block::new(Align::Start);
            block.push_run(Run::new("Label", Style::default().with_size(24.0)));
            block
        });

        let small = engine.measure(&small, Measure::unbounded());
        let large = engine.measure(&large, Measure::unbounded());

        assert!(large.height() > small.height());
    }

    #[test]
    fn repeated_measurement_reuses_cached_metrics() {
        let mut engine = Engine::new();
        let document = Document::plain("Cached Label");

        let first = engine.measure(&document, Measure::unbounded());
        let second = engine.measure(&document, Measure::unbounded());

        assert_eq!(first, second);
        assert_eq!(engine.uncached_measure_count(), 1);
        assert_eq!(engine.cache_len(), 1);
    }

    #[test]
    fn color_only_changes_reuse_cached_metrics() {
        let mut engine = Engine::new();
        let red = Document::plain("Cached Label").with_color(paint::Color::RED);
        let black = Document::plain("Cached Label").with_color(paint::Color::BLACK);

        let red = engine.measure(&red, Measure::unbounded());
        let black = engine.measure(&black, Measure::unbounded());

        assert_eq!(red, black);
        assert_eq!(engine.uncached_measure_count(), 1);
    }

    #[test]
    fn shaping_relevant_document_and_bounds_changes_use_distinct_cache_keys() {
        let mut engine = Engine::new();
        let base = styled_document("Cached Label", Align::Start, 16.0, Weight::Normal);
        let text = styled_document("Different Label", Align::Start, 16.0, Weight::Normal);
        let size = styled_document("Cached Label", Align::Start, 20.0, Weight::Normal);
        let weight = styled_document("Cached Label", Align::Start, 16.0, Weight::Bold);
        let align = styled_document("Cached Label", Align::End, 16.0, Weight::Normal);

        engine.measure(&base, Measure::unbounded());
        engine.measure(&text, Measure::unbounded());
        engine.measure(&size, Measure::unbounded());
        engine.measure(&weight, Measure::unbounded());
        engine.measure(&align, Measure::unbounded());
        engine.measure(&base, Measure::bounded(area::logical(40.0, 100.0)));

        assert_eq!(engine.uncached_measure_count(), 6);
        assert_eq!(engine.cache_len(), 6);
    }

    #[test]
    fn bounded_fifo_cache_evicts_oldest_entries() {
        let mut engine = Engine::with_cache_capacity(2);
        let first = Document::plain("First");
        let second = Document::plain("Second");
        let third = Document::plain("Third");

        engine.measure(&first, Measure::unbounded());
        engine.measure(&second, Measure::unbounded());
        engine.measure(&third, Measure::unbounded());
        engine.measure(&first, Measure::unbounded());

        assert_eq!(engine.cache_len(), 2);
        assert_eq!(engine.uncached_measure_count(), 4);
    }

    fn styled_document(text: &str, align: Align, size: f32, weight: Weight) -> Document {
        let mut block = Block::new(align);
        block.push_run(Run::new(
            text,
            Style::default().with_size(size).with_weight(weight),
        ));

        Document::from_block(block)
    }
}
