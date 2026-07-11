use super::*;

#[derive(Clone, Default)]
struct TypingDocuments {
    first: TextDocument,
    second: TextDocument,
}

impl State for TypingDocuments {}

#[test]
fn closing_window_removes_framework_owned_composition() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    app.present(window).expect("window should have a view");
    assert!(app.composition(window).is_some());

    let close = app
        .composition(window)
        .expect("composition should be retained")
        .view()
        .binding::<session::CloseWindow>()
        .expect("close command should be in the retained view")
        .action();

    app.handle_view(window, close)
        .expect("close action should be handled");

    assert!(!app.session().contains(window));
    assert!(app.composition(window).is_none());
}

#[test]
fn focused_object_responders_do_not_participate_without_focus() {
    let mut app = Runtime::new(MultiDocumentState {
        first: SaveDocument {
            dirty: true,
            save_count: 0,
        },
        second: SaveDocument {
            dirty: true,
            save_count: 0,
        },
    })
    .commands(|commands| {
        commands.register::<Save>(command::Spec::new("Save").shortcut("Ctrl+S"));
    })
    .responders(|responders| {
        responders
            .object("first", |state: &mut MultiDocumentState| &mut state.first)
            .target::<Save>();
        responders
            .object("second", |state: &mut MultiDocumentState| &mut state.second)
            .target::<Save>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Editor"));
    });

    app.start();

    let window = app.session().windows()[0].id();
    let save = app.trigger::<Save>(());

    assert!(!app.state_for(&save).is_enabled());
    assert!(!app.state_for_focused(window, &save).is_enabled());

    let error = app
        .invoke_focused(window, save)
        .output
        .expect_err("unfocused document command should not resolve");

    assert!(matches!(
        error,
        Error::MissingTarget {
            command: "app.save"
        }
    ));
    assert!(app.state().first.dirty);
    assert_eq!(app.state().first.save_count, 0);
    assert!(app.state().second.dirty);
    assert_eq!(app.state().second.save_count, 0);
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn focused_object_responder_matches_only_the_focused_target_name() {
    let mut app = Runtime::new(MultiDocumentState {
        first: SaveDocument {
            dirty: true,
            save_count: 0,
        },
        second: SaveDocument {
            dirty: true,
            save_count: 0,
        },
    })
    .commands(|commands| {
        commands.register::<Save>(command::Spec::new("Save").shortcut("Ctrl+S"));
    })
    .responders(|responders| {
        responders
            .object("first", |state: &mut MultiDocumentState| &mut state.first)
            .target::<Save>();
        responders
            .object("second", |state: &mut MultiDocumentState| &mut state.second)
            .target::<Save>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Editor"));
    });

    app.start();

    let window = app.session().windows()[0].id();
    assert!(app.focus(window, session::Focus::text("second")));

    let save = app.trigger::<Save>(());
    assert!(app.state_for_focused(window, &save).is_enabled());

    let response = app.invoke_focused(window, save);

    assert!(response.output.is_ok());
    assert!(app.state().first.dirty);
    assert_eq!(app.state().first.save_count, 0);
    assert!(!app.state().second.dirty);
    assert_eq!(app.state().second.save_count, 1);
    assert_eq!(app.revision().get(), 1);
}

#[test]
fn document_wraps_production_text_buffer_and_tracks_document_dirty() {
    let mut document = TextDocument::new_multiline();
    let saved_revision = document.saved_buffer_revision();

    assert!(document.is_multiline());
    assert_eq!(document.text(), "");
    assert!(!document.is_dirty());

    let outcome = document.apply_edit(text::edit::Edit::insert("alpha"));

    assert!(outcome.text_changed());
    assert!(outcome.selection_changed());
    assert_eq!(document.text(), "alpha");
    assert_ne!(document.buffer_revision(), saved_revision);
    assert!(document.is_dirty());
    assert_eq!(document.edit_count(), 1);

    document.mark_saved();

    assert_eq!(document.saved_buffer_revision(), document.buffer_revision());
    assert!(!document.is_dirty());
}

#[test]
fn document_edit_command_targets_text_document_and_bumps_app_revision() {
    let mut app = Runtime::new(text_editor::State::default())
        .commands(|commands| {
            commands.register::<document::ApplyEdit>(command::Spec::new("Edit"));
        })
        .responders(|responders| {
            responders
                .object("document", |state: &mut text_editor::State| {
                    &mut state.document
                })
                .target::<document::ApplyEdit>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Editor"));
        });
    let trigger = app.trigger::<document::ApplyEdit>(text::edit::Edit::insert("alpha"));
    app.start();
    let window = app.session().windows()[0].id();
    assert!(app.focus(window, session::Focus::text("document")));

    let response = app.invoke_focused(window, trigger);

    let outcome = response.output.expect("edit command should succeed");
    assert!(outcome.text_changed());
    assert_eq!(app.state().document.text(), "alpha");
    assert!(app.state().document.is_dirty());
    assert_eq!(app.state().document.edit_count(), 1);
    assert_eq!(app.revision().get(), 1);
    assert_eq!(
        app.store().changes()[0].reason(),
        &state::Reason::Command("document.apply_edit")
    );
}

#[test]
fn typing_history_does_not_coalesce_across_focused_documents() {
    let mut app = Runtime::new(TypingDocuments::default())
        .commands(|commands| {
            commands.register::<document::ApplyEdit>(command::Spec::new("Edit"));
        })
        .responders(|responders| {
            responders
                .object("first", |state: &mut TypingDocuments| &mut state.first)
                .target::<document::ApplyEdit>();
            responders
                .object("second", |state: &mut TypingDocuments| &mut state.second)
                .target::<document::ApplyEdit>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Editor"));
        });

    app.start();
    let window = app.session().windows()[0].id();

    assert!(app.focus(window, session::Focus::text("first")));
    app.invoke_focused(
        window,
        app.trigger::<document::ApplyEdit>(text::edit::Edit::insert("a")),
    )
    .output
    .expect("first document edit should succeed");

    assert!(app.focus(window, session::Focus::text("second")));
    app.invoke_focused(
        window,
        app.trigger::<document::ApplyEdit>(text::edit::Edit::insert("b")),
    )
    .output
    .expect("second document edit should succeed");

    assert_eq!(app.state().first.text(), "a");
    assert_eq!(app.state().second.text(), "b");
    assert_eq!(app.timeline().undo_depth(), 2);

    assert!(app.undo(), "second document edit should undo independently");
    assert_eq!(app.state().first.text(), "a");
    assert_eq!(app.state().second.text(), "");
}

#[test]
fn selection_only_document_edit_bumps_app_revision_without_document_dirty() {
    let mut app = Runtime::new(text_editor::State {
        document: TextDocument::from_text("alpha"),
        ..text_editor::State::default()
    })
    .commands(|commands| {
        commands.register::<document::ApplyEdit>(command::Spec::new("Edit"));
    })
    .responders(|responders| {
        responders
            .object("document", |state: &mut text_editor::State| {
                &mut state.document
            })
            .target::<document::ApplyEdit>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Editor"));
    });
    let trigger = app.trigger::<document::ApplyEdit>(text::edit::Edit::SelectAll);
    app.start();
    let window = app.session().windows()[0].id();
    assert!(app.focus(window, session::Focus::text("document")));

    let response = app.invoke_focused(window, trigger);

    let outcome = response.output.expect("select all should succeed");
    assert!(!outcome.text_changed());
    assert!(outcome.selection_changed());
    assert_eq!(app.state().document.text(), "alpha");
    assert!(!app.state().document.is_dirty());
    assert_eq!(app.state().document.edit_count(), 0);
    assert_eq!(app.revision().get(), 1);
}

#[test]
fn document_text_command_outcome_is_framework_owned() {
    let mut document = TextDocument::from_text("alpha");
    let mut clipboard = Clipboard::default();

    let selected = document.apply_edit(text::edit::Edit::SelectAll);
    assert!(!selected.text_changed());
    assert!(selected.selection_changed());

    let copied = document.apply_action(text::edit::Action::Copy, &mut clipboard);

    assert!(!copied.text_changed());
    assert!(!copied.selection_changed());
    assert!(copied.clipboard_changed());
    assert!(!copied.unavailable());
    assert_eq!(
        clipboard
            .text()
            .expect("clipboard read should succeed")
            .as_deref(),
        Some("alpha")
    );
    assert_eq!(document.text(), "alpha");
    assert_eq!(document.edit_count(), 0);
}

#[test]
fn clipboard_failures_preserve_cut_text_and_differ_from_empty_paste() {
    let mut document = TextDocument::from_text("alpha");
    document.apply_edit(text::edit::Edit::SelectAll);
    let mut unavailable = Clipboard::unavailable_system();

    let cut = document.apply_action(text::edit::Action::Cut, &mut unavailable);

    assert_eq!(document.text(), "alpha");
    assert!(!cut.text_changed());
    assert!(!cut.clipboard_changed());
    assert!(cut.unavailable());
    assert_eq!(
        unavailable.put(&clipboard::Text::new("replacement")),
        Err(clipboard::Error::Unavailable)
    );
    assert_eq!(unavailable.text(), Err(clipboard::Error::Unavailable));

    let failed_paste = document.apply_action(text::edit::Action::Paste, &mut unavailable);
    assert_eq!(document.text(), "alpha");
    assert!(failed_paste.unavailable());

    let mut empty = Clipboard::default();
    let empty_paste = document.apply_action(text::edit::Action::Paste, &mut empty);
    assert_eq!(document.text(), "alpha");
    assert!(!empty_paste.unavailable());
    assert!(!empty_paste.buffer_changed());
}

#[test]
fn paste_command_owns_optimistic_clipboard_availability() {
    let without_clipboard = Context::default();
    assert!(!document::Paste::availability(&without_clipboard).is_enabled());

    let mut empty = Clipboard::default();
    let empty_context = Context::with_clipboard(&mut empty);
    assert!(!document::Paste::availability(&empty_context).is_enabled());

    empty
        .put(&clipboard::Text::new("available"))
        .expect("in-memory clipboard write should succeed");
    assert!(document::Paste::availability(&empty_context).is_enabled());

    let mut unavailable = Clipboard::unavailable_system();
    let unavailable_context = Context::with_clipboard(&mut unavailable);
    assert!(document::Paste::availability(&unavailable_context).is_enabled());
}

#[test]
fn document_owns_text_state_separately_from_text_storage() {
    let mut document = TextDocument::from_multiline_text("alpha beta");
    let initial = document.text_state();

    let moved = document.apply_edit(text::edit::Edit::set_position(text::buffer::Position::new(
        0,
    )));

    assert!(moved.selection_changed());
    assert_ne!(document.text_state(), initial);
    assert_eq!(document.buffer().initial_state(), initial);

    let selected = document.apply_edit(text::edit::Edit::pointer(
        text::edit::PointerEditKind::Drag,
        text::buffer::Position::new("alpha".len()),
    ));

    assert!(selected.selection_changed());
    assert_eq!(document.selected_text().as_deref(), Some("alpha"));

    let snapshot = document.clone();
    document.apply_edit(text::edit::Edit::set_position(text::buffer::Position::new(
        0,
    )));

    assert_eq!(snapshot.text(), document.text());
    assert_ne!(snapshot.text_state(), document.text_state());
    assert_eq!(snapshot.selected_text().as_deref(), Some("alpha"));
    assert_eq!(document.selected_text(), None);
}

#[test]
fn document_new_file_resets_text_path_and_dirty_state() {
    let path = temp_text_path("document_reset.txt");
    let mut document = TextDocument::from_multiline_text("alpha\nbeta");
    document.save_to(path.clone()).expect("save should succeed");
    document.apply_edit(text::edit::Edit::insert("gamma"));

    assert!(document.path().is_some());
    assert!(document.is_dirty());

    document.new_file();

    assert_eq!(document.text(), "");
    assert!(document.is_multiline());
    assert_eq!(document.path(), None);
    assert!(!document.is_dirty());
    assert_eq!(document.edit_count(), 0);

    let _ = std::fs::remove_file(path);
}

#[test]
fn document_save_to_and_open_path_keep_document_clean() {
    let path = temp_text_path("document_roundtrip.txt");
    let mut document = TextDocument::new_multiline();
    document.apply_edit(text::edit::Edit::insert("alpha\nbeta"));

    assert!(document.is_dirty());

    document.save_to(path.clone()).expect("save should succeed");

    assert_eq!(document.path(), Some(path.as_path()));
    assert!(!document.is_dirty());

    let mut opened = TextDocument::default();
    opened.open_path(path.clone()).expect("open should succeed");

    assert_eq!(opened.text(), "alpha\nbeta");
    assert_eq!(opened.path(), Some(path.as_path()));
    assert!(!opened.is_dirty());
    assert_eq!(opened.edit_count(), 0);

    let _ = std::fs::remove_file(path);
}

#[test]
fn open_crlf_document_edits_and_atomically_saves_while_open() {
    let path = temp_text_path("document_crlf_roundtrip.txt");
    std::fs::write(&path, "one\r\ntwo\r\nthree\n").expect("CRLF fixture should be writable");
    let mut document = TextDocument::default();
    document
        .open_path(path.clone())
        .expect("CRLF file should open");

    assert_eq!(document.text(), "one\r\ntwo\r\nthree\n");
    document.apply_edit(text::edit::Edit::insert_line_break());
    document
        .save_to(path.clone())
        .expect("an owned open document should atomically replace its source file");

    assert_eq!(
        std::fs::read(&path).expect("saved CRLF file should read"),
        b"one\r\ntwo\r\nthree\n\r\n"
    );
    assert!(!document.is_dirty());

    let _ = std::fs::remove_file(path);
}

#[test]
fn document_save_snapshot_keeps_identity_and_captured_revision_together() {
    let path = temp_text_path("document_versioned_save.txt");
    let mut document = TextDocument::from_text("alpha");
    let identity = document.identity();
    let snapshot = document.save_snapshot();
    let version = snapshot.version();

    assert_eq!(version.identity(), identity);
    assert_eq!(version.revision(), document.buffer_revision());
    assert_eq!(document.clone().identity(), identity);

    document.apply_edit(text::edit::Edit::insert("!"));
    snapshot
        .write_to(&path)
        .expect("snapshot save should succeed");
    assert!(document.record_saved_version_at(version, path.clone()));

    assert_eq!(
        std::fs::read_to_string(&path).expect("saved snapshot should be readable"),
        "alpha"
    );
    assert_eq!(document.path(), Some(path.as_path()));
    assert_eq!(document.saved_buffer_revision(), version.revision());
    assert!(
        document.is_dirty(),
        "the post-snapshot edit must remain dirty"
    );

    let other = TextDocument::from_text("other");
    assert_ne!(other.identity(), identity);
    assert!(!document.record_saved_version_at(other.version(), path.clone()));

    let before_new = document.identity();
    document.new_file();
    assert_ne!(document.identity(), before_new);

    let _ = std::fs::remove_file(path);
}

#[test]
fn document_save_replaces_existing_file_without_leaving_temporary_sibling() {
    let path = temp_text_path("document_atomic_replace.txt");
    std::fs::write(&path, "old contents").expect("old fixture should be writable");
    let mut document = TextDocument::from_text("new contents");

    document
        .save_to(path.clone())
        .expect("atomic replacement should succeed");

    assert_eq!(
        std::fs::read_to_string(&path).expect("replacement should be readable"),
        "new contents"
    );
    let temporary_prefix = format!(
        ".{}.wgpu_l3-save-",
        path.file_name()
            .expect("test path should name a file")
            .to_string_lossy()
    );
    let leftovers = std::fs::read_dir(
        path.parent()
            .expect("temporary test path should have a parent"),
    )
    .expect("temporary directory should be readable")
    .filter_map(Result::ok)
    .filter(|entry| {
        entry
            .file_name()
            .to_string_lossy()
            .starts_with(&temporary_prefix)
    })
    .collect::<Vec<_>>();
    assert!(
        leftovers.is_empty(),
        "temporary save siblings must be cleaned"
    );
    assert!(!document.is_dirty());

    let _ = std::fs::remove_file(path);
}

#[test]
fn text_editor_file_commands_flow_through_runtime_responders() {
    let path = temp_text_path("runtime_file_commands.txt");
    std::fs::write(&path, "opened").expect("fixture file should be writable");

    let mut app = text_editor::app(text_editor::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    assert!(app.focus(window, session::Focus::text("document")));

    let save_without_path = app.trigger::<document::SaveFile>(());

    assert!(!app.state_for(&save_without_path).is_enabled());

    let open = app.trigger::<document::OpenPath>(path.clone());
    let open_response = app.invoke(open);

    assert_eq!(open_response.output.expect("open should resolve"), Ok(()));
    assert_eq!(app.state().document.text(), "opened");
    assert_eq!(app.state().document.path(), Some(path.as_path()));
    assert!(!app.state().document.is_dirty());
    assert_eq!(
        app.state().last_status,
        format!("opened {}", text_editor::display_path(&path))
    );
    assert_eq!(app.revision().get(), 1);

    let edit = app.trigger::<document::ApplyEdit>(text::edit::Edit::insert("!"));
    let edit_response = app.invoke_focused(window, edit);

    assert!(
        edit_response
            .output
            .expect("edit should resolve")
            .text_changed()
    );
    assert_eq!(app.state().document.text(), "opened!");
    assert!(app.state().document.is_dirty());
    assert_eq!(app.revision().get(), 2);

    let save = app.trigger::<document::SaveFile>(());

    assert!(app.state_for(&save).is_enabled());

    let save_response = app.invoke(save);

    assert_eq!(save_response.output.expect("save should resolve"), Ok(()));
    assert_eq!(app.pending_tasks(), 1);
    assert_eq!(
        app.state().last_status,
        format!("saving {}", text_editor::display_path(&path))
    );
    assert_eq!(
        std::fs::read_to_string(&path).expect("file should not be rewritten before task runs"),
        "opened"
    );
    assert!(app.state().document.is_dirty());
    assert_eq!(app.revision().get(), 3);

    let save_task = app.run_next_task().expect("save task should run");
    assert_eq!(app.pending_tasks(), 0);
    assert_eq!(save_task.status(), task::Status::Completed);
    assert!(save_task.changed_state());
    assert_eq!(
        std::fs::read_to_string(&path).expect("saved file should be readable"),
        "opened!"
    );
    assert!(!app.state().document.is_dirty());
    assert_eq!(
        app.state().last_status,
        format!("saved {}", text_editor::display_path(&path))
    );
    assert_eq!(app.revision().get(), 4);

    let new_file = app.trigger::<document::NewFile>(());
    let new_response = app.invoke(new_file);

    new_response.output.expect("new file should resolve");
    assert_eq!(app.state().document.text(), "");
    assert_eq!(app.state().document.path(), None);
    assert!(!app.state().document.is_dirty());
    assert_eq!(app.state().last_status, "new file");
    assert_eq!(app.revision().get(), 5);

    let _ = std::fs::remove_file(path);
}

#[test]
fn text_editor_save_completion_keeps_newer_edits_dirty() {
    let path = temp_text_path("save_completion_revision.txt");
    let mut app = text_editor::app(text_editor::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    assert!(app.focus(window, session::Focus::text("document")));
    app.invoke_focused(
        window,
        app.trigger::<document::ApplyEdit>(text::edit::Edit::insert("alpha")),
    )
    .output
    .expect("initial edit should succeed");
    let saved_version = app.state().document.version();

    app.invoke(app.trigger::<document::SaveToPath>(path.clone()))
        .output
        .expect("save command should resolve")
        .expect("save should schedule");
    app.invoke_focused(
        window,
        app.trigger::<document::ApplyEdit>(text::edit::Edit::insert("!")),
    )
    .output
    .expect("newer edit should succeed");

    let completion = app.run_next_task().expect("save task should complete");

    assert!(completion.changed_state());
    assert_eq!(
        std::fs::read_to_string(&path).expect("saved snapshot should be readable"),
        "alpha"
    );
    assert_eq!(app.state().document.text(), "alpha!");
    assert_eq!(app.state().document.path(), Some(path.as_path()));
    assert_eq!(
        app.state().document.saved_buffer_revision(),
        saved_version.revision()
    );
    assert!(app.state().document.is_dirty());
    assert_eq!(
        app.state().last_status,
        format!(
            "saved {}; newer edits remain unsaved",
            text_editor::display_path(&path)
        )
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn text_editor_save_completion_cannot_mark_a_replacement_document_saved() {
    let path = temp_text_path("save_completion_document_identity.txt");
    let mut app = text_editor::app(text_editor::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    assert!(app.focus(window, session::Focus::text("document")));
    app.invoke_focused(
        window,
        app.trigger::<document::ApplyEdit>(text::edit::Edit::insert("old document")),
    )
    .output
    .expect("edit should succeed");
    let saving_identity = app.state().document.identity();
    app.invoke(app.trigger::<document::SaveToPath>(path.clone()))
        .output
        .expect("save command should resolve")
        .expect("save should schedule");

    app.invoke(app.trigger::<document::NewFile>(()))
        .output
        .expect("new file should succeed");
    assert_ne!(app.state().document.identity(), saving_identity);
    let revision_before_completion = app.revision();

    let completion = app.run_next_task().expect("old save task should complete");

    assert!(!completion.changed_state());
    assert_eq!(app.revision(), revision_before_completion);
    assert_eq!(app.state().document.text(), "");
    assert_eq!(app.state().document.path(), None);
    assert!(!app.state().document.is_dirty());
    assert_eq!(app.state().last_status, "new file");
    assert_eq!(
        std::fs::read_to_string(&path).expect("old snapshot should still reach disk"),
        "old document"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn text_editor_latest_save_generation_owns_completion() {
    let first_path = temp_text_path("save_generation_first.txt");
    let second_path = temp_text_path("save_generation_second.txt");
    let mut app = text_editor::app(text_editor::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    assert!(app.focus(window, session::Focus::text("document")));
    app.invoke_focused(
        window,
        app.trigger::<document::ApplyEdit>(text::edit::Edit::insert("first")),
    )
    .output
    .expect("first edit should succeed");
    app.invoke(app.trigger::<document::SaveToPath>(first_path.clone()))
        .output
        .expect("first save command should resolve")
        .expect("first save should schedule");
    let first_generation = app.state().save_generation;

    app.invoke_focused(
        window,
        app.trigger::<document::ApplyEdit>(text::edit::Edit::insert(" second")),
    )
    .output
    .expect("second edit should succeed");
    app.invoke(app.trigger::<document::SaveToPath>(second_path.clone()))
        .output
        .expect("second save command should resolve")
        .expect("second save should schedule");
    assert!(app.state().save_generation > first_generation);
    let waiting_status = format!("saving {}", text_editor::display_path(&second_path));

    let stale = app
        .run_next_task()
        .expect("first save task should complete");

    assert!(!stale.changed_state());
    assert_eq!(app.state().document.path(), None);
    assert!(app.state().document.is_dirty());
    assert_eq!(app.state().last_status, waiting_status);
    assert_eq!(
        std::fs::read_to_string(&first_path).expect("first save should reach disk"),
        "first"
    );

    let current = app
        .run_next_task()
        .expect("second save task should complete");

    assert!(current.changed_state());
    assert_eq!(app.state().document.path(), Some(second_path.as_path()));
    assert!(!app.state().document.is_dirty());
    assert_eq!(
        std::fs::read_to_string(&second_path).expect("second save should reach disk"),
        "first second"
    );
    assert_eq!(
        app.state().last_status,
        format!("saved {}", text_editor::display_path(&second_path))
    );

    let _ = std::fs::remove_file(first_path);
    let _ = std::fs::remove_file(second_path);
}

#[test]
fn text_editor_restore_clears_pending_save_task_and_transient_dialog() {
    let path = temp_text_path("restore_clears_pending_save.txt");
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    assert!(app.composition(window).is_some());
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");
    let text_area = text_area_node(projected.root()).expect("text area should be in the view");
    let scroll_target = text_area
        .pointer_target()
        .expect("text area should have a scroll target");

    app.handle_input(window, Input::focus(focus))
        .expect("focus input should be handled");
    app.handle_input(
        window,
        Input::text_edit(text::edit::Edit::insert("restored")),
    )
    .expect("text edit input should be handled");

    let snapshot = app.snapshot();

    app.handle_view(
        window,
        text_area
            .scroll_action(interaction::ScrollDelta::vertical(96))
            .expect("text area should expose scroll"),
    )
    .expect("scroll should be handled");
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("window should have interaction state")
            .scroll()
            .offset(&scroll_target),
        interaction::ScrollOffset::new(0, 96)
    );

    let file = projected
        .menus()
        .into_iter()
        .find(|menu| menu.label_text() == Some("File"))
        .expect("file menu should be in the view");
    app.handle_view(
        window,
        file.pointer_down_action()
            .expect("file menu should expose pointer down"),
    )
    .expect("pointer down should be handled");
    assert!(
        app.session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().pressed())
            .is_some()
    );

    app.handle_input(window, Input::shortcut("Ctrl+S"))
        .expect("save shortcut should request save dialog");
    assert_eq!(
        app.session().file_dialog(window),
        Some(session::FileDialog::SaveAs)
    );

    app.handle_input(window, Input::file_path_selected(Some(path.clone())))
        .expect("save path should schedule save");
    assert_eq!(app.pending_tasks(), 1);
    assert!(app.state().last_status.starts_with("saving "));
    app.diagnostics_mut(window)
        .expect("window should have diagnostics")
        .frame
        .full_redraws = 42;

    let change = app.restore(snapshot);

    assert_eq!(change.reason(), &state::Reason::Restore);
    assert_eq!(app.state().document.text(), "restored");
    assert_eq!(app.state().last_status, "edit");
    assert_eq!(app.session().file_dialog(window), None);
    assert!(app.composition(window).is_none());
    let interaction = app
        .session()
        .interaction(window)
        .expect("restored window should have interaction state");
    assert_eq!(interaction.pointer().hovered(), None);
    assert_eq!(interaction.pointer().pressed(), None);
    assert_eq!(interaction.pointer().capture(), None);
    assert_eq!(
        interaction.scroll().offset(&scroll_target),
        interaction::ScrollOffset::default()
    );
    assert_eq!(app.pending_tasks(), 0);
    assert!(app.run_next_task().is_none());
    assert!(!path.exists());
    assert_eq!(
        app.diagnostics(window)
            .expect("restored window should have diagnostics")
            .frame
            .full_redraws,
        0
    );
    assert!(!app.is_dirty());
    assert_eq!(app.store().saved_revision(), app.revision());

    let _ = std::fs::remove_file(path);
}

#[test]
fn text_editor_open_menu_requests_dialog_and_selected_path_opens_document() {
    let path = temp_text_path("runtime_open_dialog.txt");
    std::fs::write(&path, "opened from dialog").expect("fixture file should be writable");
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let open = projected
        .binding::<document::OpenFile>()
        .expect("open command should be in the view");

    assert!(open.is_enabled());
    assert_eq!(open.state().label.as_deref(), Some("Open"));
    assert_eq!(
        open.state().shortcut.map(|shortcut| shortcut.as_str()),
        Some("Primary+O")
    );

    let effect = app
        .activate_in(window, open)
        .expect("open command should activate");

    assert_eq!(effect, response::Effect::OpenFileDialog);
    assert_eq!(app.state().last_status, "choosing file");
    assert_eq!(app.revision().get(), 1);
    assert_eq!(
        app.session().file_dialog(window),
        Some(session::FileDialog::Open)
    );
    let requests = app.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].window(), window);
    assert_eq!(
        requests[0].kind(),
        session::RequestKind::FileDialog(session::FileDialog::Open)
    );

    let outcome = app
        .handle_input(window, Input::file_path_selected(Some(path.clone())))
        .expect("selected file should be handled");

    assert!(outcome.is_handled());
    assert!(outcome.changed_state());
    assert_eq!(app.state().document.text(), "opened from dialog");
    assert_eq!(app.state().document.path(), Some(path.as_path()));
    assert!(!app.state().document.is_dirty());
    assert_eq!(
        app.state().last_status,
        format!("opened {}", text_editor::display_path(&path))
    );
    assert_eq!(app.session().file_dialog(window), None);
    assert!(app.requests().is_empty());
    assert_eq!(app.revision().get(), 2);

    let _ = std::fs::remove_file(path);
}

#[test]
fn sequenced_view_actions_preserve_all_effects() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");
    let open = projected
        .binding::<document::OpenFile>()
        .expect("open command should be in the view")
        .action();
    assert!(app.clear_redraw_request(window));

    let outcome = app
        .handle_view(
            window,
            view::Action::sequence([view::Action::focus(focus), open]),
        )
        .expect("sequenced focus and open should be handled");

    assert!(outcome.is_handled());
    assert!(outcome.changed_state());
    assert!(outcome.effect().contains_invalidation());
    assert!(outcome.effect().contains(&response::Effect::OpenFileDialog));
    assert_eq!(app.state().last_status, "choosing file");
    assert_eq!(app.session().focused(window), Some(focus));
    assert!(app.session().windows()[0].redraw_requested());
    assert_eq!(
        app.session().file_dialog(window),
        Some(session::FileDialog::Open)
    );
}

#[test]
fn text_editor_open_dialog_cancel_updates_status_without_touching_document() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let open = projected
        .binding::<document::OpenFile>()
        .expect("open command should be in the view");

    let effect = app
        .activate_in(window, open)
        .expect("open command should activate");

    assert_eq!(effect, response::Effect::OpenFileDialog);
    assert_eq!(app.state().last_status, "choosing file");
    assert_eq!(
        app.session().file_dialog(window),
        Some(session::FileDialog::Open)
    );

    let outcome = app
        .handle_input(window, Input::file_path_selected(None))
        .expect("canceled open dialog should be handled");

    assert!(outcome.is_handled());
    assert!(outcome.changed_state());
    assert_eq!(app.state().document.text(), "");
    assert_eq!(app.state().document.path(), None);
    assert!(!app.state().document.is_dirty());
    assert_eq!(app.state().last_status, "open canceled");
    assert_eq!(app.session().file_dialog(window), None);
    assert_eq!(app.revision().get(), 2);
    assert_eq!(
        app.store().changes()[1].reason(),
        &state::Reason::Notification(
            <document::OpenDialogCanceled as notification::Notification>::NAME
        )
    );
}

#[test]
fn text_editor_save_menu_for_untitled_dirty_document_requests_save_dialog() {
    let path = temp_text_path("runtime_save_dialog.txt");
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

    let projected = app
        .present(window)
        .expect("window should still have a view");
    let save = projected
        .binding::<document::SaveFile>()
        .expect("save command should be in the view");

    assert!(save.is_enabled());

    let effect = app
        .activate_in(window, save)
        .expect("save command should activate");

    assert_eq!(effect, response::Effect::SaveFileDialog);
    assert_eq!(app.state().last_status, "choosing save location");
    assert_eq!(app.revision().get(), 2);
    assert_eq!(
        app.session().file_dialog(window),
        Some(session::FileDialog::SaveAs)
    );
    let requests = app.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].window(), window);
    assert_eq!(
        requests[0].kind(),
        session::RequestKind::FileDialog(session::FileDialog::SaveAs)
    );

    let outcome = app
        .handle_input(window, Input::file_path_selected(Some(path.clone())))
        .expect("selected save path should be handled");

    assert!(outcome.is_handled());
    assert!(outcome.changed_state());
    assert_eq!(app.pending_tasks(), 1);
    assert_eq!(app.state().document.text(), "unsaved");
    assert_eq!(app.state().document.path(), None);
    assert!(app.state().document.is_dirty());
    assert_eq!(
        app.state().last_status,
        format!("saving {}", text_editor::display_path(&path))
    );
    assert_eq!(app.session().file_dialog(window), None);
    assert!(app.requests().is_empty());
    assert_eq!(app.revision().get(), 3);
    assert!(app.clear_redraw_request(window));

    let save_task = app.run_next_task().expect("save task should run");
    assert_eq!(app.pending_tasks(), 0);
    assert_eq!(save_task.status(), task::Status::Completed);
    assert!(save_task.changed_state());
    assert_eq!(
        std::fs::read_to_string(&path).expect("saved file should be readable"),
        "unsaved"
    );
    assert_eq!(app.state().document.path(), Some(path.as_path()));
    assert!(!app.state().document.is_dirty());
    assert_eq!(
        app.state().last_status,
        format!("saved {}", text_editor::display_path(&path))
    );
    assert_eq!(app.revision().get(), 4);
    assert!(app.session().windows()[0].redraw_requested());

    let _ = std::fs::remove_file(path);
}

#[test]
fn text_editor_save_dialog_cancel_updates_status_without_saving() {
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

    let projected = app
        .present(window)
        .expect("window should still have a view");
    let save = projected
        .binding::<document::SaveFile>()
        .expect("save command should be in the view");

    let effect = app
        .activate_in(window, save)
        .expect("save command should activate");

    assert_eq!(effect, response::Effect::SaveFileDialog);
    assert_eq!(app.state().last_status, "choosing save location");

    let outcome = app
        .handle_input(window, Input::file_path_selected(None))
        .expect("canceled save dialog should be handled");

    assert!(outcome.is_handled());
    assert!(outcome.changed_state());
    assert_eq!(app.state().document.text(), "unsaved");
    assert_eq!(app.state().document.path(), None);
    assert!(app.state().document.is_dirty());
    assert_eq!(app.state().last_status, "save canceled");
    assert_eq!(app.session().file_dialog(window), None);
    assert_eq!(app.revision().get(), 3);
    assert_eq!(
        app.store().changes()[2].reason(),
        &state::Reason::Notification(
            <document::SaveDialogCanceled as notification::Notification>::NAME
        )
    );
}

#[test]
fn text_editor_dialog_cancel_notifications_are_not_palette_commands() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    app.present(window).expect("initial view should present");
    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");

    let projected = app.present(window).expect("palette should project");
    let labels = projected.labels();

    assert!(!labels.contains(&"Open Canceled"));
    assert!(!labels.contains(&"Save Canceled"));
}

