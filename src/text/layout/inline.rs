use std::cell::RefCell;
use std::rc::Rc;

use glyphon::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping};

use super::super::document::{self, Align, Document, Style, TextDirection, Weight};
use super::shaping_cache::ShapingCache;
use super::system;
use crate::icon;
use crate::text;

const TEXT_CACHE_CAPACITY: usize = 2048;
const ICON_CACHE_CAPACITY: usize = 512;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct InlineStats {
    pub text_cache_hits: usize,
    pub text_cache_misses: usize,
    pub text_shape_calls: usize,
    pub icon_cache_hits: usize,
    pub icon_cache_misses: usize,
    pub icon_shape_calls: usize,
}

pub(crate) struct InlineCache {
    font_system: FontSystem,
    text: ShapingCache<TextKey, CachedText>,
    icons: ShapingCache<IconKey, CachedIcon>,
}

#[derive(Clone)]
pub(crate) struct PreparedText {
    pub buffer: Rc<RefCell<Buffer>>,
    pub default_color: glyphon::Color,
    pub content_height: f32,
    pub stats: InlineStats,
}

#[derive(Clone)]
pub(crate) struct PreparedIcon {
    pub buffer: Rc<RefCell<Buffer>>,
    pub line_height: f32,
    pub stats: InlineStats,
}

#[derive(Clone)]
struct CachedText {
    buffer: Rc<RefCell<Buffer>>,
    content_height: f32,
}

