use std::any::TypeId;

use crate::text;

use super::{
    command as framework_command,
    context::{Context as CommandContext, Source},
    diagnostics::Diagnostics,
    interaction, responder, session, window,
};

mod action;
mod binding;
mod control;
mod style;

pub use action::{Action, FocusDirection};
pub use binding::Binding;
pub use control::{Button, Checkbox, Control, Radio, Slider, TextArea, TextBox, Wrap};
pub use style::{Align, Dimension, Padding, Style};

#[derive(Clone)]
pub struct View {
    root: Node,
}

#[derive(Clone)]
pub struct Presentation {
    window: window::Id,
    view: View,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Context {
    window: window::Id,
    diagnostics: Diagnostics,
    interaction: interaction::Interaction,
}

#[derive(Clone)]
pub struct Node {
    role: Role,
    id: Option<interaction::Id>,
    axis: Option<Axis>,
    style: Style,
    label: Option<String>,
    binding: Option<Binding>,
    control: Option<Control>,
    text_area: Option<TextArea>,
    children: Vec<Node>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Root,
    Stack,
    MenuBar,
    Menu,
    Command,
    Separator,
    TextArea,
    Button,
    Checkbox,
    Radio,
    Slider,
    TextBox,
    Panel,
    Popup,
    Label,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl View {
    pub fn new(root: Node) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Node {
        &self.root
    }

    pub fn bindings(&self) -> Vec<&Binding> {
        let mut bindings = Vec::new();
        self.root.collect_bindings(&mut bindings);
        bindings
    }

    pub fn command<C: framework_command::Command>(&self) -> Option<&Binding> {
        let command_type = TypeId::of::<C>();
        self.bindings()
            .into_iter()
            .find(|binding| binding.command_type() == command_type)
    }

    pub fn text_areas(&self) -> Vec<&TextArea> {
        let mut text_areas = Vec::new();
        self.root.collect_text_areas(&mut text_areas);
        text_areas
    }

    pub fn buttons(&self) -> Vec<&Button> {
        let mut buttons = Vec::new();
        self.root.collect_buttons(&mut buttons);
        buttons
    }

    pub fn checkboxes(&self) -> Vec<&Checkbox> {
        let mut checkboxes = Vec::new();
        self.root.collect_checkboxes(&mut checkboxes);
        checkboxes
    }

    pub fn radios(&self) -> Vec<&Radio> {
        let mut radios = Vec::new();
        self.root.collect_radios(&mut radios);
        radios
    }

    pub fn sliders(&self) -> Vec<&Slider> {
        let mut sliders = Vec::new();
        self.root.collect_sliders(&mut sliders);
        sliders
    }

    pub fn text_boxes(&self) -> Vec<&TextBox> {
        let mut text_boxes = Vec::new();
        self.root.collect_text_boxes(&mut text_boxes);
        text_boxes
    }

    pub fn contains_focus(&self, focus: session::Focus) -> bool {
        self.root.contains_focus(focus)
    }

    pub fn focus_order(&self) -> Vec<session::Focus> {
        let mut order = Vec::new();
        self.root.collect_focus_order(&mut order);
        order
    }

    pub fn next_focus(
        &self,
        current: Option<session::Focus>,
        direction: FocusDirection,
    ) -> Option<session::Focus> {
        let order = self.focus_order();
        if order.is_empty() {
            return None;
        }

        let index =
            current.and_then(|focus| order.iter().position(|candidate| *candidate == focus));
        match (index, direction) {
            (Some(index), FocusDirection::Forward) => Some(order[(index + 1) % order.len()]),
            (Some(0), FocusDirection::Backward) => order.last().copied(),
            (Some(index), FocusDirection::Backward) => Some(order[index - 1]),
            (None, FocusDirection::Forward) => order.first().copied(),
            (None, FocusDirection::Backward) => order.last().copied(),
        }
    }

    pub(super) fn text_commit_action(&self, focus: session::Focus, text: String) -> Option<Action> {
        self.root.text_commit_action(focus, text)
    }

    pub(super) fn text_box_text(&self, focus: session::Focus) -> Option<&str> {
        self.root.text_box_for_focus(focus).map(TextBox::text)
    }

    pub(super) fn text_input_target(&self, focus: session::Focus) -> Option<interaction::Target> {
        self.root.text_input_target_for_focus(focus)
    }

    pub fn menus(&self) -> Vec<&Node> {
        let mut menus = Vec::new();
        self.root.collect_menus(&mut menus);
        menus
    }

    pub fn labels(&self) -> Vec<&str> {
        let mut labels = Vec::new();
        self.root.collect_labels(&mut labels);
        labels
    }

    pub fn popups(&self) -> Vec<&Node> {
        let mut popups = Vec::new();
        self.root.collect_popups(&mut popups);
        popups
    }

    pub(super) fn project_interaction(&mut self, interaction: &interaction::Interaction) {
        self.root.project_interaction(interaction);

        if let Some(menu) = interaction.open_menu() {
            let popup = self
                .root
                .popup_for_menu(menu)
                .unwrap_or_else(|| Node::popup(menu.id(), menu.label()));
            self.root.children.push(popup);
        }
    }

    pub(super) fn project_focus(&mut self, focus: Option<session::Focus>) {
        self.root.project_focus(focus);
    }

    pub(super) fn resolve_commands(
        &mut self,
        registry: &framework_command::Registry,
        chain: &mut responder::Chain<'_, impl super::state::State>,
        cx: &CommandContext,
    ) {
        self.root.resolve_commands(registry, chain, cx);
        self.root.prune_hidden_commands();
    }

    pub(super) fn visit_bindings_mut(&mut self, mut visit: impl FnMut(&mut Binding)) {
        self.root.visit_bindings_mut(&mut visit);
    }
}

impl Presentation {
    pub(super) fn new(window: window::Id, view: View) -> Self {
        Self { window, view }
    }

