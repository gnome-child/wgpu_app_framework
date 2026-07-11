use super::super::{
    binding::Binding,
    control::{Button, Checkbox, Control, Radio, Slider, TextArea, TextBox},
    style::Style,
};
use super::{Axis, FloatingPlacement, NativePopupMaterialPreference, Node, Role};
use crate::{interaction, subject};

impl Node {
    pub(crate) fn role(&self) -> Role {
        self.role
    }

    pub fn axis(&self) -> Option<Axis> {
        self.axis
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    pub(crate) fn floating_placement(&self) -> FloatingPlacement {
        self.floating_placement
    }

    pub(crate) fn force_overlay_group(&self) -> bool {
        self.force_overlay_group
    }

    pub(crate) fn native_popup_material_preference(&self) -> NativePopupMaterialPreference {
        self.native_popup_material_preference
    }

    pub fn subject(&self) -> Option<&subject::Segment> {
        self.subject.as_ref()
    }

    pub fn label_text(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub(crate) fn world_text_overflow(&self) -> Option<crate::text::Overflow> {
        match self.text_kind {
            super::TextKind::Author => None,
            super::TextKind::World(overflow) => Some(overflow),
        }
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

    pub fn focus_visible(&self) -> bool {
        self.focus_visible
            || self.text_area_model().is_some_and(TextArea::focus_visible)
            || self.text_box_model().is_some_and(TextBox::focus_visible)
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn is_active_item(&self) -> bool {
        self.active_item
    }

    pub(crate) fn scroll_offset(&self) -> interaction::ScrollOffset {
        self.scroll_offset
    }

    pub(crate) fn virtual_list_model(&self) -> Option<&crate::virtual_list::Model> {
        self.virtual_list.as_ref()
    }

    pub(crate) fn provided_row(&self) -> Option<super::ProvidedRow> {
        self.provided_row
    }

    pub(crate) fn table_row(&self) -> Option<crate::table::Row> {
        self.table_row
    }

    pub(crate) fn table_cell(&self) -> Option<crate::table::Cell> {
        self.table_cell
    }

    pub(crate) fn table_header_cell(&self) -> Option<crate::table::HeaderCell> {
        self.table_header_cell
    }

    pub(crate) fn table_divider(&self) -> Option<crate::table::Divider> {
        self.table_divider
    }

    pub(crate) fn table_model(&self) -> Option<&crate::table::Model> {
        self.table_model.as_ref()
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