#[test]
fn text_editor_view_toggle_commands_target_app_state() {
    let mut app = text_editor::runtime(text_editor::State::default());
    let wrap = app.trigger::<text_editor::ToggleWrapText>(());
    let debug = app.trigger::<text_editor::ToggleDebugPanel>(());

    assert_eq!(app.state_for(&wrap).checked, Some(true));
    assert_eq!(app.state_for(&debug).checked, Some(false));

    let wrap_response = app.invoke(wrap);
    let debug_response = app.invoke(debug);

    wrap_response.output.expect("wrap should resolve");
    debug_response.output.expect("debug should resolve");
    assert!(!app.state().wrap_text);
    assert!(app.state().show_debug_panel);
    assert_eq!(app.revision().get(), 2);

    let wrap = app.trigger::<text_editor::ToggleWrapText>(());
    let debug = app.trigger::<text_editor::ToggleDebugPanel>(());

    assert_eq!(app.state_for(&wrap).checked, Some(false));
    assert_eq!(app.state_for(&debug).checked, Some(true));
}

#[test]
fn text_editor_view_resolves_command_bindings_from_runtime() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");

    assert_eq!(projected.root().role(), view::Role::Root);
    assert!(projected.labels().contains(&"File"));
    assert!(projected.labels().contains(&"View"));
    assert!(!projected.labels().contains(&"Debug"));

    let save = projected
        .binding::<document::SaveFile>()
        .expect("save command should be in the view");

    assert_eq!(save.command_name(), document::SaveFile::NAME);
    assert!(!save.is_enabled());
    assert_eq!(save.state().label.as_deref(), Some("Save"));
    assert_eq!(
        save.state().shortcut.map(|shortcut| shortcut.as_str()),
        Some("Primary+S")
    );

    let wrap = projected
        .binding::<text_editor::ToggleWrapText>()
        .expect("wrap command should be in the view");
    let debug = projected
        .binding::<text_editor::ToggleDebugPanel>()
        .expect("debug command should be in the view");

    assert_eq!(wrap.state().checked, Some(true));
    assert_eq!(debug.state().checked, Some(false));
    assert_eq!(projected.text_areas()[0].buffer().text(), "");
    assert_eq!(projected.text_areas()[0].wrap(), view::Wrap::Word);
    assert_eq!(
        projected.text_areas()[0].focus(),
        Some(session::Focus::text("document"))
    );
    let focus = projected.text_areas()[0]
        .focus()
        .expect("text area should declare a focus target");
    assert!(app.focus(window, focus));

    let edit = app.trigger::<document::ApplyEdit>(text::edit::Edit::insert("alpha"));
    app.invoke_focused(window, edit)
        .output
        .expect("edit should resolve through document target");
    let toggle_wrap = app.trigger::<text_editor::ToggleWrapText>(());
    app.invoke(toggle_wrap)
        .output
        .expect("wrap should resolve through app target");
    let toggle_debug = app.trigger::<text_editor::ToggleDebugPanel>(());
    app.invoke(toggle_debug)
        .output
        .expect("debug should resolve through app target");

    let projected = app
        .present(window)
        .expect("window should still have a view");

    assert!(projected.labels().contains(&"Debug"));
    assert_eq!(projected.text_areas()[0].buffer().text(), "alpha");
    assert_eq!(projected.text_areas()[0].wrap(), view::Wrap::None);
    assert_eq!(
        projected
            .binding::<text_editor::ToggleWrapText>()
            .expect("wrap command should remain in the view")
            .state()
            .checked,
        Some(false)
    );
    assert_eq!(
        projected
            .binding::<text_editor::ToggleDebugPanel>()
            .expect("debug command should remain in the view")
            .state()
            .checked,
        Some(true)
    );
}

