use std::collections::HashMap;

pub mod control;
mod layouting;
mod painting;

use crate::geometry::{area, point};
use crate::{action, layout, paint, text, window};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(&'static str);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path {
    ids: Vec<Id>,
}

impl Id {
    pub const fn new(value: &'static str) -> Self {
        Self(value)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl Path {
    pub fn new(ids: impl Into<Vec<Id>>) -> Self {
        Self { ids: ids.into() }
    }

    pub fn root(id: Id) -> Self {
        Self { ids: vec![id] }
    }

    pub fn child(&self, id: Id) -> Self {
        let mut ids = self.ids.clone();
        ids.push(id);
        Self { ids }
    }

    pub fn push(&mut self, id: Id) {
        self.ids.push(id);
    }

    pub fn ids(&self) -> &[Id] {
        &self.ids
    }

    pub fn leaf(&self) -> Option<Id> {
        self.ids.last().copied()
    }
}

impl From<Id> for Path {
    fn from(value: Id) -> Self {
        Self::root(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Left,
    Right,
    Middle,
    Other(u16),
}

#[derive(Debug, Clone, PartialEq)]
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
        target: Option<Path>,
    },
    PointerEntered {
        target: Path,
    },
    PointerLeft {
        target: Path,
    },
    PointerDown {
        position: point::Logical,
        target: Option<Path>,
        button: Button,
    },
    PointerUp {
        position: point::Logical,
        target: Option<Path>,
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
    pub label: Option<text::Document>,
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
    pub label_color: Option<paint::Color>,
    pub disabled_label_color: Option<paint::Color>,
    pub padding: layout::Insets,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Interactivity {
    pub hit_test: bool,
    pub focusable: bool,
    pub actionable: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Interaction {
    pub hovered: Option<Path>,
    pub focused: Option<Path>,
    pub pressed: Option<Path>,
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

    pub fn actions(&self) -> HashMap<Path, action::Id> {
        let mut actions = HashMap::new();

        if let Some(root) = self.root.as_ref() {
            collect_actions(root, &Path::root(root.id), &mut actions);
        }

        actions
    }

    pub fn interactivity(&self) -> HashMap<Path, Interactivity> {
        let mut interactivity = HashMap::new();

        if let Some(root) = self.root.as_ref() {
            collect_interactivity(root, &Path::root(root.id), &mut interactivity);
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
}

impl Default for Interactivity {
    fn default() -> Self {
        Self::NONE
    }
}

fn collect_actions(node: &Node, path: &Path, actions: &mut HashMap<Path, action::Id>) {
    if let Some(action) = node.action {
        actions.insert(path.clone(), action);
    }

    for child in &node.children {
        collect_actions(child, &path.child(child.id), actions);
    }
}

fn collect_interactivity(
    node: &Node,
    path: &Path,
    interactivity: &mut HashMap<Path, Interactivity>,
) {
    interactivity.insert(path.clone(), node.interactivity);

    for child in &node.children {
        collect_interactivity(child, &path.child(child.id), interactivity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT: Id = Id::new("root");
    const A: Id = Id::new("a");
    const B: Id = Id::new("b");
    const C: Id = Id::new("c");
    const CLICK: action::Id = action::Id::new("click");

    fn layout(tree: &Tree) -> layout::Box {
        tree.layout(area::logical(100.0, 80.0))
            .expect("tree should have root")
    }

    fn path(id: Id) -> Path {
        Path::from(id)
    }

    fn quad(scene: &paint::Scene, index: usize) -> paint::Quad {
        match scene.items().get(index) {
            Some(paint::Item::Quad(quad)) => *quad,
            item => panic!("expected quad item at {index}, got {item:?}"),
        }
    }

    fn text(scene: &paint::Scene, index: usize) -> &paint::Text {
        match scene.items().get(index) {
            Some(paint::Item::Text(text)) => text,
            item => panic!("expected text item at {index}, got {item:?}"),
        }
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
    fn layout_assigns_stable_paths() {
        let root = Node::container(ROOT, layout::Axis::Vertical)
            .with_child(Node::container(A, layout::Axis::Vertical).with_child(Node::leaf(B)));
        let mut tree = Tree::new();

        tree.set_root(root);
        let layout = layout(&tree);

        assert_eq!(layout.path, Path::new(vec![ROOT]));
        assert_eq!(layout.children[0].path, Path::new(vec![ROOT, A]));
        assert_eq!(
            layout.children[0].children[0].path,
            Path::new(vec![ROOT, A, B])
        );
    }

    #[test]
    fn duplicate_child_ids_under_different_parents_have_distinct_paths() {
        let root = Node::container(ROOT, layout::Axis::Vertical)
            .with_child(Node::container(A, layout::Axis::Vertical).with_child(Node::leaf(C)))
            .with_child(Node::container(B, layout::Axis::Vertical).with_child(Node::leaf(C)));
        let mut tree = Tree::new();

        tree.set_root(root);
        let layout = layout(&tree);

        assert_ne!(
            layout.children[0].children[0].path,
            layout.children[1].children[0].path
        );
        assert_eq!(
            layout.children[0].children[0].path,
            Path::new(vec![ROOT, A, C])
        );
        assert_eq!(
            layout.children[1].children[0].path,
            Path::new(vec![ROOT, B, C])
        );
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
            layout.hit_test_where(point::logical(5.0, 25.0), |path| interactivity
                .get(path)
                .is_some_and(|interactivity| interactivity.hit_test)),
            Some(Path::new(vec![ROOT, B]))
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
            layout.hit_test_where(point::logical(90.0, 70.0), |path| interactivity
                .get(path)
                .is_some_and(|interactivity| interactivity.hit_test)),
            None
        );
        assert_eq!(
            layout.hit_test_where(point::logical(5.0, 5.0), |path| interactivity
                .get(path)
                .is_some_and(|interactivity| interactivity.hit_test)),
            Some(Path::new(vec![ROOT, A]))
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

        assert_eq!(scene.items().len(), 2);
        assert_eq!(quad(&scene, 0).rect, layout.rect);
        assert_eq!(quad(&scene, 1).rect, layout.children[0].rect);
    }

    #[test]
    fn labeled_button_stores_label_document() {
        let button = control::labeled_button(A, CLICK, "Activate");

        let label = button.label.expect("button should have a label");
        assert_eq!(label.blocks()[0].align(), text::Align::Center);
        assert_eq!(label.blocks()[0].runs()[0].text(), "Activate");
    }

    #[test]
    fn tree_paint_emits_label_after_node_background() {
        let root = Node::container(ROOT, layout::Axis::Vertical)
            .with_background(paint::Color::BLACK)
            .with_child(control::labeled_button(A, CLICK, "Activate"));
        let mut tree = Tree::new();
        let mut scene = paint::Scene::new();
        let mut registry = action::Registry::new();
        let window = window::Id::new(1);

        registry.register(action::Action::new(CLICK, "Click"));
        tree.set_root(root);
        let layout = layout(&tree);
        tree.paint(
            &layout,
            &registry,
            window,
            Interaction::default(),
            &mut scene,
        );

        assert_eq!(scene.items().len(), 3);
        assert_eq!(quad(&scene, 0).rect, layout.rect);
        assert_eq!(quad(&scene, 1).rect, layout.children[0].rect);
        assert_eq!(text(&scene, 2).rect, layout.children[0].rect);
        assert_eq!(
            text(&scene, 2).document.blocks()[0].runs()[0].text(),
            "Activate"
        );
    }

    #[test]
    fn later_sibling_quad_renders_after_button_label() {
        let root = Node::container(ROOT, layout::Axis::Vertical)
            .with_child(control::labeled_button(A, CLICK, "Activate"))
            .with_child(Node::leaf(B).with_background(paint::Color::RED));
        let mut tree = Tree::new();
        let mut scene = paint::Scene::new();
        let mut registry = action::Registry::new();
        let window = window::Id::new(1);

        registry.register(action::Action::new(CLICK, "Click"));
        tree.set_root(root);
        let layout = layout(&tree);
        tree.paint(
            &layout,
            &registry,
            window,
            Interaction::default(),
            &mut scene,
        );

        assert_eq!(scene.items().len(), 3);
        assert_eq!(quad(&scene, 0).rect, layout.children[0].rect);
        assert_eq!(text(&scene, 1).rect, layout.children[0].rect);
        assert_eq!(quad(&scene, 2).rect, layout.children[1].rect);
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
            action::Context::path(window, path(A)),
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
            quad(&scene, 0).style.fill,
            Some(paint::Fill::Brush(paint::Brush::Solid(paint::Color::BLACK)))
        );
    }

    #[test]
    fn disabled_button_uses_disabled_label_color() {
        let root = control::labeled_button(A, CLICK, "Disabled");
        let mut tree = Tree::new();
        let mut scene = paint::Scene::new();
        let mut registry = action::Registry::new();
        let window = window::Id::new(1);

        registry.register(action::Action::new(CLICK, "Click"));
        registry.set_state(
            CLICK,
            action::Context::path(window, path(A)),
            action::State::disabled(),
        );
        tree.set_root(root.clone());
        let layout = layout(&tree);
        tree.paint(
            &layout,
            &registry,
            window,
            Interaction::default(),
            &mut scene,
        );

        assert_eq!(
            text(&scene, 1).document.blocks()[0].runs()[0].style().color,
            root.style
                .disabled_label_color
                .expect("control has disabled label color")
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
                hovered: Some(path(A)),
                focused: None,
                pressed: None,
            },
            &mut scene,
        );

        assert_eq!(
            quad(&scene, 0).style.fill,
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
                hovered: Some(path(B)),
                focused: Some(path(A)),
                pressed: None,
            },
            &mut scene,
        );

        assert_eq!(
            quad(&scene, 0).style.fill,
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
            action::Context::path(window, path(A)),
            action::State::active(),
        );
        tree.set_root(root.clone());
        let layout = layout(&tree);
        tree.paint(
            &layout,
            &registry,
            window,
            Interaction {
                hovered: Some(path(A)),
                focused: Some(path(A)),
                pressed: None,
            },
            &mut scene,
        );

        assert_eq!(
            quad(&scene, 0).style.fill,
            Some(paint::Fill::Brush(paint::Brush::Solid(
                root.style
                    .active_background
                    .expect("control has active color")
            )))
        );
    }

    #[test]
    fn enabled_inactive_action_node_uses_base_background() {
        let root = control::button(A, CLICK);
        let mut tree = Tree::new();
        let mut scene = paint::Scene::new();
        let mut registry = action::Registry::new();
        let window = window::Id::new(1);

        registry.register(action::Action::new(CLICK, "Click"));
        registry.set_state(
            CLICK,
            action::Context::path(window, path(A)),
            action::State {
                enabled: true,
                active: false,
            },
        );
        tree.set_root(root.clone());
        let layout = layout(&tree);
        tree.paint(
            &layout,
            &registry,
            window,
            Interaction::default(),
            &mut scene,
        );

        assert_eq!(
            quad(&scene, 0).style.fill,
            Some(paint::Fill::Brush(paint::Brush::Solid(
                root.style.background.expect("control has base color")
            )))
        );
    }
}
