use super::*;

#[test]
fn text_editor_menu_open_state_is_framework_owned_interaction() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let file = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should be in the view");
    let action = file.menu_action().expect("menu should expose an action");

    assert!(app.clear_redraw_request(window));

    let outcome = app
        .handle_view(window, action.clone())
        .expect("menu action should be handled");

    assert!(outcome.is_handled());
    assert!(!outcome.changed_state());
    assert!(outcome.effect().contains_invalidation());
    let interaction: &Interaction = app
        .session()
        .interaction(window)
        .expect("window should have interaction state");

    assert_eq!(
        interaction.open_menu().map(|menu| menu.label()),
        Some("File")
    );
    assert!(app.session().windows()[0].redraw_requested());
    assert_eq!(app.revision(), state::Revision::initial());
    let projected = app
        .present(window)
        .expect("window should still have a view");

    assert_eq!(projected.floating_panels().len(), 1);
    assert_eq!(projected.floating_panels()[0].label_text(), None);

    app.clear_redraw_request(window);
    app.handle_view(window, action)
        .expect("second menu action should be handled");

    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.open_menu()),
        None
    );
    let projected = app
        .present(window)
        .expect("window should still have a view");

    assert!(projected.floating_panels().is_empty());
    assert!(app.session().windows()[0].redraw_requested());
}

#[test]
fn hovering_another_menu_title_switches_open_menu() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let initial = app
        .render_scene(window, size)
        .expect("text editor should render");
    let file = labeled_frame(initial.layout(), view::Role::Menu, "File");
    let edit = labeled_frame(initial.layout(), view::Role::Menu, "Edit");

    app.pointer_down_at(window, size, frame_point(file))
        .expect("file menu pointer down should be handled");
    app.pointer_up_at(window, size, frame_point(file))
        .expect("file menu pointer up should open the menu");
    app.render_scene(window, size)
        .expect("open file menu should render");

    let switched = app
        .pointer_move_at(window, size, frame_point(edit))
        .expect("edit menu hover should be handled");

    assert!(switched.is_handled());
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.open_menu())
            .map(|menu| menu.label()),
        Some("Edit")
    );
}

#[test]
fn pointer_down_outside_menu_surface_closes_open_menu() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let initial = app
        .render_scene(window, size)
        .expect("text editor should render");
    let file = labeled_frame(initial.layout(), view::Role::Menu, "File");

    app.pointer_down_at(window, size, frame_point(file))
        .expect("file menu pointer down should be handled");
    app.pointer_up_at(window, size, frame_point(file))
        .expect("file menu pointer up should open the menu");
    let opened = app
        .render_scene(window, size)
        .expect("open file menu should render");
    let text_area = opened
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");

    let outside_popup = geometry::Point::new(
        size.width().saturating_sub(2),
        text_area.rect().y().saturating_add(80),
    );

    app.pointer_down_at(window, size, outside_popup)
        .expect("outside pointer down should be handled");

    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.open_menu()),
        None
    );
}

#[test]
fn menu_command_activation_closes_framework_owned_menu_state() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let file = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should be in the view");
    app.handle_view(
        window,
        file.menu_action().expect("menu should expose an action"),
    )
    .expect("menu action should be handled");

    let projected = app
        .present(window)
        .expect("window should still have a view");
    let open = projected
        .binding::<document::OpenFile>()
        .expect("open command should be in the view")
        .action();

    app.handle_view(window, open)
        .expect("open action should be handled");

    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.open_menu()),
        None
    );
    assert_eq!(
        app.session().file_dialog(window),
        Some(session::FileDialog::Open)
    );
    assert_eq!(app.state().last_status, "choosing file");
    assert_eq!(app.revision().get(), 1);
}

