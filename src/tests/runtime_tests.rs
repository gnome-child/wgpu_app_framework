use super::*;

fn successful_render_report() -> diagnostics::RenderReport {
    diagnostics::RenderReport::new(Duration::ZERO, Duration::ZERO, Instant::now())
}

#[test]
fn store_starts_clean_with_initial_revision() {
    let store = state::Store::new(EditorState::default());

    assert_eq!(store.revision(), state::Revision::initial());
    assert_eq!(store.saved_revision(), state::Revision::initial());
    assert!(!store.is_dirty());
    assert!(store.changes().is_empty());
}

#[test]
fn skipped_presentation_retains_visible_geometry_and_retries_the_same_epoch() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let size = geometry::Size::new(760, 660);
    let candidate = app
        .render_scene(window, size)
        .expect("first candidate should prepare");
    let epoch = candidate.epoch();

    app.finish_render_report(
        window,
        epoch,
        candidate.invalidation(),
        candidate.layout(),
        successful_render_report().with_presented(false),
    );

    assert!(app.presented_layout(window).is_none());
    assert_eq!(app.acknowledged_presentation_epoch(window), None);
    assert_eq!(
        app.session()
            .window(window)
            .expect("window should remain")
            .desired_presentation_epoch(),
        epoch,
        "retrying the same truth must not mint a freshness epoch"
    );
    assert!(app.session().window(window).unwrap().redraw_requested());

    let retry = app
        .render_scene(window, size)
        .expect("pending invalidation should prepare a retry");
    assert_eq!(retry.epoch(), epoch);
    app.finish_render_report(
        window,
        retry.epoch(),
        retry.invalidation(),
        retry.layout(),
        successful_render_report(),
    );

    assert_eq!(app.acknowledged_presentation_epoch(window), Some(epoch));
    assert_eq!(
        app.presented_layout(window).map(layout::Layout::size),
        Some(size)
    );
}

#[test]
fn older_successful_receipt_cannot_replace_newer_presented_geometry() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let older_size = geometry::Size::new(500, 400);
    let newer_size = geometry::Size::new(700, 600);
    let older = app
        .render_scene(window, older_size)
        .expect("older candidate should prepare");

    app.request_redraw(window);
    let newer = app
        .render_scene(window, newer_size)
        .expect("newer candidate should prepare");
    assert!(newer.epoch() > older.epoch());
    app.finish_render_report(
        window,
        newer.epoch(),
        newer.invalidation(),
        newer.layout(),
        successful_render_report(),
    );
    app.finish_render_report(
        window,
        older.epoch(),
        older.invalidation(),
        older.layout(),
        successful_render_report(),
    );

    assert_eq!(
        app.acknowledged_presentation_epoch(window),
        Some(newer.epoch())
    );
    assert_eq!(
        app.presented_layout(window).map(layout::Layout::size),
        Some(newer_size)
    );
}

#[test]
fn model_revision_desired_epoch_and_acknowledged_epoch_are_independent_facts() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let revision = app.revision();
    let desired = app
        .session()
        .window(window)
        .unwrap()
        .desired_presentation_epoch();

    app.request_redraw(window);
    let next_desired = app
        .session()
        .window(window)
        .unwrap()
        .desired_presentation_epoch();
    assert!(next_desired > desired);
    assert_eq!(app.revision(), revision);
    assert_eq!(app.acknowledged_presentation_epoch(window), None);

    let candidate = app
        .render_scene(window, geometry::Size::new(760, 660))
        .expect("candidate should prepare");
    app.finish_render_report(
        window,
        candidate.epoch(),
        candidate.invalidation(),
        candidate.layout(),
        successful_render_report(),
    );
    assert_eq!(app.revision(), revision);
    assert_eq!(
        app.acknowledged_presentation_epoch(window),
        Some(next_desired)
    );

    app.change(state::Reason::programmatic("model-only witness"), |state| {
        state.clicks += 1;
    });
    assert!(app.revision() > revision);
    assert_eq!(
        app.acknowledged_presentation_epoch(window),
        Some(next_desired),
        "model truth may advance while visible geometry remains older"
    );
}