#[derive(Clone)]
struct CachedIcon {
    buffer: Rc<RefCell<Buffer>>,
    line_height: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextKey {
    text: String,
    size: u32,
    weight: Weight,
    direction: TextDirection,
    align: Align,
    wrap: WrapKey,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct IconKey {
    glyph: icon::Glyph,
    size: u32,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum WrapKey {
    None,
    WordOrGlyph,
}

impl InlineCache {
    pub(crate) fn new() -> Self {
        Self {
            font_system: system::font_system(),
            text: ShapingCache::new(TEXT_CACHE_CAPACITY, "inline text"),
            icons: ShapingCache::new(ICON_CACHE_CAPACITY, "inline icon"),
        }
    }

    pub(crate) fn font_system_mut(&mut self) -> &mut FontSystem {
        &mut self.font_system
    }

    pub(crate) fn prepare_text(
        &mut self,
        document: &Document,
        width: f32,
        height: f32,
        wrap: glyphon::Wrap,
    ) -> Option<PreparedText> {
        let Some((key, color)) = TextKey::new(document, width, height, wrap) else {
            let prepared = system::prepare_document_buffer(
                &mut self.font_system,
                document,
                width,
                height,
                wrap,
            )?;
            return Some(PreparedText {
                buffer: Rc::new(RefCell::new(prepared.buffer)),
                default_color: prepared.default_color,
                content_height: prepared.content_height,
                stats: InlineStats {
                    text_shape_calls: 1,
                    ..InlineStats::default()
                },
            });
        };

        let shaped = self
            .text
            .shape(&mut self.font_system, key, true, prepare_cached_text)?;
        let cached = shaped.value;

        Some(PreparedText {
            buffer: cached.buffer,
            default_color: system::color(color),
            content_height: cached.content_height,
            stats: if shaped.cache_hit {
                InlineStats {
                    text_cache_hits: 1,
                    ..InlineStats::default()
                }
            } else {
                InlineStats {
                    text_cache_misses: 1,
                    text_shape_calls: 1,
                    ..InlineStats::default()
                }
            },
        })
    }

    pub(crate) fn prepare_icon(
        &mut self,
        glyph: icon::Glyph,
        size: f32,
        width: f32,
        height: f32,
    ) -> Option<PreparedIcon> {
        let key = IconKey::new(glyph, size, width, height);

        let shaped = self
            .icons
            .shape(&mut self.font_system, key, true, prepare_cached_icon)?;
        let cached = shaped.value;

        Some(PreparedIcon {
            buffer: cached.buffer,
            line_height: cached.line_height,
            stats: if shaped.cache_hit {
                InlineStats {
                    icon_cache_hits: 1,
                    ..InlineStats::default()
                }
            } else {
                InlineStats {
                    icon_cache_misses: 1,
                    icon_shape_calls: 1,
                    ..InlineStats::default()
                }
            },
        })
    }
}

impl Default for InlineCache {
    fn default() -> Self {
        Self::new()
    }
}

impl InlineStats {
    pub(crate) fn add(&mut self, other: Self) {
        self.text_cache_hits += other.text_cache_hits;
        self.text_cache_misses += other.text_cache_misses;
        self.text_shape_calls += other.text_shape_calls;
        self.icon_cache_hits += other.icon_cache_hits;
        self.icon_cache_misses += other.icon_cache_misses;
        self.icon_shape_calls += other.icon_shape_calls;
    }
}

impl TextKey {
    fn new(
        document: &Document,
        width: f32,
        height: f32,
        wrap: glyphon::Wrap,
    ) -> Option<(Self, text::Color)> {
        let block = single_non_empty_block(document)?;
        let run = single_non_empty_run(block)?;
        let style = run.style();
        let text = run.text().to_owned();

        Some((
            Self {
                text,
                size: finite_bits(style.size().max(1.0)),
                weight: style.weight(),
                direction: style.direction(),
                align: block.align(),
                wrap: WrapKey::from_glyphon(wrap),
                width: finite_bits(width.max(0.0)),
                height: finite_bits(height.max(0.0)),
            },
            style.color(),
        ))
    }

    fn style(&self) -> Style {
        Style::default()
            .with_size(f32::from_bits(self.size).max(1.0))
            .with_weight(self.weight)
            .with_direction(self.direction)
    }
}

impl IconKey {
    fn new(glyph: icon::Glyph, size: f32, width: f32, height: f32) -> Self {
        Self {
            glyph,
            size: finite_bits(size.max(1.0)),
            width: finite_bits(width.max(0.0)),
            height: finite_bits(height.max(0.0)),
        }
    }
}

impl WrapKey {
    fn from_glyphon(wrap: glyphon::Wrap) -> Self {
        match wrap {
            glyphon::Wrap::None => Self::None,
            glyphon::Wrap::WordOrGlyph => Self::WordOrGlyph,
            _ => Self::WordOrGlyph,
        }
    }

    fn to_glyphon(self) -> glyphon::Wrap {
        match self {
            Self::None => glyphon::Wrap::None,
            Self::WordOrGlyph => glyphon::Wrap::WordOrGlyph,
        }
    }
}

fn single_non_empty_block(document: &Document) -> Option<&document::Block> {
    let mut blocks = document.blocks().iter().filter(|block| !block.is_empty());
    let block = blocks.next()?;

    blocks.next().is_none().then_some(block)
}

fn single_non_empty_run(block: &document::Block) -> Option<&document::Run> {
    let mut runs = block.runs().iter().filter(|run| !run.is_empty());
    let run = runs.next()?;

    runs.next().is_none().then_some(run)
}

fn prepare_cached_text(font_system: &mut FontSystem, key: &TextKey) -> Option<CachedText> {
    let style = key.style();
    let font_size = style.size().max(1.0);
    let line_height = font_size * 1.25;
    let mut buffer = Buffer::new(font_system, Metrics::relative(font_size, 1.25));
    let attrs = attrs_for_shape(style);
    let align = system::align(key.align, style.direction().resolve_for_text(&key.text));

    buffer.set_size(
        font_system,
        Some(f32::from_bits(key.width).max(0.0)),
        Some(f32::from_bits(key.height).max(0.0)),
    );
    buffer.set_wrap(font_system, key.wrap.to_glyphon());
    buffer.set_rich_text(
        font_system,
        vec![(key.text.as_str(), attrs.clone())],
        &attrs,
        Shaping::Advanced,
        Some(align),
    );
    buffer.shape_until_scroll(font_system, false);

    Some(CachedText {
        content_height: system::prepared_content_height(&buffer, line_height),
        buffer: Rc::new(RefCell::new(buffer)),
    })
}

fn prepare_cached_icon(font_system: &mut FontSystem, key: &IconKey) -> Option<CachedIcon> {
    let character = key.glyph.character()?;
    let font_size = f32::from_bits(key.size).max(1.0);
    let line_height = font_size;
    let buffer_height = f32::from_bits(key.height).max(0.0).min(line_height);
    let width = f32::from_bits(key.width).max(0.0);
    let mut buffer = Buffer::new(font_system, Metrics::relative(font_size, 1.0));
    let attrs = Attrs::new()
        .family(Family::Name(key.glyph.family()))
        .metrics(Metrics::relative(font_size, 1.0));
    let text = character.to_string();

    buffer.set_size(font_system, Some(width), Some(buffer_height.max(0.0)));
    buffer.set_rich_text(
        font_system,
        vec![(text.as_str(), attrs.clone())],
        &attrs,
        Shaping::Basic,
        Some(glyphon::cosmic_text::Align::Center),
    );
    buffer.shape_until_scroll(font_system, false);

    Some(CachedIcon {
        buffer: Rc::new(RefCell::new(buffer)),
        line_height,
    })
}

fn attrs_for_shape(style: Style) -> Attrs<'static> {
    Attrs::new()
        .family(Family::SansSerif)
        .weight(system::weight(style.weight()))
        .metrics(Metrics::relative(style.size().max(1.0), 1.25))
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
    use crate::text::Color;
    use crate::text::document::{Block, Run};

    fn document(text: &str, color: Color, size: f32, weight: Weight) -> Document {
        let mut block = Block::new(Align::Start);
        block.push_run(Run::new(
            text,
            Style::default()
                .with_color(color)
                .with_size(size)
                .with_weight(weight),
        ));
        Document::from_block(block)
    }

    #[test]
    fn text_cache_ignores_color() {
        let mut cache = InlineCache::new();
        let red = document("Label", Color::RED, 12.0, Weight::Normal);
        let black = document("Label", Color::BLACK, 12.0, Weight::Normal);

        let first = cache
            .prepare_text(&red, 120.0, 22.0, glyphon::Wrap::None)
            .expect("text should prepare");
        let second = cache
            .prepare_text(&black, 120.0, 22.0, glyphon::Wrap::None)
            .expect("text should prepare");

        assert_eq!(first.stats.text_cache_misses, 1);
        assert_eq!(first.stats.text_shape_calls, 1);
        assert_eq!(second.stats.text_cache_hits, 1);
        assert_eq!(second.stats.text_shape_calls, 0);
        assert_eq!(second.default_color, system::color(Color::BLACK));
    }

    #[test]
    fn text_cache_misses_when_metrics_change() {
        let mut cache = InlineCache::new();
        let normal = document("Label", Color::BLACK, 12.0, Weight::Normal);
        let bold = document("Label", Color::BLACK, 12.0, Weight::Bold);

        let _ = cache.prepare_text(&normal, 120.0, 22.0, glyphon::Wrap::None);
        let changed = cache
            .prepare_text(&bold, 120.0, 22.0, glyphon::Wrap::None)
            .expect("text should prepare");

        assert_eq!(changed.stats.text_cache_misses, 1);
        assert_eq!(changed.stats.text_shape_calls, 1);
    }

    #[test]
    fn multi_run_text_uses_uncached_path() {
        let mut cache = InlineCache::new();
        let mut block = Block::new(Align::Start);
        block.push_run(Run::new(
            "Red",
            Style::default().with_color(Color::RED).with_size(12.0),
        ));
        block.push_run(Run::new(
            "Black",
            Style::default().with_color(Color::BLACK).with_size(12.0),
        ));
        let document = Document::from_block(block);

        let first = cache
            .prepare_text(&document, 120.0, 22.0, glyphon::Wrap::None)
            .expect("text should prepare");
        let second = cache
            .prepare_text(&document, 120.0, 22.0, glyphon::Wrap::None)
            .expect("text should prepare");

        assert_eq!(first.stats.text_cache_hits, 0);
        assert_eq!(second.stats.text_cache_hits, 0);
        assert_eq!(second.stats.text_shape_calls, 1);
    }

    #[test]
    fn icon_cache_ignores_color_by_only_keying_shape_inputs() {
        let mut cache = InlineCache::new();
        let glyph = icon::Icon::phosphor(icon::Id::new("command"))
            .glyph()
            .expect("command icon should resolve");

        let first = cache
            .prepare_icon(glyph, 12.0, 18.0, 18.0)
            .expect("icon should prepare");
        let second = cache
            .prepare_icon(glyph, 12.0, 18.0, 18.0)
            .expect("icon should prepare");

        assert_eq!(first.stats.icon_cache_misses, 1);
        assert_eq!(first.stats.icon_shape_calls, 1);
        assert_eq!(second.stats.icon_cache_hits, 1);
        assert_eq!(second.stats.icon_shape_calls, 0);
    }
}
