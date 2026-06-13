use std::collections::HashMap;

use crate::geometry::{Rect, area, point};
use crate::{action, layout, paint, window};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(&'static str);

impl Id {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Left,
    Right,
    Middle,
    Other(u16),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Resized {
        area: area::Physical,
        scale_factor: f32,
    },
    ScaleFactorChanged {
        scale_factor: f32,
    },
    CloseRequested,
    Focused(bool),
    PointerMoved {
        position: point::Logical,
        target: Option<Id>,
    },
    PointerEntered {
        target: Id,
    },
    PointerLeft {
        target: Id,
    },
    PointerDown {
        position: point::Logical,
        target: Option<Id>,
        button: Button,
    },
    PointerUp {
        position: point::Logical,
        target: Option<Id>,
        button: Button,
    },
    ActionInvoked {
        action: action::Id,
        source: action::Source,
        context: action::Context,
    },
    Ignored,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tree {
    root: Option<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub id: Id,
    pub layout: Layout,
    pub style: Style,
    pub action: Option<action::Id>,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    pub width: layout::Size,
    pub height: layout::Size,
    pub direction: Option<layout::Axis>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    pub background: Option<paint::Color>,
    pub disabled_background: Option<paint::Color>,
    pub padding: layout::Insets,
}

impl Tree {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn set_root(&mut self, root: Node) {
        self.root = Some(root);
    }

    pub fn root(&self) -> Option<&Node> {
        self.root.as_ref()
    }

    pub fn root_mut(&mut self) -> Option<&mut Node> {
        self.root.as_mut()
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn layout(&self, area: area::Logical) -> Option<layout::Box> {
        self.root
            .as_ref()
            .map(|root| layout_node(root, point::logical(0.0, 0.0), area))
    }

    pub fn actions(&self) -> HashMap<Id, action::Id> {
        let mut actions = HashMap::new();

        if let Some(root) = self.root.as_ref() {
            collect_actions(root, &mut actions);
        }

        actions
    }

    pub fn paint(
        &self,
        layout: &layout::Box,
        actions: &action::Registry,
        window: window::Id,
        scene: &mut paint::Scene,
    ) {
        if let Some(root) = self.root.as_ref() {
            paint_node(root, layout, actions, window, scene);
        }
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

impl Node {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            layout: Layout::default(),
            style: Style::default(),
            action: None,
            children: Vec::new(),
        }
    }

    pub fn leaf(id: Id) -> Self {
        Self::new(id)
    }

    pub fn container(id: Id, axis: layout::Axis) -> Self {
        Self::new(id).with_direction(axis)
    }

    pub fn with_layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_size(mut self, width: layout::Size, height: layout::Size) -> Self {
        self.layout.width = width;
        self.layout.height = height;
        self
    }

    pub fn with_direction(mut self, axis: layout::Axis) -> Self {
        self.layout.direction = Some(axis);
        self
    }

    pub fn with_background(mut self, color: paint::Color) -> Self {
        self.style.background = Some(color);
        self
    }

    pub fn with_disabled_background(mut self, color: paint::Color) -> Self {
        self.style.disabled_background = Some(color);
        self
    }

    pub fn with_padding(mut self, padding: layout::Insets) -> Self {
        self.style.padding = padding;
        self
    }

    pub fn with_action(mut self, action: action::Id) -> Self {
        self.action = Some(action);
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

impl Default for Layout {
    fn default() -> Self {
        Self {
            width: layout::Size::Fill,
            height: layout::Size::Fill,
            direction: None,
        }
    }
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: None,
            disabled_background: None,
            padding: layout::Insets::ZERO,
        }
    }
}

fn collect_actions(node: &Node, actions: &mut HashMap<Id, action::Id>) {
    if let Some(action) = node.action {
        actions.insert(node.id, action);
    }

    for child in &node.children {
        collect_actions(child, actions);
    }
}

fn layout_node(node: &Node, origin: point::Logical, available: area::Logical) -> layout::Box {
    let width = resolve_size(node.layout.width, available.width());
    let height = resolve_size(node.layout.height, available.height());
    let rect = Rect::new(origin, area::logical(width, height));
    let padding = node.style.padding;
    let content_origin = point::logical(origin.x() + padding.left, origin.y() + padding.top);
    let content_area = area::logical(
        (width - padding.horizontal()).max(0.0),
        (height - padding.vertical()).max(0.0),
    );
    let children = match node.layout.direction {
        Some(layout::Axis::Vertical) => {
            layout_vertical_children(node, content_origin, content_area)
        }
        Some(layout::Axis::Horizontal) => {
            layout_horizontal_children(node, content_origin, content_area)
        }
        None => node
            .children
            .iter()
            .map(|child| layout_node(child, content_origin, content_area))
            .collect(),
    };

    layout::Box::new(node.id, rect, children)
}

fn resolve_size(size: layout::Size, available: f32) -> f32 {
    match size {
        layout::Size::Fit => 0.0,
        layout::Size::Fill => available.max(0.0),
        layout::Size::Fixed(value) => value.max(0.0),
    }
}

fn layout_vertical_children(
    node: &Node,
    origin: point::Logical,
    available: area::Logical,
) -> Vec<layout::Box> {
    let fixed_height: f32 = node
        .children
        .iter()
        .map(|child| match child.layout.height {
            layout::Size::Fixed(value) => value.max(0.0),
            _ => 0.0,
        })
        .sum();
    let fill_count = node
        .children
        .iter()
        .filter(|child| !matches!(child.layout.height, layout::Size::Fixed(_)))
        .count();
    let fill_height = if fill_count == 0 {
        0.0
    } else {
        ((available.height() - fixed_height).max(0.0)) / fill_count as f32
    };
    let mut y = origin.y();
    let mut children = Vec::with_capacity(node.children.len());

    for child in &node.children {
        let height = match child.layout.height {
            layout::Size::Fixed(value) => value.max(0.0),
            _ => fill_height,
        };
        let child_area = area::logical(resolve_size(child.layout.width, available.width()), height);
        children.push(layout_node(
            child,
            point::logical(origin.x(), y),
            child_area,
        ));
        y += height;
    }

    children
}

fn layout_horizontal_children(
    node: &Node,
    origin: point::Logical,
    available: area::Logical,
) -> Vec<layout::Box> {
    let fixed_width: f32 = node
        .children
        .iter()
        .map(|child| match child.layout.width {
            layout::Size::Fixed(value) => value.max(0.0),
            _ => 0.0,
        })
        .sum();
    let fill_count = node
        .children
        .iter()
        .filter(|child| !matches!(child.layout.width, layout::Size::Fixed(_)))
        .count();
    let fill_width = if fill_count == 0 {
        0.0
    } else {
        ((available.width() - fixed_width).max(0.0)) / fill_count as f32
    };
    let mut x = origin.x();
    let mut children = Vec::with_capacity(node.children.len());

    for child in &node.children {
        let width = match child.layout.width {
            layout::Size::Fixed(value) => value.max(0.0),
            _ => fill_width,
        };
        let child_area =
            area::logical(width, resolve_size(child.layout.height, available.height()));
        children.push(layout_node(
            child,
            point::logical(x, origin.y()),
            child_area,
        ));
        x += width;
    }

    children
}

fn paint_node(
    node: &Node,
    layout: &layout::Box,
    actions: &action::Registry,
    window: window::Id,
    scene: &mut paint::Scene,
) {
    if let Some(background) = resolved_background(node, actions, window) {
        scene.push_quad(paint::Quad {
            rect: layout.rect,
            style: paint::Style {
                fill: Some(paint::Fill::Brush(paint::Brush::Solid(background))),
                stroke: None,
                tint: None,
            },
        });
    }

    for (child, child_layout) in node.children.iter().zip(&layout.children) {
        paint_node(child, child_layout, actions, window, scene);
    }
}

fn resolved_background(
    node: &Node,
    actions: &action::Registry,
    window: window::Id,
) -> Option<paint::Color> {
    let background = node.style.background?;

    if let Some(action) = node.action {
        let state = actions.state(
            action,
            action::Context {
                window,
                target: Some(node.id),
            },
        );

        if !state.enabled {
            return Some(
                node.style
                    .disabled_background
                    .unwrap_or_else(|| dim(background)),
            );
        }
    }

    Some(background)
}

fn dim(color: paint::Color) -> paint::Color {
    paint::Color {
        r: color.r * 0.45,
        g: color.g * 0.45,
        b: color.b * 0.45,
        a: color.a,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT: Id = Id::new("root");
    const A: Id = Id::new("a");
    const B: Id = Id::new("b");
    const CLICK: action::Id = action::Id::new("click");

    fn layout(tree: &Tree) -> layout::Box {
        tree.layout(area::logical(100.0, 80.0))
            .expect("tree should have root")
    }

    #[test]
    fn fixed_and_fill_vertical_layout() {
        let mut root = Node::container(ROOT, layout::Axis::Vertical);
        root.push_child(Node::leaf(A).with_size(layout::Size::Fill, layout::Size::Fixed(20.0)));
        root.push_child(Node::leaf(B));

        let mut tree = Tree::new();
        tree.set_root(root);
        let layout = layout(&tree);

        assert_eq!(layout.children[0].rect.area, area::logical(100.0, 20.0));
        assert_eq!(layout.children[1].rect.area, area::logical(100.0, 60.0));
        assert_eq!(layout.children[1].rect.origin, point::logical(0.0, 20.0));
    }

    #[test]
    fn padding_offsets_children() {
        let root = Node::container(ROOT, layout::Axis::Vertical)
            .with_padding(layout::Insets::splat(10.0))
            .with_child(Node::leaf(A));

        let mut tree = Tree::new();
        tree.set_root(root);
        let layout = layout(&tree);

        assert_eq!(layout.children[0].rect.origin, point::logical(10.0, 10.0));
        assert_eq!(layout.children[0].rect.area, area::logical(80.0, 60.0));
    }

    #[test]
    fn horizontal_layout_distributes_fill_width() {
        let root = Node::container(ROOT, layout::Axis::Horizontal)
            .with_child(Node::leaf(A))
            .with_child(Node::leaf(B));

        let mut tree = Tree::new();
        tree.set_root(root);
        let layout = layout(&tree);

        assert_eq!(layout.children[0].rect.area, area::logical(50.0, 80.0));
        assert_eq!(layout.children[1].rect.origin, point::logical(50.0, 0.0));
    }

    #[test]
    fn deepest_hit_test_target_is_returned() {
        let root = Node::container(ROOT, layout::Axis::Vertical)
            .with_child(Node::leaf(A).with_size(layout::Size::Fill, layout::Size::Fixed(20.0)))
            .with_child(Node::leaf(B));

        let mut tree = Tree::new();
        tree.set_root(root);
        let layout = layout(&tree);

        assert_eq!(layout.hit_test(point::logical(5.0, 25.0)), Some(B));
    }

    #[test]
    fn tree_renders_background_quads_in_layout_order() {
        let root = Node::container(ROOT, layout::Axis::Vertical)
            .with_background(paint::Color::BLACK)
            .with_child(Node::leaf(A).with_background(paint::Color::RED));
        let mut tree = Tree::new();
        let mut scene = paint::Scene::new();
        let registry = action::Registry::new();

        tree.set_root(root);
        let layout = layout(&tree);
        tree.paint(&layout, &registry, window::Id::new(1), &mut scene);

        assert_eq!(scene.quads().len(), 2);
        assert_eq!(scene.quads()[0].rect, layout.rect);
        assert_eq!(scene.quads()[1].rect, layout.children[0].rect);
    }

    #[test]
    fn disabled_action_node_renders_disabled_background() {
        let root = Node::leaf(A)
            .with_action(CLICK)
            .with_background(paint::Color::RED)
            .with_disabled_background(paint::Color::BLACK);
        let mut tree = Tree::new();
        let mut scene = paint::Scene::new();
        let mut registry = action::Registry::new();
        let window = window::Id::new(1);

        registry.register(action::Action::new(CLICK, "Click"));
        registry.set_state(
            CLICK,
            action::Context {
                window,
                target: Some(A),
            },
            action::State::disabled(),
        );
        tree.set_root(root);
        let layout = layout(&tree);
        tree.paint(&layout, &registry, window, &mut scene);

        assert_eq!(
            scene.quads()[0].style.fill,
            Some(paint::Fill::Brush(paint::Brush::Solid(paint::Color::BLACK)))
        );
    }
}
