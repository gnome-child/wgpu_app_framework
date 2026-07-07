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
        (view::node::Role::Menu, Some("Edit")),
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
            .handle_input(
                window,
                Input::key_down(input::Key::Tab, input::Modifiers::default()),
            )
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

    app.handle_input(
        window,
        Input::key_down(input::Key::Tab, input::Modifiers::default()),
    )
    .expect("tab should focus Controls menu");
    let menu_focus = app
        .session()
        .focused(window)
        .expect("controls menu should be focused");
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
    app.handle_input(
        window,
        Input::key_down(input::Key::Tab, input::Modifiers::default()),
    )
    .expect("tab should move into open popup");

    let popup_focus = app
        .render_scene(window, size)
        .expect("focused popup item should render");
    let focused = focused_frame(popup_focus.layout());
    assert_eq!(focused.role(), view::node::Role::Binding);
    assert_eq!(focused.label_text(), Some("Click"));
    assert_focus_outline(popup_focus.scene(), focused);
    assert_focus_outline_after_text(popup_focus.scene(), focused, "Click");

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
    let restored = app
        .session()
        .focused(window)
        .expect("menu close should restore the menu title focus");
    assert!(restored.same_target(&menu_focus));
    assert_eq!(restored.reason(), session::Reason::Keyboard);
    assert_eq!(restored.visibility(), session::Visibility::Visible);
}

#[test]
fn focused_menu_title_outline_paints_below_open_floating_menu() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    app.render_scene(window, size)
        .expect("control gallery should render before menu focus");

    app.handle_input(
        window,
        Input::key_down(input::Key::Tab, input::Modifiers::default()),
    )
    .expect("tab should focus Controls menu");
    app.handle_input(
        window,
        Input::key_down(input::Key::Enter, input::Modifiers::default()),
    )
    .expect("enter should open focused menu");

    let presentation = app
        .render_scene(window, size)
        .expect("open menu should render");
    let menu = find_frame(
        presentation.layout(),
        view::node::Role::Menu,
        Some("Controls"),
    );

    assert_focus_outline(presentation.scene(), menu);
    assert_focus_outline_before_text(presentation.scene(), menu, "Click");
}

#[test]
fn pointer_down_on_inert_space_clears_widget_focus() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    app.render_scene(window, size)
        .expect("control gallery should render before focus");

    app.handle_input(
        window,
        Input::key_down(input::Key::Tab, input::Modifiers::default()),
    )
    .expect("tab should focus the first menu");
    assert!(app.session().focused(window).is_some());

    let cleared = app
        .pointer_down_at(window, size, geometry::Point::new(758, 518))
        .expect("background pointer down should be handled");

    assert!(cleared.is_handled());
    assert!(!cleared.changed_state());
    assert_eq!(app.session().focused(window), None);
}

#[test]
fn pointer_focus_is_hidden_and_tab_continues_from_pointer_target() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    let initial = app
        .render_scene(window, size)
        .expect("control gallery should render before pointer focus");
    let wrap_text = find_frame(
        initial.layout(),
        view::node::Role::Checkbox,
        Some("Wrap text"),
    );
    let point = center(wrap_text.active_rect());

    let pointed = app
        .pointer_down_at(window, size, point)
        .expect("pointer down should focus the checkbox");

    assert!(pointed.is_handled());
    assert_eq!(
        app.session()
            .focused(window)
            .map(|focus| (focus.reason(), focus.visibility())),
        Some((session::Reason::Pointer, session::Visibility::Hidden))
    );

    let hidden_focus = app
        .render_scene(window, size)
        .expect("pointer-focused checkbox should render");
    let focused = focused_frame(hidden_focus.layout());
    assert_eq!(focused.role(), view::node::Role::Checkbox);
    assert_eq!(focused.label_text(), Some("Wrap text"));
    assert_no_focus_outline(hidden_focus.scene(), focused);

    app.handle_input(
        window,
        Input::key_down(input::Key::Tab, input::Modifiers::default()),
    )
    .expect("tab should continue from pointer-focused checkbox");

    let keyboard_focus = app
        .render_scene(window, size)
        .expect("keyboard-focused checkbox should render");
    let focused = focused_frame(keyboard_focus.layout());
    assert_eq!(focused.role(), view::node::Role::Checkbox);
    assert_eq!(focused.label_text(), Some("Show grid"));
    assert_focus_outline(keyboard_focus.scene(), focused);
}

