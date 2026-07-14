use super::super::{
    binding::Binding,
    control::{Button, Checkbox, Radio, Slider, TextArea, TextBox},
    style::Style,
};
use super::content::{MenuBar, Panel, Scroll};
use super::{
    Axis, Content, FloatingPlacement, NativePopupMaterialPreference, Node, Participation, Role,
    TablePart, WorldText,
};
use crate::{command, context::Source, interaction, subject};

impl Node {
    pub fn root() -> Self {
        Self::new(Content::Root)
    }

    pub fn stack(axis: Axis) -> Self {
        Self::new(Content::Stack).with_axis(axis)
    }

    pub fn menu_bar() -> Self {
        Self::new(Content::MenuBar(MenuBar::Ordinary))
    }

    pub(crate) fn standard_menu_bar() -> Self {
        Self::new(Content::MenuBar(MenuBar::Standard(Vec::new())))
    }

    pub(crate) fn push_standard_menu_extension(
        &mut self,
        extension: super::standard_menu::Extension,
    ) {
        assert!(
            self.standard_menu_extensions().is_some(),
            "standard-menu extensions require a standard menu bar"
        );
        if let Some(extensions) = self.standard_menu_extensions_mut() {
            extensions.push(extension);
        }
    }

    pub fn menu(id: impl Into<interaction::Id>, label: impl Into<String>) -> Self {
        Self::new(Content::Menu).with_id(id).with_label(label)
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
        Self::new(Content::Binding).with_binding(Binding::new::<C>(args, Source::Button))
    }

