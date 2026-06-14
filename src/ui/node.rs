use crate::{action, layout, paint, text};

use super::{Id, Path};

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    id: Id,
    layout: Layout,
    style: Style,
    interactivity: Interactivity,
    action: Option<action::Id>,
    label: Option<text::Document>,
    children: Vec<Node>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    width: layout::Size,
    height: layout::Size,
    direction: Option<layout::Axis>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    background: Option<paint::Color>,
    hover_background: Option<paint::Color>,
    focus_background: Option<paint::Color>,
    active_background: Option<paint::Color>,
    disabled_background: Option<paint::Color>,
    label_color: Option<paint::Color>,
    disabled_label_color: Option<paint::Color>,
    padding: layout::Insets,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interactivity {
    hit_test: bool,
    focusable: bool,
    actionable: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Interaction {
    hovered: Option<Path>,
    focused: Option<Path>,
    pressed: Option<Path>,
}

impl Node {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            layout: Layout::default(),
            style: Style::default(),
            interactivity: Interactivity::default(),
            action: None,
            label: None,
            children: Vec::new(),
        }
    }

    pub fn leaf(id: Id) -> Self {
        Self::new(id)
    }

    pub fn container(id: Id, axis: layout::Axis) -> Self {
        Self::new(id).with_direction(axis)
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn layout(&self) -> Layout {
        self.layout
    }

    pub fn style(&self) -> Style {
        self.style
    }

    pub fn interactivity(&self) -> Interactivity {
        self.interactivity
    }

    pub fn action(&self) -> Option<action::Id> {
        self.action
    }

    pub fn label(&self) -> Option<&text::Document> {
        self.label.as_ref()
    }

    pub fn children(&self) -> &[Node] {
        &self.children
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_size(mut self, width: layout::Size, height: layout::Size) -> Self {
        self.layout = self.layout.with_size(width, height);
        self
    }

    pub fn with_direction(mut self, axis: layout::Axis) -> Self {
        self.layout = self.layout.with_direction(axis);
        self
    }

    pub fn with_background(mut self, color: paint::Color) -> Self {
        self.style.background = Some(color);
        self
    }

    pub fn with_hover_background(mut self, color: paint::Color) -> Self {
        self.style.hover_background = Some(color);
        self
    }

    pub fn with_focus_background(mut self, color: paint::Color) -> Self {
        self.style.focus_background = Some(color);
        self
    }

    pub fn with_active_background(mut self, color: paint::Color) -> Self {
        self.style.active_background = Some(color);
        self
    }

    pub fn with_disabled_background(mut self, color: paint::Color) -> Self {
        self.style.disabled_background = Some(color);
        self
    }

    pub fn with_label_color(mut self, color: paint::Color) -> Self {
        self.style.label_color = Some(color);
        self
    }

    pub fn with_disabled_label_color(mut self, color: paint::Color) -> Self {
        self.style.disabled_label_color = Some(color);
        self
    }

    pub fn with_padding(mut self, padding: layout::Insets) -> Self {
        self.style.padding = padding;
        self
    }

    pub fn with_label(mut self, label: text::Document) -> Self {
        self.label = Some(label);
        self
    }

    pub fn with_action(mut self, action: action::Id) -> Self {
        self.action = Some(action);
        self
    }

    pub fn with_interactivity(mut self, interactivity: Interactivity) -> Self {
        self.interactivity = interactivity;
        self
    }

    pub fn hit_testable(mut self, hit_test: bool) -> Self {
        self.interactivity = self.interactivity.with_hit_test(hit_test);
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.interactivity = self.interactivity.with_focusable(focusable);
        self
    }

    pub fn actionable(mut self, actionable: bool) -> Self {
        self.interactivity = self.interactivity.with_actionable(actionable);
        self
    }

    pub fn push_child(&mut self, child: Node) {
        self.children.push(child);
    }

    pub fn with_child(mut self, child: Node) -> Self {
        self.push_child(child);
        self
    }
}

impl Layout {
    pub const fn new(width: layout::Size, height: layout::Size) -> Self {
        Self {
            width,
            height,
            direction: None,
        }
    }

    pub const fn width(self) -> layout::Size {
        self.width
    }

    pub const fn height(self) -> layout::Size {
        self.height
    }

    pub const fn direction(self) -> Option<layout::Axis> {
        self.direction
    }

    pub const fn with_size(mut self, width: layout::Size, height: layout::Size) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub const fn with_direction(mut self, direction: layout::Axis) -> Self {
        self.direction = Some(direction);
        self
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new(layout::Size::Fill, layout::Size::Fill)
    }
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn background(self) -> Option<paint::Color> {
        self.background
    }

    pub fn hover_background(self) -> Option<paint::Color> {
        self.hover_background
    }

    pub fn focus_background(self) -> Option<paint::Color> {
        self.focus_background
    }

    pub fn active_background(self) -> Option<paint::Color> {
        self.active_background
    }

    pub fn disabled_background(self) -> Option<paint::Color> {
        self.disabled_background
    }

    pub fn label_color(self) -> Option<paint::Color> {
        self.label_color
    }

    pub fn disabled_label_color(self) -> Option<paint::Color> {
        self.disabled_label_color
    }

    pub fn padding(self) -> layout::Insets {
        self.padding
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: None,
            hover_background: None,
            focus_background: None,
            active_background: None,
            disabled_background: None,
            label_color: None,
            disabled_label_color: None,
            padding: layout::Insets::ZERO,
        }
    }
}

impl Interactivity {
    pub const NONE: Self = Self {
        hit_test: false,
        focusable: false,
        actionable: false,
    };

    pub const CONTROL: Self = Self {
        hit_test: true,
        focusable: true,
        actionable: true,
    };

    pub const fn hit_test(self) -> bool {
        self.hit_test
    }

    pub const fn focusable(self) -> bool {
        self.focusable
    }

    pub const fn actionable(self) -> bool {
        self.actionable
    }

    pub const fn with_hit_test(mut self, hit_test: bool) -> Self {
        self.hit_test = hit_test;
        self
    }

    pub const fn with_focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub const fn with_actionable(mut self, actionable: bool) -> Self {
        self.actionable = actionable;
        self
    }
}

impl Default for Interactivity {
    fn default() -> Self {
        Self::NONE
    }
}

impl Interaction {
    pub fn new(hovered: Option<Path>, focused: Option<Path>, pressed: Option<Path>) -> Self {
        Self {
            hovered,
            focused,
            pressed,
        }
    }

    pub fn hovered(&self) -> Option<&Path> {
        self.hovered.as_ref()
    }

    pub fn focused(&self) -> Option<&Path> {
        self.focused.as_ref()
    }

    pub fn pressed(&self) -> Option<&Path> {
        self.pressed.as_ref()
    }
}
