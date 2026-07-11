use super::super::{
    binding::Binding,
    control::{Button, Checkbox, Control, Radio, Slider, TextArea, TextBox},
    style::Style,
};
use super::{Axis, FloatingPlacement, NativePopupMaterialPreference, Node, Role};
use crate::{command, context::Source, interaction, subject};

impl Node {
    pub fn root() -> Self {
        Self::new(Role::Root)
    }

    pub fn stack(axis: Axis) -> Self {
        Self::new(Role::Stack).with_axis(axis)
    }

    pub fn menu_bar() -> Self {
        Self::new(Role::MenuBar)
    }

    pub fn menu(id: impl Into<interaction::Id>, label: impl Into<String>) -> Self {
        Self::new(Role::Menu).with_id(id).with_label(label)
    }

    pub fn bound<C>() -> Self
    where
        C: command::Command<Args = ()>,
    {
        Self::bound_with_args::<C>(())
    }

    pub fn menu_bound<C>() -> Self
    where
        C: command::Command<Args = ()>,
    {
        Self::menu_bound_with_args::<C>(())
    }

    pub fn bound_with_args<C>(args: C::Args) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        Self::new(Role::Binding).with_binding(Binding::new::<C>(args, Source::Button))
    }

    pub fn menu_bound_with_args<C>(args: C::Args) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        Self::new(Role::Binding).with_binding(Binding::new::<C>(args, Source::Menu))
    }

    pub fn separator() -> Self {
        Self::new(Role::Separator)
    }

    pub fn text_area(text: impl Into<String>) -> Self {
        Self::text_area_state(TextArea::new(text))
    }

    pub fn text_area_state(text_area: TextArea) -> Self {
        Self::new(Role::TextArea).with_control(Control::TextArea(text_area))
    }

    pub fn button(label: impl Into<String>) -> Self {
        Self::button_state(Button::new(label))
    }

    pub fn button_state(button: Button) -> Self {
        let label = button.label().to_owned();
        Self::new(Role::Button)
            .with_label(label)
            .with_control(Control::Button(button))
    }

    pub fn checkbox(label: impl Into<String>, checked: bool) -> Self {
        Self::checkbox_state(Checkbox::new(label, checked))
    }

    pub fn checkbox_state(checkbox: Checkbox) -> Self {
        let label = checkbox.label().to_owned();
        Self::new(Role::Checkbox)
            .with_label(label)
            .with_control(Control::Checkbox(checkbox))
    }

    pub fn radio(label: impl Into<String>, selected: bool) -> Self {
        Self::radio_state(Radio::new(label, selected))
    }

    pub fn radio_state(radio: Radio) -> Self {
        let label = radio.label().to_owned();
        Self::new(Role::Radio)
            .with_label(label)
            .with_control(Control::Radio(radio))
    }

    pub fn slider(label: impl Into<String>, value: f64, start: f64, end: f64) -> Self {
        Self::slider_state(Slider::new(label, value, start, end))
    }

    pub fn slider_state(slider: Slider) -> Self {
        let label = slider.display_label();
        Self::new(Role::Slider)
            .with_label(label)
            .with_control(Control::Slider(slider))
    }

    pub fn text_box(text: impl Into<String>) -> Self {
        Self::text_box_state(TextBox::new(text))
    }

    pub fn text_box_state(text_box: TextBox) -> Self {
        Self::new(Role::TextBox).with_control(Control::TextBox(text_box))
    }

    pub fn panel() -> Self {
        Self::new(Role::Panel)
    }

    pub fn scroll() -> Self {
        Self::new(Role::Scroll).with_axis(Axis::Vertical)
    }

    pub(crate) fn virtual_list(model: crate::virtual_list::Model) -> Self {
        Self::new(Role::VirtualList)
            .with_id(model.id())
            .with_axis(Axis::Vertical)
            .with_virtual_list(model)
    }

    pub(crate) fn table(id: interaction::Id) -> Self {
        Self::new(Role::Table)
            .with_axis(Axis::Vertical)
            .with_interaction_id(id)
    }

    pub fn floating_panel(id: impl Into<interaction::Id>) -> Self {
        Self::new(Role::FloatingPanel).with_id(id)
    }

    pub fn label(label: impl Into<String>) -> Self {
        Self::new(Role::Label).with_label(label)
    }

    /// Creates text supplied by the world outside the program. Its overflow
    /// behavior must be explicit because the program cannot promise it fits.
    pub fn world_text(label: impl Into<String>, overflow: crate::text::Overflow) -> Self {
        Self::new(Role::Label)
            .with_label(label)
            .with_text_kind(super::TextKind::World(overflow))
    }

    pub(crate) fn section_header(label: impl Into<String>) -> Self {
        Self::new(Role::SectionHeader).with_label(label)
    }

    pub fn child(mut self, child: Node) -> Self {
        self.children.push(child);
        self
    }

    pub(in crate::view) fn push_child(&mut self, child: Node) {
        self.children.push(child);
    }

    pub(crate) fn with_provided_row(
        mut self,
        list: interaction::Id,
        key: crate::virtual_list::Key,
        index: usize,
    ) -> Self {
        self.provided_row = Some(super::ProvidedRow { list, key, index });
        self
    }

    pub(crate) fn with_table_row(mut self, row: crate::table::Row) -> Self {
        self.table_row = Some(row);
        self
    }

    pub(crate) fn with_table_cell(mut self, cell: crate::table::Cell) -> Self {
        self.table_cell = Some(cell);
        self
    }

    pub(crate) fn with_table_header_cell(mut self, cell: crate::table::HeaderCell) -> Self {
        self.table_header_cell = Some(cell);
        self
    }

    pub(crate) fn with_table_model(mut self, model: crate::table::Model) -> Self {
        self.table_model = Some(model);
        self
    }

    pub(crate) fn with_table_edit(mut self, edit: crate::table::Edit) -> Self {
        self.table_edit = Some(edit);
        self
    }

    pub(crate) fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    fn with_text_kind(mut self, text_kind: super::TextKind) -> Self {
        self.text_kind = text_kind;
        self
    }

    pub(crate) fn with_layout_axis(mut self, axis: Axis) -> Self {
        if self.role != Role::FloatingPanel && self.role != Role::Scroll {
            self.role = Role::Stack;
        }
        self.axis = Some(axis);
        self
    }

    pub(crate) fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub(crate) fn with_floating_placement(mut self, placement: FloatingPlacement) -> Self {
        self.floating_placement = placement;
        self
    }

    pub(crate) fn with_force_overlay_group(mut self, force: bool) -> Self {
        self.force_overlay_group = force;
        self
    }

    pub(crate) fn with_native_popup_material_preference(
        mut self,
        preference: NativePopupMaterialPreference,
    ) -> Self {
        self.native_popup_material_preference = preference;
        self
    }

    pub(crate) fn with_subject(mut self, subject: subject::Segment) -> Self {
        self.subject = Some(subject);
        self
    }

    pub(crate) fn bind_command<C>(mut self, args: C::Args, source: Source) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        self.binding = Some(Binding::new::<C>(args, source));
        self
    }

    pub(crate) fn bind_trigger(mut self, trigger: command::AnyTrigger, source: Source) -> Self {
        self.binding = Some(Binding::from_trigger(trigger, source));
        self
    }

    pub(crate) fn with_selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub(crate) fn bind_slider_trigger(
        mut self,
        value: f64,
        source: Source,
        slider_trigger: command::AnyValueTrigger<f64>,
    ) -> Self {
        self.binding = Some(Binding::slider(value, source, slider_trigger));
        self
    }

    pub(crate) fn bind_text_trigger(
        mut self,
        text: String,
        source: Source,
        text_trigger: command::AnyValueTrigger<String>,
    ) -> Self {
        self.binding = Some(Binding::text(text, source, text_trigger));
        self
    }

    fn with_control(mut self, control: Control) -> Self {
        self.control = Some(control);
        self
    }

    fn with_virtual_list(mut self, model: crate::virtual_list::Model) -> Self {
        self.virtual_list = Some(model);
        self
    }

    pub fn with_interaction_id(mut self, id: impl Into<interaction::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    fn new(role: Role) -> Self {
        Self {
            role,
            id: None,
            axis: None,
            style: Style::default(),
            floating_placement: FloatingPlacement::Default,
            force_overlay_group: false,
            native_popup_material_preference: NativePopupMaterialPreference::System,
            subject: None,
            label: None,
            text_kind: super::TextKind::Author,
            binding: None,
            control: None,
            focused: false,
            focus_visible: false,
            selected: false,
            active_item: false,
            scroll_offset: interaction::ScrollOffset::default(),
            virtual_list: None,
            provided_row: None,
            table_row: None,
            table_cell: None,
            table_header_cell: None,
            table_model: None,
            table_edit: None,
            table_edit_error: None,
            children: Vec::new(),
        }
    }

    fn with_axis(mut self, axis: Axis) -> Self {
        self.axis = Some(axis);
        self
    }

    fn with_id(mut self, id: impl Into<interaction::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    fn with_binding(mut self, binding: Binding) -> Self {
        self.binding = Some(binding);
        self
    }
}
