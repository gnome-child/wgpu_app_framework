use super::*;

#[test]
fn text_editor_text_area_focus_routes_edits_through_runtime() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");

    assert!(app.focus(window, focus));
    assert_eq!(app.session().focused(window), Some(focus));

    let edit = app.trigger::<document::ApplyEdit>(text::edit::Edit::insert("alpha"));
    let response = app.invoke_focused(window, edit);

    assert!(
        response
            .output
            .expect("focused edit should resolve")
            .text_changed()
    );
    assert_eq!(app.state().document.text(), "alpha");
    assert!(app.state().document.is_dirty());
    assert_eq!(app.revision().get(), 1);
    assert_eq!(
        app.store().changes()[0].reason(),
        &state::Reason::Command("document.apply_edit")
    );

    let projected = app
        .present(window)
        .expect("window should still have a view");

    assert_eq!(projected.text_areas()[0].buffer().text(), "alpha");
}

#[test]
fn text_editor_input_flow_is_framework_owned() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");

    let focus_outcome = app
        .handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");

    assert!(focus_outcome.is_handled());
    assert!(!focus_outcome.changed_state());
    assert_eq!(focus_outcome.effect(), &response::Effect::Repaint);

    let edit_outcome = app
        .handle_input(window, Input::text_edit(text::edit::Edit::insert("alpha")))
        .expect("text edit input should be handled");

    assert!(edit_outcome.is_handled());
    assert!(edit_outcome.changed_state());
    assert_eq!(app.state().document.text(), "alpha");
    assert_eq!(app.state().last_status, "edit");
    assert_eq!(app.revision().get(), 1);

    let projected = app
        .present(window)
        .expect("window should still have a view");

    assert_eq!(projected.text_areas()[0].buffer().text(), "alpha");
}

#[test]
fn text_editor_key_down_routes_editing_keys_to_focused_document() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_input(window, Input::text_commit("abc"))
        .expect("text commit should be handled");

    let moved = app
        .handle_input(
            window,
            Input::key_down(input::Key::ArrowLeft, input::Modifiers::default()),
        )
        .expect("arrow key should be handled");

    assert!(moved.is_handled());
    assert!(moved.changed_state());
    assert_eq!(app.state().document.text(), "abc");
    assert_eq!(app.state().document.position().index, 2);

    let deleted = app
        .handle_input(
            window,
            Input::key_down(input::Key::Backspace, input::Modifiers::default()),
        )
        .expect("backspace should be handled");

    assert!(deleted.is_handled());
    assert!(deleted.changed_state());
    assert_eq!(app.state().document.text(), "ac");
    assert_eq!(app.state().last_status, "edit");
    assert_eq!(app.revision().get(), 3);
}

#[test]
fn text_editor_vertical_key_motion_uses_layout_caret_map() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_input(window, Input::text_commit("abc\ndef"))
        .expect("text commit should be handled");

    assert_eq!(app.state().document.position().index, "abc\ndef".len());

    let moved_up = app
        .handle_input(
            window,
            Input::key_down(input::Key::ArrowUp, input::Modifiers::default()),
        )
        .expect("arrow up should be handled");

    assert!(moved_up.is_handled());
    assert!(moved_up.changed_state());
    assert_eq!(app.state().document.position().index, "abc".len());

    let moved_down = app
        .handle_input(
            window,
            Input::key_down(input::Key::ArrowDown, input::Modifiers::default()),
        )
        .expect("arrow down should be handled");

    assert!(moved_down.is_handled());
    assert!(moved_down.changed_state());
    assert_eq!(app.state().document.position().index, "abc\ndef".len());
}

