use super::*;

#[test]
fn present_pending_uses_revision_staleness_across_windows_after_command_undo() {
    let mut app = Runtime::new(EditorState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("First"));
            cx.open_window(window::Options::new("Second"));
        })
        .view(|state, _| {
            View::new(
                view::Node::root()
                    .child(view::Node::label(format!("events {}", state.event_count)))
                    .child(view::Node::bound::<timeline::Undo>()),
            )
        });

    app.start();

    let first = app.session().windows()[0].id();
    let second = app.session().windows()[1].id();
    assert_eq!(app.present_pending().len(), 2);
    assert!(app.present_pending().is_empty());

    app.change(state::Reason::programmatic("edit"), |state| {
        state.event_count = 1;
    });

    let edited = app.present_pending();
    assert_eq!(edited.len(), 2);
    assert!(
        edited
            .iter()
            .all(|presentation| { presentation.view().labels().contains(&"events 1") })
    );
    assert!(app.present_pending().is_empty());

    let undo = edited
        .iter()
        .find(|presentation| presentation.window() == first)
        .expect("first window should have presented")
        .view()
        .binding::<timeline::Undo>()
        .expect("undo command should be in the view")
        .clone();

    assert!(undo.is_enabled());

    let effect = app
        .activate_in(first, &undo)
        .expect("undo command should activate");

    assert_eq!(effect, response::Effect::Repaint);
    assert_eq!(app.state().event_count, 0);
    assert_eq!(app.revision().get(), 2);
    assert!(
        app.session()
            .window(first)
            .expect("first window should exist")
            .redraw_requested()
    );
    assert!(
        !app.session()
            .window(second)
            .expect("second window should exist")
            .redraw_requested()
    );

    let pending = app.present_pending();
    let pending_windows = pending
        .iter()
        .map(view::Presentation::window)
        .collect::<Vec<_>>();

    assert_eq!(pending.len(), 2);
    assert!(pending_windows.contains(&first));
    assert!(pending_windows.contains(&second));
    assert!(
        pending
            .iter()
            .all(|presentation| { presentation.view().labels().contains(&"events 0") })
    );
    assert!(app.present_pending().is_empty());
}

#[test]
fn text_editor_host_drains_presentations_and_platform_requests() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let work = app.drain();

    assert_eq!(work.presentations().len(), 1);
    assert!(work.requests().is_empty());
    assert_eq!(work.pending_tasks(), 0);
    assert_eq!(work.task_completions(), 0);

    let open = work.presentations()[0]
        .view()
        .binding::<document::OpenFile>()
        .expect("open command should be in the presented view")
        .action();

    app.handle_view(window, open)
        .expect("open action should be handled");

    let work = app.drain();

    assert_eq!(work.presentations().len(), 1);
    assert_eq!(work.requests().len(), 1);
    assert_eq!(work.pending_tasks(), 0);
    assert_eq!(work.task_completions(), 0);
    assert_eq!(work.requests()[0].window(), window);
    assert_eq!(
        work.requests()[0].kind(),
        session::RequestKind::FileDialog(session::FileDialog::Open)
    );
    assert!(!app.session().windows()[0].redraw_requested());
    assert_eq!(
        app.session().file_dialog(window),
        Some(session::FileDialog::Open)
    );
}

#[test]
fn text_editor_host_work_reports_pending_tasks() {
    let path = temp_text_path("scratch_host_work_pending_task.txt");
    let mut app = text_editor::app(text_editor::State::default());

    app.start();
    let _ = app.drain();

    let response = app.invoke(app.trigger::<document::SaveToPath>(path.clone()));

    assert_eq!(response.output.expect("save should resolve"), Ok(()));
    assert_eq!(app.pending_tasks(), 1);
    assert_eq!(app.pending_task_completions(), 0);

    let work = app.drain();

    assert_eq!(work.pending_tasks(), 1);
    assert_eq!(work.task_completions(), 0);
    assert!(!work.is_empty());

    let scene_work = app.drain_scenes(|_| geometry::Size::new(800, 600));

    assert!(scene_work.presentations().is_empty());
    assert_eq!(scene_work.pending_tasks(), 1);
    assert_eq!(scene_work.task_completions(), 0);
    assert!(!scene_work.is_empty());

    assert!(app.complete_next_task().is_some());
    assert_eq!(app.pending_tasks(), 0);
    assert_eq!(app.pending_task_completions(), 1);

    let work = app.drain();

    assert_eq!(work.pending_tasks(), 0);
    assert_eq!(work.task_completions(), 1);
    assert!(!work.is_empty());

    let outcome = app
        .dispatch_next_task_completion()
        .expect("save completion should dispatch");

    assert!(outcome.changed_state());
    assert_eq!(app.pending_tasks(), 0);
    assert_eq!(app.pending_task_completions(), 0);

    let work = app.drain();

    assert_eq!(work.pending_tasks(), 0);
    assert_eq!(work.task_completions(), 0);

    let _ = std::fs::remove_file(path);
}

