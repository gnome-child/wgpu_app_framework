use std::{marker::PhantomData, ops::RangeInclusive};

use crate::text;

use super::{command, context::Source, interaction, session, view};

pub trait Widget {
    fn into_node(self) -> view::Node;
}

pub struct Ui {
    nodes: Vec<view::Node>,
}

pub struct Root {
    node: view::Node,
}

pub struct Element {
    node: view::Node,
    layout: Layout,
    width: Option<Size>,
    height: Option<Size>,
}

pub struct Stack {
    node: view::Node,
}

pub struct MenuBar {
    node: view::Node,
}

pub struct Menu {
    node: view::Node,
}

pub struct Command<C: command::Command> {
    args: C::Args,
    placement: CommandPlacement,
    _command: PhantomData<C>,
}

pub struct Button {
    label: String,
    binding: Option<TriggerBinding>,
}

pub struct Checkbox {
    label: String,
    checked: bool,
    binding: Option<TriggerBinding>,
}

pub struct Radio {
    label: String,
    selected: bool,
    binding: Option<TriggerBinding>,
}

pub struct Slider {
    label: String,
    value: f64,
    range: RangeInclusive<f64>,
    binding: Option<SliderBinding>,
}

pub struct TextBox {
    text: String,
    placeholder: Option<String>,
    focus: Option<session::Focus>,
    binding: Option<TextBoxBinding>,
}

pub struct Separator;

pub struct TextArea {
    buffer: text::Buffer,
    edit_state: text::edit::State,
    wrap: view::Wrap,
    id: Option<interaction::Id>,
    focus: Option<session::Focus>,
}

pub struct Panel {
    node: view::Node,
}