#[test]
fn text_editor_key_down_extends_selection_and_delete_removes_it() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_input(window, Input::text_commit("abc"))
        .expect("text commit should be handled");
    app.handle_input(
        window,
        Input::key_down(
            input::Key::ArrowLeft,
            input::Modifiers::new(true, false, false, false),
        ),
    )
    .expect("shift-arrow should be handled");

    assert_eq!(app.state().document.selected_text().as_deref(), Some("c"));

    app.handle_input(
        window,
        Input::key_down(input::Key::Delete, input::Modifiers::default()),
    )
    .expect("delete should be handled");

    assert_eq!(app.state().document.text(), "ab");
    assert_eq!(app.state().document.selected_text(), None);
}

#[test]
fn text_editor_key_down_commits_printable_characters_to_focused_document() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    let outcome = app
        .handle_input(
            window,
            Input::key_down(input::Key::Character('x'), input::Modifiers::default()),
        )
        .expect("character key should commit text");

    assert!(outcome.is_handled());
    assert!(outcome.changed_state());
    assert_eq!(app.state().document.text(), "x");

    let spaced = app
        .handle_input(
            window,
            Input::key_down(input::Key::Space, input::Modifiers::default()),
        )
        .expect("space key should commit text");

    assert!(spaced.is_handled());
    assert!(spaced.changed_state());
    assert_eq!(app.state().document.text(), "x ");

    let controlled = app
        .handle_input(
            window,
            Input::key_down(
                input::Key::Character('y'),
                input::Modifiers::new(false, true, false, false),
            ),
        )
        .expect("control character key should not commit text");

    assert!(!controlled.is_handled());
    assert_eq!(app.state().document.text(), "x ");

    let inserted_text = app
        .handle_input(
            window,
            Input::key_down_with_text(
                input::Key::Other,
                input::Modifiers::default(),
                Some("é".to_owned()),
            ),
        )
        .expect("inserted key text should commit through key input");

    assert!(inserted_text.is_handled());
    assert!(inserted_text.changed_state());
    assert_eq!(app.state().document.text(), "x é");
}

#[test]
fn focused_text_area_renders_focus_outline_and_controls_caret_visibility() {
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text("focus"),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(480, 180);
    let focus = app
        .present(window)
        .expect("view should expose focus target")
        .text_areas()[0]
        .focus()
        .expect("text area should expose a focus target");
    let unfocused = app
        .render_scene(window, size)
        .expect("unfocused scene should render");
    let text_area = unfocused
        .layout()
        .find_role(view::node::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let text_area_rect = text_area.rect();
    let text_area_layout = text_area
        .text_area_layout()
        .expect("text area should have text layout");

    assert!(text_area_layout.layout().caret().is_none());
    assert!(!unfocused.scene().outlines().iter().any(|outline| {
        outline.rect() == text_area_rect && outline.color().channels() == (76, 132, 255, 255)
    }));

    app.handle_input(window, Input::focus(focus))
        .expect("focus should be handled");

    let focused = app
        .render_scene(window, size)
        .expect("focused scene should render");
    let focused_text_area = focused
        .layout()
        .find_role(view::node::Role::TextArea)
        .into_iter()
        .next()
        .expect("focused text area should be laid out");

    assert!(focused_text_area.is_focused());
    assert!(
        focused
            .scene()
            .outlines()
            .iter()
            .any(|outline| outline.rect() == focused_text_area.rect()
                && outline.color().channels() == (76, 132, 255, 255))
    );
}

#[test]
fn text_editor_key_down_escape_uses_cancel_flow_for_preedit() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_input(
        window,
        Input::text_preedit(text::edit::Preedit::new("x", Some((0, 1)))),
    )
    .expect("preedit input should be handled");

    let escaped = app
        .handle_input(
            window,
            Input::key_down(input::Key::Escape, input::Modifiers::default()),
        )
        .expect("escape should be handled as cancel");

    assert!(escaped.is_handled());
    assert!(!escaped.changed_state());
    assert_eq!(escaped.effect(), &response::Effect::Repaint);
    assert_eq!(app.session().focused(window), Some(focus));
    assert!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .text_input()
            .preedit()
            .is_none()
    );
}

