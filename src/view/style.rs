use super::super::scene;

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    width: Option<Dimension>,
    height: Option<Dimension>,
    max_height: Option<i32>,
    gap: Option<i32>,
    padding: Padding,
    align_items: Align,
    justify_content: Align,
    background: Option<scene::Brush>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    Fit,
    Flexible { weight: u16, minimum: i32 },
    Fixed(i32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Padding {
    top: i32,
    right: i32,
    bottom: i32,
    left: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_width(mut self, width: Dimension) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: Dimension) -> Self {
        self.height = Some(height);
        self
    }

    pub fn with_max_height(mut self, height: i32) -> Self {
        self.max_height = Some(height.max(0));
        self
    }

    pub fn with_gap(mut self, gap: i32) -> Self {
        self.gap = Some(gap.max(0));
        self
    }

    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_align_items(mut self, align: Align) -> Self {
        self.align_items = align;
        self
    }

    pub fn with_justify_content(mut self, align: Align) -> Self {
        self.justify_content = align;
        self
    }

    pub fn with_background(mut self, background: scene::Brush) -> Self {
        self.background = Some(background);
        self
    }

    pub fn width(&self) -> Option<Dimension> {
        self.width
    }

    pub fn height(&self) -> Option<Dimension> {
        self.height
    }

    pub fn max_height(&self) -> Option<i32> {
        self.max_height
    }

    pub fn gap(&self) -> i32 {
        self.gap.unwrap_or_default()
    }

    pub fn gap_override(&self) -> Option<i32> {
        self.gap
    }

    pub fn padding(&self) -> Padding {
        self.padding
    }

    pub fn align_items(&self) -> Align {
        self.align_items
    }

    pub fn justify_content(&self) -> Align {
        self.justify_content
    }

    pub fn background(&self) -> Option<scene::Brush> {
        self.background
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            max_height: None,
            gap: None,
            padding: Padding::zero(),
            align_items: Align::Stretch,
            justify_content: Align::Start,
            background: None,
        }
    }
}

impl Dimension {
    pub fn fit() -> Self {
        Self::Fit
    }

    pub fn grow() -> Self {
        Self::Flexible {
            weight: 1,
            minimum: 0,
        }
    }

    pub fn weight(value: u16) -> Self {
        Self::Flexible {
            weight: value.max(1),
            minimum: 0,
        }
    }

    pub fn fixed(value: i32) -> Self {
        Self::Fixed(value.max(0))
    }

    pub fn percent(value: f32) -> Self {
        Self::Percent(value.clamp(0.0, 1.0))
    }

    /// Preserves at least `minimum` logical pixels when this flexible
    /// dimension is allocated under overflow pressure.
    pub fn minimum(self, minimum: i32) -> Self {
        match self {
            Self::Flexible { weight, .. } => Self::Flexible {
                weight,
                minimum: minimum.max(0),
            },
            _ => self,
        }
    }

    pub(crate) fn flexible(self) -> Option<(u16, i32)> {
        match self {
            Self::Flexible { weight, minimum } => Some((weight, minimum)),
            _ => None,
        }
    }
}

impl Padding {
    pub fn zero() -> Self {
        Self {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        }
    }

    pub fn all(value: i32) -> Self {
        let value = value.max(0);
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn symmetric(horizontal: i32, vertical: i32) -> Self {
        let horizontal = horizontal.max(0);
        let vertical = vertical.max(0);
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub fn edges(top: i32, right: i32, bottom: i32, left: i32) -> Self {
        Self {
            top: top.max(0),
            right: right.max(0),
            bottom: bottom.max(0),
            left: left.max(0),
        }
    }

    pub fn top(self) -> i32 {
        self.top
    }

    pub fn right(self) -> i32 {
        self.right
    }

    pub fn bottom(self) -> i32 {
        self.bottom
    }

    pub fn left(self) -> i32 {
        self.left
    }

    pub fn horizontal(self) -> i32 {
        self.left.saturating_add(self.right)
    }

    pub fn vertical(self) -> i32 {
        self.top.saturating_add(self.bottom)
    }
}

impl Default for Padding {
    fn default() -> Self {
        Self::zero()
    }
}
