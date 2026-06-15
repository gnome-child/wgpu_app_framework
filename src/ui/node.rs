use std::collections::HashMap;

use crate::{action, geometry, icon, layout, menu, paint, text, widget};

use super::{Backdrop, Id, Path, focus};

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    id: Id,
    layout: Layout,
    style: Style,
    interactivity: Interactivity,
    intent: Option<Intent>,
    action: Option<action::Id>,
    action_target: ActionTarget,
    responders: Vec<action::Id>,
    command_scope: bool,
    label: Option<text::Document>,
    icon: Option<icon::Icon>,
    icon_size: Option<f32>,
    menu_bar: Option<menu::Bar>,
    clip: bool,
    scroll: Option<widget::Scroll>,
    children: Vec<Node>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    width: layout::Size,
    height: layout::Size,
    direction: Option<layout::Axis>,
    gap: f32,
    align: layout::Align,
    cross_align: layout::Align,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    background: Option<paint::Brush>,
    radius: geometry::rect::Radius,
    stroke: Option<paint::Stroke>,
    shadow: Option<Shadow>,
    backdrop: Option<Backdrop>,
    hover_background: Option<paint::Brush>,
    focus_background: Option<paint::Brush>,
    active_background: Option<paint::Brush>,
    busy_background: Option<paint::Brush>,
    disabled_background: Option<paint::Brush>,
    hover_tint: Option<paint::Brush>,
    pressed_tint: Option<paint::Brush>,
    active_tint: Option<paint::Brush>,
    busy_tint: Option<paint::Brush>,
    disabled_tint: Option<paint::Brush>,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    brush: paint::Brush,
    blur: f32,
    spread: f32,
    offset: geometry::point::Logical,
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
    Captured,
    Window,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    Action(action::Id),
    OpenMenu(menu::Id),
    OpenSubmenu(menu::Id),
    CloseSubmenu,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Interaction {
    hovered: Option<Path>,
    focused: Option<Path>,
    focus_visibility: focus::Visibility,
    pressed: Option<Path>,
    command_target: Option<action::Context>,
    command_scope_captures: HashMap<Path, action::Context>,
    open_menu: Option<menu::Id>,
    open_submenu: Option<menu::Id>,
    pointer_position: Option<geometry::point::Logical>,
}

impl Node {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            layout: Layout::default(),
            style: Style::default(),
            interactivity: Interactivity::default(),
            intent: None,
            action: None,
            action_target: ActionTarget::default(),
            responders: Vec::new(),
            command_scope: false,
            label: None,
            icon: None,
            icon_size: None,
            menu_bar: None,
            clip: false,
            scroll: None,
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

    pub fn intent(&self) -> Option<Intent> {
        self.intent
    }

    pub fn action_target(&self) -> ActionTarget {
        self.action_target
    }

    pub fn responders(&self) -> &[action::Id] {
        &self.responders
    }