#[test]
fn text_editor_key_down_without_focus_is_ignored() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let outcome = app
        .handle_input(
            window,
            Input::key_down(input::Key::Backspace, input::Modifiers::default()),
        )
        .expect("unfocused key edit should be ignored");

    assert!(!outcome.is_handled());
    assert_eq!(app.state().document.text(), "");
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn text_editor_key_down_dispatches_registered_command_shortcuts() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_input(window, Input::text_commit("unsaved"))
        .expect("text commit should be handled");

    let saved = app
        .handle_input(
            window,
            Input::key_down(
                input::Key::Character('s'),
                input::Modifiers::new(false, true, false, false),
            ),
        )
        .expect("ctrl+s should dispatch save command");

    assert!(saved.is_handled());
    assert!(saved.changed_state());
    assert_eq!(
        app.session().file_dialog(window),
        Some(session::FileDialog::SaveAs)
    );
    assert_eq!(app.state().last_status, "choosing save location");
}

#[test]
fn text_editor_key_down_edit_shortcuts_use_focused_responder_and_timeline() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_input(window, Input::text_commit("alpha"))
        .expect("text commit should be handled");
    app.handle_input(
        window,
        Input::key_down(
            input::Key::Character('a'),
            input::Modifiers::new(false, true, false, false),
        ),
    )
    .expect("ctrl+a should dispatch select all");

    assert_eq!(
        app.state().document.selected_text().as_deref(),
        Some("alpha")
    );

    app.handle_input(
        window,
        Input::key_down(
            input::Key::Character('x'),
            input::Modifiers::new(false, true, false, false),
        ),
    )
    .expect("ctrl+x should dispatch cut");

    assert_eq!(app.state().document.text(), "");
    assert_eq!(app.clipboard().text().as_deref(), Some("alpha"));
    assert!(app.clipboard().contains::<clipboard::Text>());
    assert_eq!(
        app.clipboard()
            .get::<clipboard::Text>()
            .expect("clipboard should contain text payload")
            .as_str(),
        "alpha"
    );

    app.handle_input(
        window,
        Input::key_down(
            input::Key::Character('z'),
            input::Modifiers::new(false, true, false, false),
        ),
    )
    .expect("ctrl+z should dispatch undo");

    assert_eq!(app.state().document.text(), "alpha");
    assert_eq!(app.state().last_status, "select all");
}

#[test]
fn text_editor_key_down_alt_f4_dispatches_framework_close_window() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    assert!(app.session().contains(window));

    let closed = app
        .handle_input(
            window,
            Input::key_down(
                input::Key::F4,
                input::Modifiers::new(false, false, true, false),
            ),
        )
        .expect("alt+f4 should dispatch close window");

    assert!(closed.is_handled());
    assert!(!closed.changed_state());
    assert_eq!(closed.effect(), &response::Effect::Repaint);
    assert!(!app.session().contains(window));
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn text_editor_save_shortcut_dispatches_by_registered_command_type() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_input(
        window,
        Input::text_edit(text::edit::Edit::insert("unsaved")),
    )
    .expect("text edit input should be handled");

    let outcome = app
        .handle_input(window, Input::shortcut("Ctrl+S"))
        .expect("save shortcut should resolve");

    assert!(outcome.is_handled());
    assert!(outcome.changed_state());
    assert_eq!(outcome.effect(), &response::Effect::SaveFileDialog);
    assert_eq!(app.state().last_status, "choosing save location");
    assert_eq!(
        app.session().file_dialog(window),
        Some(session::FileDialog::SaveAs)
    );
    let requests = app.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0].kind(),
        session::RequestKind::FileDialog(session::FileDialog::SaveAs)
    );
    assert_eq!(app.revision().get(), 2);
}

