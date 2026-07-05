use super::*;

#[test]
fn text_box_on_submit_invokes_command_with_draft_text() {
    let focus = session::Focus::text("search");
    let mut app = Runtime::new(TextBoxSubmitState {
        submitted: "Mar".to_owned(),
        ..TextBoxSubmitState::default()
    })
    .commands(|commands| {
        commands.register::<SubmitText>(command::Spec::new("Submit Text"));
    })
    .responders(|responders| {
        responders.app().target::<SubmitText>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Text Box"));
    })
    .view(move |state, _| {
        widget::view(|ui| {
            ui.text_box(
                widget::TextBox::new(state.submitted.clone())
                    .placeholder("Search")
                    .focus(focus)
                    .on_submit::<SubmitText>(),
            );
        })
    });

    app.start();

    let window = app.session().windows()[0].id();
    app.render_scene(window, geometry::Size::new(240, 80))
        .expect("text box view should render");
    app.handle_input(window, Input::focus(focus))
        .expect("text box focus should be handled");

    let submitted = app
        .handle_input(window, Input::text_commit("s"))
        .expect("text box commit should be handled");

    assert!(submitted.is_handled());
    assert!(submitted.changed_state());
    assert_eq!(app.state().submitted, "Mars");
    assert_eq!(app.state().source, Some(context::Source::Input));
    assert_eq!(app.revision().get(), 1);
}