#[test]
fn pointer_opened_menu_resolves_items_against_preserved_document_focus() {
    let mut app = text_editor::app(text_editor::State::default());
    app.start();

    let window = app.session().windows()[0].id();
    let size = text_editor::window_size();
    app.render_scene(window, size)
        .expect("text editor should render before menu interaction");

    let document_focus = session::Focus::text("document");
    app.handle_input(window, Input::focus(document_focus))
        .expect("document focus should be handled");
    for character in "abc".chars() {
        app.handle_input(
            window,
            Input::key_down_with_text(
                input::Key::Character(character),
                input::Modifiers::default(),
                Some(character.to_string()),
            ),
        )
        .expect("typing should edit the document");
    }
    app.handle_input(window, Input::shortcut("Ctrl+A"))
        .expect("select all should create a copyable document selection");

    let before_menu = app
        .render_scene(window, size)
        .expect("text editor should render before opening menu");
    let edit_menu = find_frame(before_menu.layout(), view::node::Role::Menu, Some("Edit"));
    let point = center(edit_menu.rect());
    app.pointer_down_at(window, size, point)
        .expect("menu pointer down should be handled");
    app.pointer_up_at(window, size, point)
        .expect("menu pointer up should open the menu");

    let opened = app
        .render_scene(window, size)
        .expect("open edit menu should render");
    let copy = find_frame(opened.layout(), view::node::Role::Binding, Some("Copy"));
    assert!(
        copy.is_enabled(),
        "menu copy command should resolve against preserved document focus"
    );
    let document = find_frame(opened.layout(), view::node::Role::TextArea, None);
    assert_focus_outline(opened.scene(), document);
    assert_focus_outline_before_text(opened.scene(), document, "Copy");

    let focused = app
        .session()
        .focused(window)
        .expect("document focus should be preserved while pointer menu is open");
    assert!(focused.same_target(&document_focus));
}

#[test]
fn control_gallery_edit_menu_operates_on_focused_text_box() {
    let mut state = control_gallery::State {
        query: "alpha".to_owned(),
        ..control_gallery::State::default()
    };
    state.last_status = "ready".to_owned();
    let mut app = control_gallery::app(state);
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    let query_focus = session::Focus::text("control_gallery.query");
    app.render_scene(window, size)
        .expect("control gallery should render before text box focus");
    app.handle_input(window, Input::focus(query_focus))
        .expect("query text box should focus");

    open_gallery_edit_menu(&mut app, window, size);
    let selected = activate_gallery_edit_binding(&mut app, window, size, "Select All");
    assert!(selected.is_handled());
    assert!(!selected.changed_state());

    open_gallery_edit_menu(&mut app, window, size);
    let copied = activate_gallery_edit_binding(&mut app, window, size, "Copy");
    assert!(copied.is_handled());
    assert_eq!(app.clipboard().text().as_deref(), Some("alpha"));
    assert_eq!(app.state().query, "alpha");

    open_gallery_edit_menu(&mut app, window, size);
    let cut = activate_gallery_edit_binding(&mut app, window, size, "Cut");
    assert!(cut.is_handled());
    assert!(!cut.changed_state());
    assert_eq!(app.state().query, "alpha");
    assert_eq!(text_draft(&app, window, query_focus).text(), "");

    open_gallery_edit_menu(&mut app, window, size);
    let undo_presentation = app
        .render_scene(window, size)
        .expect("open edit menu should render undo state");
    let undo = find_frame(
        undo_presentation.layout(),
        view::node::Role::Binding,
        Some("Undo"),
    );
    assert!(undo.is_enabled(), "text box undo should be menu-enabled");
    let undone = activate_gallery_edit_binding(&mut app, window, size, "Undo");
    assert!(undone.is_handled());
    assert!(!undone.changed_state());
    assert_eq!(app.state().query, "alpha");
    assert_eq!(text_draft(&app, window, query_focus).text(), "alpha");

    open_gallery_edit_menu(&mut app, window, size);
    let redo_presentation = app
        .render_scene(window, size)
        .expect("open edit menu should render redo state");
    let redo = find_frame(
        redo_presentation.layout(),
        view::node::Role::Binding,
        Some("Redo"),
    );
    assert!(redo.is_enabled(), "text box redo should be menu-enabled");
    let redone = activate_gallery_edit_binding(&mut app, window, size, "Redo");
    assert!(redone.is_handled());
    assert!(!redone.changed_state());
    assert_eq!(app.state().query, "alpha");
    assert_eq!(text_draft(&app, window, query_focus).text(), "");

    let committed = app
        .pointer_down_at(window, size, geometry::Point::new(758, 518))
        .expect("background pointer down should commit the text box draft");
    assert!(committed.is_handled());
    assert!(committed.changed_state());
    assert_eq!(app.state().query, "");
}

