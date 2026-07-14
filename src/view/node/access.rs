use super::super::{
    binding::Binding,
    control::{Button, Checkbox, Radio, Slider, TextArea, TextBox},
    style::Style,
};
use super::{Axis, FloatingPlacement, NativePopupMaterialPreference, Node, PanelPolicy, Role};
use crate::{interaction, subject};

static INTERACTIVE_PANEL_POLICY: PanelPolicy = PanelPolicy::Interactive;

impl Node {
    pub(crate) fn role(&self) -> Role {
        self.content.role()
    }

    pub fn axis(&self) -> Option<Axis> {
        self.axis
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    pub(crate) fn floating_placement(&self) -> FloatingPlacement {
        self.content
            .panel()
            .map_or(FloatingPlacement::Default, |panel| panel.placement)
    }

    pub(crate) fn panel_attachment(&self) -> Option<super::PanelAttachment> {
        self.content.panel().and_then(|panel| panel.attachment)
    }

    pub(crate) fn popup_context(&self) -> Option<crate::popup::ContextFingerprint> {
        self.content.panel().and_then(|panel| panel.popup_context)
    }

    pub(crate) fn panel_policy(&self) -> &PanelPolicy {
        self.content
            .panel()
            .map_or(&INTERACTIVE_PANEL_POLICY, |panel| &panel.policy)
    }

    pub(crate) fn auxiliary_hint(&self) -> Option<&super::super::Hint> {
        self.content
            .panel()
            .and_then(|panel| panel.policy.auxiliary_hint())
    }

    pub(crate) fn force_overlay_group(&self) -> bool {
        self.content
            .panel()
            .is_some_and(|panel| panel.force_overlay_group)
    }

    pub(crate) fn native_popup_material_preference(&self) -> NativePopupMaterialPreference {
        self.content
            .panel()
            .map_or(NativePopupMaterialPreference::System, |panel| {
                panel.native_material
            })
    }

    pub(crate) fn table_header_presentation(&self) -> Option<crate::table::HeaderPresentation> {
        self.table_header_presentation
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
            super::TextKind::World(world) => Some(world.overflow),
        }
    }

    pub(crate) fn world_text_wrap(&self) -> Option<super::super::Wrap> {
        match self.text_kind {
            super::TextKind::World(world) => Some(world.wrap),
            super::TextKind::Author => None,
        }
    }

    pub(crate) fn world_text_align(&self) -> Option<super::super::Align> {
        match self.text_kind {
            super::TextKind::World(world) => Some(world.align),
            super::TextKind::Author => None,
        }
    }

    pub fn binding(&self) -> Option<&Binding> {
        self.binding.as_ref()
    }

    pub(crate) fn context_binding(&self) -> Option<&Binding> {
        self.context_binding.as_ref()
    }

    pub fn is_hidden_binding(&self) -> bool {
        self.binding.as_ref().is_some_and(Binding::is_hidden)
    }

    pub fn text_area_model(&self) -> Option<&TextArea> {
        self.content.text_area()
    }

    pub fn is_focused(&self) -> bool {
        self.focus_presentation().is_focused()
    }

    pub fn focus_visible(&self) -> bool {
        self.focus_presentation().is_visible()
    }

    pub(crate) fn focus_presentation(&self) -> super::super::focus::Presentation {
        if let Some(text_area) = self.text_area_model() {
            return text_area.focus_presentation();
        }
        if let Some(text_box) = self.text_box_model() {
            return text_box.focus_presentation();
        }
        self.focus_presentation
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn is_active_item(&self) -> bool {
        self.active_item
    }

    pub(crate) fn scroll_offset(&self) -> interaction::ScrollOffset {
        self.content.scroll_offset()
    }

    pub(crate) fn virtual_list_model(&self) -> Option<&crate::virtual_list::Model> {
        self.content.virtual_list()
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

    pub(crate) fn table_model(&self) -> Option<&crate::table::Model> {
        self.content.table_model()
    }

    pub(crate) fn text_commit(&self) -> Option<&super::super::TextCommit> {
        self.content.text_commit()
    }

    pub(crate) fn participation(&self) -> Option<super::Participation> {
        self.participation
    }

    pub fn button_model(&self) -> Option<&Button> {
        self.content.button()
    }

    pub fn checkbox_model(&self) -> Option<&Checkbox> {
        self.content.checkbox()
    }

    pub fn radio_model(&self) -> Option<&Radio> {
        self.content.radio()
    }

    pub fn slider_model(&self) -> Option<&Slider> {
        self.content.slider()
    }

    pub fn text_box_model(&self) -> Option<&TextBox> {
        self.content.text_box()
    }

    pub fn id(&self) -> Option<interaction::Id> {
        self.id
    }

    pub fn children(&self) -> &[Node] {
        &self.children
    }

    pub(super) fn text_area_model_mut(&mut self) -> Option<&mut TextArea> {
        self.content.text_area_mut()
    }

    pub(super) fn text_box_model_mut(&mut self) -> Option<&mut TextBox> {
        self.content.text_box_mut()
    }

    pub(super) fn virtual_list_model_mut(&mut self) -> Option<&mut crate::virtual_list::Model> {
        self.content.virtual_list_mut()
    }

    pub(super) fn set_scroll_offset(&mut self, offset: interaction::ScrollOffset) {
        self.content.set_scroll_offset(offset);
    }

    pub(super) fn standard_menu_extensions(&self) -> Option<&[super::StandardMenuExtension]> {
        self.content.standard_menu_extensions()
    }

    pub(super) fn standard_menu_extensions_mut(
        &mut self,
    ) -> Option<&mut Vec<super::StandardMenuExtension>> {
        self.content.standard_menu_extensions_mut()
    }
}
