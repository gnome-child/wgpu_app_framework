use super::super::{
    binding::Binding,
    control::{Button, Checkbox, Control, Radio, Slider, TextArea, TextBox},
    style::Style,
};
use super::{Axis, Node, Role};
use crate::scratch::interaction;

impl Node {
    pub fn role(&self) -> Role {
        self.role
    }

    pub fn axis(&self) -> Option<Axis> {
        self.axis
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    pub fn label_text(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn binding(&self) -> Option<&Binding> {
        self.binding.as_ref()
    }

    pub fn is_hidden_binding(&self) -> bool {
        self.binding.as_ref().is_some_and(Binding::is_hidden)
    }

    pub fn text_area_model(&self) -> Option<&TextArea> {
        match self.control.as_ref()? {
            Control::TextArea(text_area) => Some(text_area),
            Control::Button(_)
            | Control::Checkbox(_)
            | Control::Radio(_)
            | Control::Slider(_)
            | Control::TextBox(_) => None,
        }
    }

    pub fn is_focused(&self) -> bool {
        self.focused
            || self.text_area_model().is_some_and(TextArea::is_focused)
            || self.text_box_model().is_some_and(TextBox::is_focused)
    }

    pub fn button_model(&self) -> Option<&Button> {
        match self.control.as_ref()? {
            Control::Button(button) => Some(button),
            Control::Checkbox(_)
            | Control::Radio(_)
            | Control::Slider(_)
            | Control::TextArea(_)
            | Control::TextBox(_) => None,
        }
    }

    pub fn checkbox_model(&self) -> Option<&Checkbox> {
        match self.control.as_ref()? {
            Control::Checkbox(checkbox) => Some(checkbox),
            Control::Button(_)
            | Control::Radio(_)
            | Control::Slider(_)
            | Control::TextArea(_)
            | Control::TextBox(_) => None,
        }
    }

    pub fn radio_model(&self) -> Option<&Radio> {
        match self.control.as_ref()? {
            Control::Radio(radio) => Some(radio),
            Control::Button(_)
            | Control::Checkbox(_)
            | Control::Slider(_)
            | Control::TextArea(_)
            | Control::TextBox(_) => None,
        }
    }

    pub fn slider_model(&self) -> Option<&Slider> {
        match self.control.as_ref()? {
            Control::Slider(slider) => Some(slider),
            Control::Button(_)
            | Control::Checkbox(_)
            | Control::Radio(_)
            | Control::TextArea(_)
            | Control::TextBox(_) => None,
        }
    }

    pub fn text_box_model(&self) -> Option<&TextBox> {
        match self.control.as_ref()? {
            Control::TextBox(text_box) => Some(text_box),
            Control::Button(_)
            | Control::Checkbox(_)
            | Control::Radio(_)
            | Control::Slider(_)
            | Control::TextArea(_) => None,
        }
    }

    pub fn id(&self) -> Option<interaction::Id> {
        self.id
    }

    pub fn children(&self) -> &[Node] {
        &self.children
    }
}
