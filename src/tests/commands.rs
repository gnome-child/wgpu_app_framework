use super::*;

#[test]
fn standard_editing_is_an_enumerable_declinable_set() {
    let editing = document::Editing::standard();
    let members = editing
        .members()
        .map(|member| {
            (
                member.command_name(),
                member.spec().display_name(),
                member.spec().standard_role(),
                member
                    .spec()
                    .declared_key_chord()
                    .map(|chord| chord.as_str()),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        members,
        vec![
            ("document.apply_edit", "Edit", None, None),
            (
                "edit.cut",
                "Cut",
                Some(command::Standard::Cut),
                Some("Standard::Cut")
            ),
            (
                "edit.copy",
                "Copy",
                Some(command::Standard::Copy),
                Some("Standard::Copy")
            ),
            (
                "edit.paste",
                "Paste",
                Some(command::Standard::Paste),
                Some("Standard::Paste")
            ),
            (
                "edit.delete",
                "Delete",
                Some(command::Standard::Delete),
                Some("Standard::Delete")
            ),
            (
                "edit.select_all",
                "Select All",
                Some(command::Standard::SelectAll),
                Some("Standard::SelectAll")
            ),
        ]
    );

    let declined = document::Editing::standard()
        .without::<document::Delete>()
        .members()
        .map(command::Member::command_name)
        .collect::<Vec<_>>();
    assert!(!declined.contains(&"edit.delete"));
    assert_eq!(declined.len(), members.len() - 1);
}

#[test]
fn authored_menu_bars_keep_explicit_order_while_command_registration_stays_nonvisual() {
    let mut authored = Runtime::new(SourceState::default())
        .commands(|commands| {
            commands
                .register::<RecordSource>(command::Spec::new("First"))
                .register::<DisabledRecordSource>(command::Spec::new("Second"));
        })
        .responders(|responders| {
            responders
                .app()
                .target::<RecordSource>()
                .target::<DisabledRecordSource>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Authored menu"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.menu_bar(|ui| {
                    ui.menu("menu.file", "File", |ui| {
                        ui.add(widget::Binding::<RecordSource>::menu());
                        ui.separator();
                        ui.add(widget::Binding::<DisabledRecordSource>::menu());
                    });
                    ui.menu("menu.edit", "Edit", |_| {});
                });
            })
        });

    authored.start();
    let window = authored.session().windows()[0].id();
    let projected = authored
        .present(window)
        .expect("authored bar should project");
    let bar = find_view_node(projected.root(), view::Role::MenuBar)
        .expect("explicit authoring should create one menu bar");
    assert_eq!(
        bar.children()
            .iter()
            .map(view::Node::label_text)
            .collect::<Vec<_>>(),
        vec![Some("File"), Some("Edit")]
    );
    assert_eq!(
        bar.children()[0]
            .children()
            .iter()
            .map(view::Node::role)
            .collect::<Vec<_>>(),
        vec![
            view::Role::Binding,
            view::Role::Separator,
            view::Role::Binding,
        ]
    );

    let mut no_bar = Runtime::new(SourceState::default())
        .commands(|commands| {
            commands.register::<RecordSource>(command::Spec::new("Registered"));
        })
        .responders(|responders| {
            responders.app().target::<RecordSource>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("No ambient menu"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.label("Content only");
            })
        });

    no_bar.start();
    let window = no_bar.session().windows()[0].id();
    let projected = no_bar
        .present(window)
        .expect("content-only view should project");
    assert!(
        find_view_node(projected.root(), view::Role::MenuBar).is_none(),
        "registration alone must never create ambient menu UI"
    );
}