#[test]
fn text_editor_debug_panel_reads_framework_diagnostics_from_view_context() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let diagnostics: &mut Diagnostics = app
        .diagnostics_mut(window)
        .expect("window should have framework diagnostics");
    diagnostics.text.author_text_overflows = 2;
    diagnostics.text.text_area_paint_layout_calls = 7;
    diagnostics.text.text_area_metrics_layout_calls = 3;
    diagnostics.scroll.wheel_events = 5;
    diagnostics.scroll.text_area_viewports = 2;
    diagnostics.frame.full_redraws = 11;
    diagnostics.frame.view_rebuilds = 12;
    diagnostics.frame.layout_reuses = 13;
    diagnostics.frame.text_area_render_surfaces = 4;

    app.invoke(app.trigger::<text_editor::ToggleDebugPanel>(()))
        .output
        .expect("debug toggle should resolve");

    let projected = app.present(window).expect("window should have a view");
    let labels = projected.labels();

    assert!(
        labels
            .iter()
            .any(|label| label.contains("Text layout: author overflows 2, paint 7, metrics 3"))
    );
    assert!(labels.iter().any(|label| label.contains("Scroll: wheel 5")));
    assert!(
        labels
            .iter()
            .any(|label| label.contains("text area viewports 2"))
    );
    assert!(
        labels
            .iter()
            .any(|label| label.contains("Frames: full 11, rebuilds 13"))
    );
    assert!(labels.iter().any(|label| label.contains("text surfaces 4")));
}

