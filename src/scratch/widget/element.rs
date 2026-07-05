use crate::scratch::{command, context::Source, interaction, view};

use super::{Direction, Layout, Ui, Widget};

pub struct Element {
    node: view::Node,
    layout: Layout,
    width: Option<view::style::Dimension>,
    height: Option<view::style::Dimension>,
}

impl Element {
    pub fn new() -> Self {
        Self {
            node: view::Node::panel(),
            layout: Layout::default(),
            width: None,
            height: None,
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

    pub fn width(mut self, size: view::style::Dimension) -> Self {
        self.width = Some(size);
        self
    }

    pub fn height(mut self, size: view::style::Dimension) -> Self {
        self.height = Some(size);
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

    pub fn trigger<C>(mut self, args: C::Args) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        self.node = self.node.bind_command::<C>(args, Source::Button);
        self
    }

    pub fn layout_state(&self) -> &Layout {
        &self.layout
    }

    pub fn width_state(&self) -> Option<view::style::Dimension> {
        self.width
    }

    pub fn height_state(&self) -> Option<view::style::Dimension> {
        self.height
    }

    fn apply_layout_direction(mut self) -> Self {
        self.node = match self.layout.direction() {
            Direction::Row => self.node.with_layout_axis(view::node::Axis::Horizontal),
            Direction::Column => self.node.with_layout_axis(view::node::Axis::Vertical),
        };
        self
    }
}

impl Default for Element {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Element {
    fn into_node(self) -> view::Node {
        let mut style = view::Style::new()
            .with_gap(self.layout.gap_value())
            .with_padding(self.layout.padding_value())
            .with_align_items(self.layout.align_items_value())
            .with_justify_content(self.layout.justify_content_value());

        if let Some(width) = self.width {
            style = style.with_width(width);
        }
        if let Some(height) = self.height {
            style = style.with_height(height);
        }

        self.node.with_style(style)
    }
}