    pub fn window(&self) -> window::Id {
        self.window
    }

    pub fn view(&self) -> &View {
        &self.view
    }

    pub fn into_view(self) -> View {
        self.view
    }
}

impl Context {
    pub(super) fn new(
        window: window::Id,
        diagnostics: Diagnostics,
        interaction: interaction::Interaction,
    ) -> Self {
        Self {
            window,
            diagnostics,
            interaction,
        }
    }

    pub fn window(&self) -> window::Id {
        self.window
    }

    pub fn diagnostics(&self) -> &Diagnostics {
        &self.diagnostics
    }

    pub fn interaction(&self) -> &interaction::Interaction {
        &self.interaction
    }
}

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

    pub fn command<C>() -> Self
    where
        C: framework_command::Command<Args = ()>,
    {
        Self::command_with_args::<C>(())
    }

    pub fn menu_command<C>() -> Self
    where
        C: framework_command::Command<Args = ()>,
    {
        Self::menu_command_with_args::<C>(())
    }

    pub fn command_with_args<C>(args: C::Args) -> Self
    where
        C: framework_command::Command,
        C::Args: Clone,
    {
        Self::new(Role::Command).with_binding(Binding::new::<C>(args, Source::Button))
    }

    pub fn menu_command_with_args<C>(args: C::Args) -> Self
    where
        C: framework_command::Command,
        C::Args: Clone,
    {
        Self::new(Role::Command).with_binding(Binding::new::<C>(args, Source::Menu))
    }

    pub fn separator() -> Self {
        Self::new(Role::Separator)
    }

    pub fn text_area(text: impl Into<String>) -> Self {
        Self::new(Role::TextArea).with_text_area(TextArea::new(text))
    }