#[test]
fn shell_drains_text_editor_to_scene_work_and_requests() {
    let mut shell = Shell::new(text_editor::app(text_editor::State::default()));

    shell.start();

    let window = shell.runtime().session().windows()[0].id();
    assert!(shell.set_window_size(window, geometry::Size::new(640, 480)));

    let work = shell.drain();

    assert_eq!(work.presentations().len(), 1);
    assert_eq!(work.presentations()[0].window(), window);
    assert_eq!(
        work.presentations()[0].layout().size(),
        geometry::Size::new(640, 480)
    );
    assert!(work.requests().is_empty());
    assert!(
        work.presentations()[0]
            .scene()
            .texts()
            .iter()
            .any(|text| text.value() == "File")
    );

    shell
        .handle_input(window, Input::shortcut("Ctrl+O"))
        .expect("open shortcut should be handled");
    let work = shell.drain();

    assert_eq!(work.requests().len(), 1);
    assert_eq!(work.requests()[0].window(), window);
    assert_eq!(
        work.requests()[0].kind(),
        session::RequestKind::FileDialog(session::FileDialog::Open)
    );
}

#[test]
fn shell_routes_coordinate_input_and_task_completions() {
    let path = temp_text_path("scratch_shell_task_completion.txt");
    let mut shell = Shell::new(text_editor::app(text_editor::State::default()));

    shell.start();

    let window = shell.runtime().session().windows()[0].id();
    let _ = shell.drain();

    shell
        .pointer_down(window, geometry::Point::new(10, 10))
        .expect("pointer down should be routed");
    shell
        .pointer_up(window, geometry::Point::new(10, 10))
        .expect("pointer up should be routed");

    assert_eq!(
        shell
            .runtime()
            .session()
            .interaction(window)
            .and_then(|interaction| interaction.open_menu())
            .map(|menu| menu.label()),
        Some("File")
    );

    let trigger = shell
        .runtime()
        .trigger::<document::SaveToPath>(path.clone());
    let response = shell.runtime_mut().invoke(trigger);

    assert_eq!(response.output.expect("save should resolve"), Ok(()));
    assert_eq!(shell.runtime().pending_tasks(), 1);
    assert_eq!(shell.drain().pending_tasks(), 1);

    assert!(shell.complete_next_task().is_some());
    let work = shell.drain();

    assert_eq!(work.pending_tasks(), 0);
    assert_eq!(work.task_completions(), 1);

    let outcome = shell
        .dispatch_next_task_completion()
        .expect("save completion should dispatch");

    assert!(outcome.changed_state());
    assert_eq!(shell.runtime().pending_task_completions(), 0);
    assert!(path.exists());

    let _ = std::fs::remove_file(path);
}