#[test]
fn cancel_input_closes_open_menu_before_clearing_focus() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");
    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    let file = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should be in the view");
    app.handle_view(
        window,
        file.menu_action().expect("menu should expose an action"),
    )
    .expect("menu action should be handled");

    assert_eq!(app.session().focused(window), Some(focus));
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.open_menu())
            .map(|menu| menu.label()),
        Some("File")
    );
    assert!(app.clear_redraw_request(window));

    let outcome = app
        .handle_input(window, Input::cancel())
        .expect("cancel input should be handled");

    assert!(outcome.is_handled());
    assert!(!outcome.changed_state());
    assert!(outcome.effect().contains_invalidation());
    assert_eq!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.open_menu()),
        None
    );
    assert_eq!(app.session().focused(window), Some(focus));
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn cancel_input_clears_focus_when_no_menu_is_open() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");
    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    assert_eq!(app.session().focused(window), Some(focus));
    assert!(app.clear_redraw_request(window));

    let outcome = app
        .handle_input(window, Input::cancel())
        .expect("cancel input should be handled");

    assert!(outcome.is_handled());
    assert!(!outcome.changed_state());
    assert!(outcome.effect().contains_invalidation());
    assert_eq!(app.session().focused(window), None);
    assert!(app.session().windows()[0].redraw_requested());
    assert_eq!(app.revision(), state::Revision::initial());

    let outcome = app
        .handle_input(window, Input::cancel())
        .expect("second cancel input should be ignored");

    assert!(!outcome.is_handled());
    assert!(!outcome.changed_state());
    assert_eq!(outcome.effect(), &response::Effect::None);
}

#[test]
fn pointer_actions_update_framework_owned_hover_and_press_state() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let file = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should be in the view");
    let target = file
        .pointer_target()
        .expect("file menu should have a pointer target");
    let pointer_move = file
        .pointer_move_action()
        .expect("file menu should expose pointer move");
    let pointer_down = file
        .pointer_down_action()
        .expect("file menu should expose pointer down");

    let moved = app
        .handle_view(window, pointer_move)
        .expect("pointer move should be handled");

    assert!(moved.is_handled());
    assert!(!moved.changed_state());
    assert!(moved.effect().contains_invalidation());
    let interaction: &Interaction = app
        .session()
        .interaction(window)
        .expect("window should have interaction state");
    assert_eq!(interaction.pointer().hovered(), Some(&target));
    assert_eq!(interaction.pointer().pressed(), None);
    assert_eq!(interaction.pointer().capture(), None);

    let pressed = app
        .handle_view(window, pointer_down)
        .expect("pointer down should be handled");

    assert!(pressed.is_handled());
    assert!(!pressed.changed_state());
    assert!(pressed.effect().contains_invalidation());
    let interaction = app
        .session()
        .interaction(window)
        .expect("window should have interaction state");
    assert_eq!(interaction.pointer().hovered(), Some(&target));
    assert_eq!(interaction.pointer().pressed(), Some(&target));
    assert_eq!(interaction.pointer().capture(), None);

    let left = app
        .handle_view(window, view::Action::pointer_left())
        .expect("pointer left should be handled");

    assert!(left.is_handled());
    assert!(!left.changed_state());
    assert!(left.effect().contains_invalidation());
    let interaction = app
        .session()
        .interaction(window)
        .expect("window should have interaction state");
    assert_eq!(interaction.pointer().hovered(), None);
    assert_eq!(interaction.pointer().pressed(), None);
    assert_eq!(interaction.pointer().capture(), None);
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn text_area_pointer_down_starts_framework_pointer_capture() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let target = text_area
        .pointer_target()
        .expect("text area should have a pointer target");

    assert!(target.captures());

    let outcome = app
        .handle_view(
            window,
            text_area
                .pointer_down_action()
                .expect("text area should expose pointer down"),
        )
        .expect("text area pointer down should be handled");

    assert!(outcome.is_handled());
    assert!(!outcome.changed_state());
    assert!(outcome.effect().contains_invalidation());
    let pointer = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .pointer();

    assert_eq!(pointer.hovered(), Some(&target));
    assert_eq!(pointer.pressed(), Some(&target));
    assert_eq!(
        pointer.capture().map(|capture| capture.target()),
        Some(&target)
    );
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn text_area_scroll_action_updates_framework_owned_scroll_state() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let target = text_area
        .pointer_target()
        .expect("text area should have a scroll target");
    let revision = app.revision();

    let scrolled = app
        .handle_view(
            window,
            text_area
                .scroll_action(interaction::ScrollDelta::vertical(120))
                .expect("text area should expose scroll"),
        )
        .expect("scroll should be handled");

    assert!(scrolled.is_handled());
    assert!(!scrolled.changed_state());
    assert!(scrolled.effect().contains_invalidation());
    assert_eq!(app.revision(), revision);
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .scroll()
            .offset(&target),
        interaction::ScrollOffset::new(0, 120)
    );
    {
        let diagnostics = app
            .diagnostics(window)
            .expect("window should have diagnostics after scrolling");
        assert_eq!(diagnostics.scroll.wheel_events, 1);
        assert_eq!(diagnostics.scroll.scroll_offset_changes, 1);
        assert_eq!(diagnostics.scroll.scroll_redraw_requests, 1);
    }

    let scrolled_again = app
        .handle_input(
            window,
            Input::scroll(target.clone(), interaction::ScrollDelta::new(8, -20)),
        )
        .expect("scroll input should be handled");

    assert!(scrolled_again.is_handled());
    assert!(!scrolled_again.changed_state());
    assert!(scrolled_again.effect().contains_invalidation());
    assert_eq!(app.revision(), revision);
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .scroll()
            .offset(&target),
        interaction::ScrollOffset::new(8, 100)
    );
    {
        let diagnostics = app
            .diagnostics(window)
            .expect("window should retain diagnostics after scrolling again");
        assert_eq!(diagnostics.scroll.wheel_events, 2);
        assert_eq!(diagnostics.scroll.scroll_offset_changes, 2);
        assert_eq!(diagnostics.scroll.scroll_redraw_requests, 2);
    }
}

