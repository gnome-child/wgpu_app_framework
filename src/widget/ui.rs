use crate::{interaction, view};

use super::{
    Button, Checkbox, Element, Label, Menu, MenuBar, Radio, Separator, Slider, StandardMenuBar,
    TextArea, TextBox, Widget,
};

pub struct Ui {
    nodes: Vec<view::Node>,
}

impl Ui {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn add(&mut self, child: impl Widget) -> &mut Self {
        self.nodes.push(child.into_node());
        self
    }

    pub fn child(&mut self, child: impl Widget) -> &mut Self {
        self.add(child)
    }

    pub fn context_menu(&mut self, child: impl Widget) -> &mut Self {
        self.add(super::context_menu(child))
    }

    pub fn row(&mut self, children: impl FnOnce(&mut Ui)) -> &mut Self {
        self.add(Element::new().row().children(children))
    }

    pub fn column(&mut self, children: impl FnOnce(&mut Ui)) -> &mut Self {
        self.add(Element::new().column().children(children))
    }

    pub fn overlay(&mut self, children: impl FnOnce(&mut Ui)) -> &mut Self {
        self.add(Element::new().overlay().children(children))
    }

    pub fn scroll(&mut self, children: impl FnOnce(&mut Ui)) -> &mut Self {
        self.add(crate::Scroll::new().children(children))
    }

    pub fn menu_bar(&mut self, children: impl FnOnce(&mut Ui)) -> &mut Self {
        self.add(MenuBar::new().children(children))
    }

    /// Builds the conventional menu bar from registered command meaning.
    ///
    /// The bar is opt-in: registering commands alone never creates UI.
    pub fn standard_menu_bar(&mut self) -> &mut Self {
        self.add(StandardMenuBar::new())
    }

    /// Places a conventional bar and applies explicit authored deviations.
    pub fn standard_menu_bar_with(
        &mut self,
        extensions: impl FnOnce(&mut StandardMenuBar),
    ) -> &mut Self {
        let mut bar = StandardMenuBar::new();
        extensions(&mut bar);
        self.add(bar)
    }

    pub fn menu(
        &mut self,
        id: impl Into<interaction::Id>,
        label: impl Into<String>,
        children: impl FnOnce(&mut Ui),
    ) -> &mut Self {
        self.add(Menu::new(id, label).children(children))
    }

    pub fn separator(&mut self) -> &mut Self {
        self.add(Separator::new())
    }

    pub fn label(&mut self, label: impl Into<String>) -> &mut Self {
        self.add(Label::new(label))
    }

    pub fn button(&mut self, button: Button) -> &mut Self {
        self.add(button)
    }

    pub fn checkbox(&mut self, checkbox: Checkbox) -> &mut Self {
        self.add(checkbox)
    }

    pub fn radio(&mut self, radio: Radio) -> &mut Self {
        self.add(radio)
    }

    pub fn slider(&mut self, slider: Slider) -> &mut Self {
        self.add(slider)
    }

    pub fn text_box(&mut self, text_box: TextBox) -> &mut Self {
        self.add(text_box)
    }

    pub fn text_area(&mut self, text_area: TextArea) -> &mut Self {
        self.add(text_area)
    }

    pub(super) fn into_nodes(self) -> Vec<view::Node> {
        self.nodes
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}
