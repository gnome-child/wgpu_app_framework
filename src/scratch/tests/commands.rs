use super::*;

#[test]
fn key_down_shortcuts_are_matched_from_registered_specs() {
    let mut app = Runtime::new(SourceState::default())
        .commands(|commands| {
            commands.register::<RecordSource>(command::Spec::new("Record").shortcut("Ctrl+R"));
        })
        .responders(|responders| {
            responders.app().target::<RecordSource>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Sources"));
        });

    app.start();

    let window = app.session().windows()[0].id();
    let outcome = app
        .handle_input(
            window,
            Input::key_down(
                input::Key::Character('r'),
                input::Modifiers::new(false, true, false, false),
            ),
        )
        .expect("keydown should dispatch registered shortcut");

    assert!(outcome.is_handled());
    assert!(outcome.changed_state());
    assert_eq!(app.state().sources, vec![context::Source::Shortcut]);
    assert_eq!(app.revision().get(), 1);
}

#[test]
fn duplicate_shortcuts_are_ambiguous_instead_of_first_registration_winning() {
    let mut app = Runtime::new(EditorState {
        project: Project {
            dirty: true,
            save_count: 0,
        },
        ..EditorState::default()
    })
    .commands(|commands| {
        commands
            .register::<Save>(command::Spec::new("Save").shortcut("Ctrl+S"))
            .register::<Ping>(command::Spec::new("Ping").shortcut("Ctrl+S"));
    })
    .responders(|responders| {
        responders.app().target::<Save>().target::<Ping>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Keymap"));
    });

    app.start();

    let window = app.session().windows()[0].id();
    let error = app
        .handle_input(window, Input::shortcut("Ctrl+S"))
        .expect_err("ambiguous shortcut should not dispatch");

    match error {
        Error::AmbiguousShortcut { shortcut, commands } => {
            assert_eq!(shortcut, "Ctrl+S");
            assert_eq!(commands, vec!["app.save", "app.ping"]);
        }
        error => panic!("unexpected error: {error}"),
    }
    assert!(app.state().project.dirty);
    assert_eq!(app.state().project.save_count, 0);
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn key_down_shortcut_dispatch_still_requires_unit_command_args() {
    let mut app = Runtime::new(EditorState::default())
        .commands(|commands| {
            commands.register::<OpenNamed>(command::Spec::new("Open Named").shortcut("Ctrl+P"));
        })
        .responders(|responders| {
            responders.app().target::<OpenNamed>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Keymap"));
        });

    app.start();

    let window = app.session().windows()[0].id();
    let error = app
        .handle_input(
            window,
            Input::key_down(
                input::Key::Character('p'),
                input::Modifiers::new(false, true, false, false),
            ),
        )
        .expect_err("keydown shortcut cannot synthesize command args");

    assert!(matches!(
        error,
        Error::ShortcutRequiresArgs {
            shortcut: "Ctrl+P",
            command: "app.open_named",
        }
    ));
    assert_eq!(app.state().event_count, 0);
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn shortcut_dispatch_requires_unit_command_args() {
    let mut app = Runtime::new(EditorState::default())
        .commands(|commands| {
            commands.register::<OpenNamed>(command::Spec::new("Open Named").shortcut("Ctrl+P"));
        })
        .responders(|responders| {
            responders.app().target::<OpenNamed>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Keymap"));
        });

    app.start();

    let trigger = app.trigger::<OpenNamed>("alpha".to_owned());
    let command_state = app.state_for(&trigger);

    assert!(command_state.is_enabled());
    assert_eq!(command_state.label.as_deref(), Some("Open Named"));
    assert_eq!(command_state.shortcut, None);

    let window = app.session().windows()[0].id();
    let error = app
        .handle_input(window, Input::shortcut("Ctrl+P"))
        .expect_err("shortcut cannot synthesize command args");

    assert!(matches!(
        error,
        Error::ShortcutRequiresArgs {
            shortcut: "Ctrl+P",
            command: "app.open_named",
        }
    ));
    assert_eq!(app.state().event_count, 0);
    assert_eq!(app.revision(), state::Revision::initial());

    let response = app.invoke(trigger);

    assert_eq!(response.output.expect("typed trigger should carry args"), 5);
    assert_eq!(app.state().event_count, 5);
    assert_eq!(app.revision().get(), 1);
}

#[test]
fn re_registering_command_replaces_its_shortcut_binding() {
    let mut app = Runtime::new(EditorState {
        project: Project {
            dirty: true,
            save_count: 0,
        },
        ..EditorState::default()
    })
    .commands(|commands| {
        commands
            .register::<Save>(command::Spec::new("Save").shortcut("Ctrl+S"))
            .register::<Save>(command::Spec::new("Save").shortcut("Ctrl+Shift+S"));
    })
    .responders(|responders| {
        responders.app().target::<Save>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Keymap"));
    });

    app.start();

    let window = app.session().windows()[0].id();
    let old = app
        .handle_input(window, Input::shortcut("Ctrl+S"))
        .expect("old shortcut should be ignored");

    assert!(!old.is_handled());
    assert!(app.state().project.dirty);
    assert_eq!(app.revision(), state::Revision::initial());

    let new = app
        .handle_input(window, Input::shortcut("Ctrl+Shift+S"))
        .expect("new shortcut should dispatch");

    assert!(new.is_handled());
    assert!(!app.state().project.dirty);
    assert_eq!(app.state().project.save_count, 1);
    assert_eq!(app.revision().get(), 1);
}

#[test]
fn session_windows_are_framework_owned_runtime_state() {
    assert!(Session::default().windows().is_empty());

    let mut app = Runtime::new(EditorState::default()).started(|cx| {
        let window = cx.open_window(window::Options::new("Editor"));
        assert!(!cx.request_redraw(window));
    });

    app.start();

    let window = app.session().windows()[0].id();
    assert_eq!(app.session().windows()[0].title(), "Editor");
    assert!(app.session().windows()[0].redraw_requested());
    assert_eq!(app.revision(), state::Revision::initial());

    app.emit(());

    assert!(app.session().contains(window));
}

#[test]
fn command_availability_does_not_mutate_model_or_revision() {
    let mut app = Runtime::new(EditorState {
        document: SaveDocument {
            dirty: true,
            save_count: 0,
        },
        ..EditorState::default()
    })
    .commands(|commands| {
        commands.register::<Save>(command::Spec::new("Save").shortcut("Ctrl+S"));
    })
    .responders(|responders| {
        responders
            .object("document", |state: &mut EditorState| &mut state.document)
            .target::<Save>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Editor"));
    });
    let revision = app.revision();
    let trigger = app.trigger::<Save>(());
    app.start();
    let window = app.session().windows()[0].id();
    assert!(app.focus(window, session::Focus::text("document")));

    let state = app.state_for_focused(window, &trigger);

    assert!(state.is_enabled());
    assert_eq!(app.revision(), revision);
    assert!(app.state().document.dirty);
    assert_eq!(app.state().document.save_count, 0);
}

#[test]
fn command_observer_runs_after_successful_command_and_shares_revision() {
    let mut app = Runtime::new(EditorState {
        document: SaveDocument {
            dirty: true,
            save_count: 0,
        },
        ..EditorState::default()
    })
    .commands(|commands| {
        commands.register::<Save>(command::Spec::new("Save"));
    })
    .responders(|responders| {
        responders
            .object("document", |state: &mut EditorState| &mut state.document)
            .target::<Save>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Editor"));
    })
    .observe::<Save>(|state, _: &(), observation| {
        assert_eq!(observation.source(), context::Source::Programmatic);
        assert_eq!(observation.effect(), &response::Effect::Repaint);
        assert!(observation.command_changed_state());
        state.event_count += 1;
        observation.mark_changed();
    });
    app.start();
    let window = app.session().windows()[0].id();
    assert!(app.focus(window, session::Focus::text("document")));

    let response = app.invoke_focused(window, app.trigger::<Save>(()));

    assert!(response.output.is_ok());
    assert!(response.changed_state());
    assert_eq!(app.state().document.save_count, 1);
    assert_eq!(app.state().event_count, 1);
    assert_eq!(app.revision().get(), 1);
    assert_eq!(app.store().changes().len(), 1);
}

#[test]
fn observer_changed_state_commits_even_when_command_output_is_unchanged() {
    let mut app = Runtime::new(EditorState::default())
        .commands(|commands| {
            commands.register::<Ping>(command::Spec::new("Ping"));
        })
        .responders(|responders| {
            responders.app().target::<Ping>();
        })
        .observe::<Ping>(|state, _: &(), observation| {
            assert!(!observation.command_changed_state());
            state.event_count += 1;
            observation.mark_changed();
        });

    let response = app.invoke(app.trigger::<Ping>(()));

    assert!(response.output.is_ok());
    assert!(response.changed_state());
    assert_eq!(app.state().event_count, 1);
    assert_eq!(app.revision().get(), 1);
    assert_eq!(
        app.store().changes()[0].reason(),
        &state::Reason::Command("app.ping")
    );
}

#[test]
fn command_state_queries_do_not_notify_observers() {
    let mut app = Runtime::new(EditorState {
        document: SaveDocument {
            dirty: true,
            save_count: 0,
        },
        ..EditorState::default()
    })
    .commands(|commands| {
        commands.register::<Save>(command::Spec::new("Save"));
    })
    .responders(|responders| {
        responders
            .object("document", |state: &mut EditorState| &mut state.document)
            .target::<Save>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Editor"));
    })
    .observe::<Save>(|state, _: &(), observation| {
        state.event_count += 1;
        observation.mark_changed();
    });
    let save = app.trigger::<Save>(());
    app.start();
    let window = app.session().windows()[0].id();
    assert!(app.focus(window, session::Focus::text("document")));

    assert!(app.state_for_focused(window, &save).is_enabled());

    assert_eq!(app.state().event_count, 0);
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn field_responder_target_invokes_by_type_and_bumps_revision_when_changed() {
    let mut app = Runtime::new(EditorState {
        document: SaveDocument {
            dirty: true,
            save_count: 0,
        },
        ..EditorState::default()
    })
    .commands(|commands| {
        commands.register::<Save>(command::Spec::new("Save").shortcut("Ctrl+S"));
    })
    .responders(|responders| {
        responders
            .object("document", |state: &mut EditorState| &mut state.document)
            .target::<Save>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Editor"));
    });
    app.start();
    let window = app.session().windows()[0].id();
    assert!(app.focus(window, session::Focus::text("document")));

    let response = app.invoke_focused(window, app.trigger::<Save>(()));

    assert!(response.output.is_ok());
    assert_eq!(response.effect, response::Effect::Repaint);
    assert_eq!(app.revision().get(), 1);
    assert!(app.is_dirty());
    assert!(!app.state().document.dirty);
    assert_eq!(app.state().document.save_count, 1);
    assert_eq!(
        app.store().changes()[0].reason(),
        &state::Reason::Command("app.save")
    );
}

#[test]
fn unchanged_command_response_does_not_bump_revision() {
    let mut app = Runtime::new(EditorState::default())
        .commands(|commands| {
            commands.register::<Ping>(command::Spec::new("Ping"));
        })
        .responders(|responders| {
            responders.app().target::<Ping>();
        });
    let trigger = app.trigger::<Ping>(());

    let response = app.invoke(trigger);

    assert!(response.output.is_ok());
    assert_eq!(app.revision(), state::Revision::initial());
    assert!(!app.is_dirty());
}

#[test]
fn app_and_field_targets_can_be_configured_without_command_ids() {
    let mut app = Runtime::new(EditorState {
        document: SaveDocument {
            dirty: true,
            save_count: 0,
        },
        ..EditorState::default()
    })
    .commands(|commands| {
        commands
            .register::<text_editor::ToggleWrapText>(command::Spec::new("Wrap text"))
            .register::<Save>(command::Spec::new("Save"));
    })
    .responders(|responders| {
        responders.app().target::<text_editor::ToggleWrapText>();
        responders
            .object("document", |state: &mut EditorState| &mut state.document)
            .target::<Save>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Editor"));
    });
    app.start();
    let window = app.session().windows()[0].id();
    assert!(app.focus(window, session::Focus::text("document")));

    let toggle = app.trigger::<text_editor::ToggleWrapText>(());
    assert!(app.invoke(toggle).output.is_ok());
    assert!(app.state().wrap_text);

    app.mark_saved();

    let save = app.trigger::<Save>(());
    assert!(app.invoke_focused(window, save).output.is_ok());
    assert!(!app.state().document.dirty);
    assert_eq!(app.state().document.save_count, 1);
}

#[test]
fn focused_responder_wins_over_app_when_builder_registers_app_first() {
    let mut app = Runtime::new(EditorState {
        document: SaveDocument {
            dirty: true,
            save_count: 0,
        },
        project: Project {
            dirty: true,
            save_count: 0,
        },
        ..EditorState::default()
    })
    .commands(|commands| {
        commands.register::<Save>(command::Spec::new("Save"));
    })
    .responders(|responders| {
        responders.app().target::<Save>();
        responders
            .object("document", |state: &mut EditorState| &mut state.document)
            .target::<Save>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Editor"));
    });
    app.start();
    let window = app.session().windows()[0].id();
    assert!(app.focus(window, session::Focus::text("document")));

    let save = app.trigger::<Save>(());
    let response = app.invoke_focused(window, save);

    assert!(response.output.is_ok());
    assert!(!app.state().document.dirty);
    assert_eq!(app.state().document.save_count, 1);
    assert!(app.state().project.dirty);
    assert_eq!(app.state().project.save_count, 0);
}

#[test]
fn presentation_clears_stale_focus_before_resolving_command_state() {
    let mut app = Runtime::new(EditorState {
        document: SaveDocument {
            dirty: true,
            save_count: 0,
        },
        project: Project {
            dirty: false,
            save_count: 0,
        },
        wrap_text: true,
        ..EditorState::default()
    })
    .commands(|commands| {
        commands.register::<Save>(command::Spec::new("Save"));
    })
    .responders(|responders| {
        responders.app().target::<Save>();
        responders
            .object("document", |state: &mut EditorState| &mut state.document)
            .target::<Save>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Editor"));
    })
    .view(|state, _| {
        let mut root =
            view::Node::root().child(view::Node::menu_bar().child(
                view::Node::menu("menu.file", "File").child(view::Node::menu_bound::<Save>()),
            ));

        if state.wrap_text {
            root = root.child(view::Node::text_area_state(
                view::control::TextArea::new("").with_focus(session::Focus::text("document")),
            ));
        }

        View::new(root)
    });

    app.start();

    let window = app.session().windows()[0].id();
    assert!(app.focus(window, session::Focus::text("document")));

    let projected = app.present(window).expect("window should have a view");
    assert!(
        projected
            .binding::<Save>()
            .expect("save command should be in the view")
            .is_enabled()
    );

    app.change(state::Reason::programmatic("hide_document"), |state| {
        state.wrap_text = false;
    });

    let projected = app
        .present(window)
        .expect("window should still have a view");

    assert_eq!(app.session().focused(window), None);
    assert!(
        !projected
            .binding::<Save>()
            .expect("save command should remain in the view")
            .is_enabled()
    );
    assert!(app.state().document.dirty);
    assert!(!app.state().project.dirty);
}

#[test]
fn presentation_is_retained_as_framework_owned_composition() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    assert!(app.composition(window).is_none());

    let projected = app.present(window).expect("window should have a view");
    let composition: &Composition = app
        .composition(window)
        .expect("presenting should retain a composition");

    assert_eq!(composition.window(), window);
    assert_eq!(composition.view().labels(), projected.labels());
    assert!(
        composition
            .view()
            .binding::<document::OpenFile>()
            .expect("open command should be retained")
            .is_enabled()
    );
    assert!(composition.view().popups().is_empty());

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

    app.present(window)
        .expect("window should still have a view after opening a menu");
    let composition = app
        .composition(window)
        .expect("composition should update after presenting");

    assert_eq!(composition.view().popups().len(), 1);
    assert_eq!(composition.view().popups()[0].label_text(), Some("File"));
}