#[test]
fn text_editor_shell_entrypoint_runs_host_loop_step() {
    let path = temp_text_path("text_editor_shell_step.txt");
    let mut shell = text_editor::shell(text_editor::State::default());

    shell.start();

    let window = shell.runtime().session().windows()[0].id();
    let initial = shell.drain();

    assert_eq!(initial.presentations().len(), 1);
    assert_eq!(initial.presentations()[0].window(), window);

    let trigger = shell
        .runtime()
        .trigger::<document::SaveToPath>(path.clone());
    let response = shell.runtime_mut().invoke(trigger);

    assert_eq!(response.output.expect("save should resolve"), Ok(()));
    assert_eq!(shell.runtime().pending_tasks(), 1);

    let work = shell.step();

    assert_eq!(work.pending_tasks(), 0);
    assert_eq!(work.task_completions(), 0);
    assert_eq!(work.presentations().len(), 1);
    assert!(path.exists());
    assert_eq!(
        shell.runtime().state().last_status,
        format!("saved {}", text_editor::compact_path(&path))
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn text_editor_shell_handles_file_dialog_selection() {
    let path = temp_text_path("text_editor_shell_open.txt");
    std::fs::write(&path, "opened through shell").expect("fixture file should be writable");
    let mut shell = text_editor::shell(text_editor::State::default());

    shell.start();

    let window = shell.runtime().session().windows()[0].id();
    let _ = shell.drain();

    shell
        .handle_input(window, Input::shortcut("Ctrl+O"))
        .expect("open shortcut should be handled");
    let work = shell.drain();

    assert_eq!(work.requests().len(), 1);
    assert_eq!(
        work.requests()[0].kind(),
        session::RequestKind::FileDialog(session::FileDialog::Open)
    );

    let outcome = shell
        .file_path_selected(window, Some(path.clone()))
        .expect("selected path should be handled");

    assert!(outcome.is_handled());
    assert!(outcome.changed_state());
    assert_eq!(
        shell.runtime().state().document.text(),
        "opened through shell"
    );
    assert_eq!(
        shell.runtime().state().document.path(),
        Some(path.as_path())
    );
    assert_eq!(
        shell.runtime().state().last_status,
        format!("opened {}", text_editor::compact_path(&path))
    );
    assert!(!shell.drain().presentations().is_empty());

    let _ = std::fs::remove_file(path);
}

#[test]
fn text_editor_shell_event_surface_drives_save_flow() {
    let path = temp_text_path("text_editor_shell_event_save.txt");
    let _ = std::fs::remove_file(&path);
    let mut shell = text_editor::shell(text_editor::State::default());

    let started = shell
        .handle_event(shell::Event::Started)
        .expect("started event should drain initial work");

    assert_eq!(started.opened_windows().len(), 1);
    assert_eq!(
        started.opened_windows()[0].title(),
        text_editor::WINDOW_TITLE
    );
    assert_eq!(
        started.opened_windows()[0].size(),
        text_editor::window_size()
    );
    assert_eq!(
        started.opened_windows()[0].canvas_color(),
        text_editor::CANVAS_COLOR
    );
    assert_eq!(started.presentations().len(), 1);
    assert_eq!(
        started.presentations()[0].layout().size(),
        text_editor::window_size()
    );
    assert_eq!(
        started.presentations()[0].scene().clear(),
        text_editor::CANVAS_COLOR
    );

    let window = started.opened_windows()[0].id();
    let resized = shell
        .handle_event(shell::Event::WindowResized {
            window,
            size: geometry::Size::new(640, 480),
        })
        .expect("resize event should drain presentation work");
    let presentation = resized
        .presentations()
        .iter()
        .find(|presentation| presentation.window() == window)
        .expect("resized window should present");
    let text_area = presentation
        .layout()
        .find_role(view::node::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let point = geometry::Point::new(text_area.rect().x() + 4, text_area.rect().y() + 4);

    shell
        .handle_event(shell::Event::PointerDown { window, point })
        .expect("pointer down event should focus the text area");
    shell
        .handle_event(shell::Event::TextCommitted {
            window,
            text: "abc".to_owned(),
        })
        .expect("text commit event should edit the document");

    assert_eq!(shell.runtime().state().document.text(), "abc");
    assert!(shell.runtime().state().document.is_dirty());

    let save = shell
        .handle_event(shell::Event::KeyDown {
            window,
            key: input::Key::Character('s'),
            modifiers: input::Modifiers::new(false, true, false, false),
            text: None,
        })
        .expect("ctrl+s should request a save path");

    assert_eq!(save.requests().len(), 1);
    assert_eq!(save.requests()[0].window(), window);
    assert_eq!(
        save.requests()[0].kind(),
        session::RequestKind::FileDialog(session::FileDialog::SaveAs)
    );

    let selected = shell
        .handle_event(shell::Event::FilePathSelected {
            window,
            path: Some(path.clone()),
        })
        .expect("selected path should start the save task");

    assert_eq!(selected.pending_tasks(), 1);
    assert_eq!(
        shell.runtime().state().last_status,
        format!("saving {}", text_editor::compact_path(&path))
    );
    assert!(selected.needs_poll());

    let finished = shell
        .handle_event(shell::Event::Poll)
        .expect("poll event should run and dispatch task work");

    assert_eq!(finished.pending_tasks(), 0);
    assert_eq!(finished.task_completions(), 0);
    assert!(!finished.needs_poll());
    assert!(path.exists());
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "abc");
    assert_eq!(
        shell.runtime().state().document.path(),
        Some(path.as_path())
    );
    assert!(!shell.runtime().state().document.is_dirty());
    assert_eq!(
        shell.runtime().state().last_status,
        format!("saved {}", text_editor::compact_path(&path))
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn text_editor_shell_event_surface_reports_window_lifecycle() {
    let mut shell = text_editor::shell(text_editor::State::default());

    let started = shell
        .handle_event(shell::Event::Started)
        .expect("started event should drain initial work");

    assert_eq!(started.opened_windows().len(), 1);
    assert!(started.closed_windows().is_empty());
    assert!(!started.is_empty());

    let window = started.opened_windows()[0].id();
    let closed = shell
        .handle_event(shell::Event::CloseRequested { window })
        .expect("close event should drain lifecycle work");

    assert!(closed.opened_windows().is_empty());
    assert_eq!(closed.closed_windows(), &[window]);
    assert!(closed.presentations().is_empty());
    assert!(!closed.is_empty());
    assert!(!shell.runtime().session().contains(window));
}

#[test]
fn shell_reports_existing_runtime_windows_on_first_drain() {
    let mut app = text_editor::app(text_editor::State::default());
    app.start();
    let window = app.session().windows()[0].id();
    let mut shell = Shell::new(app);

    let work = shell.drain();

    assert_eq!(work.opened_windows().len(), 1);
    assert_eq!(work.opened_windows()[0].id(), window);
    assert_eq!(work.opened_windows()[0].title(), text_editor::WINDOW_TITLE);
    assert_eq!(work.opened_windows()[0].size(), text_editor::window_size());
    assert_eq!(work.presentations().len(), 1);
}

#[test]
fn text_editor_host_adapter_consumes_shell_work_end_to_end() {
    let path = temp_text_path("text_editor_host_adapter_save.txt");
    let _ = std::fs::remove_file(&path);
    let mut host = Host::new(text_editor::shell(text_editor::State::default()));

    let started = host.start().expect("host should start shell");

    assert_eq!(started.opened_windows().len(), 1);
    assert_eq!(host.windows().len(), 1);
    let _: &host::Window = &host.windows()[0];
    let window = host.windows()[0].id();
    assert_eq!(host.windows()[0].title(), text_editor::WINDOW_TITLE);
    assert_eq!(host.windows()[0].size(), text_editor::window_size());
    assert!(host.presentation(window).is_some());

    host.handle_event(host::Event::window(
        window,
        host::WindowEvent::Resized {
            size: geometry::Size::new(640, 480),
        },
    ))
    .expect("resize should present at the new size");

    assert_eq!(
        host.window(window)
            .expect("host window should exist")
            .size(),
        geometry::Size::new(640, 480)
    );

    let text_area = host
        .presentation(window)
        .expect("host should retain latest presentation")
        .layout()
        .find_role(view::node::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let point = geometry::Point::new(text_area.rect().x() + 4, text_area.rect().y() + 4);

    host.handle_event(host::Event::window(
        window,
        host::WindowEvent::PointerDown { point },
    ))
    .expect("pointer down should focus text area");
    host.handle_event(host::Event::window(
        window,
        host::WindowEvent::TextCommitted {
            text: "host save".to_owned(),
        },
    ))
    .expect("text commit should edit document");

    assert_eq!(host.shell().runtime().state().document.text(), "host save");

    host.handle_event(host::Event::window(
        window,
        host::WindowEvent::KeyDown {
            key: input::Key::Character('s'),
            modifiers: input::Modifiers::new(false, true, false, false),
            text: None,
        },
    ))
    .expect("save shortcut should request file path");

    assert_eq!(host.requests().len(), 1);
    assert_eq!(
        host.requests()[0].kind(),
        session::RequestKind::FileDialog(session::FileDialog::SaveAs)
    );

    let selected = host
        .handle_event(host::Event::FilePathSelected {
            window,
            path: Some(path.clone()),
        })
        .expect("selected save path should schedule save");

    assert!(selected.needs_poll());
    assert!(host.needs_poll());
    assert!(host.requests().is_empty());

    host.poll().expect("poll should complete save task");

    assert!(!host.needs_poll());
    assert_eq!(
        std::fs::read_to_string(&path).expect("saved file should be readable"),
        "host save"
    );
    assert_eq!(
        host.shell().runtime().state().last_status,
        format!("saved {}", text_editor::compact_path(&path))
    );
    assert!(host.presentation(window).is_some());

    let closed = host
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::CloseRequested,
        ))
        .expect("close should update host window registry");

    assert_eq!(closed.closed_windows(), &[window]);
    assert!(host.windows().is_empty());
    assert!(host.presentation(window).is_none());

    let _ = std::fs::remove_file(path);
}