#[test]
fn text_area_interaction_id_scrolls_without_focus() {
    let document = (0..120)
        .map(|line| format!("preview line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let buffer = text::Buffer::from_multiline_text(document);
    let edit_state = buffer.initial_state();
    let mut app = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Preview"));
        })
        .view(move |_, _| {
            View::new(
                view::Node::root().child(
                    view::Node::text_area_state(view::control::TextArea::from_buffer(
                        buffer.clone(),
                        edit_state,
                    ))
                    .with_interaction_id("preview"),
                ),
            )
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let presentation = app
        .render_scene(window, size)
        .expect("initial preview scene should render");
    let text_area = presentation
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("preview text area should be laid out");
    let target = text_area
        .target()
        .expect("preview text area should expose a scroll target")
        .clone();
    let point = geometry::Point::new(text_area.rect().x() + 4, text_area.rect().y() + 4);

    assert_eq!(app.session().focused(window), None);
    assert_eq!(target, interaction::Target::text_area_id("preview"));

    let scrolled = app
        .scroll_at(window, size, point, interaction::ScrollDelta::vertical(96))
        .expect("preview scroll should route by hit test");

    assert!(scrolled.is_handled());
    assert!(!scrolled.changed_state());
    assert!(scrolled.effect().contains_invalidation());

    let presentation = app
        .render_scene(window, size)
        .expect("scrolled preview scene should render");

    assert_eq!(app.session().focused(window), None);
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .scroll()
            .offset(&target),
        interaction::ScrollOffset::new(0, 96)
    );
    let text_area = presentation
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("preview text area should be laid out after scrolling");
    assert_eq!(
        text_area
            .text_area_layout()
            .expect("preview should use text area layout")
            .layout()
            .scroll_y(),
        96.0
    );
}

#[test]
fn text_input_preedit_is_framework_owned_and_projected_into_text_area() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let focus = text_area
        .text_area_model()
        .and_then(view::control::TextArea::focus)
        .expect("text area should declare a focus target");
    let target = text_area
        .pointer_target()
        .expect("text area should have an interaction target");
    let preedit = text::edit::Preedit::new("世界", Some((0, "世".len())));

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    let outcome = app
        .handle_input(window, Input::text_preedit(preedit.clone()))
        .expect("preedit input should be handled");

    assert!(outcome.is_handled());
    assert!(!outcome.changed_state());
    assert!(outcome.effect().contains_invalidation());
    assert_eq!(app.revision(), state::Revision::initial());
    let text_input = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .text_input();
    assert_eq!(text_input.target(), Some(&target));
    assert_eq!(text_input.preedit(), Some(&preedit));

    let projected = app
        .present(window)
        .expect("window should project interaction into a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");

    assert_eq!(
        text_area
            .text_area_model()
            .expect("node should contain text area")
            .preedit(),
        Some(&preedit)
    );
}

