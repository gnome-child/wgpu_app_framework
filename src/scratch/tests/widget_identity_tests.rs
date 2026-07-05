use super::*;

#[test]
fn command_control_pointer_identity_is_shared_by_node_and_layout_paths() {
    let explicit = widget::Widget::into_node(
        widget::Element::new()
            .id("command.explicit")
            .trigger::<OpenNamed>("explicit".to_owned()),
    );
    let explicit_target = explicit
        .pointer_target()
        .expect("explicit id command control should expose a node target");
    let explicit_view = View::new(view::Node::root().child(explicit));
    let mut layout_engine = layout::engine::Engine::new();
    let explicit_layout = layout::Layout::compose(
        &explicit_view,
        geometry::Size::new(160, 80),
        &mut layout_engine,
    );
    let explicit_hit = explicit_layout
        .hit_test(geometry::Point::new(1, 1))
        .expect("explicit id command control should be hit");

    assert_eq!(explicit_hit.target(), Some(&explicit_target));

    let path_bound = widget::Widget::into_node(
        widget::Button::new("Path").trigger::<OpenNamed>("path".to_owned()),
    );
    assert!(
        path_bound.pointer_target().is_none(),
        "id-less command controls require layout path context"
    );

    let path_view = View::new(view::Node::root().child(path_bound));
    let path_layout =
        layout::Layout::compose(&path_view, geometry::Size::new(160, 80), &mut layout_engine);
    let path_hit = path_layout
        .hit_test(geometry::Point::new(1, 1))
        .expect("path-bound command control should be hit");

    assert_eq!(
        path_hit
            .target()
            .expect("layout path should derive a command target")
            .kind(),
        interaction::target::Kind::Command
    );
}