#[test]
fn text_box_undo_history_survives_blur_while_controls_use_app_undo() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    let query_focus = session::Focus::text("control_gallery.query");
    app.render_scene(window, size)
        .expect("control gallery should render before text input");
    app.handle_input(window, Input::focus(query_focus))
        .expect("query text box should focus");

    let typed = app
        .handle_input(window, Input::text_commit("abc"))
        .expect("typing should update query text box");
    assert!(typed.is_handled());
    assert!(!typed.changed_state());
    assert_eq!(text_draft(&app, window, query_focus).text(), "abc");
    assert_eq!(app.state().query, "");
    assert_eq!(app.timeline().undo_depth(), 0);

    let presentation = app
        .render_scene(window, size)
        .expect("control gallery should render before checkbox click");
    let wrap_text = find_frame(
        presentation.layout(),
        view::node::Role::Checkbox,
        Some("Wrap text"),
    );
    let point = center(wrap_text.active_rect());
    pointer_down_then_present(&mut app, window, size, point);
    let toggled = pointer_up_then_present(&mut app, window, size, point);

    assert!(toggled.is_handled());
    assert!(toggled.changed_state());
    assert!(!app.state().wrap);
    assert_eq!(app.state().query, "abc");
    assert_eq!(app.timeline().undo_depth(), 2);
    assert_eq!(text_draft(&app, window, query_focus).text(), "abc");

    let control_undo = app
        .handle_input(window, Input::shortcut("Ctrl+Z"))
        .expect("unfocused text box should undo the latest app change first");

    assert!(control_undo.is_handled());
    assert!(control_undo.changed_state());
    assert!(app.state().wrap);
    assert_eq!(app.state().query, "abc");

    let text_undo = app
        .handle_input(window, Input::shortcut("Ctrl+Z"))
        .expect("unfocused text box edit should participate in app undo history");

    assert!(text_undo.is_handled());
    assert!(text_undo.changed_state());
    assert_eq!(app.state().query, "");

    let restored = app
        .present(window)
        .expect("control gallery should render restored query");
    let text_box = text_box_node(restored.root())
        .and_then(view::Node::text_box_model)
        .expect("query text box should be in the restored view");
    assert_eq!(text_box.text(), "");
}

