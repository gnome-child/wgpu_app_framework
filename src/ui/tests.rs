use super::*;
use crate::geometry::{area, point};
use crate::{action, layout, paint, text, window};

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

    assert_eq!(layout.children()[0].rect().area, area::logical(100.0, 20.0));
    assert_eq!(layout.children()[1].rect().area, area::logical(100.0, 60.0));
    assert_eq!(
        layout.children()[1].rect().origin,
        point::logical(0.0, 20.0)
    );
}

#[test]
fn padding_offsets_children() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_padding(layout::Insets::splat(10.0))
        .with_child(Node::leaf(A));

    let mut tree = Tree::new();
    tree.set_root(root);
    let layout = layout(&tree);

    assert_eq!(
        layout.children()[0].rect().origin,
        point::logical(10.0, 10.0)
    );
    assert_eq!(layout.children()[0].rect().area, area::logical(80.0, 60.0));
}

#[test]
fn horizontal_layout_distributes_fill_width() {
    let root = Node::container(ROOT, layout::Axis::Horizontal)
        .with_child(Node::leaf(A))
        .with_child(Node::leaf(B));

    let mut tree = Tree::new();
    tree.set_root(root);
    let layout = layout(&tree);

    assert_eq!(layout.children()[0].rect().area, area::logical(50.0, 80.0));
    assert_eq!(
        layout.children()[1].rect().origin,
        point::logical(50.0, 0.0)
    );
}

#[test]
fn layout_assigns_stable_paths() {
    let root = Node::container(ROOT, layout::Axis::Vertical)
        .with_child(Node::container(A, layout::Axis::Vertical).with_child(Node::leaf(B)));
    let mut tree = Tree::new();

    tree.set_root(root);
    let layout = layout(&tree);

    assert_eq!(layout.path(), &Path::new(vec![ROOT]));
    assert_eq!(layout.children()[0].path(), &Path::new(vec![ROOT, A]));
    assert_eq!(
        layout.children()[0].children()[0].path(),
        &Path::new(vec![ROOT, A, B])
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
        layout.children()[0].children()[0].path(),
        layout.children()[1].children()[0].path()
    );
    assert_eq!(
        layout.children()[0].children()[0].path(),
        &Path::new(vec![ROOT, A, C])
    );
    assert_eq!(
        layout.children()[1].children()[0].path(),
        &Path::new(vec![ROOT, B, C])
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
            .is_some_and(|interactivity| interactivity.hit_test())),
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
            .is_some_and(|interactivity| interactivity.hit_test())),
        None
    );
    assert_eq!(
        layout.hit_test_where(point::logical(5.0, 5.0), |path| interactivity
            .get(path)
            .is_some_and(|interactivity| interactivity.hit_test())),
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
    let registry = action::Registry::<()>::new();

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
    assert_eq!(quad(&scene, 0).rect, layout.rect());
    assert_eq!(quad(&scene, 1).rect, layout.children()[0].rect());
}

#[test]
fn labeled_button_stores_label_document() {
    let button = control::labeled_button(A, CLICK, "Activate");

    let label = button.label().expect("button should have a label");
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
    let mut registry = action::Registry::<()>::new();
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
    assert_eq!(quad(&scene, 0).rect, layout.rect());
    assert_eq!(quad(&scene, 1).rect, layout.children()[0].rect());
    assert_eq!(text(&scene, 2).rect, layout.children()[0].rect());
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
    let mut registry = action::Registry::<()>::new();
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
    assert_eq!(quad(&scene, 0).rect, layout.children()[0].rect());
    assert_eq!(text(&scene, 1).rect, layout.children()[0].rect());
    assert_eq!(quad(&scene, 2).rect, layout.children()[1].rect());
}

#[test]
fn disabled_action_node_renders_disabled_background() {
    let root = Node::leaf(A)
        .with_action(CLICK)
        .with_background(paint::Color::RED)
        .with_disabled_background(paint::Color::BLACK);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
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
    let mut registry = action::Registry::<()>::new();
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
        root.style()
            .disabled_label_color()
            .expect("control has disabled label color")
    );
}

#[test]
fn control_hover_state_chooses_hover_background() {
    let root = control::button(A, CLICK);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(Some(path(A)), None, None),
        &mut scene,
    );

    assert_eq!(
        quad(&scene, 0).style.fill,
        Some(paint::Fill::Brush(paint::Brush::Solid(
            root.style()
                .hover_background()
                .expect("control has hover color")
        )))
    );
}

#[test]
fn control_focus_state_chooses_focus_background() {
    let root = control::button(A, CLICK);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(Some(path(B)), Some(path(A)), None),
        &mut scene,
    );

    assert_eq!(
        quad(&scene, 0).style.fill,
        Some(paint::Fill::Brush(paint::Brush::Solid(
            root.style()
                .focus_background()
                .expect("control has focus color")
        )))
    );
}

#[test]
fn control_active_state_chooses_active_background() {
    let root = control::button(A, CLICK);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
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
        Interaction::new(Some(path(A)), Some(path(A)), None),
        &mut scene,
    );

    assert_eq!(
        quad(&scene, 0).style.fill,
        Some(paint::Fill::Brush(paint::Brush::Solid(
            root.style()
                .active_background()
                .expect("control has active color")
        )))
    );
}

#[test]
fn control_busy_state_chooses_busy_background_before_active_or_hover() {
    let root = control::button(A, CLICK);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_state(
        CLICK,
        action::Context::path(window, path(A)),
        action::State::active().with_busy(true),
    );
    tree.set_root(root.clone());
    let layout = layout(&tree);
    tree.paint(
        &layout,
        &registry,
        window,
        Interaction::new(Some(path(A)), Some(path(A)), None),
        &mut scene,
    );

    assert_eq!(
        quad(&scene, 0).style.fill,
        Some(paint::Fill::Brush(paint::Brush::Solid(
            root.style()
                .busy_background()
                .expect("control has busy color")
        )))
    );
}

#[test]
fn busy_button_uses_busy_label_color() {
    let root = control::labeled_button(A, CLICK, "Working");
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_busy(CLICK, action::Context::path(window, path(A)), true);
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
        root.style()
            .busy_label_color()
            .expect("control has busy label color")
    );
}

#[test]
fn enabled_inactive_action_node_uses_base_background() {
    let root = control::button(A, CLICK);
    let mut tree = Tree::new();
    let mut scene = paint::Scene::new();
    let mut registry = action::Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(action::Action::new(CLICK, "Click"));
    registry.set_state(
        CLICK,
        action::Context::path(window, path(A)),
        action::State::new(true, false),
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
            root.style().background().expect("control has base color")
        )))
    );
}
