use crate::paint;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ResolvedTextDirection {
    Ltr,
    Rtl,
}

impl ResolvedTextDirection {
    pub(crate) fn is_rtl(self) -> bool {
        matches!(self, Self::Rtl)
    }
}

fn auto_text_direction(text: &str) -> ResolvedTextDirection {
    unicode_bidi::BidiInfo::new(text, None)
        .paragraphs
        .first()
        .map(|paragraph| {
            if paragraph.level.is_rtl() {
                ResolvedTextDirection::Rtl
            } else {
                ResolvedTextDirection::Ltr
            }
        })
        .unwrap_or(ResolvedTextDirection::Ltr)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum TextDirection {
    #[default]
    Auto,
    Ltr,
    Rtl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Body,
    Label,
    Control,
    Menu,
    Placeholder,
}

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    size: f32,
    color: paint::Color,
    weight: Weight,
    direction: TextDirection,
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

impl TextDirection {
    pub(crate) fn resolve_for_text(self, text: &str) -> ResolvedTextDirection {
        match self {
            Self::Ltr => ResolvedTextDirection::Ltr,
            Self::Rtl => ResolvedTextDirection::Rtl,
            Self::Auto => auto_text_direction(text),
        }
    }
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

    pub fn first_style(&self) -> Option<Style> {
        let first_run_style = self
            .blocks
            .iter()
            .flat_map(Block::runs)
            .next()
            .map(Run::style);

        self.blocks
            .iter()
            .flat_map(Block::runs)
            .find(|run| !run.is_empty())
            .map(Run::style)
            .or(first_run_style)
    }

    pub fn with_color(mut self, color: paint::Color) -> Self {
        for block in &mut self.blocks {
            for run in &mut block.runs {
                run.style = run.style.with_color(color);
            }
        }
        self
    }

    pub fn with_size(mut self, size: f32) -> Self {
        for block in &mut self.blocks {
            for run in &mut block.runs {
                run.style = run.style.with_size(size);
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

    pub fn direction(self) -> TextDirection {
        self.direction
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

    pub fn with_direction(mut self, direction: TextDirection) -> Self {
        self.direction = direction;
        self
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            size: 16.0,
            color: paint::Color::rgb(0.92, 0.94, 0.98),
            weight: Weight::Normal,
            direction: TextDirection::Auto,
        }
    }
}

pub(crate) fn block_direction(block: &Block) -> ResolvedTextDirection {
    let text = block.runs().iter().map(Run::text).collect::<String>();
    block
        .runs()
        .iter()
        .find(|run| !run.is_empty())
        .map(|run| run.style().direction().resolve_for_text(&text))
        .unwrap_or(ResolvedTextDirection::Ltr)
}