#[test]
fn text_editor_render_records_live_text_and_frame_diagnostics() {
    let document = (0..80)
        .map(|line| format!("diagnostic line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut app = text_editor::app(text_editor::State {
        document: TextDocument::from_multiline_text(document),
        ..text_editor::State::default()
    });

    app.start();

    let window = app.session().windows()[0].id();
    assert_eq!(
        app.diagnostics(window)
            .expect("window should have diagnostics")
            .frame
            .full_redraws,
        0
    );

    app.render_scene(window, geometry::Size::new(800, 600))
        .expect("scene should render");

    let diagnostics = app
        .diagnostics(window)
        .expect("window should have diagnostics after render");
    assert_eq!(diagnostics.frame.full_redraws, 1);
    assert_eq!(diagnostics.scroll.text_area_viewports, 1);
    assert!(diagnostics.text.text_area_paint_layout_calls > 0);
    assert!(diagnostics.text.text_area_visible_logical_lines > 0);
    assert!(diagnostics.text.text_area_render_surface_calls > 0);
    assert!(diagnostics.frame.text_area_render_surfaces > 0);

    app.invoke(app.trigger::<text_editor::ToggleDebugPanel>(()))
        .output
        .expect("debug toggle should resolve");

    let projected = app.present(window).expect("window should have a view");
    let labels = projected.labels();

    assert!(labels.iter().any(|label| label.contains("Frames: full 1")));
    assert!(
        labels
            .iter()
            .any(|label| label.contains("Text layout: author overflows "))
    );
}

