use crate::{interaction, scene, subject, view};

use super::{Direction, Layout, Ui, Widget};

pub struct Scroll {
    node: view::Node,
    layout: Layout,
    width: Option<view::style::Dimension>,
    height: Option<view::style::Dimension>,
    max_height: Option<i32>,
    background: Option<scene::Brush>,
}

impl Scroll {
    pub fn new() -> Self {
        Self {
            node: view::Node::scroll(),
            layout: Layout::default(),
            width: None,
            height: None,
            max_height: None,
            background: None,
        }
    }

    pub fn id(mut self, id: impl Into<interaction::Id>) -> Self {
        self.node = self.node.with_interaction_id(id);
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.node = self.node.with_label(label);
        self
    }

    pub fn subject(mut self, subject: subject::Segment) -> Self {
        self.node = self.node.with_subject(subject);
        self
    }

    pub fn width(mut self, size: view::style::Dimension) -> Self {
        self.width = Some(size);
        self
    }

    pub fn height(mut self, size: view::style::Dimension) -> Self {
        self.height = Some(size);
        self
    }

    pub fn max_height(mut self, height: i32) -> Self {
        self.max_height = Some(height.max(0));
        self
    }

    pub fn background(mut self, background: scene::Brush) -> Self {
        self.background = Some(background);
        self
    }

    pub fn layout(mut self, configure: impl FnOnce(Layout) -> Layout) -> Self {
        self.layout = configure(self.layout);
        self.apply_layout_direction()
    }

    pub fn row(mut self) -> Self {
        self.layout = self.layout.row();
        self.apply_layout_direction()
    }

    pub fn column(mut self) -> Self {
        self.layout = self.layout.column();
        self.apply_layout_direction()
    }

    pub fn overlay(mut self) -> Self {
        self.layout = self.layout.overlay();
        self.apply_layout_direction()
    }

    pub fn children(mut self, children: impl FnOnce(&mut Ui)) -> Self {
        let mut ui = Ui::new();
        children(&mut ui);
        for child in ui.into_nodes() {
            self.node = self.node.child(child);
        }
        self
    }

    pub fn child(mut self, child: impl Widget) -> Self {
        self.node = self.node.child(child.into_node());
        self
    }

    fn apply_layout_direction(mut self) -> Self {
        self.node = match self.layout.direction() {
            Direction::Row => self.node.with_layout_axis(view::node::Axis::Horizontal),
            Direction::Column => self.node.with_layout_axis(view::node::Axis::Vertical),
            Direction::Overlay => self.node.with_layout_axis(view::node::Axis::Overlay),
        };
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
        let mut style = view::Style::new()
            .with_padding(self.layout.padding_value())
            .with_align_items(self.layout.align_items_value())
            .with_justify_content(self.layout.justify_content_value());

        if let Some(gap) = self.layout.gap_override() {
            style = style.with_gap(gap);
        }
        if let Some(width) = self.width {
            style = style.with_width(width);
        }
        if let Some(height) = self.height {
            style = style.with_height(height);
        }
        if let Some(max_height) = self.max_height {
            style = style.with_max_height(max_height);
        }
        if let Some(background) = self.background {
            style = style.with_background(background);
        }

        self.node.with_style(style)
    }
}
