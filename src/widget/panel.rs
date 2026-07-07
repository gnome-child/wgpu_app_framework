use crate::{command, interaction, scene, subject, view};

use super::{Element, Layout, Ui, Widget};

pub struct Panel {
    element: Element,
}

pub struct Floating {
    panel: Panel,
}

impl Floating {
    pub fn new(id: impl Into<interaction::Id>) -> Self {
        Self {
            panel: Panel::from_node(view::Node::floating_panel(id)),
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.panel = self.panel.label(label);
        self
    }

    pub fn subject(mut self, subject: subject::Segment) -> Self {
        self.panel = self.panel.subject(subject);
        self
    }

    pub fn width(mut self, size: view::Dimension) -> Self {
        self.panel = self.panel.width(size);
        self
    }

    pub fn height(mut self, size: view::Dimension) -> Self {
        self.panel = self.panel.height(size);
        self
    }

    pub fn max_height(mut self, height: i32) -> Self {
        self.panel = self.panel.max_height(height);
        self
    }

    pub fn background(mut self, background: scene::Brush) -> Self {
        self.panel = self.panel.background(background);
        self
    }

    pub fn layout(mut self, configure: impl FnOnce(Layout) -> Layout) -> Self {
        self.panel = self.panel.layout(configure);
        self
    }

    pub fn row(mut self) -> Self {
        self.panel = self.panel.row();
        self
    }

    pub fn column(mut self) -> Self {
        self.panel = self.panel.column();
        self
    }

    pub fn overlay(mut self) -> Self {
        self.panel = self.panel.overlay();
        self
    }

    pub fn children(mut self, children: impl FnOnce(&mut Ui)) -> Self {
        self.panel = self.panel.children(children);
        self
    }

    pub fn child(mut self, child: impl Widget) -> Self {
        self.panel = self.panel.child(child);
        self
    }
}

impl Widget for Floating {
    fn into_node(self) -> view::Node {
        self.panel.into_node()
    }
}

impl Panel {
    pub fn new() -> Self {
        Self::from_node(view::Node::panel())
    }

    fn from_node(node: view::Node) -> Self {
        Self {
            element: Element::from_node(node),
        }
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

    pub fn trigger<C>(mut self, args: C::Args) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        self.element = self.element.trigger::<C>(args);
        self
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Panel {
    fn into_node(self) -> view::Node {
        self.element.into_node()
    }
}