#[test]
fn text_editor_disabled_shortcut_claims_and_returns_disabled_error() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let error = app
        .handle_input(window, Input::shortcut("Ctrl+S"))
        .expect_err("disabled save shortcut should fail");

    assert!(matches!(
        error,
        Error::Disabled {
            command: "document.save_file"
        }
    ));
    assert_eq!(app.session().file_dialog(window), None);
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn text_editor_edit_and_timeline_shortcuts_use_focus_and_history() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_input(window, Input::text_edit(text::edit::Edit::insert("alpha")))
        .expect("text edit input should be handled");

    app.handle_input(window, Input::shortcut("Ctrl+A"))
        .expect("select-all shortcut should resolve");
    app.handle_input(window, Input::shortcut("Ctrl+X"))
        .expect("cut shortcut should resolve");

    assert_eq!(app.state().document.text(), "");
    assert_eq!(app.clipboard().text().as_deref(), Some("alpha"));
    assert_eq!(app.state().last_status, "cut");
    assert_eq!(app.revision().get(), 3);

    app.handle_input(window, Input::shortcut("Ctrl+Z"))
        .expect("undo shortcut should resolve");

    assert_eq!(app.state().document.text(), "alpha");
    assert_eq!(app.state().last_status, "select all");
    assert_eq!(app.revision().get(), 4);

    app.handle_input(window, Input::shortcut("Ctrl+Shift+Z"))
        .expect("redo shortcut should resolve");

    assert_eq!(app.state().document.text(), "");
    assert_eq!(app.state().last_status, "cut");
    assert_eq!(app.revision().get(), 5);

    app.handle_input(window, Input::shortcut("Ctrl+V"))
        .expect("paste shortcut should resolve");

    assert_eq!(app.state().document.text(), "alpha");
    assert_eq!(app.state().last_status, "paste");
    assert_eq!(app.revision().get(), 6);
}

#[test]
fn text_editor_text_drop_input_groups_drop_and_source_cleanup() {
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text("abcd"),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");

    let outcome = app
        .handle_input(
            window,
            Input::text_drop_with_source_cleanup(
                text::edit::Edit::insert_at(4, "bc"),
                text::edit::Edit::replace_range(1..3, ""),
            ),
        )
        .expect("drop input should be handled");

    assert!(outcome.is_handled());
    assert!(outcome.changed_state());
    assert_eq!(app.state().document.text(), "adbc");
    assert!(app.state().document.is_dirty());
    assert_eq!(app.state().document.edit_count(), 2);
    assert_eq!(app.revision().get(), 1);
    assert_eq!(
        app.store().changes()[0].reason(),
        &state::Reason::Event("text_drop")
    );

    assert!(app.undo());
    assert_eq!(app.state().document.text(), "abcd");
    assert_eq!(app.revision().get(), 2);
}

#[test]
fn text_editor_input_edits_can_be_undone_and_redone() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_input(window, Input::text_edit(text::edit::Edit::insert("alpha")))
        .expect("text edit input should be handled");

    assert_eq!(app.state().document.text(), "alpha");
    assert!(app.timeline().can_undo());

    assert!(app.undo());

    assert_eq!(app.state().document.text(), "");
    assert!(!app.state().document.is_dirty());
    assert!(app.timeline().can_redo());

    assert!(app.redo());

    assert_eq!(app.state().document.text(), "alpha");
    assert!(app.state().document.is_dirty());
    assert_eq!(app.revision().get(), 3);
}

#[test]
fn text_editor_typing_commits_coalesce_into_one_timeline_entry() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_input(window, Input::text_commit("a"))
        .expect("first text commit should be handled");
    app.handle_input(window, Input::text_commit("b"))
        .expect("second text commit should be handled");
    app.handle_input(window, Input::text_commit("c"))
        .expect("third text commit should be handled");

    assert_eq!(app.state().document.text(), "abc");
    assert_eq!(app.timeline().undo_depth(), 1);
    assert_eq!(app.revision().get(), 3);

    assert!(app.undo(), "coalesced typing should undo as one entry");

    assert_eq!(app.state().document.text(), "");
    assert_eq!(app.timeline().undo_depth(), 0);
    assert_eq!(app.timeline().redo_depth(), 1);

    assert!(app.redo(), "coalesced typing should redo as one entry");

    assert_eq!(app.state().document.text(), "abc");
    assert_eq!(app.timeline().undo_depth(), 1);
}

