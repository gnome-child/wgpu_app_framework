use std::collections::HashMap;

pub mod control;
mod layouting;
mod painting;

use crate::geometry::{area, point};
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
    pub interactivity: Interactivity,
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
    pub hover_background: Option<paint::Color>,
    pub focus_background: Option<paint::Color>,
    pub active_background: Option<paint::Color>,
    pub disabled_background: Option<paint::Color>,
    pub padding: layout::Insets,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interactivity {
    pub hit_test: bool,
    pub focusable: bool,
    pub actionable: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Interaction {
    pub hovered: Option<Id>,
    pub focused: Option<Id>,
    pub pressed: Option<Id>,
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
        self.root.as_ref().map(|root| layouting::tree(root, area))
    }

    pub fn actions(&self) -> HashMap<Id, action::Id> {
        let mut actions = HashMap::new();

        if let Some(root) = self.root.as_ref() {
            collect_actions(root, &mut actions);
        }

        actions
    }

    pub fn interactivity(&self) -> HashMap<Id, Interactivity> {
        let mut interactivity = HashMap::new();

        if let Some(root) = self.root.as_ref() {
            collect_interactivity(root, &mut interactivity);
        }

        interactivity
    }

    pub fn paint(
        &self,
        layout: &layout::Box,
        actions: &action::Registry,
        window: window::Id,
        interaction: Interaction,
        scene: &mut paint::Scene,
    ) {
        if let Some(root) = self.root.as_ref() {
            painting::tree(root, layout, actions, window, interaction, scene);
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
            interactivity: Interactivity::default(),
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

    pub fn with_padding(mut self, padding: layout::Insets) -> Self {
        self.style.padding = padding;
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
        self.interactivity.hit_test = hit_test;
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.interactivity.focusable = focusable;
        self
    }

    pub fn actionable(mut self, actionable: bool) -> Self {
        self.interactivity.actionable = actionable;
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
            hover_background: None,
            focus_background: None,
            active_background: None,
            disabled_background: None,
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
}

impl Default for Interactivity {
    fn default() -> Self {
        Self::NONE
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

fn collect_interactivity(node: &Node, interactivity: &mut HashMap<Id, Interactivity>) {
    interactivity.insert(node.id, node.interactivity);

    for child in &node.children {
        collect_interactivity(child, interactivity);
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
            .with_child(
                Node::leaf(A)
                    .with_size(layout::Size::Fill, layout::Size::Fixed(20.0))
                    .hit_testable(true),
            )
            .with_child(Node::leaf(B).hit_testable(true));

        let mut tree = Tree::new();
        tree.set_root(root);
        let layout = layout(&tree);

        let interactivity = tree.interactivity();

        assert_eq!(
            layout.hit_test_where(point::logical(5.0, 25.0), |id| interactivity
                .get(&id)
                .is_some_and(|interactivity| interactivity.hit_test)),
            Some(B)
        );
    }

    #[test]
    fn passive_parent_does_not_become_hit_target() {
        let root = Node::container(ROOT, layout::Axis::Vertical).with_child(
            control::button(A, CLICK).with_size(layout::Size::Fill, layout::Size::Fixed(20.0)),
        );
        let mut tree = Tree::new();

        tree.set_root(root);
        let layout = layout(&tree);
        let interactivity = tree.interactivity();

        assert_eq!(
            layout.hit_test_where(point::logical(90.0, 70.0), |id| interactivity
                .get(&id)
                .is_some_and(|interactivity| interactivity.hit_test)),
            None
        );
        assert_eq!(
            layout.hit_test_where(point::logical(5.0, 5.0), |id| interactivity
                .get(&id)
                .is_some_and(|interactivity| interactivity.hit_test)),
            Some(A)
        );
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
        tree.paint(
            &layout,
            &registry,
            window::Id::new(1),
            Interaction::default(),
            &mut scene,
        );

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
        tree.paint(
            &layout,
            &registry,
            window,
            Interaction::default(),
            &mut scene,
        );

        assert_eq!(
            scene.quads()[0].style.fill,
            Some(paint::Fill::Brush(paint::Brush::Solid(paint::Color::BLACK)))
        );
    }

    #[test]
    fn control_hover_state_chooses_hover_background() {
        let root = control::button(A, CLICK);
        let mut tree = Tree::new();
        let mut scene = paint::Scene::new();
        let mut registry = action::Registry::new();
        let window = window::Id::new(1);

        registry.register(action::Action::new(CLICK, "Click"));
        tree.set_root(root.clone());
        let layout = layout(&tree);
        tree.paint(
            &layout,
            &registry,
            window,
            Interaction {
                hovered: Some(A),
                focused: None,
                pressed: None,
            },
            &mut scene,
        );

        assert_eq!(
            scene.quads()[0].style.fill,
            Some(paint::Fill::Brush(paint::Brush::Solid(
                root.style
                    .hover_background
                    .expect("control has hover color")
            )))
        );
    }

    #[test]
    fn control_focus_state_chooses_focus_background() {
        let root = control::button(A, CLICK);
        let mut tree = Tree::new();
        let mut scene = paint::Scene::new();
        let mut registry = action::Registry::new();
        let window = window::Id::new(1);

        registry.register(action::Action::new(CLICK, "Click"));
        tree.set_root(root.clone());
        let layout = layout(&tree);
        tree.paint(
            &layout,
            &registry,
            window,
            Interaction {
                hovered: Some(B),
                focused: Some(A),
                pressed: None,
            },
            &mut scene,
        );

        assert_eq!(
            scene.quads()[0].style.fill,
            Some(paint::Fill::Brush(paint::Brush::Solid(
                root.style
                    .focus_background
                    .expect("control has focus color")
            )))
        );
    }

    #[test]
    fn control_active_state_chooses_active_background() {
        let root = control::button(A, CLICK);
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
            action::State::active(),
        );
        tree.set_root(root.clone());
        let layout = layout(&tree);
        tree.paint(
            &layout,
            &registry,
            window,
            Interaction {
                hovered: Some(A),
                focused: Some(A),
                pressed: None,
            },
            &mut scene,
        );

        assert_eq!(
            scene.quads()[0].style.fill,
            Some(paint::Fill::Brush(paint::Brush::Solid(
                root.style
                    .active_background
                    .expect("control has active color")
            )))
        );
    }
}