#[test]
fn standard_menu_bar_derives_ordinary_live_menu_bindings_on_explicit_request() {
    let mut app = Runtime::new(SourceState::default())
        .keymap(keymap::Profile::windows())
        .commands(|commands| {
            commands
                .register::<RecordSource>(command::Spec::standard(command::Standard::New))
                .register::<Ping>(command::Spec::standard(command::Standard::Open))
                .register::<DisabledRecordSource>(command::Spec::standard(command::Standard::Save));
        })
        .responders(|responders| {
            responders
                .app()
                .target::<RecordSource>()
                .target::<DisabledRecordSource>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Derived menu"));
        })
        .view(|_, _| {
            widget::view(|ui| {
                ui.standard_menu_bar();
                ui.label("Content");
            })
        });

    app.start();
    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("derived bar should project");
    let bar = find_view_node(projected.root(), view::Role::MenuBar)
        .expect("the explicit request should create one ordinary menu bar");
    let file = bar
        .children()
        .iter()
        .find(|node| node.label_text() == Some("File"))
        .expect("registered File roles should derive a File menu");

    assert_eq!(file.role(), view::Role::Menu);
    assert_eq!(
        file.children()
            .iter()
            .map(view::Node::role)
            .collect::<Vec<_>>(),
        vec![
            view::Role::Binding,
            view::Role::Binding,
            view::Role::Binding,
            view::Role::Separator,
            view::Role::Binding,
        ],
        "the platform topology alone should insert the section separator"
    );
    let binding = |name| {
        file.children()
            .iter()
            .filter_map(view::Node::binding)
            .find(|binding| binding.command_name() == name)
            .expect("registered role should remain in the bar")
    };
    assert!(binding(RecordSource::NAME).is_enabled());
    assert!(
        !binding(Ping::NAME).is_enabled(),
        "an unclaimed registered role remains visible but disabled"
    );
    assert!(!binding(DisabledRecordSource::NAME).is_enabled());
    assert!(
        projected
            .bindings()
            .iter()
            .all(|binding| binding.command_name() != session::OpenCommandPalette::NAME),
        "the palette role has no conventional-bar slot"
    );

    app.activate_in(window, binding(RecordSource::NAME))
        .expect("derived bindings should invoke through the live menu route");
    assert_eq!(app.state().sources, vec![context::Source::Menu]);
}

#[test]
fn typing_history_group_carries_the_text_owned_coalesce_window() {
    let typing =
        <document::ApplyEdit as Command>::history_group(&text::edit::Edit::Insert("a".to_owned()))
            .expect("typing edit should declare a history group");
    let generic = command::HistoryGroup::new("generic");

    assert_eq!(
        typing.coalesce_window(),
        text::edit::TYPING_UNDO_COALESCE_WINDOW
    );
    assert_eq!(generic.coalesce_window(), Duration::from_millis(1000));
    assert_ne!(
        generic
            .clone()
            .with_coalesce_window(Duration::from_millis(250)),
        generic
    );
}

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
fn mac_profile_dispatches_primary_shortcut_with_command_not_control() {
    let mut app = Runtime::new(SourceState::default())
        .keymap(keymap::Profile::mac())
        .commands(|commands| {
            commands.register::<RecordSource>(command::Spec::new("Record").shortcut("Primary+S"));
        })
        .responders(|responders| {
            responders.app().target::<RecordSource>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Keymap"));
        });

    app.start();
    let window = app.session().windows()[0].id();
    let control = app
        .handle_input(
            window,
            Input::key_down(
                input::Key::Character('s'),
                input::Modifiers::new(false, true, false, false),
            ),
        )
        .expect("control should be a valid key input");

    assert!(!control.is_handled());
    assert!(app.state().sources.is_empty());

    let command = app
        .handle_input(
            window,
            Input::key_down(
                input::Key::Character('s'),
                input::Modifiers::new(false, false, false, true),
            ),
        )
        .expect("command should dispatch primary shortcut");

    assert!(command.is_handled());
    assert_eq!(app.state().sources, vec![context::Source::Shortcut]);
}

#[test]
fn windows_profile_dispatches_primary_shortcut_with_control_not_command() {
    let mut app = Runtime::new(SourceState::default())
        .keymap(keymap::Profile::windows())
        .commands(|commands| {
            commands.register::<RecordSource>(command::Spec::new("Record").shortcut("Primary+S"));
        })
        .responders(|responders| {
            responders.app().target::<RecordSource>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Keymap"));
        });

    app.start();
    let window = app.session().windows()[0].id();
    let command = app
        .handle_input(
            window,
            Input::key_down(
                input::Key::Character('s'),
                input::Modifiers::new(false, false, false, true),
            ),
        )
        .expect("command should be a valid key input");

    assert!(!command.is_handled());
    assert!(app.state().sources.is_empty());

    let control = app
        .handle_input(
            window,
            Input::key_down(
                input::Key::Character('s'),
                input::Modifiers::new(false, true, false, false),
            ),
        )
        .expect("control should dispatch primary shortcut");

    assert!(control.is_handled());
    assert_eq!(app.state().sources, vec![context::Source::Shortcut]);
}