#[test]
fn window_teardown_removes_acknowledged_geometry() {
    let mut app = control_gallery::app(control_gallery::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let candidate = app
        .render_scene(window, geometry::Size::new(760, 660))
        .expect("candidate should prepare");
    app.finish_render_report(
        window,
        candidate.epoch(),
        candidate.invalidation(),
        candidate.layout(),
        successful_render_report(),
    );
    assert_eq!(app.window_residues(window).presented_geometry, 1);

    app.invoke_focused(window, app.trigger::<session::CloseWindow>(()))
        .output
        .expect("window close should succeed");

    assert!(!app.session().contains(window));
    assert_eq!(app.window_residues(window).presented_geometry, 0);
    assert!(app.presented_layout(window).is_none());
}

#[test]
fn window_options_expose_explicit_framework_defaults() {
    let defaults = window::Options::default();

    assert_eq!(defaults.title(), window::DEFAULT_TITLE);
    assert_eq!(defaults.inner_size(), window::Options::default_inner_size());
    assert_eq!(
        defaults.canvas_color(),
        window::Options::default_canvas_color()
    );
}

#[test]
fn runtime_change_bumps_revision_once_and_marks_dirty() {
    let mut app = Runtime::new(EditorState::default());

    let change = app.change(state::Reason::event("edit"), |state| {
        state.event_count += 1;
    });

    assert_eq!(app.state().event_count, 1);
    assert_eq!(app.revision().get(), 1);
    assert!(app.is_dirty());
    assert_eq!(change.revision(), app.revision());
    assert_eq!(change.reason(), &state::Reason::Event("edit"));
    assert_eq!(app.store().changes(), &[change]);
}

#[test]
fn mark_saved_clears_dirty_without_changing_revision() {
    let mut app = Runtime::new(EditorState::default()).started(|cx| {
        cx.open_window(window::Options::new("Editor"));
    });
    app.start();
    let window = app.session().windows()[0].id();
    assert!(app.clear_redraw_request(window));

    app.change(state::Reason::programmatic("load fixture"), |state| {
        state.event_count += 1;
    });
    let revision = app.revision();
    assert!(app.session().windows()[0].redraw_requested());
    assert!(app.clear_redraw_request(window));

    app.mark_saved();

    assert_eq!(app.revision(), revision);
    assert!(!app.is_dirty());
    assert!(app.session().windows()[0].redraw_requested());
}

#[test]
fn runtime_undo_redo_restores_state_snapshots() {
    let mut app = Runtime::new(EditorState::default());

    app.change(state::Reason::programmatic("toggle wrap"), |state| {
        state.wrap_text = true;
    });

    let _: &Timeline<EditorState> = app.timeline();

    assert!(app.state().wrap_text);
    assert!(app.timeline().can_undo());
    assert_eq!(app.timeline().undo_depth(), 1);
    assert_eq!(app.revision().get(), 1);

    assert!(app.undo());

    assert!(!app.state().wrap_text);
    assert!(app.timeline().can_redo());
    assert_eq!(app.timeline().redo_depth(), 1);
    assert_eq!(app.revision().get(), 2);
    assert_eq!(app.store().changes()[1].reason(), &state::Reason::Undo);

    assert!(app.redo());

    assert!(app.state().wrap_text);
    assert!(app.timeline().can_undo());
    assert!(!app.timeline().can_redo());
    assert_eq!(app.revision().get(), 3);
    assert_eq!(app.store().changes()[2].reason(), &state::Reason::Redo);
}

#[test]
fn runtime_retention_bounds_change_log_and_undo_snapshots() {
    let mut app = Runtime::new(EditorState::default())
        .retention(runtime::Retention::new().changes(2).snapshots(2));

    for event_count in 1..=4 {
        app.change(state::Reason::programmatic("tick"), |state| {
            state.event_count = event_count;
        });
    }

    assert_eq!(app.state().event_count, 4);
    assert_eq!(app.revision().get(), 4);
    assert!(app.is_dirty());
    assert_eq!(app.store().change_limit(), 2);
    assert_eq!(app.timeline().snapshot_limit(), 2);
    assert_eq!(app.store().changes().len(), 2);
    assert_eq!(app.store().changes()[0].revision().get(), 3);
    assert_eq!(app.store().changes()[1].revision().get(), 4);
    assert_eq!(app.timeline().undo_depth(), 2);

    assert!(app.undo());
    assert!(app.undo());
    assert!(!app.undo());

    assert_eq!(app.state().event_count, 2);
    assert_eq!(app.revision().get(), 6);
    assert_eq!(app.store().changes().len(), 2);
    assert!(
        app.store()
            .changes()
            .iter()
            .all(|change| change.reason() == &state::Reason::Undo)
    );
    assert_eq!(app.timeline().redo_depth(), 2);

    assert!(app.redo());
    assert!(app.redo());
    assert!(!app.redo());

    assert_eq!(app.state().event_count, 4);
    assert_eq!(app.revision().get(), 8);
    assert_eq!(app.timeline().undo_depth(), 2);
}

#[test]
fn runtime_retention_can_drop_all_diagnostic_history() {
    let mut app = Runtime::new(EditorState::default())
        .retention(runtime::Retention::new().changes(0).snapshots(0));

    app.change(state::Reason::programmatic("tick"), |state| {
        state.event_count = 1;
    });

    assert_eq!(app.state().event_count, 1);
    assert_eq!(app.revision().get(), 1);
    assert!(app.is_dirty());
    assert!(app.store().changes().is_empty());
    assert_eq!(app.timeline().undo_depth(), 0);
    assert!(!app.undo());
}

#[test]
fn ignored_history_command_does_not_snapshot_model() {
    let state = CloneCountState::default();
    let clone_count = state.count();
    let mut app = Runtime::new(state)
        .commands(|commands| {
            commands.register::<IgnoredPing>(command::Spec::new("Ignored Ping"));
        })
        .responders(|responders| {
            responders.app().target::<IgnoredPing>();
        });

    let response = app.invoke(app.trigger::<IgnoredPing>(()));

    response.output.expect("ignored command should resolve");
    assert_eq!(clone_count.get(), 0);
    assert_eq!(app.revision(), state::Revision::initial());
    assert!(app.store().changes().is_empty());
    assert_eq!(app.timeline().undo_depth(), 0);
}

#[test]
fn ignored_history_changed_response_advances_revision_without_snapshot() {
    let state = CloneCountState::default();
    let clone_count = state.count();
    let mut app = Runtime::new(state)
        .commands(|commands| {
            commands.register::<IgnoredMutation>(command::Spec::new("Ignored Mutation"));
        })
        .responders(|responders| {
            responders.app().target::<IgnoredMutation>();
        });

    app.invoke(app.trigger::<IgnoredMutation>(()))
        .output
        .expect("ignored mutation should resolve");

    assert_eq!(app.state().value, 1);
    assert_eq!(app.revision().get(), 1);
    assert!(app.is_dirty());
    assert_eq!(app.store().changes().len(), 1);
    assert_eq!(
        app.store().changes()[0].reason(),
        &state::Reason::command(IgnoredMutation::NAME)
    );
    assert_eq!(app.timeline().undo_depth(), 0);
    assert_eq!(clone_count.get(), 0);
}

#[test]
fn committed_timeline_commands_skip_unused_transaction_snapshot() {
    let state = CloneCountState::default();
    let clone_count = state.count();
    let mut app = Runtime::new(state);

    app.change(state::Reason::programmatic("seed"), |state| {
        state.value = 1;
    });
    clone_count.set(0);

    assert!(app.undo());

    assert_eq!(app.state().value, 0);
    assert_eq!(app.revision().get(), 2);
    assert_eq!(clone_count.get(), 1);
    assert_eq!(app.timeline().redo_depth(), 1);

    clone_count.set(0);
    assert!(app.redo());

    assert_eq!(app.state().value, 1);
    assert_eq!(app.revision().get(), 3);
    assert_eq!(clone_count.get(), 1);
    assert_eq!(app.timeline().undo_depth(), 1);
}

#[test]
fn committed_history_user_override_advances_revision_without_snapshot() {
    let state = CloneCountState::default();
    let clone_count = state.count();
    let mut app = Runtime::new(state).responders(|responders| {
        responders.app().target::<timeline::Undo>();
    });

    app.invoke(app.trigger::<timeline::Undo>(()))
        .output
        .expect("user undo override should resolve");

    assert_eq!(app.state().value, 1);
    assert_eq!(app.revision().get(), 1);
    assert!(app.is_dirty());
    assert_eq!(app.store().changes().len(), 1);
    assert_eq!(
        app.store().changes()[0].reason(),
        &state::Reason::command(timeline::Undo::NAME)
    );
    assert_eq!(app.timeline().undo_depth(), 0);
    assert_eq!(app.timeline().redo_depth(), 0);
    assert_eq!(clone_count.get(), 0);
}

#[test]
fn retained_transaction_snapshot_skips_clone_for_unchanged_automatic_commands() {
    let state = CloneCountState::default();
    let clone_count = state.count();
    let mut app = Runtime::new(state)
        .commands(|commands| {
            commands
                .register::<OpenNamed>(command::Spec::new("Open Named"))
                .register::<Ping>(command::Spec::new("Ping"));
        })
        .responders(|responders| {
            responders.app().target::<OpenNamed>().target::<Ping>();
        });

    app.invoke(app.trigger::<OpenNamed>("seed".to_owned()))
        .output
        .expect("changed command should resolve");
    clone_count.set(0);

    app.invoke(app.trigger::<Ping>(()))
        .output
        .expect("unchanged command should resolve");
    app.invoke(app.trigger::<Ping>(()))
        .output
        .expect("second unchanged command should resolve");

    assert_eq!(clone_count.get(), 0);
    assert_eq!(app.state().value, 4);
    assert_eq!(app.revision().get(), 1);
    assert_eq!(app.timeline().undo_depth(), 1);
}

#[test]
fn runtime_snapshot_restore_replaces_model_session_and_marks_clean() {
    let mut app = Runtime::new(EditorState::default()).started(|cx| {
        cx.open_window(window::Options::new("First"));
        cx.open_window(window::Options::new("Second"));
    });

    app.start();

    let first = app.session().windows()[0].id();
    let second = app.session().windows()[1].id();
    assert!(app.focus(second, session::Focus::text("document")));
    app.change(state::Reason::programmatic("seed"), |state| {
        state.event_count = 7;
        state.wrap_text = true;
    });
    app.mark_saved();

    let snapshot = app.snapshot();

    assert_eq!(snapshot.state().model().event_count, 7);
    assert_eq!(snapshot.session().windows().len(), 2);
    assert_eq!(snapshot.session().windows()[1].title(), "Second");
    assert_eq!(
        snapshot.session().windows()[1].focus(),
        Some(session::Focus::text("document"))
    );

    app.change(state::Reason::programmatic("dirty"), |state| {
        state.event_count = 99;
        state.wrap_text = false;
    });
    app.invoke(app.trigger::<session::CloseWindow>(()))
        .output
        .expect("close window should resolve");
    assert!(!app.session().contains(first));
    assert!(app.session().contains(second));
    app.diagnostics_mut(second)
        .expect("remaining window diagnostics should exist")
        .frame
        .full_redraws = 99;
    assert!(app.is_dirty());

    let change = app.restore(snapshot);

    assert_eq!(change.reason(), &state::Reason::Restore);
    assert_eq!(app.state().event_count, 7);
    assert!(app.state().wrap_text);
    assert_eq!(app.session().windows().len(), 2);
    assert_eq!(app.session().windows()[0].id(), first);
    assert_eq!(app.session().windows()[0].title(), "First");
    assert_eq!(app.session().windows()[1].id(), second);
    assert_eq!(
        app.session().focused(second),
        Some(session::Focus::text("document"))
    );
    assert!(!app.is_dirty());
    assert_eq!(app.store().saved_revision(), app.revision());
    assert!(!app.timeline().can_undo());
    assert_eq!(
        app.diagnostics(second)
            .expect("restored window should have diagnostics")
            .frame
            .full_redraws,
        0
    );
}

#[test]
fn runtime_snapshot_restore_clears_transient_animation_schedule() {
    let mut app = text_editor::app(text_editor::State::default());
    app.start();

    let window = app.session().windows()[0].id();
    let snapshot = app.snapshot();
    let presented = app.present(window).expect("window should have a view");
    let focus = presented.text_areas()[0]
        .focus()
        .expect("text area should declare focus");
    app.handle_input(window, Input::focus(focus))
        .expect("focus should be handled");
    let target = interaction::Target::text_area(focus);
    let epoch = app
        .session()
        .interaction(window)
        .and_then(|interaction| interaction.text_input().caret_epoch_for(&target))
        .expect("focused text area should store a caret epoch");
    app.render_scene_at(window, geometry::Size::new(480, 180), epoch)
        .expect("focused window should render");
    assert_ne!(
        app.animation_schedule(),
        crate::animation::Schedule::Idle,
        "focused caret should seed transient animation state"
    );

    app.restore(snapshot);

    assert_eq!(
        app.animation_schedule(),
        crate::animation::Schedule::Idle,
        "restoring an unfocused snapshot must not retain the later caret schedule"
    );
    assert!(
        app.session()
            .window(window)
            .is_some_and(session::Window::redraw_requested),
        "restore should redraw after dropping cached presentation state"
    );
}

#[test]
fn runtime_save_and_load_use_app_defined_persistence() {
    let mut app = Runtime::new(EditorState::default()).started(|cx| {
        cx.open_window(window::Options::new("Editor"));
    });

    app.start();

    let window = app.session().windows()[0].id();
    assert!(app.focus(window, session::Focus::text("document")));
    app.change(state::Reason::programmatic("prepare snapshot"), |state| {
        state.event_count = 42;
        state.wrap_text = true;
    });
    assert!(app.is_dirty());

    let mut persistence = EditorPersistence::default();
    let saved_revision = app
        .save(&mut persistence)
        .expect("persistence save should succeed");

    assert_eq!(saved_revision, app.revision());
    assert!(!app.is_dirty());
    assert_eq!(app.store().saved_revision(), app.revision());
    assert_eq!(
        persistence.data.as_deref(),
        Some("42|true|0:Editor:document")
    );

    app.change(state::Reason::programmatic("dirty after save"), |state| {
        state.event_count = 1;
        state.wrap_text = false;
    });
    app.invoke(app.trigger::<session::CloseWindow>(()))
        .output
        .expect("close window should resolve");
    assert!(app.is_dirty());
    assert!(app.session().windows().is_empty());

    let change = app
        .load(&mut persistence)
        .expect("persistence load should succeed");

    assert_eq!(change.reason(), &state::Reason::Load);
    assert_eq!(app.state().event_count, 42);
    assert!(app.state().wrap_text);
    assert_eq!(app.session().windows().len(), 1);
    let restored = app.session().windows()[0].id();
    assert_eq!(restored, window);
    assert_eq!(app.session().windows()[0].title(), "Editor");
    assert_eq!(
        app.session().focused(restored),
        Some(session::Focus::text("document"))
    );
    assert!(!app.is_dirty());
    assert_eq!(app.store().saved_revision(), app.revision());
    assert!(!app.timeline().can_undo());
}

#[test]
fn failed_runtime_save_does_not_mark_state_clean() {
    let mut app = Runtime::new(EditorState::default());
    app.change(state::Reason::programmatic("dirty"), |state| {
        state.event_count = 1;
    });
    let mut persistence = EditorPersistence {
        fail_save: true,
        ..EditorPersistence::default()
    };

    let error = app
        .save(&mut persistence)
        .expect_err("failed persistence save should surface");

    assert_eq!(error, "save failed");
    assert!(app.is_dirty());
    assert_eq!(app.store().saved_revision(), state::Revision::initial());
    assert_eq!(persistence.data, None);
}

#[test]
fn timeline_commands_are_framework_registered_and_invoke_runtime_history() {
    let mut app = Runtime::new(EditorState::default());
    let undo = app.trigger::<timeline::Undo>(());
    let redo = app.trigger::<timeline::Redo>(());

    assert!(!app.state_for(&undo).is_enabled());
    assert_eq!(app.state_for(&undo).label.as_deref(), Some("Undo"));
    assert_eq!(
        app.state_for(&undo)
            .shortcut
            .map(|shortcut| shortcut.as_str()),
        Some("Standard::Undo")
    );
    assert!(!app.state_for(&redo).is_enabled());

    app.change(state::Reason::programmatic("toggle wrap"), |state| {
        state.wrap_text = true;
    });

    assert!(app.state_for(&undo).is_enabled());
    assert!(!app.state_for(&redo).is_enabled());

    app.invoke(undo).output.expect("undo should resolve");

    assert!(!app.state().wrap_text);
    let undo = app.trigger::<timeline::Undo>(());
    assert!(!app.state_for(&undo).is_enabled());
    assert!(app.state_for(&redo).is_enabled());
    assert_eq!(app.store().changes()[1].reason(), &state::Reason::Undo);

    app.invoke(redo).output.expect("redo should resolve");

    assert!(app.state().wrap_text);
    assert_eq!(app.store().changes()[2].reason(), &state::Reason::Redo);
}

#[test]
fn started_callback_runs_once_and_mutates_through_runtime_context() {
    let mut app = Runtime::new(EditorState::default()).started(|cx| {
        cx.change(state::Reason::programmatic("started"), |state| {
            state.event_count += 1;
        });
    });

    app.start();
    app.start();

    assert_eq!(app.state().event_count, 1);
    assert_eq!(app.revision().get(), 1);
    assert!(app.is_dirty());
}

#[test]
fn event_and_view_callbacks_replace_user_owned_application_object() {
    let mut app = Runtime::new(EditorState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Editor"));
        })
        .event(|cx, event: EditorEvent| match event {
            EditorEvent::Edited => {
                cx.change(state::Reason::event("edited"), |state| {
                    state.event_count += 1;
                    state.document.dirty = true;
                });
            }
            EditorEvent::Saved => {
                cx.change(state::Reason::event("saved"), |state| {
                    state.document.dirty = false;
                });
                cx.mark_saved();
            }
        })
        .view(|state, _window| {
            format!(
                "events={} dirty={}",
                state.event_count, state.document.dirty
            )
        });

    app.start();

    let window = app.session().windows()[0].id();
    assert_eq!(app.render(window).as_deref(), Some("events=0 dirty=false"));
    assert!(app.clear_redraw_request(window));

    app.emit(EditorEvent::Edited);

    assert_eq!(app.render(window).as_deref(), Some("events=1 dirty=true"));
    assert_eq!(app.revision().get(), 1);
    assert!(app.is_dirty());
    assert!(app.session().windows()[0].redraw_requested());
    assert!(app.clear_redraw_request(window));

    app.emit(EditorEvent::Saved);

    assert_eq!(app.render_all()[0].1, "events=1 dirty=false");
    assert_eq!(app.revision().get(), 2);
    assert!(!app.is_dirty());
    assert!(app.session().windows()[0].redraw_requested());
}