    pub fn text_area_state(text_area: TextArea) -> Self {
        Self::new(Role::TextArea).with_text_area(text_area)
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
        let label = checkbox.display_label();
        Self::new(Role::Checkbox)
            .with_label(label)
            .with_control(Control::Checkbox(checkbox))
    }

    pub fn radio(label: impl Into<String>, selected: bool) -> Self {
        Self::radio_state(Radio::new(label, selected))
    }

    pub fn radio_state(radio: Radio) -> Self {
        let label = radio.display_label();
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

    pub fn popup(id: impl Into<interaction::Id>, label: impl Into<String>) -> Self {
        Self::new(Role::Popup).with_id(id).with_label(label)
    }

    pub fn label(label: impl Into<String>) -> Self {
        Self::new(Role::Label).with_label(label)
    }

    pub fn child(mut self, child: Node) -> Self {
        self.children.push(child);
        self
    }

    pub(super) fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub(super) fn with_layout_axis(mut self, axis: Axis) -> Self {
        self.role = Role::Stack;
        self.axis = Some(axis);
        self
    }

    pub(super) fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub(super) fn bind_command<C>(mut self, args: C::Args, source: Source) -> Self
    where
        C: framework_command::Command,
        C::Args: Clone,
    {
        self.binding = Some(Binding::new::<C>(args, source));
        self
    }

    pub(super) fn bind_trigger(
        mut self,
        trigger: framework_command::AnyTrigger,
        source: Source,
    ) -> Self {
        self.binding = Some(Binding::from_trigger(trigger, source));
        self
    }

    pub(super) fn bind_slider_trigger(
        mut self,
        value: f64,
        source: Source,
        slider_trigger: framework_command::AnyValueTrigger<f64>,
    ) -> Self {
        self.binding = Some(Binding::slider(value, source, slider_trigger));
        self
    }

    pub(super) fn bind_text_trigger(
        mut self,
        text: String,
        source: Source,
        text_trigger: framework_command::AnyValueTrigger<String>,
    ) -> Self {
        self.binding = Some(Binding::text(text, source, text_trigger));
        self
    }

    pub fn with_text_area(mut self, text_area: TextArea) -> Self {
        self.text_area = Some(text_area);
        self
    }

    pub(super) fn with_control(mut self, control: Control) -> Self {
        self.control = Some(control);
        self
    }

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
        self.text_area.as_ref()
    }

    pub fn is_focused(&self) -> bool {
        self.text_area.as_ref().is_some_and(TextArea::is_focused)
            || self.text_box_model().is_some_and(TextBox::is_focused)
    }

    pub fn button_model(&self) -> Option<&Button> {
        match self.control.as_ref()? {
            Control::Button(button) => Some(button),
            Control::Checkbox(_) | Control::Radio(_) | Control::Slider(_) | Control::TextBox(_) => {
                None
            }
        }
    }

    pub fn checkbox_model(&self) -> Option<&Checkbox> {
        match self.control.as_ref()? {
            Control::Checkbox(checkbox) => Some(checkbox),
            Control::Button(_) | Control::Radio(_) | Control::Slider(_) | Control::TextBox(_) => {
                None
            }
        }
    }

    pub fn radio_model(&self) -> Option<&Radio> {
        match self.control.as_ref()? {
            Control::Radio(radio) => Some(radio),
            Control::Button(_)
            | Control::Checkbox(_)
            | Control::Slider(_)
            | Control::TextBox(_) => None,
        }
    }

    pub fn slider_model(&self) -> Option<&Slider> {
        match self.control.as_ref()? {
            Control::Slider(slider) => Some(slider),
            Control::Button(_) | Control::Checkbox(_) | Control::Radio(_) | Control::TextBox(_) => {
                None
            }
        }
    }

    pub fn text_box_model(&self) -> Option<&TextBox> {
        match self.control.as_ref()? {
            Control::TextBox(text_box) => Some(text_box),
            Control::Button(_) | Control::Checkbox(_) | Control::Radio(_) | Control::Slider(_) => {
                None
            }
        }
    }

