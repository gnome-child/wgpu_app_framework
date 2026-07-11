use crate::{interaction, scene, subject, view};

use super::{Element, Layout, Ui, Widget};

pub struct Scroll {
    element: Element,
}

impl Scroll {
    pub fn new() -> Self {
        Self {
            element: Element::from_node(view::Node::scroll()),
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
}

impl Default for Scroll {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Scroll {
    fn into_node(self) -> view::Node {
        self.element.into_node()
    }
}