#[test]
fn lifecycle_callbacks_can_spawn_framework_owned_tasks() {
    let mut app = Runtime::new(EditorState::default())
        .started(|cx| {
            assert!(cx.spawn(Task::ready(EditorEvent::Edited)).is_some());
        })
        .event(|cx, event: EditorEvent| match event {
            EditorEvent::Edited => {
                cx.change(state::Reason::event("edited"), |state| {
                    state.event_count += 1;
                    state.document.dirty = true;
                });
                assert!(cx.spawn(Task::ready(EditorEvent::Saved)).is_some());
            }
            EditorEvent::Saved => {
                cx.change(state::Reason::event("saved"), |state| {
                    state.document.dirty = false;
                });
            }
        });

    app.start();

    assert_eq!(app.pending_tasks(), 1);

    let edited = app.run_next_task().expect("edited task should run");

    assert_eq!(edited.status(), task::Status::Completed);
    assert!(edited.changed_state());
    assert_eq!(app.state().event_count, 1);
    assert!(app.state().document.dirty);
    assert_eq!(app.pending_tasks(), 1);
    assert_eq!(app.revision().get(), 1);

    let saved = app.run_next_task().expect("saved task should run");

    assert_eq!(saved.status(), task::Status::Completed);
    assert!(saved.changed_state());
    assert_eq!(app.state().event_count, 1);
    assert!(!app.state().document.dirty);
    assert_eq!(app.pending_tasks(), 0);
    assert_eq!(app.revision().get(), 2);
}

