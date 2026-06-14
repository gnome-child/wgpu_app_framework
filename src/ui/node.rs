use crate::{action, geometry, icon, layout, paint, text};

use super::{Id, Path, focus};

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    id: Id,
    layout: Layout,
    style: Style,
    interactivity: Interactivity,
    action: Option<action::Id>,
    action_target: ActionTarget,
    label: Option<text::Document>,
    icon: Option<icon::Icon>,
    icon_size: Option<f32>,
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
    radius: geometry::rect::Radius,
    stroke: Option<paint::Stroke>,
    hover_background: Option<paint::Color>,
    focus_background: Option<paint::Color>,
    active_background: Option<paint::Color>,
    busy_background: Option<paint::Color>,
    disabled_background: Option<paint::Color>,
    hover_tint: Option<paint::Color>,
    pressed_tint: Option<paint::Color>,
    active_tint: Option<paint::Color>,
    busy_tint: Option<paint::Color>,
    disabled_tint: Option<paint::Color>,
    focus_outline: Option<FocusOutline>,
    label_color: Option<paint::Color>,
    busy_label_color: Option<paint::Color>,
    disabled_label_color: Option<paint::Color>,
    padding: layout::Insets,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusOutline {
    brush: paint::Brush,
    width: f32,
    offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interactivity {
    hit_test: bool,
    focusable: bool,
    actionable: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ActionTarget {
    #[default]
    Origin,
    Command,
    Window,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Interaction {
    hovered: Option<Path>,
    focused: Option<Path>,
    focus_visibility: focus::Visibility,
    pressed: Option<Path>,
    command_target: Option<action::Context>,
}

impl Node {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            layout: Layout::default(),
            style: Style::default(),
            interactivity: Interactivity::default(),
            action: None,
            action_target: ActionTarget::default(),
            label: None,
            icon: None,
            icon_size: None,
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

    pub fn action_target(&self) -> ActionTarget {
        self.action_target
    }

    pub fn label(&self) -> Option<&text::Document> {
        self.label.as_ref()
    }

    pub fn icon(&self) -> Option<icon::Icon> {
        self.icon
    }

    pub fn icon_size(&self) -> Option<f32> {
        self.icon_size
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

    pub fn with_radius(mut self, radius: geometry::rect::Radius) -> Self {
        self.style.radius = radius;
        self
    }

    pub fn with_stroke(mut self, stroke: paint::Stroke) -> Self {
        self.style.stroke = Some(stroke);
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

    pub fn with_busy_background(mut self, color: paint::Color) -> Self {
        self.style.busy_background = Some(color);
        self
    }

    pub fn with_disabled_background(mut self, color: paint::Color) -> Self {
        self.style.disabled_background = Some(color);
        self
    }

    pub fn with_hover_tint(mut self, color: paint::Color) -> Self {
        self.style.hover_tint = Some(color);
        self
    }

    pub fn with_pressed_tint(mut self, color: paint::Color) -> Self {
        self.style.pressed_tint = Some(color);
        self
    }

    pub fn with_active_tint(mut self, color: paint::Color) -> Self {
        self.style.active_tint = Some(color);
        self
    }

    pub fn with_busy_tint(mut self, color: paint::Color) -> Self {
        self.style.busy_tint = Some(color);
        self
    }

    pub fn with_disabled_tint(mut self, color: paint::Color) -> Self {
        self.style.disabled_tint = Some(color);
        self
    }

    pub fn with_focus_outline(mut self, color: paint::Color, width: f32, offset: f32) -> Self {
        self.style.focus_outline = Some(FocusOutline {
            brush: paint::Brush::Solid(color),
            width,
            offset,
        });
        self
    }

    pub fn with_label_color(mut self, color: paint::Color) -> Self {
        self.style.label_color = Some(color);
        self
    }

    pub fn with_busy_label_color(mut self, color: paint::Color) -> Self {
        self.style.busy_label_color = Some(color);
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

    pub fn with_icon(mut self, icon: icon::Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn with_icon_size(mut self, size: f32) -> Self {
        self.icon_size = Some(size);
        self
    }

    pub fn with_action(mut self, action: action::Id) -> Self {
        self.action = Some(action);
        self
    }

    pub fn with_action_target(mut self, target: ActionTarget) -> Self {
        self.action_target = target;
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

    pub fn radius(self) -> geometry::rect::Radius {
        self.radius
    }

    pub fn stroke(self) -> Option<paint::Stroke> {
        self.stroke
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

    pub fn busy_background(self) -> Option<paint::Color> {
        self.busy_background
    }

    pub fn disabled_background(self) -> Option<paint::Color> {
        self.disabled_background
    }

    pub fn hover_tint(self) -> Option<paint::Color> {
        self.hover_tint
    }

    pub fn pressed_tint(self) -> Option<paint::Color> {
        self.pressed_tint
    }

    pub fn active_tint(self) -> Option<paint::Color> {
        self.active_tint
    }

    pub fn busy_tint(self) -> Option<paint::Color> {
        self.busy_tint
    }

    pub fn disabled_tint(self) -> Option<paint::Color> {
        self.disabled_tint
    }

    pub fn focus_outline(self) -> Option<FocusOutline> {
        self.focus_outline
    }

    pub fn label_color(self) -> Option<paint::Color> {
        self.label_color
    }

    pub fn busy_label_color(self) -> Option<paint::Color> {
        self.busy_label_color
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
            radius: geometry::rect::Radius::none(),
            stroke: None,
            hover_background: None,
            focus_background: None,
            active_background: None,
            busy_background: None,
            disabled_background: None,
            hover_tint: None,
            pressed_tint: None,
            active_tint: None,
            busy_tint: None,
            disabled_tint: None,
            focus_outline: None,
            label_color: None,
            busy_label_color: None,
            disabled_label_color: None,
            padding: layout::Insets::ZERO,
        }
    }
}

impl FocusOutline {
    pub fn brush(self) -> paint::Brush {
        self.brush
    }

    pub fn width(self) -> f32 {
        self.width
    }

    pub fn offset(self) -> f32 {
        self.offset
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
            focus_visibility: focus::Visibility::Visible,
            pressed,
            command_target: None,
        }
    }

    pub fn with_focus_visibility(mut self, visibility: focus::Visibility) -> Self {
        self.focus_visibility = visibility;
        self
    }

    pub fn with_command_target(mut self, target: action::Context) -> Self {
        self.command_target = Some(target);
        self
    }

    pub fn hovered(&self) -> Option<&Path> {
        self.hovered.as_ref()
    }

    pub fn focused(&self) -> Option<&Path> {
        self.focused.as_ref()
    }

    pub fn focus_visibility(&self) -> focus::Visibility {
        self.focus_visibility
    }

    pub fn pressed(&self) -> Option<&Path> {
        self.pressed.as_ref()
    }

    pub fn command_target(&self) -> Option<&action::Context> {
        self.command_target.as_ref()
    }
}
