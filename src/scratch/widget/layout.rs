use super::super::view;

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    direction: Direction,
    gap: Option<i32>,
    padding: view::style::Padding,
    align_items: view::style::Align,
    justify_content: view::style::Align,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Row,
    Column,
    Overlay,
}

impl Layout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn row(mut self) -> Self {
        self.direction = Direction::Row;
        self
    }

    pub fn column(mut self) -> Self {
        self.direction = Direction::Column;
        self
    }

    pub fn overlay(mut self) -> Self {
        self.direction = Direction::Overlay;
        self
    }

    pub fn gap(mut self, gap: i32) -> Self {
        self.gap = Some(gap.max(0));
        self
    }

    pub fn padding(mut self, padding: view::style::Padding) -> Self {
        self.padding = padding;
        self
    }

    pub fn align_items(mut self, align: view::style::Align) -> Self {
        self.align_items = align;
        self
    }

    pub fn justify_content(mut self, align: view::style::Align) -> Self {
        self.justify_content = align;
        self
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn gap_value(&self) -> i32 {
        self.gap.unwrap_or_default()
    }

    pub fn gap_override(&self) -> Option<i32> {
        self.gap
    }

    pub fn padding_value(&self) -> view::style::Padding {
        self.padding
    }

    pub fn align_items_value(&self) -> view::style::Align {
        self.align_items
    }

    pub fn justify_content_value(&self) -> view::style::Align {
        self.justify_content
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            direction: Direction::Column,
            gap: None,
            padding: view::style::Padding::zero(),
            align_items: view::style::Align::Stretch,
            justify_content: view::style::Align::Start,
        }
    }
}
