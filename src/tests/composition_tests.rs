use super::*;
use crate::composition::{Tree, tree};

#[test]
fn transient_text_projection_advances_the_existing_nodes_content_revision() {
    let focus = session::Focus::text("projected.text");
    let window = window::Id::new(1);
    let authored = || {
        View::new(view::Node::root().child(view::Node::text_box_state(
            view::TextBox::new("").with_focus(focus),
        )))
    };
    let mut store = composition::Store::default();
    store.install(window, authored());
    let installed = store.install(window, authored());
    assert!(installed.changes().is_empty());
    let text = installed.tree().root().children()[0].node_id();
    let initial = installed.tree().node(text).unwrap().content_revision();
    let mut interaction = interaction::Interaction::new(8);
    let target = focus
        .text_target()
        .expect("text focus should identify its draft target");
    assert!(interaction.activate_text_draft(target, "q"));

    let installed = store
        .get_mut(window)
        .expect("the installed composition should remain available");
    installed.project_transient_state(Some(&interaction), Some(focus));
    let projected = installed.tree().node(text).unwrap().content_revision();

    assert_eq!(projected.get(), initial.get() + 1);
    assert_eq!(installed.changes().changed(), &[text]);

    installed.project_transient_state(Some(&interaction), Some(focus));
    assert_eq!(
        installed.tree().node(text).unwrap().content_revision(),
        projected,
        "an unchanged transient projection must not mint another revision"
    );

    let installed = store.install(window, authored());
    assert_eq!(
        installed.tree().node(text).unwrap().content_revision(),
        projected,
        "rebuilding the same authored view must not erase or re-mint projected state"
    );
}

#[test]
fn explicit_ids_preserve_node_ids_across_sibling_movement() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);

    let first = store.install(window, labeled_view(["a", "b"]));
    let a = node_id(&first.tree().root().children()[0]);
    let b = node_id(&first.tree().root().children()[1]);
    let a_revision = first.tree().root().children()[0].content_revision();
    let b_revision = first.tree().root().children()[1].content_revision();

    let second = store.install(window, labeled_view(["b", "a"]));
    assert_eq!(node_id(&second.tree().root().children()[0]), b);
    assert_eq!(node_id(&second.tree().root().children()[1]), a);
    assert_eq!(
        second.tree().root().children()[0].content_revision(),
        b_revision
    );
    assert_eq!(
        second.tree().root().children()[1].content_revision(),
        a_revision
    );
    assert!(second.changes().is_empty());
}

#[test]
fn one_sibling_content_change_mints_only_that_nodes_revision() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);

    let first = store.install(window, labeled_content_view("Before"));
    let root_revision = first.tree().root().content_revision();
    let stable = node_id(&first.tree().root().children()[0]);
    let stable_revision = first.tree().root().children()[0].content_revision();
    let changed = node_id(&first.tree().root().children()[1]);
    let changed_revision = first.tree().root().children()[1].content_revision();

    let second = store.install(window, labeled_content_view("After"));

    assert_eq!(second.changes().changed(), &[changed]);
    assert!(second.changes().added().is_empty());
    assert!(second.changes().removed().is_empty());
    assert_eq!(second.tree().root().content_revision(), root_revision);
    assert_eq!(node_id(&second.tree().root().children()[0]), stable);
    assert_eq!(
        second.tree().root().children()[0].content_revision(),
        stable_revision
    );
    assert_eq!(node_id(&second.tree().root().children()[1]), changed);
    assert_eq!(
        second.tree().root().children()[1].content_revision().get(),
        changed_revision.get() + 1
    );
}

#[test]
fn role_position_preserves_node_ids_for_identical_rebuilds() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);

    let first = store.install(window, anonymous_button_view());
    let button = node_id(&first.tree().root().children()[0]);

    let second = store.install(window, anonymous_button_view());
    assert_eq!(node_id(&second.tree().root().children()[0]), button);
    assert!(second.changes().is_empty());
}

#[test]
fn idless_sibling_insertions_remain_positional_by_design() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);

    let first = store.install(window, anonymous_label_view(["one", "two"]));
    let first_position_id = node_id(&first.tree().root().children()[0]);
    let second_position_id = node_id(&first.tree().root().children()[1]);

    let second = store.install(window, anonymous_label_view(["zero", "one", "two"]));
    assert_eq!(
        node_id(&second.tree().root().children()[0]),
        first_position_id
    );
    assert_eq!(
        node_id(&second.tree().root().children()[1]),
        second_position_id
    );
    assert_eq!(second.changes().added().len(), 1);
    assert!(second.changes().removed().is_empty());
}

#[test]
fn explicit_ids_do_not_survive_cross_parent_reparenting_in_v1() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);

    let first = store.install(window, reparenting_view(true));
    let moved = node_id(&first.tree().root().children()[0].children()[0]);

    let second = store.install(window, reparenting_view(false));
    let readded = node_id(&second.tree().root().children()[1].children()[0]);
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
    let (first, _) = Tree::new(
        &View::new(view::Node::root().child(view::Node::label("Old").with_interaction_id("old"))),
        &mut next_node_id,
    );
    let old_root = node_id(first.root());
    let old_child = node_id(&first.root().children()[0]);

    let (second, changes) = first.reconcile(
        &View::new(view::Node::root().child(view::Node::stack(view::Axis::Vertical))),
        &mut next_node_id,
    );

    assert_eq!(node_id(second.root()), old_root);
    assert_ne!(node_id(&second.root().children()[0]), old_child);
    assert_eq!(changes.removed(), &[old_child]);
    assert_eq!(changes.removed_elements(), &[interaction::Id::new("old")]);
}