#[test]
fn spawned_tasks_have_status_and_complete_through_runtime_events() {
    let mut app = Runtime::new(EditorState::default())
        .commands(|commands| {
            commands.register::<SpawnEditorEvent>(command::Spec::new("Spawn Event"));
        })
        .responders(|responders| {
            responders.app().target::<SpawnEditorEvent>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Tasks"));
        })
        .event(|cx, event: EditorEvent| match event {
            EditorEvent::Edited => {
                cx.change(state::Reason::event("edited"), |state| {
                    state.event_count += 1;
                });
            }
            EditorEvent::Saved => {
                cx.change(state::Reason::event("saved"), |state| {
                    state.document.dirty = false;
                });
            }
        });

    app.start();
    let window = app.session().windows()[0].id();
    assert!(app.clear_redraw_request(window));

    let task = app
        .invoke(app.trigger::<SpawnEditorEvent>(EditorEvent::Edited))
        .output
        .expect("spawn command should resolve")
        .expect("task should be accepted");

    assert_eq!(app.pending_tasks(), 1);
    assert_eq!(app.pending_task_completions(), 0);
    assert_eq!(app.task_status(task), Some(task::Status::Pending));

    let outcome = app.run_next_task().expect("task should run");

    assert_eq!(app.pending_tasks(), 0);
    assert_eq!(app.pending_task_completions(), 0);
    assert_eq!(outcome.id(), task);
    assert_eq!(outcome.status(), task::Status::Completed);
    assert!(outcome.changed_state());
    assert_eq!(app.task_status(task), Some(task::Status::Completed));
    assert_eq!(app.state().event_count, 1);
    assert_eq!(app.revision().get(), 1);
    assert!(app.session().windows()[0].redraw_requested());
}