#[test]
fn text_editor_view_command_activation_invokes_typed_target() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("window should have a view");
    let wrap = projected
        .binding::<text_editor::ToggleWrapText>()
        .expect("wrap command should be in the view");

    let effect = app.activate(wrap).expect("wrap command should activate");

    assert_eq!(effect, response::Effect::None);
    assert!(!app.state().wrap_text);
    assert_eq!(app.revision().get(), 1);
    assert_eq!(
        app.store().changes()[0].reason(),
        &state::Reason::Command("view.toggle_wrap_text")
    );

    let projected = app
        .present(window)
        .expect("window should still have a view");
    let save = projected
        .binding::<document::SaveFile>()
        .expect("save command should be in the view");

    let error = app
        .activate(save)
        .expect_err("disabled save should not activate");

    assert!(matches!(
        error,
        Error::Disabled {
            command: "document.save_file"
        }
    ));
    assert_eq!(app.revision().get(), 1);
}

#[test]
fn text_editor_view_actions_are_owned_host_events() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    assert!(app.clear_redraw_request(window));
    assert!(!app.session().windows()[0].redraw_requested());

    let wrap_action = {
        let projected = app.present(window).expect("window should have a view");
        projected
            .binding::<text_editor::ToggleWrapText>()
            .expect("wrap command should be in the view")
            .action()
    };

    let wrap_outcome = app
        .handle_view(window, wrap_action)
        .expect("wrap view action should be handled");

    assert!(wrap_outcome.is_handled());
    assert!(wrap_outcome.changed_state());
    assert_eq!(wrap_outcome.effect(), &response::Effect::None);
    assert!(!app.state().wrap_text);
    assert_eq!(app.revision().get(), 1);
    assert!(app.session().windows()[0].redraw_requested());
    assert!(app.clear_redraw_request(window));

    let focus_action = {
        let projected = app
            .present(window)
            .expect("window should still have a view");
        projected.text_areas()[0]
            .focus_action()
            .expect("text area should expose a focus action")
    };

    let focus_outcome = app
        .handle_view(window, focus_action)
        .expect("focus view action should be handled");

    assert!(focus_outcome.is_handled());
    assert!(!focus_outcome.changed_state());
    assert!(focus_outcome.effect().contains_invalidation());
    assert_eq!(
        app.session().focused(window),
        Some(session::Focus::text("document"))
    );
    assert!(app.session().windows()[0].redraw_requested());
    assert!(app.clear_redraw_request(window));

    let edit_outcome = app
        .handle_view(
            window,
            view::Action::text_edit(text::edit::Edit::insert("host action")),
        )
        .expect("text edit view action should be handled");

    assert!(edit_outcome.is_handled());
    assert!(edit_outcome.changed_state());
    assert_eq!(app.state().document.text(), "host action");
    assert_eq!(app.state().last_status, "edit");
    assert_eq!(app.revision().get(), 2);
    assert!(app.session().windows()[0].redraw_requested());
}