#[test]
fn every_view_has_one_stable_root_before_auxiliary_panels_are_projected() {
    let content = view::Node::label("Content").with_interaction_id("stable.content");
    let view = View::new(content);

    assert_eq!(view.root().role(), view::Role::Root);
    assert_eq!(view.root().children().len(), 1);
    assert_eq!(view.root().children()[0].label_text(), Some("Content"));
}

#[test]
fn removed_nodes_and_elements_are_reported_for_pruning() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);

    let first = store.install(window, labeled_view(["a", "b"]));
    let removed = node_id(&first.tree().root().children()[1]);

    let second = store.install(window, labeled_view(["a"]));
    assert_eq!(second.changes().removed(), &[removed]);
    assert_eq!(
        second.changes().removed_elements(),
        &[interaction::Id::new("b")]
    );
}

#[test]
fn retired_duplicate_node_does_not_report_a_still_present_table_cell_removed() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);
    let cell = crate::table::Cell::new(
        interaction::Id::new("retained.table"),
        crate::list::Key::new(7),
        interaction::Id::new("value"),
    );
    let duplicate = || view::Node::label("Value").with_table_cell(cell);
    store.install(
        window,
        View::new(
            view::Node::root().child(
                view::Node::stack(view::Axis::Vertical)
                    .child(duplicate())
                    .child(duplicate()),
            ),
        ),
    );

    let reconciled = store.install(
        window,
        View::new(
            view::Node::root().child(view::Node::stack(view::Axis::Vertical).child(duplicate())),
        ),
    );

    assert_eq!(reconciled.changes().removed().len(), 1);
    assert!(reconciled.changes().removed_table_cells().is_empty());
}

#[test]
fn recycled_list_slot_preserves_row_and_cell_nodes_across_logical_rebind() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);
    let slot = crate::list::Slot::from_test_value(7);

    let first = store.install(window, recycled_table_row_view(slot, 10, "Record 10"));
    let first_row = first.tree().root().children()[0].children()[0].node_id();
    let first_cell = first.tree().root().children()[0].children()[0].children()[0].node_id();

    let rebound = store.install(window, recycled_table_row_view(slot, 90, "Record 90"));
    let rebound_row = rebound.tree().root().children()[0].children()[0].node_id();
    let rebound_cell = rebound.tree().root().children()[0].children()[0].children()[0].node_id();

    assert_eq!(
        rebound_row, first_row,
        "the recycled row slot is presentation identity"
    );
    assert_eq!(
        rebound_cell, first_cell,
        "stable structure inside the recycled slot must retain renderer identity"
    );
    assert!(rebound.changes().added().is_empty());
    assert!(rebound.changes().removed().is_empty());
    assert_eq!(rebound.changes().departed(), &[first_row, first_cell]);
    assert_eq!(
        rebound.changes().removed_table_cells(),
        &[crate::table::Cell::new(
            interaction::Id::new("recycled.table"),
            crate::list::Key::new(10),
            interaction::Id::new("name"),
        )],
        "logical table state must retire while presentation identity is recycled"
    );
}

#[test]
fn removed_idless_text_box_reports_its_focus_element_for_draft_pruning() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);
    let focus = session::Focus::text("conditional.text");
    let first = View::new(view::Node::root().child(view::Node::text_box_state(
        view::TextBox::new("").with_focus(focus),
    )));
    store.install(window, first);

    let second = store.install(window, View::new(view::Node::root()));

    assert_eq!(
        second.changes().removed_elements(),
        &[interaction::Id::new("conditional.text")]
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
        layout::PopupSurfaces::InFrame,
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
    let (retained, _) = Tree::new(&view, &mut next_node_id);
    let layout = tree::Layout::new(&view);

    assert!(node_id(retained.root()).is_retained());
    assert!(!layout.root().node_id().is_retained());
    assert_ne!(node_id(retained.root()), layout.root().node_id());
}

#[test]
fn focused_subject_path_comes_from_composition_ancestry() {
    let mut store = composition::Store::default();
    let window = window::Id::new(1);
    let composition = store.install(
        window,
        View::new(view::Node::root().child(view::Node::text_area_state(
            view::TextArea::new("").with_focus(session::Focus::text("document")),
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

fn labeled_content_view(changing: &'static str) -> View {
    View::new(
        view::Node::root()
            .child(view::Node::label("Stable").with_interaction_id("stable"))
            .child(view::Node::label(changing).with_interaction_id("changing")),
    )
}

fn node_id(node: &tree::Node) -> tree::NodeId {
    node.node_id()
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

fn recycled_table_row_view(slot: crate::list::Slot, key: u64, label: &'static str) -> View {
    let table = interaction::Id::new("recycled.table");
    let column = interaction::Id::new("name");
    let key = crate::list::Key::new(key);
    let cell = crate::table::Cell::new(table, key, column);
    let row = view::Node::stack(view::Axis::Horizontal)
        .with_provided_row(table, key, slot, key.value() as usize)
        .child(view::Node::label(label).with_table_cell(cell));
    View::new(view::Node::root().child(view::Node::stack(view::Axis::Vertical).child(row)))
}