#[test]
fn text_box_submit_with_maps_committed_text_into_custom_command_args() {
    let focus = session::Focus::text("filter");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .commands(|commands| {
            commands.register::<SubmitMappedText>(command::Spec::new("Submit Mapped Text"));
        })
        .responders(|responders| {
            responders.app().target::<SubmitMappedText>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Mapped Text Box"));
        })
        .view(move |state, _| {
            widget::view(|ui| {
                ui.text_box(
                    widget::TextBox::new(state.submitted.clone())
                        .focus(focus)
                        .submit_with::<SubmitMappedText, _>(|text| TextSubmitArgs {
                            normalized: text.trim().to_ascii_lowercase(),
                            raw: text,
                        }),
                );
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    app.render_scene(window, geometry::Size::new(240, 80))
        .expect("mapped text box view should render");
    app.handle_input(window, Input::focus(focus))
        .expect("mapped text box focus should be handled");

    let submitted = app
        .handle_input(window, Input::text_commit("  Rust  "))
        .expect("mapped text box commit should be handled");

    assert!(submitted.is_handled());
    assert!(submitted.changed_state());
    assert_eq!(app.state().submitted, "  Rust  ");
    assert_eq!(app.state().normalized, "rust");
    assert_eq!(app.state().source, Some(context::Source::Input));
    assert_eq!(app.revision().get(), 1);
}

#[test]
fn unbound_text_box_commit_updates_framework_owned_draft() {
    let focus = session::Focus::text("plain-text-box");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Plain Text Box"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(widget::TextBox::new("").focus(focus));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    app.render_scene(window, geometry::Size::new(240, 80))
        .expect("plain text box view should render");
    app.handle_input(window, Input::focus(focus))
        .expect("plain text box focus should be handled");

    let submitted = app
        .handle_input(window, Input::text_commit("ignored"))
        .expect("plain text box commit should not error");

    assert!(submitted.is_handled());
    assert!(!submitted.changed_state());
    assert_eq!(submitted.effect(), &response::Effect::Repaint);
    assert_eq!(app.state().submitted, "");
    assert_eq!(app.revision(), state::Revision::initial());

    let target = interaction::Target::text_area(focus);
    let draft = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .text_input()
        .draft_for(&target)
        .expect("text box input should own a draft");

    assert_eq!(draft.text(), "ignored");
    assert_eq!(draft.cursor(), "ignored".len());

    let projected = app
        .present(window)
        .expect("text box draft should project into the view");
    let text_box = text_box_node(projected.root()).expect("text box should be in the view");
    let text_box = text_box
        .text_box_model()
        .expect("node should contain text box state");

    assert_eq!(text_box.text(), "ignored");
    assert_eq!(text_box.cursor(), Some("ignored".len()));
}

#[test]
fn character_key_updates_focused_text_box_draft_and_focus_outline() {
    let focus = session::Focus::text("find");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Text Box Key Input"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(widget::TextBox::new("").focus(focus));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    app.present(window)
        .expect("view should be presented before key input");
    app.handle_input(window, Input::focus(focus))
        .expect("text box focus should be handled");

    let typed = app
        .handle_input(
            window,
            Input::key_down(input::Key::Character('q'), input::Modifiers::default()),
        )
        .expect("character key should update text box draft");

    assert!(typed.is_handled());
    assert!(!typed.changed_state());
    assert_eq!(typed.effect(), &response::Effect::Repaint);
    assert_eq!(app.revision(), state::Revision::initial());

    let target = interaction::Target::text_area(focus);
    let draft = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .text_input()
        .draft_for(&target)
        .expect("text box should retain a draft");

    assert_eq!(draft.text(), "q");
    assert_eq!(draft.cursor(), 1);

    let rendered = app
        .render_scene(window, geometry::Size::new(240, 80))
        .expect("focused text box should render");
    let text_box = rendered
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("text box should be laid out");

    assert!(text_box.is_focused());
    assert!(
        rendered
            .scene()
            .outlines()
            .iter()
            .any(|outline| outline.rect() == text_box.rect()
                && outline.color().channels() == (76, 132, 255, 255))
    );
}

#[test]
fn tab_key_moves_focus_through_current_view_order() {
    let first = session::Focus::text("first");
    let document = session::Focus::text("document");
    let second = session::Focus::text("second");
    let mut app = Runtime::new(text_editor::State {
        document: TextDocument::from_multiline_text("body"),
        ..text_editor::State::default()
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Tab Navigation"));
    })
    .view(move |state, _| {
        widget::view(|ui| {
            ui.column(|ui| {
                ui.text_box(widget::TextBox::new("first").focus(first));
                ui.text_area(
                    widget::TextArea::from_buffer(
                        state.document.buffer().clone(),
                        state.document.edit_state(),
                    )
                    .focus(document),
                );
                ui.text_box(widget::TextBox::new("second").focus(second));
            });
        })
    });

    app.start();

    let window = app.session().windows()[0].id();
    app.present(window)
        .expect("view should be presented before key navigation");

    let focused_first = app
        .handle_input(
            window,
            Input::key_down(input::Key::Tab, input::Modifiers::default()),
        )
        .expect("tab should navigate to first focusable node");

    assert!(focused_first.is_handled());
    assert!(!focused_first.changed_state());
    assert_eq!(focused_first.effect(), &response::Effect::Repaint);
    assert_eq!(app.session().focused(window), Some(first));

    app.handle_input(
        window,
        Input::key_down(input::Key::Tab, input::Modifiers::default()),
    )
    .expect("tab should navigate to the document");

    assert_eq!(app.session().focused(window), Some(document));

    let document_text = app.state().document.text();
    app.handle_input(
        window,
        Input::key_down(input::Key::Tab, input::Modifiers::default()),
    )
    .expect("tab from the document should navigate instead of editing text");

    assert_eq!(app.session().focused(window), Some(second));
    assert_eq!(app.state().document.text(), document_text);

    app.handle_input(
        window,
        Input::key_down(input::Key::Tab, input::Modifiers::default()),
    )
    .expect("tab should wrap to first focusable node");

    assert_eq!(app.session().focused(window), Some(first));

    app.handle_input(
        window,
        Input::key_down(
            input::Key::Tab,
            input::Modifiers::new(true, false, false, false),
        ),
    )
    .expect("shift-tab should wrap backward");

    assert_eq!(app.session().focused(window), Some(second));

    app.handle_input(
        window,
        Input::key_down(
            input::Key::Tab,
            input::Modifiers::new(true, false, false, false),
        ),
    )
    .expect("shift-tab should navigate backward through focus order");

    assert_eq!(app.session().focused(window), Some(document));
}

#[test]
fn text_box_cursor_moves_before_next_text_commit() {
    let focus = session::Focus::text("find");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Text Box Cursor"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(widget::TextBox::new("ab").focus(focus));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    app.render_scene(window, geometry::Size::new(240, 80))
        .expect("text box view should render");
    app.handle_input(window, Input::focus(focus))
        .expect("text box focus should be handled");
    app.handle_input(window, Input::text_commit("c"))
        .expect("text box commit should be handled");
    app.handle_input(
        window,
        Input::key_down(input::Key::ArrowLeft, input::Modifiers::default()),
    )
    .expect("left arrow should be handled by the text box");
    app.handle_input(window, Input::text_commit("X"))
        .expect("second text box commit should be handled");

    let target = interaction::Target::text_area(focus);
    let draft = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .text_input()
        .draft_for(&target)
        .expect("text box input should retain a draft");

    assert_eq!(draft.text(), "abXc");
    assert_eq!(draft.cursor(), "abX".len());
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn text_box_pointer_click_positions_framework_owned_draft_caret() {
    let focus = session::Focus::text("find");
    let mut app = Runtime::new(TextBoxSubmitState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Text Box Pointer"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.text_box(widget::TextBox::new("abcd").focus(focus));
            })
        });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(240, 80);
    let presentation = app
        .render_scene(window, size)
        .expect("text box view should render");
    let text_box = presentation
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("text box should be laid out");
    let rect = text_box.rect();
    let left_edge = geometry::Point::new(rect.x() + 1, rect.y() + rect.height() / 2);

    let clicked = app
        .pointer_down_at(window, size, left_edge)
        .expect("text box pointer down should be handled");

    assert!(clicked.is_handled());
    assert!(!clicked.changed_state());

    let target = interaction::Target::text_area(focus);
    let draft = app
        .session()
        .interaction(window)
        .expect("window should have interaction state")
        .text_input()
        .draft_for(&target)
        .expect("text box pointer click should create a draft");

    assert_eq!(draft.text(), "abcd");
    assert_eq!(draft.cursor(), 0);

    let committed = app
        .handle_input(window, Input::text_commit("X"))
        .expect("text box commit should be handled");

    assert!(committed.is_handled());
    assert!(!committed.changed_state());

    let draft = app
        .session()
        .interaction(window)
        .expect("window should keep interaction state")
        .text_input()
        .draft_for(&target)
        .expect("text box draft should remain active");

    assert_eq!(draft.text(), "Xabcd");
    assert_eq!(draft.cursor(), "X".len());

    let projected = app
        .present(window)
        .expect("text box draft should project into the view");
    let text_box = text_box_node(projected.root()).expect("text box should be in the view");
    let text_box = text_box
        .text_box_model()
        .expect("node should contain text box state");

    assert_eq!(text_box.text(), "Xabcd");
    assert_eq!(text_box.cursor(), Some("X".len()));
    assert_eq!(app.revision(), state::Revision::initial());
}