#[test]
fn text_editor_punctuation_commit_breaks_typing_coalescing() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_input(window, Input::text_commit("a"))
        .expect("letter commit should be handled");
    app.handle_input(window, Input::text_commit("!"))
        .expect("punctuation commit should be handled");

    assert_eq!(app.state().document.text(), "a!");
    assert_eq!(app.timeline().undo_depth(), 2);

    assert!(app.undo(), "punctuation should undo independently");

    assert_eq!(app.state().document.text(), "a");
    assert_eq!(app.timeline().undo_depth(), 1);
    assert_eq!(app.timeline().redo_depth(), 1);
}

#[test]
fn text_editor_edit_menu_undo_redo_commands_use_runtime_timeline() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");

    assert!(projected.labels().contains(&"Edit"));
    assert!(
        !projected
            .binding::<timeline::Undo>()
            .expect("undo command should be in the view")
            .is_enabled()
    );

    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");
    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_input(window, Input::text_edit(text::edit::Edit::insert("alpha")))
        .expect("text edit input should be handled");

    let projected = app
        .present(window)
        .expect("window should still have a view");
    let undo = projected
        .binding::<timeline::Undo>()
        .expect("undo command should be in the view");

    assert!(undo.is_enabled());
    assert_eq!(undo.state().label.as_deref(), Some("Undo"));
    assert_eq!(
        undo.state().shortcut.map(|shortcut| shortcut.as_str()),
        Some("Ctrl+Z")
    );

    let effect = app
        .activate_in(window, undo)
        .expect("undo command should activate");

    assert_eq!(effect, response::Effect::Repaint);
    assert_eq!(app.state().document.text(), "");
    assert_eq!(app.revision().get(), 2);

    let projected = app
        .present(window)
        .expect("window should still have a view");
    let redo = projected
        .binding::<timeline::Redo>()
        .expect("redo command should be in the view");

    assert!(redo.is_enabled());

    app.activate_in(window, redo)
        .expect("redo command should activate");

    assert_eq!(app.state().document.text(), "alpha");
    assert_eq!(app.revision().get(), 3);
}