#[test]
fn completed_tasks_are_dispatched_as_framework_owned_events() {
    let mut app = Runtime::new(EditorState::default())
        .commands(|commands| {
            commands.register::<SpawnEditorEvent>(command::Spec::new("Spawn Event"));
        })
        .responders(|responders| {
            responders.app().target::<SpawnEditorEvent>();
        })
        .event(|cx, event: EditorEvent| {
            if let EditorEvent::Edited = event {
                cx.change(state::Reason::event("edited"), |state| {
                    state.event_count += 1;
                });
            }
        });

    let task = app
        .invoke(app.trigger::<SpawnEditorEvent>(EditorEvent::Edited))
        .output
        .expect("spawn command should resolve")
        .expect("task should be accepted");

    assert_eq!(app.complete_next_task(), Some(task));
    assert_eq!(app.pending_tasks(), 0);
    assert_eq!(app.pending_task_completions(), 1);
    assert_eq!(app.task_status(task), Some(task::Status::Completed));
    assert_eq!(app.state().event_count, 0);
    assert_eq!(app.revision(), state::Revision::initial());

    let outcome = app
        .dispatch_next_task_completion()
        .expect("completion should dispatch");

    assert_eq!(outcome.id(), task);
    assert!(outcome.changed_state());
    assert_eq!(app.pending_task_completions(), 0);
    assert_eq!(app.state().event_count, 1);
    assert_eq!(app.revision().get(), 1);
}