    pub fn is_command_scope(&self) -> bool {
        self.command_scope
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

    pub fn menu_bar(&self) -> Option<&menu::Bar> {
        self.menu_bar.as_ref()
    }

    pub fn clips(&self) -> bool {
        self.clip
    }

    pub fn scroll(&self) -> Option<widget::Scroll> {
        self.scroll
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

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.layout = self.layout.with_gap(gap);
        self
    }

    pub fn with_align(mut self, align: layout::Align) -> Self {
        self.layout = self.layout.with_align(align);
        self
    }

    pub fn with_cross_align(mut self, align: layout::Align) -> Self {
        self.layout = self.layout.with_cross_align(align);
        self
    }

    pub fn with_background(mut self, brush: impl Into<paint::Brush>) -> Self {
        self.style.background = Some(brush.into());
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

    pub fn with_shadow(
        mut self,
        brush: impl Into<paint::Brush>,
        blur: f32,
        spread: f32,
        offset: geometry::point::Logical,
    ) -> Self {
        self.style.shadow = Some(Shadow {
            brush: brush.into(),
            blur,
            spread,
            offset,
        });
        self
    }

    pub fn with_backdrop(mut self, backdrop: Backdrop) -> Self {
        self.style.backdrop = Some(backdrop);
        self
    }

    pub fn with_hover_background(mut self, brush: impl Into<paint::Brush>) -> Self {
        self.style.hover_background = Some(brush.into());
        self
    }

    pub fn with_focus_background(mut self, brush: impl Into<paint::Brush>) -> Self {
        self.style.focus_background = Some(brush.into());
        self
    }

    pub fn with_active_background(mut self, brush: impl Into<paint::Brush>) -> Self {
        self.style.active_background = Some(brush.into());
        self
    }

    pub fn with_busy_background(mut self, brush: impl Into<paint::Brush>) -> Self {
        self.style.busy_background = Some(brush.into());
        self
    }

    pub fn with_disabled_background(mut self, brush: impl Into<paint::Brush>) -> Self {
        self.style.disabled_background = Some(brush.into());
        self
    }

    pub fn with_hover_tint(mut self, brush: impl Into<paint::Brush>) -> Self {
        self.style.hover_tint = Some(brush.into());
        self
    }

    pub fn with_pressed_tint(mut self, brush: impl Into<paint::Brush>) -> Self {
        self.style.pressed_tint = Some(brush.into());
        self
    }

    pub fn with_active_tint(mut self, brush: impl Into<paint::Brush>) -> Self {
        self.style.active_tint = Some(brush.into());
        self
    }

    pub fn with_busy_tint(mut self, brush: impl Into<paint::Brush>) -> Self {
        self.style.busy_tint = Some(brush.into());
        self
    }

    pub fn with_disabled_tint(mut self, brush: impl Into<paint::Brush>) -> Self {
        self.style.disabled_tint = Some(brush.into());
        self
    }

    pub fn with_focus_outline(
        mut self,
        brush: impl Into<paint::Brush>,
        width: f32,
        offset: f32,
    ) -> Self {
        self.style.focus_outline = Some(FocusOutline {
            brush: brush.into(),
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

    pub fn clipped(self) -> Self {
        self.with_clip(true)
    }

    pub fn with_clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    pub fn with_scroll(mut self, scroll: widget::Scroll) -> Self {
        self.scroll = Some(scroll);
        self
    }

    pub fn with_scroll_offset(mut self, offset: geometry::point::Logical) -> Self {
        self.scroll = Some(self.scroll.unwrap_or_default().with_offset(offset));
        self
    }

    pub fn with_scroll_bars(mut self, bars: widget::scroll::Bars) -> Self {
        self.scroll = Some(self.scroll.unwrap_or_default().with_bars(bars));
        self
    }

    pub fn with_scroll_style(mut self, style: widget::scroll::Style) -> Self {
        self.scroll = Some(self.scroll.unwrap_or_default().with_style(style));
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
        self.intent = Some(Intent::Action(action));
        self.action = Some(action);
        self
    }

    pub fn with_intent(mut self, intent: Intent) -> Self {
        self.action = match intent {
            Intent::Action(action) => Some(action),
            Intent::OpenMenu(_) | Intent::OpenSubmenu(_) | Intent::CloseSubmenu => None,
        };
        self.intent = Some(intent);
        self
    }

    pub fn with_action_target(mut self, target: ActionTarget) -> Self {
        self.action_target = target;
        self
    }

    pub fn with_responder(mut self, action: action::Id) -> Self {
        self.responders.push(action);
        self
    }

    pub fn with_command_scope(mut self) -> Self {
        self.command_scope = true;
        self
    }

    pub fn with_menu_bar(mut self, bar: menu::Bar) -> Self {
        self.menu_bar = Some(bar);
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
            gap: 0.0,
            align: layout::Align::Start,
            cross_align: layout::Align::Stretch,
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

    pub const fn gap(self) -> f32 {
        self.gap
    }

    pub const fn align(self) -> layout::Align {
        self.align
    }

    pub const fn cross_align(self) -> layout::Align {
        self.cross_align
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

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }

    pub const fn with_align(mut self, align: layout::Align) -> Self {
        self.align = align;
        self
    }

    pub const fn with_cross_align(mut self, align: layout::Align) -> Self {
        self.cross_align = align;
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

    pub fn background(self) -> Option<paint::Brush> {
        self.background
    }

    pub fn radius(self) -> geometry::rect::Radius {
        self.radius
    }

    pub fn stroke(self) -> Option<paint::Stroke> {
        self.stroke
    }

    pub fn shadow(self) -> Option<Shadow> {
        self.shadow
    }

    pub fn backdrop(self) -> Option<Backdrop> {
        self.backdrop
    }

    pub fn hover_background(self) -> Option<paint::Brush> {
        self.hover_background
    }

    pub fn focus_background(self) -> Option<paint::Brush> {
        self.focus_background
    }

    pub fn active_background(self) -> Option<paint::Brush> {
        self.active_background
    }

    pub fn busy_background(self) -> Option<paint::Brush> {
        self.busy_background
    }

    pub fn disabled_background(self) -> Option<paint::Brush> {
        self.disabled_background
    }

    pub fn hover_tint(self) -> Option<paint::Brush> {
        self.hover_tint
    }

    pub fn pressed_tint(self) -> Option<paint::Brush> {
        self.pressed_tint
    }

    pub fn active_tint(self) -> Option<paint::Brush> {
        self.active_tint
    }

    pub fn busy_tint(self) -> Option<paint::Brush> {
        self.busy_tint
    }

    pub fn disabled_tint(self) -> Option<paint::Brush> {
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
            shadow: None,
            backdrop: None,
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

impl Shadow {
    pub fn brush(self) -> paint::Brush {
        self.brush
    }

    pub fn blur(self) -> f32 {
        self.blur
    }

    pub fn spread(self) -> f32 {
        self.spread
    }

    pub fn offset(self) -> geometry::point::Logical {
        self.offset
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
            command_scope_captures: HashMap::new(),
            open_menu: None,
            open_submenu: None,
            pointer_position: None,
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

    pub fn with_command_scope_captures(mut self, captures: HashMap<Path, action::Context>) -> Self {
        self.command_scope_captures = captures;
        self
    }

    pub fn with_open_menu(mut self, menu: Option<menu::Id>) -> Self {
        self.open_menu = menu;
        self
    }

    pub fn with_open_submenu(mut self, menu: Option<menu::Id>) -> Self {
        self.open_submenu = menu;
        self
    }

    pub fn with_pointer_position(mut self, position: Option<geometry::point::Logical>) -> Self {
        self.pointer_position = position;
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

    pub fn open_menu(&self) -> Option<menu::Id> {
        self.open_menu
    }

    pub fn open_submenu(&self) -> Option<menu::Id> {
        self.open_submenu
    }

    pub fn pointer_position(&self) -> Option<geometry::point::Logical> {
        self.pointer_position
    }

    pub fn captured_command_target(&self, path: &Path) -> Option<&action::Context> {
        for length in (1..=path.ids().len()).rev() {
            let candidate = Path::new(path.ids()[..length].to_vec());

            if let Some(context) = self.command_scope_captures.get(&candidate) {
                return Some(context);
            }
        }

        None
    }
}