#[test]
fn text_input_commit_routes_to_focused_document_and_clears_preedit() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let focus = text_area
        .text_area_model()
        .and_then(view::control::TextArea::focus)
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_input(
        window,
        Input::text_preedit(text::edit::Preedit::new("世", Some((0, "世".len())))),
    )
    .expect("preedit input should be handled");

    let outcome = app
        .handle_input(window, Input::text_commit("界"))
        .expect("commit input should be handled");

    assert!(outcome.is_handled());
    assert!(outcome.changed_state());
    assert!(outcome.effect().contains_invalidation());
    assert_eq!(app.state().document.text(), "界");
    assert_eq!(app.state().last_status, "edit");
    assert_eq!(app.revision().get(), 1);
    assert!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .text_input()
            .preedit()
            .is_none()
    );

    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");

    assert!(
        text_area
            .text_area_model()
            .expect("node should contain text area")
            .preedit()
            .is_none()
    );
}

#[test]
fn cancel_input_clears_text_preedit_before_clearing_focus() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let focus = text_area
        .text_area_model()
        .and_then(view::control::TextArea::focus)
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_input(
        window,
        Input::text_preedit(text::edit::Preedit::new("世", Some((0, "世".len())))),
    )
    .expect("preedit input should be handled");

    let canceled = app
        .handle_input(window, Input::cancel())
        .expect("cancel input should clear preedit");

    assert!(canceled.is_handled());
    assert!(!canceled.changed_state());
    assert!(canceled.effect().contains_invalidation());
    assert_eq!(app.session().focused(window), Some(focus));
    assert!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .text_input()
            .preedit()
            .is_none()
    );

    let canceled = app
        .handle_input(window, Input::cancel())
        .expect("second cancel input should clear focus");

    assert!(canceled.is_handled());
    assert_eq!(app.session().focused(window), None);
}

#[test]
fn text_input_preedit_is_transient_and_clears_on_restore() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let focus = text_area
        .text_area_model()
        .and_then(view::control::TextArea::focus)
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    let snapshot = app.snapshot();

    app.handle_input(
        window,
        Input::text_preedit(text::edit::Preedit::new("世", Some((0, "世".len())))),
    )
    .expect("preedit input should be handled");
    assert!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .text_input()
            .preedit()
            .is_some()
    );

    app.restore(snapshot);

    assert_eq!(app.session().focused(window), Some(focus));
    assert!(
        app.session()
            .interaction(window)
            .expect("restored window should have interaction state")
            .text_input()
            .preedit()
            .is_none()
    );
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    assert!(
        text_area
            .text_area_model()
            .expect("node should contain text area")
            .preedit()
            .is_none()
    );
}

#[test]
fn text_area_pointer_click_focuses_and_routes_cursor_edit() {
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_text("hello world"),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let focus = text_area
        .text_area_model()
        .and_then(view::control::TextArea::focus)
        .expect("text area should declare a focus target");
    let target = text_area
        .pointer_target()
        .expect("text area should have a pointer target");

    let outcome = app
        .handle_view(
            window,
            text_area
                .text_pointer_down_action(text::buffer::Position::new(5))
                .expect("text area should expose pointer click"),
        )
        .expect("text area pointer click should be handled");

    assert!(outcome.is_handled());
    assert!(outcome.changed_state());
    assert!(outcome.effect().contains_invalidation());
    let actual_focus = app
        .session()
        .focused(window)
        .expect("text area should be focused");
    assert!(actual_focus.same_target(&focus));
    assert_eq!(actual_focus.reason(), session::Reason::Pointer);
    assert_eq!(actual_focus.visibility(), session::Visibility::Hidden);
    let focused = app
        .render_scene(window, geometry::Size::new(480, 180))
        .expect("pointer-focused text area should render");
    let focused_text_area = focused
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");

    assert!(
        focused.scene().outlines().iter().any(|outline| {
            outline.rect() == focused_text_area.rect()
                && outline.color() == Theme::default().focus().color
        }),
        "pointer-focused text area should paint the focus indicator because it accepts keyboard input"
    );
    assert_eq!(app.state().document.position().index, 5);
    assert_eq!(app.state().document.selected_text(), None);
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .pointer()
            .capture()
            .map(|capture| capture.target()),
        Some(&target)
    );
}