#[test]
fn command_palette_opens_filters_and_invokes_unit_commands() {
    let mut app = Runtime::new(SourceState::default())
        .commands(|commands| {
            commands
                .register::<RecordSource>(command::Spec::new("Record").shortcut("Ctrl+R"))
                .register::<HiddenRecordSource>(command::Spec::new("Hidden"))
                .register::<DisabledRecordSource>(command::Spec::new("Disabled"))
                .register::<OpenNamed>(command::Spec::new("Open Named"));
        })
        .responders(|responders| {
            responders
                .app()
                .target::<RecordSource>()
                .target::<HiddenRecordSource>()
                .target::<DisabledRecordSource>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Palette"));
        })
        .view(|_, _| View::new(view::Node::root()));

    app.start();
    let window = app.session().windows()[0].id();
    app.present(window).expect("initial view should present");

    let opened = app
        .handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should dispatch");
    assert!(opened.is_handled());
    assert!(
        app.session()
            .interaction(window)
            .and_then(interaction::Interaction::command_palette)
            .is_some()
    );

    let projected = app.present(window).expect("palette should project");
    let labels = projected.labels();
    assert!(labels.contains(&"Record"));
    assert!(labels.contains(&"System"));
    assert!(!labels.contains(&"Command Palette"));
    assert!(!labels.contains(&"Hidden"));
    assert!(!labels.contains(&"Disabled"));
    assert!(!labels.contains(&"Open Named"));

    for ch in ['r', 'e', 'c'] {
        app.handle_input(
            window,
            Input::key_down_with_text(
                input::Key::Character(ch),
                input::Modifiers::default(),
                Some(ch.to_string()),
            ),
        )
        .expect("typing should edit palette query");
    }
    let projected = app
        .present(window)
        .expect("filtered palette should project");
    let labels = projected.labels();
    assert!(labels.contains(&"Record"));

    let invoked = app
        .handle_input(
            window,
            Input::key_down(input::Key::Enter, input::Modifiers::default()),
        )
        .expect("enter should invoke selected palette command");

    assert!(invoked.is_handled());
    assert_eq!(app.state().sources, vec![context::Source::Palette]);
    assert!(
        app.session()
            .interaction(window)
            .and_then(interaction::Interaction::command_palette)
            .is_none()
    );
}

#[test]
fn command_palette_invokes_against_captured_focus_not_query_focus() {
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
        cx.open_window(window::Options::new("Palette"));
    })
    .view(|_, _| {
        View::new(view::Node::root().child(view::Node::text_area_state(
            view::TextArea::new("").with_focus(session::Focus::text("document")),
        )))
    });

    app.start();
    let window = app.session().windows()[0].id();
    app.present(window).expect("initial view should present");
    assert!(app.focus(window, session::Focus::text("document")));

    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let projected = app.present(window).expect("palette should project");
    assert_eq!(
        app.session().focused(window),
        Some(interaction::CommandPalette::query_focus())
    );
    assert!(projected.labels().contains(&"Document"));

    for ch in ['s', 'a', 'v', 'e'] {
        app.handle_input(
            window,
            Input::key_down_with_text(
                input::Key::Character(ch),
                input::Modifiers::default(),
                Some(ch.to_string()),
            ),
        )
        .expect("typing should edit palette query");
    }
    app.present(window)
        .expect("filtered palette should project");
    app.handle_input(
        window,
        Input::key_down(input::Key::Enter, input::Modifiers::default()),
    )
    .expect("enter should invoke save");

    assert!(!app.state().document.dirty);
    assert_eq!(app.state().document.save_count, 1);
    assert!(app.state().project.dirty);
    assert_eq!(app.state().project.save_count, 0);
}