    pub fn pointer_target(&self) -> Option<interaction::Target> {
        self.pointer_target_with_path(None)
    }

    pub fn pointer_target_at_path(&self, path: &[usize]) -> Option<interaction::Target> {
        self.pointer_target_with_path(Some(path))
    }

    fn pointer_target_with_path(&self, path: Option<&[usize]>) -> Option<interaction::Target> {
        if let Some(binding) = &self.binding {
            let target = self
                .id
                .map(|id| binding.element_pointer_target(id))
                .or_else(|| path.map(|path| binding.path_pointer_target(path)))?;

            return Some(if self.role == Role::Slider {
                target.with_capture()
            } else {
                target
            });
        }

        match self.role {
            Role::Menu => self
                .id
                .zip(self.label.as_ref())
                .map(|(id, label)| interaction::Target::menu(id, label.clone())),
            Role::TextArea => self.id.map(interaction::Target::text_area_id).or_else(|| {
                self.text_area
                    .as_ref()
                    .and_then(TextArea::focus)
                    .map(interaction::Target::text_area)
            }),
            Role::TextBox => self
                .text_box_model()
                .and_then(TextBox::focus)
                .map(interaction::Target::text_area),
            Role::Popup => self
                .id
                .zip(self.label.as_ref())
                .map(|(id, label)| interaction::Target::popup(id, label.clone())),
            Role::Label => self
                .id
                .zip(self.label.as_ref())
                .map(|(id, label)| interaction::Target::label(id, label.clone())),
            Role::Root
            | Role::Stack
            | Role::MenuBar
            | Role::Command
            | Role::Separator
            | Role::Button
            | Role::Checkbox
            | Role::Radio
            | Role::Slider
            | Role::Panel => None,
        }
    }

    pub fn id(&self) -> Option<interaction::Id> {
        self.id
    }

    pub fn with_interaction_id(mut self, id: impl Into<interaction::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn label_target(&self) -> Option<interaction::Target> {
        if self.role != Role::Label {
            return None;
        }

        self.id
            .zip(self.label.as_ref())
            .map(|(id, label)| interaction::Target::label(id, label.clone()))
    }

    pub fn pointer_move_action(&self) -> Option<Action> {
        Some(Action::pointer_move(Some(self.pointer_target()?)))
    }

    pub fn pointer_down_action(&self) -> Option<Action> {
        Some(Action::pointer_down(self.pointer_target()?))
    }

    pub fn pointer_up_action(&self) -> Option<Action> {
        Some(Action::pointer_up(
            Some(self.pointer_target()?),
            self.pointer_activation_action(),
        ))
    }

    pub fn scroll_action(&self, delta: interaction::ScrollDelta) -> Option<Action> {
        Some(Action::scroll(self.pointer_target()?, delta))
    }

    pub fn text_pointer_down_action(&self, position: text::TextPosition) -> Option<Action> {
        if self.role != Role::TextArea {
            return None;
        }

        let text_area = self.text_area.as_ref()?;
        let target = self.pointer_target()?;

        Some(Action::sequence([
            Action::pointer_down(target),
            text_area.focus_action()?,
            Action::text_edit(text::edit::Edit::pointer(
                text::PointerEditKind::Click,
                position,
            )),
        ]))
    }

    pub fn text_pointer_drag_action(&self, position: text::TextPosition) -> Option<Action> {
        if self.role != Role::TextArea {
            return None;
        }

        let target = self.pointer_target()?;

        Some(Action::pointer_drag(
            Some(target.clone()),
            target,
            Some(Action::text_edit(text::edit::Edit::pointer(
                text::PointerEditKind::Drag,
                position,
            ))),
        ))
    }

    pub fn menu_action(&self) -> Option<Action> {
        if self.role != Role::Menu {
            return None;
        }

        Some(Action::toggle_menu(interaction::Menu::new(
            self.id?,
            self.label.as_deref()?,
        )))
    }

    pub fn children(&self) -> &[Node] {
        &self.children
    }

    fn new(role: Role) -> Self {
        Self {
            role,
            id: None,
            axis: None,
            style: Style::default(),
            label: None,
            binding: None,
            control: None,
            text_area: None,
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

    fn pointer_activation_action(&self) -> Option<Action> {
        if let Some(binding) = &self.binding {
            return binding.is_enabled().then(|| Action::command(binding));
        }

        match self.role {
            Role::Menu => self.menu_action(),
            Role::TextArea => self.text_area.as_ref().and_then(TextArea::focus_action),
            Role::TextBox => self.text_box_model().and_then(TextBox::focus_action),
            Role::Root
            | Role::Stack
            | Role::MenuBar
            | Role::Command
            | Role::Separator
            | Role::Button
            | Role::Checkbox
            | Role::Radio
            | Role::Slider
            | Role::Panel
            | Role::Popup
            | Role::Label => None,
        }
    }

    fn collect_bindings<'a>(&'a self, bindings: &mut Vec<&'a Binding>) {
        if let Some(binding) = &self.binding {
            bindings.push(binding);
        }

        for child in &self.children {
            child.collect_bindings(bindings);
        }
    }

    fn collect_text_areas<'a>(&'a self, text_areas: &mut Vec<&'a TextArea>) {
        if let Some(text_area) = &self.text_area {
            text_areas.push(text_area);
        }

        for child in &self.children {
            child.collect_text_areas(text_areas);
        }
    }

    fn collect_buttons<'a>(&'a self, buttons: &mut Vec<&'a Button>) {
        if let Some(button) = self.button_model() {
            buttons.push(button);
        }

        for child in &self.children {
            child.collect_buttons(buttons);
        }
    }