#[test]
fn control_gallery_undo_steps_through_mixed_controls_and_text_edits() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);

    drag_gallery_slider_to_fraction(&mut app, window, size, 0.80);
    let first_level = app.state().level;
    assert!(first_level > 70.0);

    click_gallery_frame(
        &mut app,
        window,
        size,
        view::node::Role::Checkbox,
        Some("Wrap text"),
    );
    assert!(!app.state().wrap);

    click_gallery_frame(
        &mut app,
        window,
        size,
        view::node::Role::Radio,
        Some("Inspect"),
    );
    assert_eq!(app.state().mode, control_gallery::Mode::Inspect);

    app.handle_input(
        window,
        Input::focus(session::Focus::text("control_gallery.query")),
    )
    .expect("query text box should focus");
    for character in "abc".chars() {
        app.handle_input(
            window,
            Input::key_down_with_text(
                input::Key::Character(character),
                input::Modifiers::default(),
                Some(character.to_string()),
            ),
        )
        .expect("query text box should accept native-style text key input");
    }
    assert_eq!(
        text_draft(&app, window, session::Focus::text("control_gallery.query")).text(),
        "abc"
    );
    assert_eq!(app.state().query, "");

    drag_gallery_slider_to_fraction(&mut app, window, size, 0.15);
    let second_level = app.state().level;
    assert!(second_level < first_level);
    assert_eq!(app.state().query, "abc");

    click_gallery_frame(
        &mut app,
        window,
        size,
        view::node::Role::Checkbox,
        Some("Show grid"),
    );
    assert!(app.state().grid);
    assert_eq!(app.timeline().undo_depth(), 6);

    app.handle_input(window, Input::shortcut("Ctrl+Z"))
        .expect("undo should restore grid");
    assert!(!app.state().grid);
    assert_near(app.state().level, second_level);
    assert_eq!(app.state().query, "abc");

    app.handle_input(window, Input::shortcut("Ctrl+Z"))
        .expect("undo should restore the second slider gesture");
    assert_near(app.state().level, first_level);
    assert_eq!(app.state().query, "abc");

    app.handle_input(window, Input::shortcut("Ctrl+Z"))
        .expect("undo should restore the text edit");
    assert_eq!(app.state().query, "");
    let restored = app
        .present(window)
        .expect("control gallery should project restored text state");
    let text_box = text_box_node(restored.root())
        .and_then(view::Node::text_box_model)
        .expect("query text box should be present");
    assert_eq!(text_box.text(), "");

    app.handle_input(window, Input::shortcut("Ctrl+Z"))
        .expect("undo should restore the radio selection");
    assert_eq!(app.state().mode, control_gallery::Mode::Design);

    app.handle_input(window, Input::shortcut("Ctrl+Z"))
        .expect("undo should restore the wrap checkbox");
    assert!(app.state().wrap);

    app.handle_input(window, Input::shortcut("Ctrl+Z"))
        .expect("undo should restore the first slider gesture");
    assert_near(app.state().level, 42.0);

    app.handle_input(window, Input::shortcut("Ctrl+Shift+Z"))
        .expect("redo should replay the first slider gesture");
    assert_near(app.state().level, first_level);
}

#[test]
fn edit_menu_undo_uses_committed_text_box_history_after_focus_is_cleared() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    let query_focus = session::Focus::text("control_gallery.query");
    app.render_scene(window, size)
        .expect("control gallery should render before text input");
    app.handle_input(window, Input::focus(query_focus))
        .expect("query text box should focus");
    app.handle_input(window, Input::text_commit("abc"))
        .expect("typing should update query text box");
    assert_eq!(text_draft(&app, window, query_focus).text(), "abc");
    assert_eq!(app.state().query, "");

    let cleared = app
        .pointer_down_at(window, size, geometry::Point::new(758, 518))
        .expect("background pointer down should clear focus");
    assert!(cleared.is_handled());
    assert!(cleared.changed_state());
    assert_eq!(app.session().focused(window), None);
    assert_eq!(app.state().query, "abc");
    assert_eq!(text_draft(&app, window, query_focus).text(), "abc");

    open_gallery_edit_menu(&mut app, window, size);
    let undo_presentation = app
        .render_scene(window, size)
        .expect("open edit menu should render undo state");
    let undo = find_frame(
        undo_presentation.layout(),
        view::node::Role::Binding,
        Some("Undo"),
    );
    assert!(
        undo.is_enabled(),
        "cleared focus should fall back to app-level committed text history"
    );

    let undone = activate_gallery_edit_binding(&mut app, window, size, "Undo");
    assert!(undone.is_handled());
    assert!(undone.changed_state());
    assert_eq!(app.state().query, "");
}