#[test]
fn command_palette_navigation_and_escape_restore_captured_focus() {
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
        cx.open_window(window::Options::new("Palette"));
    })
    .view(|_, _| {
        View::new(view::Node::root().child(view::Node::text_area_state(
            view::TextArea::new("").with_focus(session::Focus::text("document")),
        )))
    });

    app.start();
    let window = app.session().windows()[0].id();
    app.present(window).expect("initial view should present");
    assert!(app.focus(window, session::Focus::text("document")));

    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette shortcut should open");
    let projected = app.present(window).expect("palette should project");
    assert_eq!(selected_palette_labels(&projected), vec!["Save"]);

    app.handle_input(
        window,
        Input::key_down(input::Key::ArrowDown, input::Modifiers::default()),
    )
    .expect("arrow down should move selection");
    let projected = app.present(window).expect("palette should project");
    assert_eq!(selected_palette_labels(&projected), vec!["Close Window"]);

    app.handle_input(
        window,
        Input::key_down(input::Key::Escape, input::Modifiers::default()),
    )
    .expect("escape should close the palette");
    assert!(
        app.session()
            .interaction(window)
            .and_then(interaction::Interaction::command_palette)
            .is_none()
    );
    assert_eq!(
        app.session().focused(window),
        Some(session::Focus::text("document"))
    );
}

#[test]
fn palette_scope_gives_standard_text_commands_to_query_and_rows_to_captured_document() {
    let clipboard = Clipboard::default();
    clipboard
        .put(&clipboard::Text::new("first\r\nsecond"))
        .expect("test clipboard should accept text");
    let document_focus = session::Focus::text("document");
    let mut app = Runtime::new(text_editor::State {
        document: TextDocument::from_multiline_text("alpha beta"),
        ..text_editor::State::default()
    })
    .commands(|commands| {
        commands.install(document::Editing::standard());
    })
    .responders(|responders| {
        responders
            .object("document", |state: &mut text_editor::State| {
                &mut state.document
            })
            .target::<document::SelectAll>();
    })
    .started(|cx| {
        cx.open_window(window::Options::new("Palette Scope"));
    })
    .view(move |state, _| {
        widget::view(|ui| {
            ui.text_area(widget::TextArea::from_document(&state.document).focus(document_focus));
        })
    })
    .with_clipboard(clipboard);

    app.start();
    let window = app.session().windows()[0].id();
    let projected = app.present(window).expect("editor should present");
    assert!(text_area_node(projected.root()).is_some());
    app.handle_input(window, Input::focus(document_focus))
        .expect("document should focus");
    assert_eq!(app.state().document.selected_text(), None);

    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette should open");
    app.present(window).expect("palette should project");
    app.handle_input(window, Input::text_commit("cut"))
        .expect("typing should edit the query");
    app.handle_input(window, Input::shortcut("Ctrl+A"))
        .expect("select all should select the query");
    let projected = app.present(window).expect("selected query should project");
    let labels = projected.labels();
    assert!(!labels.contains(&"Cut"));
    assert!(!labels.contains(&"Copy"));
    app.handle_input(window, Input::text_commit("select all"))
        .expect("IME commit should edit the standard query box");

    app.handle_input(window, Input::shortcut("Ctrl+A"))
        .expect("select all should belong to the palette query scope");
    let query_focus = interaction::CommandPalette::query_focus();
    assert_eq!(
        text_draft(&app, window, query_focus).selection(),
        Some(0..10)
    );
    assert_eq!(app.state().document.selected_text(), None);
    assert_eq!(app.timeline().undo_depth(), 0);

    app.handle_input(
        window,
        Input::key_down(input::Key::Home, input::Modifiers::default()),
    )
    .expect("home should move the query caret");
    assert_eq!(text_draft(&app, window, query_focus).cursor(), 0);
    app.handle_input(
        window,
        Input::key_down(input::Key::End, input::Modifiers::default()),
    )
    .expect("end should move the query caret");
    assert_eq!(text_draft(&app, window, query_focus).cursor(), 10);
    app.handle_input(
        window,
        Input::key_down(input::Key::ArrowLeft, input::Modifiers::default()),
    )
    .expect("left should move the query caret");
    assert_eq!(text_draft(&app, window, query_focus).cursor(), 9);
    app.handle_input(
        window,
        Input::key_down(input::Key::ArrowRight, input::Modifiers::default()),
    )
    .expect("right should move the query caret");
    assert_eq!(text_draft(&app, window, query_focus).cursor(), 10);
    app.handle_input(
        window,
        Input::key_down(input::Key::ArrowDown, input::Modifiers::default()),
    )
    .expect("down should be consumed by palette result navigation");
    assert_eq!(text_draft(&app, window, query_focus).cursor(), 10);

    app.handle_input(
        window,
        Input::key_down(input::Key::Enter, input::Modifiers::default()),
    )
    .expect("enter should invoke Select All from the captured listing");
    assert_eq!(
        app.state().document.selected_text().as_deref(),
        Some("alpha beta")
    );
    let app_undo_depth = app.timeline().undo_depth();

    app.handle_input(window, Input::shortcut("Ctrl+Shift+P"))
        .expect("palette should reopen");
    app.present(window)
        .expect("reopened palette should project");
    app.handle_input(window, Input::shortcut("Ctrl+V"))
        .expect("paste should use the standard query text service");
    assert_eq!(text_draft(&app, window, query_focus).text(), "first");
    assert_eq!(app.timeline().undo_depth(), app_undo_depth);
    app.handle_input(window, Input::shortcut("Ctrl+A"))
        .expect("select all should select pasted query text");
    app.handle_input(window, Input::shortcut("Ctrl+C"))
        .expect("copy should use the query selection");
    app.handle_input(window, Input::shortcut("Ctrl+X"))
        .expect("cut should delete the query selection");
    assert_eq!(text_draft(&app, window, query_focus).text(), "");
    assert_eq!(
        app.state().document.selected_text().as_deref(),
        Some("alpha beta")
    );
    app.handle_input(window, Input::shortcut("Ctrl+V"))
        .expect("paste should restore copied query text");
    assert_eq!(text_draft(&app, window, query_focus).text(), "first");
    app.handle_input(window, Input::shortcut("Ctrl+Z"))
        .expect("undo should use query draft history");
    assert_eq!(text_draft(&app, window, query_focus).text(), "");
    assert_eq!(app.timeline().undo_depth(), app_undo_depth);

    let preedit = text::edit::Preedit::new("界", Some((0, 3)));
    app.handle_input(window, Input::text_preedit(preedit.clone()))
        .expect("palette query should accept IME preedit");
    assert_eq!(
        app.session()
            .interaction(window)
            .expect("palette should keep interaction state")
            .text_input()
            .preedit(),
        Some(&preedit)
    );
    app.handle_input(window, Input::text_commit("界"))
        .expect("palette query should accept IME commit");
    assert_eq!(text_draft(&app, window, query_focus).text(), "界");
}

