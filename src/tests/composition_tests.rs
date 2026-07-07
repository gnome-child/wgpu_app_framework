use super::*;

#[test]
fn explicit_ids_preserve_node_ids_across_sibling_movement() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);

    let first = store.install(window, labeled_view(["a", "b"]));
    let a = retained_id(&first.tree().root().children()[0]);
    let b = retained_id(&first.tree().root().children()[1]);

    let second = store.install(window, labeled_view(["b", "a"]));
    assert_eq!(retained_id(&second.tree().root().children()[0]), b);
    assert_eq!(retained_id(&second.tree().root().children()[1]), a);
    assert!(second.changes().is_empty());
}

#[test]
fn role_position_preserves_node_ids_for_identical_rebuilds() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);

    let first = store.install(window, anonymous_button_view());
    let button = retained_id(&first.tree().root().children()[0]);

    let second = store.install(window, anonymous_button_view());
    assert_eq!(retained_id(&second.tree().root().children()[0]), button);
    assert!(second.changes().is_empty());
}

#[test]
fn idless_sibling_insertions_remain_positional_by_design() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);

    let first = store.install(window, anonymous_label_view(["one", "two"]));
    let first_slot = retained_id(&first.tree().root().children()[0]);
    let second_slot = retained_id(&first.tree().root().children()[1]);

    let second = store.install(window, anonymous_label_view(["zero", "one", "two"]));
    assert_eq!(retained_id(&second.tree().root().children()[0]), first_slot);
    assert_eq!(
        retained_id(&second.tree().root().children()[1]),
        second_slot
    );
    assert_eq!(second.changes().added().len(), 1);
    assert!(second.changes().removed().is_empty());
}

#[test]
fn explicit_ids_do_not_survive_cross_parent_reparenting_in_v1() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);

    let first = store.install(window, reparenting_view(true));
    let moved = retained_id(&first.tree().root().children()[0].children()[0]);

    let second = store.install(window, reparenting_view(false));
    let readded = retained_id(&second.tree().root().children()[1].children()[0]);
    assert_ne!(readded, moved);
    assert_eq!(second.changes().removed(), &[moved]);
    assert_eq!(
        second.changes().removed_elements(),
        &[interaction::Id::new("moved")]
    );
}

#[test]
fn mismatched_old_node_reports_removed_subtree_before_rebuild() {
    let mut next_node_id = 1;
    let (first, _) = composition::Tree::new(
        &View::new(view::Node::root().child(view::Node::label("Old").with_interaction_id("old"))),
        &mut next_node_id,
    );
    let old_root = retained_id(first.root());
    let old_child = retained_id(&first.root().children()[0]);

    let (second, changes) = first.reconcile(
        &View::new(view::Node::stack(view::Axis::Vertical)),
        &mut next_node_id,
    );

    assert_ne!(retained_id(second.root()), old_root);
    assert_eq!(changes.removed(), &[old_root, old_child]);
    assert_eq!(changes.removed_elements(), &[interaction::Id::new("old")]);
}

#[test]
fn removed_nodes_and_elements_are_reported_for_pruning() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);

    let first = store.install(window, labeled_view(["a", "b"]));
    let removed = retained_id(&first.tree().root().children()[1]);

    let second = store.install(window, labeled_view(["a"]));
    assert_eq!(second.changes().removed(), &[removed]);
    assert_eq!(
        second.changes().removed_elements(),
        &[interaction::Id::new("b")]
    );
}

#[test]
fn idless_binding_hit_targets_use_retained_identity() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);
    let composition = store.install(window, anonymous_button_view());
    let mut layout_engine = layout::Engine::new();
    let layout = layout::Layout::compose_composition_with_theme_at(
        composition,
        geometry::Size::new(120, 40),
        &mut layout_engine,
        &Theme::default(),
        crate::animation::Frame::new(std::time::Instant::now()),
        crate::keymap::Profile::default(),
    );
    let hit = layout
        .hit_test(geometry::Point::new(1, 1))
        .expect("button should be hit");

    assert_eq!(
        hit.target().and_then(interaction::Target::node_id),
        Some(hit.frame().node_id())
    );
}