    fn collect_checkboxes<'a>(&'a self, checkboxes: &mut Vec<&'a Checkbox>) {
        if let Some(checkbox) = self.checkbox_model() {
            checkboxes.push(checkbox);
        }

        for child in &self.children {
            child.collect_checkboxes(checkboxes);
        }
    }

    fn collect_radios<'a>(&'a self, radios: &mut Vec<&'a Radio>) {
        if let Some(radio) = self.radio_model() {
            radios.push(radio);
        }

        for child in &self.children {
            child.collect_radios(radios);
        }
    }

    fn collect_sliders<'a>(&'a self, sliders: &mut Vec<&'a Slider>) {
        if let Some(slider) = self.slider_model() {
            sliders.push(slider);
        }

        for child in &self.children {
            child.collect_sliders(sliders);
        }
    }

    fn collect_text_boxes<'a>(&'a self, text_boxes: &mut Vec<&'a TextBox>) {
        if let Some(text_box) = self.text_box_model() {
            text_boxes.push(text_box);
        }

        for child in &self.children {
            child.collect_text_boxes(text_boxes);
        }
    }

    fn contains_focus(&self, focus: session::Focus) -> bool {
        self.text_area.as_ref().and_then(TextArea::focus) == Some(focus)
            || self.text_box_model().and_then(TextBox::focus) == Some(focus)
            || self
                .children
                .iter()
                .any(|child| child.contains_focus(focus))
    }

    fn collect_focus_order(&self, order: &mut Vec<session::Focus>) {
        if let Some(focus) = self.text_area.as_ref().and_then(TextArea::focus) {
            push_focus(order, focus);
        }

        if let Some(focus) = self.text_box_model().and_then(TextBox::focus) {
            push_focus(order, focus);
        }

        for child in &self.children {
            child.collect_focus_order(order);
        }
    }

    fn text_commit_action(&self, focus: session::Focus, text: String) -> Option<Action> {
        if self.text_box_model().and_then(TextBox::focus) == Some(focus) {
            return self
                .binding
                .as_ref()
                .and_then(|binding| binding.text_action(text));
        }

        self.children
            .iter()
            .find_map(|child| child.text_commit_action(focus, text.clone()))
    }

    fn text_box_for_focus(&self, focus: session::Focus) -> Option<&TextBox> {
        if let Some(text_box) = self
            .text_box_model()
            .filter(|text_box| text_box.focus() == Some(focus))
        {
            return Some(text_box);
        }

        self.children
            .iter()
            .find_map(|child| child.text_box_for_focus(focus))
    }

    fn text_input_target_for_focus(&self, focus: session::Focus) -> Option<interaction::Target> {
        if self.text_area.as_ref().and_then(TextArea::focus) == Some(focus) {
            return self.pointer_target();
        }

        if self.text_box_model().and_then(TextBox::focus) == Some(focus) {
            return self.pointer_target();
        }

        self.children
            .iter()
            .find_map(|child| child.text_input_target_for_focus(focus))
    }

    fn collect_menus<'a>(&'a self, menus: &mut Vec<&'a Node>) {
        if self.role == Role::Menu {
            menus.push(self);
        }

        for child in &self.children {
            child.collect_menus(menus);
        }
    }

    fn collect_labels<'a>(&'a self, labels: &mut Vec<&'a str>) {
        if let Some(label) = &self.label {
            labels.push(label);
        }

        for child in &self.children {
            child.collect_labels(labels);
        }
    }

    fn collect_popups<'a>(&'a self, popups: &mut Vec<&'a Node>) {
        if self.role == Role::Popup {
            popups.push(self);
        }

        for child in &self.children {
            child.collect_popups(popups);
        }
    }

    fn resolve_commands(
        &mut self,
        registry: &framework_command::Registry,
        chain: &mut responder::Chain<'_, impl super::state::State>,
        cx: &CommandContext,
    ) {
        if let Some(binding) = &mut self.binding {
            binding.resolve(registry, chain, cx);
        }

        for child in &mut self.children {
            child.resolve_commands(registry, chain, cx);
        }
    }

    fn prune_hidden_commands(&mut self) {
        for child in &mut self.children {
            child.prune_hidden_commands();
        }

        self.children.retain(|child| !child.is_hidden_binding());
    }

    fn visit_bindings_mut(&mut self, visit: &mut impl FnMut(&mut Binding)) {
        if let Some(binding) = &mut self.binding {
            visit(binding);
        }

        for child in &mut self.children {
            child.visit_bindings_mut(visit);
        }
    }

    fn project_interaction(&mut self, interaction: &interaction::Interaction) {
        let text_area_target = if self.role == Role::TextArea {
            self.pointer_target()
        } else {
            None
        };
        if let Some(text_area) = &mut self.text_area {
            text_area.project_interaction(interaction, text_area_target.as_ref());
        }

        if let Some(Control::TextBox(text_box)) = &mut self.control {
            text_box.project_interaction(interaction);
        }

        for child in &mut self.children {
            child.project_interaction(interaction);
        }
    }

    fn project_focus(&mut self, focus: Option<session::Focus>) {
        if let Some(text_area) = &mut self.text_area {
            text_area.project_focus(focus);
        }

        if let Some(Control::TextBox(text_box)) = &mut self.control {
            text_box.project_focus(focus);
        }

        for child in &mut self.children {
            child.project_focus(focus);
        }
    }

    fn popup_for_menu(&self, menu: &interaction::Menu) -> Option<Node> {
        if self.role == Role::Menu && self.id == Some(menu.id()) {
            let mut popup = Node::popup(menu.id(), menu.label());
            popup.children = self.children.clone();
            return Some(popup);
        }

        self.children
            .iter()
            .find_map(|child| child.popup_for_menu(menu))
    }
}

fn push_focus(order: &mut Vec<session::Focus>, focus: session::Focus) {
    if !order.contains(&focus) {
        order.push(focus);
    }
}