    pub fn menu_bound_with_args<C>(args: C::Args) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        Self::new(Content::Binding).with_binding(Binding::new::<C>(args, Source::Menu))
    }

    pub(crate) fn resolved_menu_action(action: command::ResolvedAction) -> Self {
        Self::new(Content::Binding).with_binding(Binding::from_resolved(action, Source::Menu))
    }

    pub(crate) fn resolved_bar_action(action: &command::BarAction, show_shortcut: bool) -> Self {
        Self::new(Content::Binding).with_binding(Binding::from_bar_action(action, show_shortcut))
    }

    pub fn separator() -> Self {
        Self::new(Content::Separator)
    }

    pub fn text_area(text: impl Into<String>) -> Self {
        Self::text_area_state(TextArea::new(text))
    }

    pub fn text_area_state(text_area: TextArea) -> Self {
        Self::new(Content::TextArea(text_area))
    }

    pub fn button(label: impl Into<String>) -> Self {
        Self::button_state(Button::new(label))
    }

    pub fn button_state(button: Button) -> Self {
        let label = button.label().to_owned();
        Self::new(Content::Button(button)).with_label(label)
    }

    pub fn checkbox(label: impl Into<String>, checked: bool) -> Self {
        Self::checkbox_state(Checkbox::new(label, checked))
    }

    pub fn checkbox_state(checkbox: Checkbox) -> Self {
        let label = checkbox.label().to_owned();
        Self::new(Content::Checkbox(checkbox)).with_label(label)
    }

    pub fn radio(label: impl Into<String>, selected: bool) -> Self {
        Self::radio_state(Radio::new(label, selected))
    }

    pub fn radio_state(radio: Radio) -> Self {
        let label = radio.label().to_owned();
        Self::new(Content::Radio(radio)).with_label(label)
    }

    pub fn slider(label: impl Into<String>, value: f64, start: f64, end: f64) -> Self {
        Self::slider_state(Slider::new(label, value, start, end))
    }

    pub fn slider_state(slider: Slider) -> Self {
        let label = slider.display_label();
        Self::new(Content::Slider(slider)).with_label(label)
    }

    pub fn text_box(text: impl Into<String>) -> Self {
        Self::text_box_state(TextBox::new(text))
    }

    pub fn text_box_state(text_box: TextBox) -> Self {
        Self::new(Content::TextBox {
            model: text_box,
            commit: None,
        })
    }

    pub(crate) fn text_box_state_with_commit(
        text_box: TextBox,
        commit: super::super::TextCommit,
    ) -> Self {
        Self::new(Content::TextBox {
            model: text_box,
            commit: Some(commit),
        })
    }

    pub fn panel() -> Self {
        Self::new(Content::Panel)
    }

    pub fn scroll() -> Self {
        Self::new(Content::Scroll(Scroll::Ordinary {
            offset: interaction::ScrollOffset::default(),
        }))
        .with_axis(Axis::Vertical)
    }

    pub(crate) fn table_scroll(model: crate::table::Model) -> Self {
        Self::new(Content::Scroll(Scroll::Table {
            model,
            offset: interaction::ScrollOffset::default(),
        }))
        .with_axis(Axis::Horizontal)
    }

    pub(crate) fn virtual_list(model: crate::virtual_list::Model) -> Self {
        let id = model.id();
        Self::new(Content::VirtualList {
            model,
            offset: interaction::ScrollOffset::default(),
        })
        .with_id(id)
        .with_axis(Axis::Vertical)
    }

    pub(crate) fn table(id: interaction::Id) -> Self {
        Self::new(Content::Table)
            .with_axis(Axis::Vertical)
            .with_interaction_id(id)
    }

    pub fn floating_panel(id: impl Into<interaction::Id>) -> Self {
        Self::new(Content::FloatingPanel(Panel::interactive())).with_id(id)
    }

    pub fn label(label: impl Into<String>) -> Self {
        Self::new(Content::Label).with_label(label)
    }

    /// Creates text supplied by the world outside the program. Its overflow
    /// behavior must be explicit because the program cannot promise it fits.
    pub fn world_text(label: impl Into<String>, overflow: crate::text::Overflow) -> Self {
        Self::world_text_with_policy(label, super::super::Wrap::None, overflow)
    }

    pub(crate) fn wrapped_world_text(label: impl Into<String>, wrap: super::super::Wrap) -> Self {
        Self::world_text_with_policy(label, wrap, crate::text::Overflow::Clip)
    }

    pub(crate) fn world_text_with_policy(
        label: impl Into<String>,
        wrap: super::super::Wrap,
        overflow: crate::text::Overflow,
    ) -> Self {
        Self::new(Content::Label)
            .with_label(label)
            .with_text_kind(super::TextKind::World(WorldText::new(wrap, overflow)))
    }

    pub(crate) fn with_world_text_policy(
        mut self,
        label: impl Into<String>,
        wrap: super::super::Wrap,
        overflow: crate::text::Overflow,
    ) -> Self {
        self.label = Some(label.into());
        self.text_kind = super::TextKind::World(WorldText::new(wrap, overflow));
        self
    }

    pub(crate) fn with_world_text_alignment(mut self, align: super::super::Align) -> Self {
        if let super::TextKind::World(world) = &mut self.text_kind {
            world.align = align;
        }
        self
    }

    pub(crate) fn section_header(label: impl Into<String>) -> Self {
        Self::new(Content::SectionHeader).with_label(label)
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
        self.participation = Some(Participation::Table(match self.role() {
            Role::Checkbox if self.binding.is_some() => TablePart::Toggle,
            Role::Checkbox => TablePart::PassiveToggle,
            Role::Button => TablePart::Action,
            _ => TablePart::Cell,
        }));
        self
    }

    pub(crate) fn with_table_header_cell(mut self, cell: crate::table::HeaderCell) -> Self {
        self.table_header_cell = Some(cell);
        self.participation = Some(Participation::Table(if self.binding.is_some() {
            TablePart::HeaderControl
        } else {
            TablePart::Header
        }));
        self
    }

    pub(crate) fn with_table_header_band(mut self) -> Self {
        self.participation = Some(Participation::Table(TablePart::HeaderBand));
        self
    }

    pub(crate) fn with_auxiliary_text_participation(mut self) -> Self {
        self.participation = Some(Participation::AuxiliaryText);
        self
    }

    pub(crate) fn with_table_header_presentation(
        mut self,
        presentation: crate::table::HeaderPresentation,
    ) -> Self {
        self.table_header_presentation = Some(presentation);
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
        if self.role() != Role::FloatingPanel && self.role() != Role::Scroll {
            self.content = Content::Stack;
        }
        self.axis = Some(axis);
        self
    }

    pub(crate) fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub(crate) fn with_floating_placement(mut self, placement: FloatingPlacement) -> Self {
        if let Some(panel) = self.content.panel_mut() {
            panel.placement = placement;
        } else {
            debug_assert!(false, "floating placement requires a floating panel");
        }
        self
    }

    pub(crate) fn with_panel_placement(
        mut self,
        anchor: crate::geometry::placement::Anchor,
        available: crate::geometry::Rect,
    ) -> Self {
        if let Some(panel) = self.content.panel_mut() {
            panel.attachment = Some(super::PanelAttachment::Geometry {
                anchor,
                available: Some(available),
            });
        } else {
            debug_assert!(false, "panel placement requires a floating panel");
        }
        self
    }

    pub(crate) fn with_panel_anchor(mut self, anchor: crate::geometry::placement::Anchor) -> Self {
        if let Some(panel) = self.content.panel_mut() {
            panel.attachment = Some(super::PanelAttachment::Geometry {
                anchor,
                available: None,
            });
        } else {
            debug_assert!(false, "panel anchor requires a floating panel");
        }
        self
    }

    pub(crate) fn with_pointer_panel_anchor(mut self, point: crate::geometry::Point) -> Self {
        debug_assert_eq!(self.role(), Role::FloatingPanel);
        if let Some(panel) = self.content.panel_mut() {
            panel.attachment = Some(super::PanelAttachment::Pointer(point));
        }
        self
    }

    pub(crate) fn with_panel_anchor_element(mut self, id: impl Into<interaction::Id>) -> Self {
        debug_assert_eq!(self.role(), Role::FloatingPanel);
        if let Some(panel) = self.content.panel_mut() {
            panel.attachment = Some(super::PanelAttachment::Element(id.into()));
        }
        self
    }

    pub(crate) fn with_popup_context(
        mut self,
        fingerprint: crate::popup::ContextFingerprint,
    ) -> Self {
        if let Some(panel) = self.content.panel_mut() {
            panel.popup_context = Some(fingerprint);
        } else {
            debug_assert!(false, "popup context requires a floating panel");
        }
        self
    }

    pub(crate) fn with_panel_policy(mut self, policy: super::PanelPolicy) -> Self {
        debug_assert_eq!(self.role(), Role::FloatingPanel);
        if let Some(panel) = self.content.panel_mut() {
            panel.policy = policy;
        }
        self
    }

    pub(crate) fn with_force_overlay_group(mut self, force: bool) -> Self {
        if let Some(panel) = self.content.panel_mut() {
            panel.force_overlay_group = force;
        } else {
            debug_assert!(false, "overlay grouping requires a floating panel");
        }
        self
    }

    pub(crate) fn with_native_popup_material_preference(
        mut self,
        preference: NativePopupMaterialPreference,
    ) -> Self {
        if let Some(panel) = self.content.panel_mut() {
            panel.native_material = preference;
        } else {
            debug_assert!(false, "native popup material requires a floating panel");
        }
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
        self.participation = participation_for_source(source);
        self.binding = Some(Binding::new::<C>(args, source));
        self
    }

    pub(crate) fn bind_trigger(mut self, trigger: command::AnyTrigger, source: Source) -> Self {
        self.participation = participation_for_source(source);
        self.binding = Some(Binding::from_trigger(trigger, source));
        self
    }

    pub(crate) fn bind_context_trigger(mut self, trigger: command::AnyTrigger) -> Self {
        self.context_binding = Some(Binding::from_trigger(trigger, Source::Button));
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
        self.participation = participation_for_source(source);
        self.binding = Some(Binding::slider(value, source, slider_trigger));
        self
    }

    pub fn with_interaction_id(mut self, id: impl Into<interaction::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub(crate) fn with_context_menu(mut self) -> Self {
        self.context_menu = true;
        self
    }

    fn new(content: Content) -> Self {
        Self {
            content,
            id: None,
            axis: None,
            style: Style::default(),
            subject: None,
            label: None,
            text_kind: super::TextKind::Author,
            binding: None,
            context_binding: None,
            focus_presentation: super::super::focus::Presentation::default(),
            selected: false,
            active_item: false,
            provided_row: None,
            table_row: None,
            table_cell: None,
            table_header_cell: None,
            table_header_presentation: None,
            participation: None,
            context_menu: false,
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
        self.participation = participation_for_source(binding.source());
        self.binding = Some(binding);
        self
    }
}

fn participation_for_source(source: Source) -> Option<Participation> {
    match source {
        Source::Menu => Some(Participation::MenuRow),
        Source::Palette => Some(Participation::PaletteRow),
        _ => None,
    }
}
