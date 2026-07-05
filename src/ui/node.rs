use crate::widget::menu;
use crate::{action, geometry, icon, paint, pointer, text};

use super::{Backdrop, Id, Path, focus, layout, scroll};

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    key: Option<Id>,
    kind: &'static str,
    layout: Layout,
    style: Style,
    interactivity: Interactivity,
    cursor: Cursor,
    intent: Option<Intent>,
    action: Option<action::Route>,
    action_subject: action::Subject,
    responders: Vec<action::Key>,
    responder_bindings: Vec<action::Binding>,
    action_targets: Vec<action::Target>,
    action_scope: bool,
    label: Option<text::document::Document>,
    text_surface: Option<text::Surface>,
    icon: Option<icon::Icon>,
    icon_size: Option<f32>,
    icon_requires_active: bool,
    menu_bar: Option<menu::Bar>,
    clip: bool,
    scroll: Option<scroll::Scroll>,
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
    rounding: geometry::rect::Rounding,
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
pub enum Cursor {
    #[default]
    Default,
    Text,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum CursorOverlay {
    Text(CursorOverlayText),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CursorOverlayText {
    text: String,
    offset: geometry::point::Logical,
    max_width: f32,
    alpha: f32,
}

impl CursorOverlay {
    pub(crate) fn text(text: impl Into<String>) -> Self {
        Self::Text(CursorOverlayText {
            text: text.into(),
            offset: geometry::point::logical(12.0, 16.0),
            max_width: 240.0,
            alpha: 0.65,
        })
    }
}

impl CursorOverlayText {
    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    pub(crate) fn offset(&self) -> geometry::point::Logical {
        self.offset
    }

    pub(crate) fn max_width(&self) -> f32 {
        self.max_width
    }

    pub(crate) fn alpha(&self) -> f32 {
        self.alpha
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Intent {
    Action(action::Route),
    OpenMenu(menu::Id),
    OpenSubmenu(menu::Id),
    CloseSubmenu,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Interaction {
    hovered: Option<Path>,
    focused: Option<Path>,
    text_editing_target: Option<Path>,
    focus_visibility: focus::Visibility,
    pressed: Option<Path>,
    open_menu: Option<menu::Id>,
    open_submenu: Option<menu::Id>,
    pointer_position: Option<geometry::point::Logical>,
    pointer_capture: Option<pointer::Capture>,
    text_drop_caret: Option<(Path, geometry::Rect)>,
    drag_drop_operation: crate::ui::drag_drop::Operation,
    cursor_overlay: Option<CursorOverlay>,
}

impl Node {
    pub fn new() -> Self {
        Self::kind("node")
    }

    pub fn kind(kind: &'static str) -> Self {
        Self {
            key: None,
            kind,
            layout: Layout::default(),
            style: Style::default(),
            interactivity: Interactivity::default(),
            cursor: Cursor::default(),
            intent: None,
            action: None,
            action_subject: action::Subject::default(),
            responders: Vec::new(),
            responder_bindings: Vec::new(),
            action_targets: Vec::new(),
            action_scope: false,
            label: None,
            text_surface: None,
            icon: None,
            icon_size: None,
            icon_requires_active: false,
            menu_bar: None,
            clip: false,
            scroll: None,
            children: Vec::new(),
        }
    }

    pub fn leaf() -> Self {
        Self::kind("leaf")
    }

    pub fn container(axis: layout::Axis) -> Self {
        Self::kind("container").with_direction(axis)
    }

    pub fn with_kind(mut self, kind: &'static str) -> Self {
        self.kind = kind;
        self
    }

    pub fn key(mut self, key: Id) -> Self {
        self.key = Some(key);
        self
    }

    pub fn with_key(self, key: Id) -> Self {
        self.key(key)
    }

    pub fn id(&self) -> Id {
        self.path_id(0)
    }

    pub(crate) fn path_id(&self, index: usize) -> Id {
        self.key.unwrap_or_else(|| Id::structural(self.kind, index))
    }

    pub fn explicit_key(&self) -> Option<Id> {
        self.key
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

    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    pub(crate) fn action(&self) -> Option<action::Key> {
        self.action.map(|route| route.key())
    }

    pub(crate) fn action_route(&self) -> Option<action::Route> {
        self.action
    }

    pub(crate) fn intent(&self) -> Option<Intent> {
        self.intent
    }

    pub fn action_subject(&self) -> action::Subject {
        self.action_subject
    }

    pub(crate) fn responders(&self) -> &[action::Key] {
        &self.responders
    }

    pub(crate) fn responder_bindings(&self) -> &[action::Binding] {
        &self.responder_bindings
    }

    pub(crate) fn action_targets(&self) -> &[action::Target] {
        &self.action_targets
    }

    pub fn is_action_scope(&self) -> bool {
        self.action_scope
    }

    pub fn label(&self) -> Option<&text::document::Document> {
        self.label.as_ref()
    }

    pub fn text_field(&self) -> Option<&text::Field> {
        self.text_surface.as_ref().and_then(text::Surface::as_field)
    }

    pub fn text_area(&self) -> Option<&text::Area> {
        self.text_surface.as_ref().and_then(text::Surface::as_area)
    }

    pub fn text_surface(&self) -> Option<&text::Surface> {
        self.text_surface.as_ref()
    }

    pub fn icon(&self) -> Option<icon::Icon> {
        self.icon
    }

    pub fn icon_size(&self) -> Option<f32> {
        self.icon_size
    }

    pub(crate) fn icon_requires_active(&self) -> bool {
        self.icon_requires_active
    }

    pub fn menu_bar(&self) -> Option<&menu::Bar> {
        self.menu_bar.as_ref()
    }

    pub fn clips(&self) -> bool {
        self.clip
    }

    pub fn scroll(&self) -> Option<scroll::Scroll> {
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

    pub fn size(self, width: layout::Size, height: layout::Size) -> Self {
        self.with_size(width, height)
    }

    pub fn width(mut self, width: layout::Size) -> Self {
        self.layout = self.layout.with_width(width);
        self
    }

    pub fn height(mut self, height: layout::Size) -> Self {
        self.layout = self.layout.with_height(height);
        self
    }

    pub fn with_direction(mut self, axis: layout::Axis) -> Self {
        self.layout = self.layout.with_direction(axis);
        self
    }

    pub fn direction(self, axis: layout::Axis) -> Self {
        self.with_direction(axis)
    }

    pub fn with_gap(mut self, gap: f32) -> Self {
        self.layout = self.layout.with_gap(gap);
        self
    }

    pub fn gap(self, gap: f32) -> Self {
        self.with_gap(gap)
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

    pub fn with_rounding(mut self, rounding: geometry::rect::Rounding) -> Self {
        self.style.rounding = rounding;
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

    pub fn padding(self, padding: impl Into<layout::Insets>) -> Self {
        self.with_padding(padding.into())
    }

    pub fn clipped(self) -> Self {
        self.with_clip(true)
    }

    pub fn with_clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    pub fn with_scroll(mut self, scroll: scroll::Scroll) -> Self {
        self.scroll = Some(scroll);
        self
    }

    pub fn with_scroll_offset(mut self, offset: geometry::point::Logical) -> Self {
        self.scroll = Some(self.scroll.unwrap_or_default().with_offset(offset));
        self
    }

    pub fn with_scroll_axes(mut self, axes: scroll::Axes) -> Self {
        self.scroll = Some(self.scroll.unwrap_or_default().with_axes(axes));
        self
    }

    pub fn with_scroll_bars(mut self, bars: scroll::Bars) -> Self {
        self.scroll = Some(self.scroll.unwrap_or_default().with_bars(bars));
        self
    }

    pub fn with_scroll_style(mut self, style: scroll::Style) -> Self {
        self.scroll = Some(self.scroll.unwrap_or_default().with_style(style));
        self
    }

    pub fn with_label(mut self, label: text::document::Document) -> Self {
        self.label = Some(label);
        self
    }

    pub fn with_text_field(mut self, field: impl Into<text::Field>) -> Self {
        self.text_surface = Some(text::Surface::Field(field.into()));
        self
    }

    pub fn with_text_area(mut self, area: impl Into<text::Area>) -> Self {
        self.text_surface = Some(text::Surface::Area(area.into()));
        self
    }

    pub fn with_icon(mut self, icon: icon::Icon) -> Self {
        self.icon = Some(icon);
        self.icon_requires_active = false;
        self
    }

    pub(crate) fn with_active_icon(mut self, icon: icon::Icon) -> Self {
        self.icon = Some(icon);
        self.icon_requires_active = true;
        self
    }

    pub fn with_icon_size(mut self, size: f32) -> Self {
        self.icon_size = Some(size);
        self
    }

    #[cfg(test)]
    pub(crate) fn with_action_key(mut self, key: action::Key) -> Self {
        let route = action::Route::new(key, action::Target::action(key));
        self.intent = Some(Intent::Action(route));
        self.action = Some(route);
        self
    }

    pub fn with_action_route(mut self, route: action::Route) -> Self {
        self.intent = Some(Intent::Action(route));
        self.action = Some(route);
        self
    }

    pub(crate) fn with_intent(mut self, intent: Intent) -> Self {
        self.action = match intent {
            Intent::Action(route) => Some(route),
            Intent::OpenMenu(_) | Intent::OpenSubmenu(_) | Intent::CloseSubmenu => None,
        };
        self.intent = Some(intent);
        self
    }

    pub fn with_action_subject(mut self, target: action::Subject) -> Self {
        self.action_subject = target;
        self
    }

    #[cfg(test)]
    pub(crate) fn with_responder_key(mut self, key: action::Key) -> Self {
        self.push_responder_binding(action::Binding::new(key));
        self
    }

    pub fn with_responder_binding(mut self, binding: action::Binding) -> Self {
        self.push_responder_binding(binding);
        self
    }

    pub fn with_action_target(mut self, target: action::Target) -> Self {
        self.push_action_target(target);
        self
    }

    pub fn with_action_scope(mut self) -> Self {
        self.action_scope = true;
        self
    }

    pub fn with_menu_bar(mut self, bar: menu::Bar) -> Self {
        self.menu_bar = Some(bar.with_structural_ids());
        self
    }

    pub fn with_interactivity(mut self, interactivity: Interactivity) -> Self {
        self.interactivity = interactivity;
        self
    }

    pub fn with_cursor(mut self, cursor: Cursor) -> Self {
        self.cursor = cursor;
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

    pub fn child(self, child: Node) -> Self {
        self.with_child(child)
    }

    pub fn with_children(mut self, children: impl IntoIterator<Item = Node>) -> Self {
        self.children.extend(children);
        self
    }

    fn push_responder_binding(&mut self, binding: action::Binding) {
        let key = binding.key();

        if !self.responders.contains(&key) {
            self.responders.push(key);
        }

        if let Some(existing) = self
            .responder_bindings
            .iter_mut()
            .find(|existing| existing.key() == key)
        {
            *existing = binding;
            return;
        }

        self.responder_bindings.push(binding);
    }

    fn push_action_target(&mut self, target: action::Target) {
        if !self.action_targets.contains(&target) {
            self.action_targets.push(target);
        }
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

    pub fn box_model(self, padding: layout::Insets) -> layout::BoxModel {
        layout::BoxModel::new(self.width, self.height).with_padding(padding)
    }

    pub fn strategy(self) -> layout::Layout {
        match self.direction {
            Some(layout::Axis::Horizontal) => layout::Layout::Stack(
                layout::Stack::row()
                    .with_gap(self.gap)
                    .with_align(self.align)
                    .with_cross_align(self.cross_align),
            ),
            Some(layout::Axis::Vertical) => layout::Layout::Stack(
                layout::Stack::column()
                    .with_gap(self.gap)
                    .with_align(self.align)
                    .with_cross_align(self.cross_align),
            ),
            None => layout::Layout::overlay(),
        }
    }

    pub const fn with_size(mut self, width: layout::Size, height: layout::Size) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub const fn with_width(mut self, width: layout::Size) -> Self {
        self.width = width;
        self
    }

    pub const fn with_height(mut self, height: layout::Size) -> Self {
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

    pub fn rounding(self) -> geometry::rect::Rounding {
        self.rounding
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
            rounding: geometry::rect::Rounding::none(),
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
            text_editing_target: None,
            focus_visibility: focus::Visibility::Visible,
            pressed,
            open_menu: None,
            open_submenu: None,
            pointer_position: None,
            pointer_capture: None,
            text_drop_caret: None,
            drag_drop_operation: crate::ui::drag_drop::Operation::None,
            cursor_overlay: None,
        }
    }

    pub fn with_focus_visibility(mut self, visibility: focus::Visibility) -> Self {
        self.focus_visibility = visibility;
        self
    }

    pub fn with_text_editing_target(mut self, target: Option<Path>) -> Self {
        self.text_editing_target = target;
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

    pub fn with_pointer_capture(mut self, capture: Option<pointer::Capture>) -> Self {
        self.pointer_capture = capture;
        self
    }

    pub fn with_text_drop_caret(mut self, caret: Option<(Path, geometry::Rect)>) -> Self {
        self.text_drop_caret = caret;
        self
    }

    pub fn with_drag_drop_operation(mut self, operation: crate::ui::drag_drop::Operation) -> Self {
        self.drag_drop_operation = operation;
        self
    }

    pub(crate) fn with_cursor_overlay(mut self, overlay: Option<CursorOverlay>) -> Self {
        self.cursor_overlay = overlay;
        self
    }

    pub fn hovered(&self) -> Option<&Path> {
        self.hovered.as_ref()
    }

    pub fn focused(&self) -> Option<&Path> {
        self.focused.as_ref()
    }

    pub fn text_editing_target(&self) -> Option<&Path> {
        self.text_editing_target.as_ref()
    }

    pub fn focus_visibility(&self) -> focus::Visibility {
        self.focus_visibility
    }

    pub fn pressed(&self) -> Option<&Path> {
        self.pressed.as_ref()
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

    pub fn pointer_capture(&self) -> Option<&pointer::Capture> {
        self.pointer_capture.as_ref()
    }

    pub fn text_drop_caret(&self) -> Option<(&Path, geometry::Rect)> {
        self.text_drop_caret
            .as_ref()
            .map(|(path, rect)| (path, *rect))
    }

    pub fn drag_drop_operation(&self) -> crate::ui::drag_drop::Operation {
        self.drag_drop_operation
    }

    pub(crate) fn cursor_overlay(&self) -> Option<&CursorOverlay> {
        self.cursor_overlay.as_ref()
    }
}