#[test]
fn pointer_left_preserves_captured_text_area_until_release() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let target = text_area
        .pointer_target()
        .expect("text area should have a pointer target");

    app.handle_view(
        window,
        text_area
            .pointer_down_action()
            .expect("text area should expose pointer down"),
    )
    .expect("text area pointer down should be handled");

    let left = app
        .handle_view(window, view::Action::pointer_left())
        .expect("pointer left should be handled");

    assert!(left.is_handled());
    assert!(!left.changed_state());
    assert!(left.effect().contains_invalidation());
    let pointer = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .pointer();

    assert_eq!(pointer.hovered(), None);
    assert_eq!(pointer.pressed(), Some(&target));
    assert_eq!(
        pointer.capture().map(|capture| capture.target()),
        Some(&target)
    );

    let released = app
        .handle_view(window, view::Action::pointer_up(None, None))
        .expect("pointer up should be handled");

    assert!(released.is_handled());
    let pointer = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .pointer();
    assert_eq!(pointer.hovered(), None);
    assert_eq!(pointer.pressed(), None);
    assert_eq!(pointer.capture(), None);
}

#[test]
fn cancel_input_clears_pointer_capture_before_clearing_focus() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let focus = text_area
        .text_area_model()
        .and_then(view::control::TextArea::focus)
        .expect("text area should declare a focus target");
    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_view(
        window,
        text_area
            .pointer_down_action()
            .expect("text area should expose pointer down"),
    )
    .expect("text area pointer down should be handled");
    assert!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().capture())
            .is_some()
    );

    let canceled = app
        .handle_input(window, Input::cancel())
        .expect("cancel input should be handled");

    assert!(canceled.is_handled());
    assert!(!canceled.changed_state());
    assert!(canceled.effect().contains_invalidation());
    assert_eq!(app.session().focused(window), Some(focus));
    let pointer = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .pointer();
    assert_eq!(pointer.pressed(), None);
    assert_eq!(pointer.capture(), None);
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn captured_pointer_drag_routes_text_edit_to_captured_text_area() {
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_text("hello world"),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let target = text_area
        .pointer_target()
        .expect("text area should have a pointer target");
    app.handle_view(
        window,
        text_area
            .text_pointer_down_action(text::buffer::Position::new(0))
            .expect("text area should expose pointer click"),
    )
    .expect("text area pointer down should be handled");

    let dragged = app
        .handle_view(
            window,
            text_area
                .text_pointer_drag_action(text::buffer::Position::new(5))
                .expect("text area should expose pointer drag"),
        )
        .expect("captured pointer drag should be handled");

    assert!(dragged.is_handled());
    assert!(dragged.changed_state());
    assert_eq!(dragged.effect(), &response::Effect::None);
    assert_eq!(app.state().document.text(), "hello world");
    assert_eq!(
        app.state().document.selected_text().as_deref(),
        Some("hello")
    );
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .pointer()
            .capture()
            .map(|capture| capture.target()),
        Some(&target)
    );
}

