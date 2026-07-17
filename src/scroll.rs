use crate::{interaction, scene, subject, view};

use crate::widget::{Element, Layout, Ui, Widget};

pub use crate::interaction::scroll::{Delta, Offset};

/// Visibility policy for one scrollbar axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    Always,
    Automatic,
    Never,
    External,
}

/// Whether framework-provided scrollbars overlay or consume content space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presentation {
    Overlay,
    Consuming,
}

/// Whether a scroll container requests its minimum or natural content extent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sizing {
    Minimum,
    Natural,
}

/// Logical horizontal direction for scroll operations and chrome placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
}

/// Complete authored policy for a scroll container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Configuration {
    pub(crate) horizontal_policy: Policy,
    pub(crate) vertical_policy: Policy,
    pub(crate) presentation: Presentation,
    pub(crate) horizontal_sizing: Sizing,
    pub(crate) vertical_sizing: Sizing,
    pub(crate) direction: Direction,
}

impl Configuration {
    pub const fn new(
        horizontal_policy: Policy,
        vertical_policy: Policy,
        presentation: Presentation,
        horizontal_sizing: Sizing,
        vertical_sizing: Sizing,
        direction: Direction,
    ) -> Self {
        Self {
            horizontal_policy,
            vertical_policy,
            presentation,
            horizontal_sizing,
            vertical_sizing,
            direction,
        }
    }

    pub const fn horizontal_policy(self) -> Policy {
        self.horizontal_policy
    }

    pub const fn vertical_policy(self) -> Policy {
        self.vertical_policy
    }

    pub const fn presentation(self) -> Presentation {
        self.presentation
    }

    pub const fn horizontal_sizing(self) -> Sizing {
        self.horizontal_sizing
    }

    pub const fn vertical_sizing(self) -> Sizing {
        self.vertical_sizing
    }

    pub const fn direction(self) -> Direction {
        self.direction
    }
}

pub struct Scroll {
    element: Element,
    configuration: Option<Configuration>,
}

impl Scroll {
    pub fn new() -> Self {
        Self {
            element: Element::from_node(view::Node::scroll()),
            configuration: None,
        }
    }

    pub fn configuration(mut self, configuration: Configuration) -> Self {
        self.configuration = Some(configuration);
        self
    }

    pub fn id(mut self, id: impl Into<interaction::Id>) -> Self {
        self.element = self.element.id(id);
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.element = self.element.label(label);
        self
    }

    pub fn subject(mut self, subject: subject::Segment) -> Self {
        self.element = self.element.subject(subject);
        self
    }

    pub fn width(mut self, size: view::Dimension) -> Self {
        self.element = self.element.width(size);
        self
    }

    pub fn height(mut self, size: view::Dimension) -> Self {
        self.element = self.element.height(size);
        self
    }

    pub fn max_height(mut self, height: i32) -> Self {
        self.element = self.element.max_height(height);
        self
    }

    pub fn background(mut self, background: scene::Brush) -> Self {
        self.element = self.element.background(background);
        self
    }

    pub fn layout(mut self, configure: impl FnOnce(Layout) -> Layout) -> Self {
        self.element = self.element.layout(configure);
        self
    }

    pub fn row(mut self) -> Self {
        self.element = self.element.row();
        self
    }

    pub fn column(mut self) -> Self {
        self.element = self.element.column();
        self
    }

    pub fn overlay(mut self) -> Self {
        self.element = self.element.overlay();
        self
    }

    pub fn children(mut self, children: impl FnOnce(&mut Ui)) -> Self {
        self.element = self.element.children(children);
        self
    }

    pub fn child(mut self, child: impl Widget) -> Self {
        self.element = self.element.child(child);
        self
    }
}

impl Default for Scroll {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Scroll {
    fn into_node(self) -> view::Node {
        let node = self.element.into_node();
        match self.configuration {
            Some(configuration) => node.with_scroll_configuration(configuration),
            None => node,
        }
    }
}