#[test]
fn text_editor_edit_menu_clipboard_commands_use_focused_document() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let _: &Clipboard = app.clipboard();
    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_input(window, Input::text_edit(text::edit::Edit::insert("alpha")))
        .expect("text edit input should be handled");

    let projected = app
        .present(window)
        .expect("window should still have a view");

    assert!(
        projected
            .binding::<document::SelectAll>()
            .expect("select-all command should be in the view")
            .is_enabled()
    );
    assert!(
        !projected
            .binding::<document::Copy>()
            .expect("copy command should be in the view")
            .is_enabled()
    );
    assert!(
        !projected
            .binding::<document::Paste>()
            .expect("paste command should be in the view")
            .is_enabled()
    );

    app.activate_in(
        window,
        projected
            .binding::<document::SelectAll>()
            .expect("select-all command should be in the view"),
    )
    .expect("select-all should activate");

    assert_eq!(app.revision().get(), 2);

    let projected = app
        .present(window)
        .expect("window should still have a view");
    let copy = projected
        .binding::<document::Copy>()
        .expect("copy command should be in the view");
    let cut = projected
        .binding::<document::Cut>()
        .expect("cut command should be in the view");

    assert!(copy.is_enabled());
    assert!(cut.is_enabled());
    assert_eq!(copy.state().label.as_deref(), Some("Copy"));
    assert_eq!(
        copy.state().shortcut.map(|shortcut| shortcut.as_str()),
        Some("Ctrl+C")
    );

    app.activate_in(window, copy).expect("copy should activate");

    assert_eq!(app.clipboard().text().as_deref(), Some("alpha"));
    assert_eq!(app.state().document.text(), "alpha");
    assert_eq!(app.state().last_status, "copy");
    assert_eq!(app.revision().get(), 3);

    app.activate_in(window, cut).expect("cut should activate");

    assert_eq!(app.clipboard().text().as_deref(), Some("alpha"));
    assert_eq!(app.state().document.text(), "");
    assert_eq!(app.state().last_status, "cut");
    assert_eq!(app.revision().get(), 4);

    let projected = app
        .present(window)
        .expect("window should still have a view");
    let paste = projected
        .binding::<document::Paste>()
        .expect("paste command should be in the view");

    assert!(paste.is_enabled());

    app.activate_in(window, paste)
        .expect("paste should activate");

    assert_eq!(app.state().document.text(), "alpha");
    assert_eq!(app.state().last_status, "paste");
    assert_eq!(app.revision().get(), 5);

    let projected = app
        .present(window)
        .expect("window should still have a view");
    app.activate_in(
        window,
        projected
            .binding::<document::SelectAll>()
            .expect("select-all command should be in the view"),
    )
    .expect("select-all should activate");

    let projected = app
        .present(window)
        .expect("window should still have a view");
    let delete = projected
        .binding::<document::Delete>()
        .expect("delete command should be in the view");

    assert!(delete.is_enabled());

    app.activate_in(window, delete)
        .expect("delete should activate");

    assert_eq!(app.state().document.text(), "");
    assert_eq!(app.state().last_status, "delete");
    assert_eq!(app.revision().get(), 7);
}

#[test]
fn text_editor_file_menu_exit_closes_framework_window() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let exit = projected
        .binding::<session::CloseWindow>()
        .expect("exit command should be in the view");

    assert!(exit.is_enabled());
    assert_eq!(exit.state().label.as_deref(), Some("Exit"));
    assert_eq!(
        exit.state().shortcut.map(|shortcut| shortcut.as_str()),
        Some("Alt+F4")
    );

    let effect = app
        .activate_in(window, exit)
        .expect("exit command should activate");

    assert_eq!(effect, response::Effect::Repaint);
    assert!(app.session().windows().is_empty());
    assert_eq!(app.revision(), state::Revision::initial());
    assert!(app.present(window).is_none());
}

#[test]
fn text_editor_file_menu_load_stress_text_updates_document_and_status() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    app.invoke(app.trigger::<text_editor::ToggleDebugPanel>(()))
        .output
        .expect("debug toggle should resolve");

    let projected = app.present(window).expect("window should have a view");
    let load = projected
        .binding::<text_editor::LoadStressText>()
        .expect("load stress command should be in the view");

    assert!(load.is_enabled());
    assert_eq!(load.state().label.as_deref(), Some("Load Stress Text"));

    let effect = app
        .activate_in(window, load)
        .expect("load stress command should activate");

    assert_eq!(effect, response::Effect::None);
    assert_eq!(app.state().document.text(), text_editor::STRESS_TEXT);
    assert!(app.state().document.is_dirty());
    assert_eq!(app.state().document.path(), None);
    assert_eq!(app.state().document.edit_count(), 0);
    assert_eq!(
        app.state().last_status,
        format!(
            "loaded Unicode stress fixture ({} lines)",
            text_editor::STRESS_TEXT.lines().count()
        )
    );
    assert_eq!(app.revision().get(), 2);

    let projected = app
        .present(window)
        .expect("window should still have a view");

    assert!(
        projected
            .labels()
            .iter()
            .any(|label| label.contains("File: Untitled (modified) | Wrap: on"))
    );
    assert!(
        projected
            .labels()
            .iter()
            .any(|label| label.contains("Status: loaded Unicode stress fixture"))
    );
}