pub struct Label {
    text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    direction: Direction,
    gap: i32,
    padding: Padding,
    align_items: Align,
    justify_content: Align,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Fit,
    Grow,
    Fixed(i32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Padding {
    top: i32,
    right: i32,
    bottom: i32,
    left: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

struct TriggerBinding {
    trigger: command::AnyTrigger,
    source: Source,
}

enum SliderBinding {
    Fixed(TriggerBinding),
    Change(SliderChangeBinding),
}

struct SliderChangeBinding {
    trigger: command::AnyValueTrigger<f64>,
    source: Source,
}

struct TextBoxBinding {
    trigger: command::AnyValueTrigger<String>,
    source: Source,
}

#[derive(Clone, Copy)]
enum CommandPlacement {
    Button,
    Menu,
}

impl Widget for view::Node {
    fn into_node(self) -> view::Node {
        self
    }
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

    pub fn row(&mut self, children: impl FnOnce(&mut Ui)) -> &mut Self {
        self.add(Element::new().row().children(children))
    }

    pub fn column(&mut self, children: impl FnOnce(&mut Ui)) -> &mut Self {
        self.add(Element::new().column().children(children))
    }

    pub fn menu_bar(&mut self, children: impl FnOnce(&mut Ui)) -> &mut Self {
        self.add(MenuBar::new().children(children))
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

    fn into_nodes(self) -> Vec<view::Node> {
        self.nodes
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}

impl Root {
    pub fn new() -> Self {
        Self {
            node: view::Node::root(),
        }
    }

    pub fn child(mut self, child: impl Widget) -> Self {
        self.node = self.node.child(child.into_node());
        self
    }

    pub fn children(mut self, children: impl FnOnce(&mut Ui)) -> Self {
        let mut ui = Ui::new();
        children(&mut ui);
        for child in ui.into_nodes() {
            self.node = self.node.child(child);
        }
        self
    }
}

impl Default for Root {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Root {
    fn into_node(self) -> view::Node {
        self.node
    }
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

    pub fn width(mut self, size: Size) -> Self {
        self.width = Some(size);
        self
    }

    pub fn height(mut self, size: Size) -> Self {
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

    pub fn width_state(&self) -> Option<Size> {
        self.width
    }

    pub fn height_state(&self) -> Option<Size> {
        self.height
    }

    fn apply_layout_direction(mut self) -> Self {
        self.node = match self.layout.direction {
            Direction::Row => self.node.with_layout_axis(view::Axis::Horizontal),
            Direction::Column => self.node.with_layout_axis(view::Axis::Vertical),
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
            .with_gap(self.layout.gap)
            .with_padding(self.layout.padding.into_view_padding())
            .with_align_items(self.layout.align_items.into_view_align())
            .with_justify_content(self.layout.justify_content.into_view_align());

        if let Some(width) = self.width {
            style = style.with_width(width.into_view_dimension());
        }
        if let Some(height) = self.height {
            style = style.with_height(height.into_view_dimension());
        }

        self.node.with_style(style)
    }
}

impl Stack {
    pub fn vertical() -> Self {
        Self::new(view::Axis::Vertical)
    }

    pub fn horizontal() -> Self {
        Self::new(view::Axis::Horizontal)
    }

    pub fn child(mut self, child: impl Widget) -> Self {
        self.node = self.node.child(child.into_node());
        self
    }

    pub fn children(mut self, children: impl FnOnce(&mut Ui)) -> Self {
        let mut ui = Ui::new();
        children(&mut ui);
        for child in ui.into_nodes() {
            self.node = self.node.child(child);
        }
        self
    }

    fn new(axis: view::Axis) -> Self {
        Self {
            node: view::Node::stack(axis),
        }
    }
}

impl Widget for Stack {
    fn into_node(self) -> view::Node {
        self.node
    }
}

impl MenuBar {
    pub fn new() -> Self {
        Self {
            node: view::Node::menu_bar(),
        }
    }

    pub fn child(mut self, child: impl Widget) -> Self {
        self.node = self.node.child(child.into_node());
        self
    }

    pub fn children(mut self, children: impl FnOnce(&mut Ui)) -> Self {
        let mut ui = Ui::new();
        children(&mut ui);
        for child in ui.into_nodes() {
            self.node = self.node.child(child);
        }
        self
    }
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for MenuBar {
    fn into_node(self) -> view::Node {
        self.node
    }
}

impl Menu {
    pub fn new(id: impl Into<interaction::Id>, label: impl Into<String>) -> Self {
        Self {
            node: view::Node::menu(id, label),
        }
    }

    pub fn child(mut self, child: impl Widget) -> Self {
        self.node = self.node.child(child.into_node());
        self
    }

    pub fn children(mut self, children: impl FnOnce(&mut Ui)) -> Self {
        let mut ui = Ui::new();
        children(&mut ui);
        for child in ui.into_nodes() {
            self.node = self.node.child(child);
        }
        self
    }
}

impl Widget for Menu {
    fn into_node(self) -> view::Node {
        self.node
    }
}

impl<C> Command<C>
where
    C: command::Command<Args = ()>,
{
    pub fn button() -> Self {
        Self::button_with_args(())
    }

    pub fn menu() -> Self {
        Self::menu_with_args(())
    }
}

impl<C> Command<C>
where
    C: command::Command,
{
    pub fn button_with_args(args: C::Args) -> Self {
        Self::new(args, CommandPlacement::Button)
    }

    pub fn menu_with_args(args: C::Args) -> Self {
        Self::new(args, CommandPlacement::Menu)
    }

    fn new(args: C::Args, placement: CommandPlacement) -> Self {
        Self {
            args,
            placement,
            _command: PhantomData,
        }
    }
}

impl<C> Widget for Command<C>
where
    C: command::Command,
    C::Args: Clone,
{
    fn into_node(self) -> view::Node {
        match self.placement {
            CommandPlacement::Button => view::Node::command_with_args::<C>(self.args),
            CommandPlacement::Menu => view::Node::menu_command_with_args::<C>(self.args),
        }
    }
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            binding: None,
        }
    }

    pub fn trigger<C>(mut self, args: C::Args) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        self.binding = Some(TriggerBinding::command::<C>(args, Source::Button));
        self
    }
}

impl Widget for Button {
    fn into_node(self) -> view::Node {
        let mut node = view::Node::button(self.label);
        if let Some(binding) = self.binding {
            node = node.bind_trigger(binding.trigger, binding.source);
        }
        node
    }
}

impl Checkbox {
    pub fn new(label: impl Into<String>, checked: bool) -> Self {
        Self {
            label: label.into(),
            checked,
            binding: None,
        }
    }

    pub fn trigger<C>(mut self, args: C::Args) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        self.binding = Some(TriggerBinding::command::<C>(args, Source::Button));
        self
    }
}

impl Widget for Checkbox {
    fn into_node(self) -> view::Node {
        let mut node = view::Node::checkbox(self.label, self.checked);
        if let Some(binding) = self.binding {
            node = node.bind_trigger(binding.trigger, binding.source);
        }
        node
    }
}

impl Radio {
    pub fn new(label: impl Into<String>, selected: bool) -> Self {
        Self {
            label: label.into(),
            selected,
            binding: None,
        }
    }

    pub fn trigger<C>(mut self, args: C::Args) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        self.binding = Some(TriggerBinding::command::<C>(args, Source::Button));
        self
    }
}

impl Widget for Radio {
    fn into_node(self) -> view::Node {
        let mut node = view::Node::radio(self.label, self.selected);
        if let Some(binding) = self.binding {
            node = node.bind_trigger(binding.trigger, binding.source);
        }
        node
    }
}

impl Slider {
    pub fn new(label: impl Into<String>, value: f64, range: RangeInclusive<f64>) -> Self {
        Self {
            label: label.into(),
            value,
            range,
            binding: None,
        }
    }

    pub fn trigger<C>(mut self, args: C::Args) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        self.binding = Some(SliderBinding::Fixed(TriggerBinding::command::<C>(
            args,
            Source::Button,
        )));
        self
    }

    pub fn on_change<C>(self) -> Self
    where
        C: command::Command,
        C::Args: From<f64> + Clone,
    {
        self.trigger_with::<C, _>(C::Args::from)
    }

    pub fn trigger_with<C, F>(mut self, map: F) -> Self
    where
        C: command::Command,
        C::Args: Clone,
        F: Fn(f64) -> C::Args + Send + Sync + 'static,
    {
        self.binding = Some(SliderBinding::Change(SliderChangeBinding::command::<C>(
            Source::Button,
            map,
        )));
        self
    }
}

impl Widget for Slider {
    fn into_node(self) -> view::Node {
        let start = *self.range.start();
        let end = *self.range.end();
        let mut node = view::Node::slider(self.label, self.value, start, end);
        if let Some(binding) = self.binding {
            node = binding.bind(node, self.value);
        }
        node
    }
}

impl TextBox {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            placeholder: None,
            focus: None,
            binding: None,
        }
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    pub fn focus(mut self, focus: session::Focus) -> Self {
        self.focus = Some(focus);
        self
    }

    pub fn on_submit<C>(self) -> Self
    where
        C: command::Command,
        C::Args: From<String> + Clone,
    {
        self.submit_with::<C, _>(C::Args::from)
    }

    pub fn submit_with<C, F>(mut self, map: F) -> Self
    where
        C: command::Command,
        C::Args: Clone,
        F: Fn(String) -> C::Args + Send + Sync + 'static,
    {
        self.binding = Some(TextBoxBinding::command::<C>(Source::Input, map));
        self
    }
}

impl Widget for TextBox {
    fn into_node(self) -> view::Node {
        let text = self.text;
        let mut text_box = view::TextBox::new(text.clone());
        if let Some(placeholder) = self.placeholder {
            text_box = text_box.with_placeholder(placeholder);
        }
        if let Some(focus) = self.focus {
            text_box = text_box.with_focus(focus);
        }

        let mut node = view::Node::text_box_state(text_box);
        if let Some(binding) = self.binding {
            node = binding.bind(node, text);
        }

        node
    }
}

impl Separator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Separator {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Separator {
    fn into_node(self) -> view::Node {
        view::Node::separator()
    }
}

impl TextArea {
    pub fn new(text: impl Into<String>) -> Self {
        let buffer = text::Buffer::from_multiline_text(text);
        let edit_state = buffer.edit_state();
        Self::from_buffer(buffer, edit_state)
    }

    pub fn from_buffer(buffer: text::Buffer, edit_state: text::edit::State) -> Self {
        Self {
            buffer,
            edit_state,
            wrap: view::Wrap::Word,
            id: None,
            focus: None,
        }
    }

    pub fn id(mut self, id: impl Into<interaction::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn wrap(mut self, wrap: view::Wrap) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn focus(mut self, focus: session::Focus) -> Self {
        self.focus = Some(focus);
        self
    }
}

impl Widget for TextArea {
    fn into_node(self) -> view::Node {
        let id = self.id.or_else(|| self.focus.map(session::Focus::target));
        let mut text_area =
            view::TextArea::from_buffer(self.buffer, self.edit_state).with_wrap(self.wrap);
        if let Some(focus) = self.focus {
            text_area = text_area.with_focus(focus);
        }

        let mut node = view::Node::text_area_state(text_area);
        if let Some(id) = id {
            node = node.with_interaction_id(id);
        }
        node
    }
}

impl Panel {
    pub fn new() -> Self {
        Self {
            node: view::Node::panel(),
        }
    }

    pub fn child(mut self, child: impl Widget) -> Self {
        self.node = self.node.child(child.into_node());
        self
    }

    pub fn children(mut self, children: impl FnOnce(&mut Ui)) -> Self {
        let mut ui = Ui::new();
        children(&mut ui);
        for child in ui.into_nodes() {
            self.node = self.node.child(child);
        }
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
        self.node
    }
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl Widget for Label {
    fn into_node(self) -> view::Node {
        view::Node::label(self.text)
    }
}

impl Layout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn row(mut self) -> Self {
        self.direction = Direction::Row;
        self
    }

    pub fn column(mut self) -> Self {
        self.direction = Direction::Column;
        self
    }

    pub fn gap(mut self, gap: i32) -> Self {
        self.gap = gap.max(0);
        self
    }

    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub fn align_items(mut self, align: Align) -> Self {
        self.align_items = align;
        self
    }

    pub fn justify_content(mut self, align: Align) -> Self {
        self.justify_content = align;
        self
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn gap_value(&self) -> i32 {
        self.gap
    }

    pub fn padding_value(&self) -> Padding {
        self.padding
    }

    pub fn align_items_value(&self) -> Align {
        self.align_items
    }

    pub fn justify_content_value(&self) -> Align {
        self.justify_content
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            direction: Direction::Column,
            gap: 0,
            padding: Padding::zero(),
            align_items: Align::Stretch,
            justify_content: Align::Start,
        }
    }
}

impl Size {
    pub fn fit() -> Self {
        Self::Fit
    }

    pub fn grow() -> Self {
        Self::Grow
    }

    pub fn fixed(value: i32) -> Self {
        Self::Fixed(value.max(0))
    }

    pub fn percent(value: f32) -> Self {
        Self::Percent(value.clamp(0.0, 1.0))
    }

    fn into_view_dimension(self) -> view::Dimension {
        match self {
            Self::Fit => view::Dimension::Fit,
            Self::Grow => view::Dimension::Grow,
            Self::Fixed(value) => view::Dimension::fixed(value),
            Self::Percent(value) => view::Dimension::percent(value),
        }
    }
}

impl Padding {
    pub fn zero() -> Self {
        Self {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        }
    }

    pub fn all(value: i32) -> Self {
        let value = value.max(0);
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn symmetric(horizontal: i32, vertical: i32) -> Self {
        let horizontal = horizontal.max(0);
        let vertical = vertical.max(0);
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub fn top(self) -> i32 {
        self.top
    }

    pub fn right(self) -> i32 {
        self.right
    }

    pub fn bottom(self) -> i32 {
        self.bottom
    }

    pub fn left(self) -> i32 {
        self.left
    }

    fn into_view_padding(self) -> view::Padding {
        view::Padding::edges(self.top, self.right, self.bottom, self.left)
    }
}

impl Align {
    fn into_view_align(self) -> view::Align {
        match self {
            Self::Start => view::Align::Start,
            Self::Center => view::Align::Center,
            Self::End => view::Align::End,
            Self::Stretch => view::Align::Stretch,
        }
    }
}

impl Default for Padding {
    fn default() -> Self {
        Self::zero()
    }
}

impl TriggerBinding {
    fn command<C>(args: C::Args, source: Source) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        Self {
            trigger: command::AnyTrigger::command::<C>(args),
            source,
        }
    }
}

impl SliderBinding {
    fn bind(self, node: view::Node, value: f64) -> view::Node {
        match self {
            Self::Fixed(binding) => node.bind_trigger(binding.trigger, binding.source),
            Self::Change(binding) => binding.bind(node, value),
        }
    }
}

impl SliderChangeBinding {
    fn command<C>(source: Source, map: impl Fn(f64) -> C::Args + Send + Sync + 'static) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        Self {
            trigger: command::AnyValueTrigger::command::<C>(map),
            source,
        }
    }

    fn bind(self, node: view::Node, value: f64) -> view::Node {
        node.bind_slider_trigger(value, self.source, self.trigger)
    }
}

impl TextBoxBinding {
    fn command<C>(source: Source, map: impl Fn(String) -> C::Args + Send + Sync + 'static) -> Self
    where
        C: command::Command,
        C::Args: Clone,
    {
        Self {
            trigger: command::AnyValueTrigger::command::<C>(map),
            source,
        }
    }

    fn bind(self, node: view::Node, text: String) -> view::Node {
        node.bind_text_trigger(text, self.source, self.trigger)
    }
}

pub fn view(children: impl FnOnce(&mut Ui)) -> view::View {
    view::View::new(Root::new().children(children).into_node())
}

pub fn view_node(root: impl Widget) -> view::View {
    view::View::new(root.into_node())
}