#[test]
fn app_shortcut_commits_and_deactivates_focused_text_box_before_dispatch() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    let query_focus = session::Focus::text("control_gallery.query");
    app.render_scene(window, size)
        .expect("control gallery should render before text input");
    app.handle_input(window, Input::focus(query_focus))
        .expect("query text box should focus");
    app.handle_input(window, Input::text_commit("abc"))
        .expect("typing should update query draft");

    let reset = app
        .handle_input(window, Input::shortcut("Ctrl+R"))
        .expect("reset shortcut should dispatch");

    assert!(reset.is_handled());
    assert!(reset.changed_state());
    assert_eq!(app.state().query, "");
    assert_eq!(text_draft(&app, window, query_focus).text(), "abc");

    let rendered = app
        .render_scene(window, size)
        .expect("control gallery should render reset state");
    let text_box = rendered
        .layout()
        .find_role(view::node::Role::TextBox)
        .into_iter()
        .next()
        .and_then(layout::Frame::text_box)
        .expect("query text box should be present");
    assert_eq!(
        text_box.text(),
        "",
        "reset model text should win over the inactive stale draft"
    );

    app.handle_input(window, Input::text_commit("z"))
        .expect("typing after reset should rebase the draft");
    assert_eq!(text_draft(&app, window, query_focus).text(), "z");
}

#[test]
fn app_menu_command_commits_and_deactivates_focused_text_box_before_dispatch() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 520);
    let query_focus = session::Focus::text("control_gallery.query");
    app.render_scene(window, size)
        .expect("control gallery should render before text input");
    app.handle_input(window, Input::focus(query_focus))
        .expect("query text box should focus");
    app.handle_input(window, Input::text_commit("abc"))
        .expect("typing should update query draft");

    open_gallery_menu(&mut app, window, size, "Controls");
    let reset = activate_gallery_edit_binding(&mut app, window, size, "Reset");

    assert!(reset.is_handled());
    assert!(reset.changed_state());
    assert_eq!(app.state().query, "");

    let rendered = app
        .render_scene(window, size)
        .expect("control gallery should render reset state");
    let text_box = rendered
        .layout()
        .find_role(view::node::Role::TextBox)
        .into_iter()
        .next()
        .and_then(layout::Frame::text_box)
        .expect("query text box should be present");
    assert_eq!(text_box.text(), "");
}

fn open_gallery_edit_menu(
    app: &mut Runtime<control_gallery::State, (), View>,
    window: window::Id,
    size: geometry::Size,
) {
    open_gallery_menu(app, window, size, "Edit");
}

fn open_gallery_menu(
    app: &mut Runtime<control_gallery::State, (), View>,
    window: window::Id,
    size: geometry::Size,
    label: &'static str,
) {
    let presentation = app
        .render_scene(window, size)
        .expect("control gallery should render before opening menu");
    let menu = find_frame(presentation.layout(), view::node::Role::Menu, Some(label));
    let point = center(menu.rect());

    pointer_down_then_present(app, window, size, point);
    pointer_up_then_present(app, window, size, point);
}

fn activate_gallery_edit_binding(
    app: &mut Runtime<control_gallery::State, (), View>,
    window: window::Id,
    size: geometry::Size,
    label: &'static str,
) -> input::Outcome {
    let presentation = app
        .render_scene(window, size)
        .expect("open edit menu should render binding");
    let binding = find_frame(
        presentation.layout(),
        view::node::Role::Binding,
        Some(label),
    );
    assert!(binding.is_enabled(), "{label} should be enabled");
    let point = center(binding.rect());

    pointer_down_then_present(app, window, size, point);
    pointer_up_then_present(app, window, size, point)
}

fn click_gallery_frame(
    app: &mut Runtime<control_gallery::State, (), View>,
    window: window::Id,
    size: geometry::Size,
    role: view::node::Role,
    label: Option<&str>,
) -> input::Outcome {
    let presentation = app
        .render_scene(window, size)
        .expect("control gallery should render before pointer click");
    let frame = find_frame(presentation.layout(), role, label);
    let point = center(frame.active_rect());

    pointer_down_then_present(app, window, size, point);
    pointer_up_then_present(app, window, size, point)
}