#[test]
fn pointer_drag_without_matching_capture_does_not_invoke_text_edit() {
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_text("hello world"),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");

    let dragged = app
        .handle_view(
            window,
            text_area
                .text_pointer_drag_action(text::buffer::Position::new(5))
                .expect("text area should expose pointer drag"),
        )
        .expect("uncaptured pointer drag should still be handled as pointer state");

    assert!(dragged.is_handled());
    assert!(!dragged.changed_state());
    assert!(dragged.effect().contains_invalidation());
    assert_eq!(app.state().document.text(), "hello world");
    assert_eq!(app.state().document.selected_text(), None);
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn pointer_release_over_pressed_menu_invokes_menu_action() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let file = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should be in the view");
    let pointer_down = file
        .pointer_down_action()
        .expect("file menu should expose pointer down");
    let pointer_up = file
        .pointer_up_action()
        .expect("file menu should expose pointer up");

    app.handle_view(window, pointer_down)
        .expect("pointer down should be handled");
    let released = app
        .handle_view(window, pointer_up)
        .expect("pointer up should be handled");

    assert!(released.is_handled());
    assert!(!released.changed_state());
    assert!(released.effect().contains_invalidation());
    let interaction = app
        .session()
        .interaction(window)
        .expect("window should have interaction state");
    assert_eq!(interaction.pointer().pressed(), None);
    assert_eq!(
        interaction.open_menu().map(|menu| menu.label()),
        Some("File")
    );
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn pointer_release_over_pressed_command_invokes_typed_command_binding() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let wrap_point = open_view_menu_and_wrap_command_point(&mut app, window, size);

    assert!(app.state().wrap_text);
    app.pointer_down_at(window, size, wrap_point)
        .expect("pointer down should be handled");
    let released = app
        .pointer_up_at(window, size, wrap_point)
        .expect("pointer up should be handled");

    assert!(released.is_handled());
    assert!(released.changed_state());
    assert!(!app.state().wrap_text);
    assert_eq!(app.state().last_status, "wrap text disabled");
    assert_eq!(app.revision().get(), 1);
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .pointer()
            .pressed(),
        None
    );
}

#[test]
fn pointer_release_away_from_pressed_command_does_not_invoke() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(800, 600);
    let wrap_point = open_view_menu_and_wrap_command_point(&mut app, window, size);

    app.pointer_down_at(window, size, wrap_point)
        .expect("pointer down should be handled");
    let released = app
        .handle_view(window, view::Action::pointer_up(None, None))
        .expect("pointer up should be handled");

    assert!(released.is_handled());
    assert!(!released.changed_state());
    assert!(app.state().wrap_text);
    assert_eq!(app.state().last_status, "ready");
    assert_eq!(app.revision(), state::Revision::initial());
    let interaction = app
        .session()
        .interaction(window)
        .expect("window should have interaction state");
    assert_eq!(interaction.pointer().hovered(), None);
    assert_eq!(interaction.pointer().pressed(), None);
}

#[test]
fn text_editor_host_presents_pending_redraws() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    assert!(app.session().windows()[0].redraw_requested());

    let presentations = app.present_pending();

    assert_eq!(presentations.len(), 1);
    assert_eq!(presentations[0].window(), window);
    assert_eq!(
        presentations[0].view().text_areas()[0].wrap(),
        view::control::Wrap::Word
    );
    assert!(!app.session().windows()[0].redraw_requested());
    assert!(app.present_pending().is_empty());

    let wrap_action = presentations[0]
        .view()
        .binding::<text_editor::ToggleWrapText>()
        .expect("wrap command should be in the presented view")
        .action();

    app.handle_view(window, wrap_action)
        .expect("wrap action should be handled");

    assert!(app.session().windows()[0].redraw_requested());

    let presentations = app.present_pending();

    assert_eq!(presentations.len(), 1);
    assert_eq!(presentations[0].window(), window);
    assert_eq!(
        presentations[0].view().text_areas()[0].wrap(),
        view::control::Wrap::None
    );
    assert!(!app.session().windows()[0].redraw_requested());
}

fn labeled_frame<'a>(
    layout: &'a layout::Layout,
    role: view::Role,
    label: &str,
) -> &'a layout::Frame {
    layout
        .find_role(role)
        .into_iter()
        .find(|frame| frame.label_text() == Some(label))
        .expect("labeled frame should be laid out")
}