fn selected_palette_labels(view: &View) -> Vec<&str> {
    let mut labels = Vec::new();
    collect_selected_palette_labels(view.root(), &mut labels);
    labels
}

fn collect_selected_palette_labels<'a>(node: &'a view::Node, labels: &mut Vec<&'a str>) {
    if node.is_selected()
        && node
            .binding()
            .is_some_and(|binding| binding.source() == context::Source::Palette)
        && let Some(label) = node.label_text()
    {
        labels.push(label);
    }

    for child in node.children() {
        collect_selected_palette_labels(child, labels);
    }
}

fn find_view_node(node: &view::Node, role: view::Role) -> Option<&view::Node> {
    if node.role() == role {
        return Some(node);
    }
    node.children()
        .iter()
        .find_map(|child| find_view_node(child, role))
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
fn duplicate_shortcuts_are_ambiguous_after_profile_resolution() {
    let mut app = Runtime::new(EditorState {
        project: Project {
            dirty: true,
            save_count: 0,
        },
        ..EditorState::default()
    })
    .keymap(keymap::Profile::windows())
    .commands(|commands| {
        commands
            .register::<Save>(command::Spec::new("Save").shortcut("Primary+S"))
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
        .handle_input(
            window,
            Input::key_down(
                input::Key::Character('s'),
                input::Modifiers::new(false, true, false, false),
            ),
        )
        .expect_err("resolved duplicate shortcuts should not dispatch");

    match error {
        Error::AmbiguousShortcut { shortcut, commands } => {
            assert_eq!(shortcut, "Primary+S");
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
fn closing_a_session_window_emits_one_departed_fact() {
    let mut session = Session::default();
    let window = session.open_window(window::Options::new("Departing"));

    assert!(session.close_window(window));
    assert!(!session.close_window(window));
    assert_eq!(session.take_departed(), vec![window]);
    assert!(session.take_departed().is_empty());
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
        assert!(observation.effect().contains_invalidation());
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
    assert!(response.effect.contains_invalidation());
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
                view::TextArea::new("").with_focus(session::Focus::text("document")),
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
    let composition: &composition::Composition = app
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
    assert!(composition.view().floating_panels().is_empty());

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

    assert_eq!(composition.view().floating_panels().len(), 1);
    assert_eq!(composition.view().floating_panels()[0].label_text(), None);
}