#[test]
fn view_only_layout_uses_layout_namespace_node_ids() {
    let view = anonymous_button_view();
    let mut next_node_id = 1;
    let (retained, _) = composition::Tree::new(&view, &mut next_node_id);
    let layout = composition::Tree::layout(&view);

    assert!(retained_id(retained.root()).is_retained());
    assert!(!layout.root().node_id().is_retained());
    assert_ne!(retained_id(retained.root()), layout.root().node_id());
}

#[test]
fn focused_subject_path_comes_from_composition_ancestry() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);
    let composition = store.install(
        window,
        View::new(view::Node::root().child(view::Node::text_area_state(
            view::control::TextArea::new("").with_focus(session::Focus::text("document")),
        ))),
    );
    let path = composition.subject_path_for_focus(Some(session::Focus::text("document")));
    let labels = path
        .segments()
        .iter()
        .map(subject::Segment::label)
        .collect::<Vec<_>>();

    assert_eq!(labels, vec!["Application", "Document"]);
}

#[test]
fn inferred_subject_names_keep_non_ascii_text() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);

    let composition = store.install(window, panel_subject_view("検索"));
    let subject = composition.tree().root().children()[0]
        .subject()
        .expect("panel label should infer a subject");

    assert_eq!(subject.name(), "検索");
    assert_eq!(subject.label(), "検索");
}

#[test]
fn inferred_subject_names_fallback_deterministically_for_symbol_only_labels() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);

    let first = store
        .install(window, panel_subject_view("!!!"))
        .tree()
        .root()
        .children()[0]
        .subject()
        .expect("panel label should infer a subject")
        .name()
        .to_owned();
    let second = store
        .install(window, panel_subject_view("!!!"))
        .tree()
        .root()
        .children()[0]
        .subject()
        .expect("panel label should infer a subject")
        .name()
        .to_owned();
    let different = store
        .install(window, panel_subject_view("???"))
        .tree()
        .root()
        .children()[0]
        .subject()
        .expect("panel label should infer a subject")
        .name()
        .to_owned();

    assert!(first.starts_with("subject-"));
    assert_eq!(first, second);
    assert_ne!(first, different);
}

fn labeled_view<const N: usize>(ids: [&'static str; N]) -> View {
    let mut root = view::Node::root();
    for id in ids {
        root = root.child(view::Node::label(id).with_interaction_id(id));
    }
    View::new(root)
}

fn retained_id(node: &composition::Node) -> composition::NodeId {
    node.retained_id()
        .expect("test composition should use retained node ids")
}

fn anonymous_label_view<const N: usize>(labels: [&'static str; N]) -> View {
    let mut root = view::Node::root();
    for label in labels {
        root = root.child(view::Node::label(label));
    }
    View::new(root)
}

fn reparenting_view(left: bool) -> View {
    let moved = view::Node::label("Moved").with_interaction_id("moved");
    let left_panel = if left {
        view::Node::panel()
            .with_interaction_id("left")
            .child(moved.clone())
    } else {
        view::Node::panel().with_interaction_id("left")
    };
    let right_panel = if left {
        view::Node::panel().with_interaction_id("right")
    } else {
        view::Node::panel()
            .with_interaction_id("right")
            .child(moved)
    };

    View::new(view::Node::root().child(left_panel).child(right_panel))
}

fn panel_subject_view(label: &'static str) -> View {
    View::new(view::Node::root().child(view::Node::panel().with_label(label)))
}

fn anonymous_button_view() -> View {
    View::new(view::Node::root().child(widget::Widget::into_node(
        widget::Button::new("Run").trigger::<Ping>(()),
    )))
}