fn drag_gallery_slider_to_fraction(
    app: &mut Runtime<control_gallery::State, (), View>,
    window: window::Id,
    size: geometry::Size,
    fraction: f64,
) {
    let presentation = app
        .render_scene(window, size)
        .expect("control gallery should render before slider drag");
    let slider = first_frame(presentation.layout(), view::node::Role::Slider);
    let track = slider_track_rect(slider);
    let x = track
        .x()
        .saturating_add((track.width() as f64 * fraction).round() as i32);
    let point = geometry::Point::new(x.clamp(track.x(), track.right()), track.y() + 1);

    pointer_down_then_present(app, window, size, point);
    pointer_up_then_present(app, window, size, point);
}

fn focused_frame(layout: &layout::Layout) -> &layout::Frame {
    layout
        .frames()
        .iter()
        .find(|frame| frame.is_focused())
        .expect("one frame should be focused")
}

fn first_frame(layout: &layout::Layout, role: view::node::Role) -> &layout::Frame {
    layout
        .frames()
        .iter()
        .find(|frame| frame.role() == role)
        .expect("frame should exist")
}

fn find_frame<'a>(
    layout: &'a layout::Layout,
    role: view::node::Role,
    label: Option<&str>,
) -> &'a layout::Frame {
    layout
        .frames()
        .iter()
        .find(|frame| frame.role() == role && frame.label_text() == label)
        .expect("frame should exist")
}

fn slider_track_rect(frame: &layout::Frame) -> geometry::Rect {
    let theme = Theme::default();
    layout::control::slider_track_rect(frame.rect(), frame.label_width(), &theme)
}

fn center(rect: geometry::Rect) -> geometry::Point {
    geometry::Point::new(
        rect.x().saturating_add(rect.width() / 2),
        rect.y().saturating_add(rect.height() / 2),
    )
}

fn assert_focus_outline(scene: &Scene, frame: &layout::Frame) {
    focus_outline_index(scene, frame).unwrap_or_else(|| {
        panic!(
            "focused {:?} {:?} should paint a focus outline",
            frame.role(),
            frame.label_text()
        )
    });
}

fn assert_focus_outline_after_text(scene: &Scene, frame: &layout::Frame, text: &str) {
    let outline = focus_outline_index(scene, frame).expect("focus outline should paint");
    let text = text_primitive_index(scene, text);

    assert!(
        text < outline,
        "focus outline should paint above content in the same layer"
    );
}

fn assert_focus_outline_before_text(scene: &Scene, frame: &layout::Frame, text: &str) {
    let outline = focus_outline_index(scene, frame).expect("focus outline should paint");
    let text = text_primitive_index(scene, text);

    assert!(
        outline < text,
        "focus outline should not paint above content in a later layer"
    );
}

fn focus_outline_index(scene: &Scene, frame: &layout::Frame) -> Option<usize> {
    let focus = Theme::default().focus().color;

    scene.primitives().iter().position(|primitive| {
        matches!(
            primitive,
            scene::Primitive::Outline(outline)
                if outline.rect() == frame.active_rect() && outline.color() == focus
        )
    })
}

fn text_primitive_index(scene: &Scene, value: &str) -> usize {
    scene
        .primitives()
        .iter()
        .rposition(|primitive| {
            matches!(
                primitive,
                scene::Primitive::Text(text) if text.value() == value
            )
        })
        .unwrap_or_else(|| panic!("scene text {value:?} should paint"))
}

fn assert_no_focus_outline(scene: &Scene, frame: &layout::Frame) {
    let focus = Theme::default().focus().color;
    assert!(
        !scene
            .outlines()
            .iter()
            .any(|outline| outline.rect() == frame.active_rect() && outline.color() == focus),
        "pointer-focused {:?} {:?} should hide the focus outline",
        frame.role(),
        frame.label_text()
    );
}