#[test]
fn completed_task_outcome_reports_unchanged_events() {
    let mut app = Runtime::new(EditorState::default()).started(|cx| {
        assert!(cx.spawn(Task::ready(())).is_some());
    });

    app.start();

    let outcome = app.run_next_task().expect("task should run");

    assert_eq!(outcome.status(), task::Status::Completed);
    assert!(!outcome.changed_state());
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn future_tasks_complete_through_runtime_events() {
    let mut app = Runtime::new(EditorState::default())
        .started(|cx| {
            assert!(
                cx.spawn(Task::future(async { EditorEvent::Edited }))
                    .is_some()
            );
        })
        .event(|cx, event: EditorEvent| {
            if let EditorEvent::Edited = event {
                cx.change(state::Reason::event("edited"), |state| {
                    state.event_count += 1;
                });
            }
        });

    app.start();

    assert!(app.complete_next_task().is_some());
    assert_eq!(app.pending_tasks(), 0);
    assert_eq!(app.pending_task_completions(), 1);
    assert_eq!(app.state().event_count, 0);

    let outcome = app
        .dispatch_next_task_completion()
        .expect("future completion should dispatch");

    assert!(outcome.changed_state());
    assert_eq!(app.state().event_count, 1);
}

#[test]
fn task_executor_runs_future_work_off_the_calling_thread() {
    let calling_thread = std::thread::current().id();
    let (sender, receiver) = std::sync::mpsc::channel();
    let executor = task::Executor::new();
    let work = Task::future(async move {
        (
            std::thread::current().id(),
            std::thread::current().name().map(str::to_owned),
        )
    });

    assert!(executor.spawn(move || {
        sender
            .send(work.run())
            .expect("worker result receiver should remain connected");
    }));
    let (worker_thread, worker_name) = receiver
        .recv_timeout(std::time::Duration::from_secs(2))
        .expect("worker should complete the future");

    assert_ne!(worker_thread, calling_thread);
    assert!(
        worker_name.is_some_and(|name| name.starts_with("wgpu_l3-worker-")),
        "task work should run on a named framework worker"
    );
}

#[test]
fn cancellation_discards_an_in_flight_task_completion() {
    let mut app = Runtime::new(EditorState::default())
        .commands(|commands| {
            commands.register::<SpawnEditorEvent>(command::Spec::new("Spawn Event"));
        })
        .responders(|responders| {
            responders.app().target::<SpawnEditorEvent>();
        })
        .event(|cx, event: EditorEvent| {
            if let EditorEvent::Edited = event {
                cx.change(state::Reason::event("edited"), |state| {
                    state.event_count += 1;
                });
            }
        });

    let id = app
        .invoke(app.trigger::<SpawnEditorEvent>(EditorEvent::Edited))
        .output
        .expect("spawn command should resolve")
        .expect("task should be accepted");
    let (taken_id, work) = app.take_next_task().expect("worker should take the task");
    assert_eq!(taken_id, id);
    assert_eq!(app.pending_tasks(), 1, "in-flight work remains pending");

    assert!(app.cancel_task(id));
    assert!(!app.accept_task_completion(id, work.run()));

    assert_eq!(app.task_status(id), Some(task::Status::Canceled));
    assert_eq!(app.pending_tasks(), 0);
    assert_eq!(app.pending_task_completions(), 0);
    assert_eq!(app.state().event_count, 0);
}

#[test]
fn restore_discards_an_in_flight_task_completion() {
    let mut app = Runtime::new(EditorState::default())
        .commands(|commands| {
            commands.register::<SpawnEditorEvent>(command::Spec::new("Spawn Event"));
        })
        .responders(|responders| {
            responders.app().target::<SpawnEditorEvent>();
        })
        .event(|_, _: EditorEvent| {});
    let snapshot = app.snapshot();
    let id = app
        .invoke(app.trigger::<SpawnEditorEvent>(EditorEvent::Edited))
        .output
        .expect("spawn command should resolve")
        .expect("task should be accepted");
    let (taken_id, work) = app.take_next_task().expect("worker should take the task");
    assert_eq!(taken_id, id);

    app.restore(snapshot);
    assert!(!app.accept_task_completion(id, work.run()));

    assert_eq!(app.task_status(id), Some(task::Status::Canceled));
    assert_eq!(app.pending_tasks(), 0);
    assert_eq!(app.pending_task_completions(), 0);
}

#[test]
fn pending_tasks_can_be_canceled_before_they_emit_events() {
    let mut app = Runtime::new(EditorState::default())
        .commands(|commands| {
            commands.register::<SpawnEditorEvent>(command::Spec::new("Spawn Event"));
        })
        .responders(|responders| {
            responders.app().target::<SpawnEditorEvent>();
        })
        .event(|cx, event: EditorEvent| {
            if let EditorEvent::Edited = event {
                cx.change(state::Reason::event("edited"), |state| {
                    state.event_count += 1;
                });
            }
        });

    let task = app
        .invoke(app.trigger::<SpawnEditorEvent>(EditorEvent::Edited))
        .output
        .expect("spawn command should resolve")
        .expect("task should be accepted");

    assert!(app.cancel_task(task));
    assert_eq!(app.task_status(task), Some(task::Status::Canceled));
    assert_eq!(app.pending_tasks(), 0);
    assert!(!app.cancel_task(task));
    assert!(app.run_next_task().is_none());
    assert_eq!(app.state().event_count, 0);
    assert_eq!(app.revision(), state::Revision::initial());
}

#[test]
fn restore_cancels_pending_task_ids() {
    let mut app = Runtime::new(EditorState::default())
        .commands(|commands| {
            commands.register::<SpawnEditorEvent>(command::Spec::new("Spawn Event"));
        })
        .responders(|responders| {
            responders.app().target::<SpawnEditorEvent>();
        })
        .event(|cx, event: EditorEvent| {
            if let EditorEvent::Edited = event {
                cx.change(state::Reason::event("edited"), |state| {
                    state.event_count += 1;
                });
            }
        });
    let snapshot = app.snapshot();

    let task = app
        .invoke(app.trigger::<SpawnEditorEvent>(EditorEvent::Edited))
        .output
        .expect("spawn command should resolve")
        .expect("task should be accepted");

    app.restore(snapshot);

    assert_eq!(app.task_status(task), Some(task::Status::Canceled));
    assert_eq!(app.pending_tasks(), 0);
    assert!(app.run_next_task().is_none());
    assert_eq!(app.state().event_count, 0);
}

#[test]
fn restore_discards_completed_task_events_that_were_not_dispatched() {
    let mut app = Runtime::new(EditorState::default())
        .commands(|commands| {
            commands.register::<SpawnEditorEvent>(command::Spec::new("Spawn Event"));
        })
        .responders(|responders| {
            responders.app().target::<SpawnEditorEvent>();
        })
        .event(|cx, event: EditorEvent| {
            if let EditorEvent::Edited = event {
                cx.change(state::Reason::event("edited"), |state| {
                    state.event_count += 1;
                });
            }
        });
    let snapshot = app.snapshot();

    let task = app
        .invoke(app.trigger::<SpawnEditorEvent>(EditorEvent::Edited))
        .output
        .expect("spawn command should resolve")
        .expect("task should be accepted");

    assert_eq!(app.complete_next_task(), Some(task));
    assert_eq!(app.pending_tasks(), 0);
    assert_eq!(app.pending_task_completions(), 1);

    app.restore(snapshot);

    assert_eq!(app.task_status(task), Some(task::Status::Completed));
    assert_eq!(app.pending_task_completions(), 0);
    assert!(app.dispatch_next_task_completion().is_none());
    assert_eq!(app.state().event_count, 0);
}

#[test]
fn command_context_source_tracks_invocation_origin() {
    let mut app = Runtime::new(SourceState::default())
        .commands(|commands| {
            commands.register::<RecordSource>(command::Spec::new("Record").shortcut("Ctrl+R"));
        })
        .responders(|responders| {
            responders.app().target::<RecordSource>();
        })
        .started(|cx| {
            cx.open_window(window::Options::new("Sources"));
        })
        .view(|_, _| {
            View::new(
                view::Node::root()
                    .child(
                        view::Node::menu("menu.actions", "Actions")
                            .child(view::Node::menu_bound::<RecordSource>()),
                    )
                    .child(view::Node::bound::<RecordSource>()),
            )
        });

    app.start();

    let window = app.session().windows()[0].id();
    app.invoke(app.trigger::<RecordSource>(()))
        .output
        .expect("programmatic command should resolve");

    let projected = app.present(window).expect("window should have a view");
    let menu_command = projected
        .bindings()
        .into_iter()
        .find(|command| command.source() == context::Source::Menu)
        .expect("menu command should be in the view");

    app.activate_in(window, menu_command)
        .expect("menu command should activate");

    let projected = app
        .present(window)
        .expect("window should still have a view");
    let button_command = projected
        .bindings()
        .into_iter()
        .find(|command| command.source() == context::Source::Button)
        .expect("button command should be in the view");

    app.activate_in(window, button_command)
        .expect("button command should activate");
    app.handle_input(window, Input::shortcut("Ctrl+R"))
        .expect("shortcut command should activate");

    assert_eq!(
        app.state().sources,
        vec![
            context::Source::Programmatic,
            context::Source::Menu,
            context::Source::Button,
            context::Source::Shortcut,
        ]
    );
}
