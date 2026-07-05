use super::*;

#[test]
fn tab_key_moves_focus_through_control_gallery_widgets() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    app.render_scene(window, size)
        .expect("control gallery should render before tab navigation");

    let expected = [
        (view::node::Role::Menu, Some("Controls")),
        (view::node::Role::Menu, Some("View")),
        (view::node::Role::Button, Some("Click")),
        (view::node::Role::Button, Some("Reset")),
        (view::node::Role::Checkbox, Some("Wrap text")),
        (view::node::Role::Checkbox, Some("Show grid")),
        (view::node::Role::Checkbox, Some("Advanced")),
        (view::node::Role::Radio, Some("Design")),
        (view::node::Role::Radio, Some("Inspect")),
        (view::node::Role::Radio, Some("Preview")),
        (view::node::Role::TextBox, None),
        (view::node::Role::Slider, Some("Level: 42.00")),
    ];

    for (role, label) in expected {
        let outcome = app
            .handle_input(window, Input::key_down(input::Key::Tab, input::Modifiers::default()))
            .expect("tab should navigate focus");
        assert!(outcome.is_handled());
        assert!(!outcome.changed_state());
        assert!(
            app.session().windows()[0].redraw_requested(),
            "focus changes should request a redraw"
        );

        let presentation = app
            .render_scene(window, size)
            .expect("control gallery should render focused widget");
        let focused = focused_frame(presentation.layout());

        assert_eq!(focused.role(), role);
        if let Some(label) = label {
            assert_eq!(focused.label_text(), Some(label));
        }
        assert_focus_outline(presentation.scene(), focused);
    }
}

#[test]
fn focused_menu_opens_with_enter_and_tabs_within_popup() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    app.render_scene(window, size)
        .expect("control gallery should render before menu focus");

    app.handle_input(window, Input::key_down(input::Key::Tab, input::Modifiers::default()))
        .expect("tab should focus Controls menu");
    app.handle_input(
        window,
        Input::key_down(input::Key::Enter, input::Modifiers::default()),
    )
    .expect("enter should open focused menu");

    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.open_menu())
            .map(|menu| menu.label()),
        Some("Controls")
    );

    app.render_scene(window, size)
        .expect("open menu should render popup before popup tab navigation");
    app.handle_input(window, Input::key_down(input::Key::Tab, input::Modifiers::default()))
        .expect("tab should move into open popup");

    let popup_focus = app
        .render_scene(window, size)
        .expect("focused popup item should render");
    let focused = focused_frame(popup_focus.layout());
    assert_eq!(focused.role(), view::node::Role::Binding);
    assert_eq!(focused.label_text(), Some("Click"));
    assert_focus_outline(popup_focus.scene(), focused);

    let activated = app
        .handle_input(
            window,
            Input::key_down(input::Key::Enter, input::Modifiers::default()),
        )
        .expect("enter should activate focused popup item");
    assert!(activated.is_handled());
    assert!(activated.changed_state());
    assert_eq!(app.state().clicks, 1);
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.open_menu()),
        None
    );
}

#[test]
fn pointer_down_on_inert_space_clears_widget_focus() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    app.render_scene(window, size)
        .expect("control gallery should render before focus");

    app.handle_input(window, Input::key_down(input::Key::Tab, input::Modifiers::default()))
        .expect("tab should focus the first menu");
    assert!(app.session().focused(window).is_some());

    let cleared = app
        .pointer_down_at(window, size, geometry::Point::new(758, 518))
        .expect("background pointer down should be handled");

    assert!(cleared.is_handled());
    assert!(!cleared.changed_state());
    assert_eq!(app.session().focused(window), None);
}

fn focused_frame(layout: &layout::Layout) -> &layout::frame::Frame {
    layout
        .frames()
        .iter()
        .find(|frame| frame.is_focused())
        .expect("one frame should be focused")
}

fn assert_focus_outline(scene: &Scene, frame: &layout::frame::Frame) {
    let focus = Theme::default().palette().focus;
    assert!(
        scene
            .outlines()
            .iter()
            .any(|outline| outline.rect() == frame.rect() && outline.color() == focus),
        "focused {:?} {:?} should paint a focus outline",
        frame.role(),
        frame.label_text()
    );
    assert!(
        matches!(
            scene.primitives().last(),
            Some(scene::Primitive::Outline(outline))
                if outline.rect() == frame.rect() && outline.color() == focus
        ),
        "focused {:?} {:?} outline should paint as an overlay after content",
        frame.role(),
        frame.label_text()
    );
}
