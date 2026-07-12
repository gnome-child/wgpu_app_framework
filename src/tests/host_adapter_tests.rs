use super::*;

#[test]
fn host_window_event_mapper_routes_common_window_events() {
    let document = (0..120)
        .map(|line| format!("host line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut host = Host::new(text_editor::shell(text_editor::State {
        document: TextDocument::from_multiline_text(document),
        ..text_editor::State::default()
    }));

    host.start().expect("host should start shell");
    let window = host.windows()[0].id();
    host.handle_event(host::Event::window(
        window,
        host::WindowEvent::Resized {
            size: geometry::Size::new(640, 480),
        },
    ))
    .expect("resize should update pending geometry");
    let presented = host
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("redraw should present resized geometry");
    acknowledge_host_work(&mut host, &presented);

    let (target, point) = {
        let text_area = host
            .presentation(window)
            .expect("host should retain latest presentation")
            .layout()
            .find_role(view::Role::TextArea)
            .into_iter()
            .next()
            .expect("text area should be laid out");
        let target = text_area
            .target()
            .expect("text area should expose target")
            .clone();
        let point = geometry::Point::new(text_area.rect().x() + 4, text_area.rect().y() + 4);

        (target, point)
    };

    host.handle_event(host::Event::window(
        window,
        host::WindowEvent::PointerMoved { point },
    ))
    .expect("pointer move should route by hit test");

    assert_eq!(
        host.shell()
            .runtime()
            .session()
            .interaction(window)
            .expect("window should have interaction")
            .pointer()
            .hovered(),
        Some(&target)
    );

    host.handle_event(host::Event::window(
        window,
        host::WindowEvent::PointerDown {
            point,
            button: pointer::Button::Primary,
            modifiers: input::Modifiers::default(),
        },
    ))
    .expect("pointer down should focus and capture text area");

    assert_eq!(
        host.shell()
            .runtime()
            .session()
            .interaction(window)
            .expect("window should have interaction")
            .pointer()
            .capture()
            .map(|capture| capture.target()),
        Some(&target)
    );

    let preedit = text::edit::Preedit::new("世", Some((0, "世".len())));
    host.handle_event(host::Event::window(
        window,
        host::WindowEvent::TextPreedit {
            preedit: preedit.clone(),
        },
    ))
    .expect("preedit should route to focused text input");

    assert_eq!(
        host.shell()
            .runtime()
            .session()
            .interaction(window)
            .expect("window should have interaction")
            .text_input()
            .preedit(),
        Some(&preedit)
    );

    host.handle_event(host::Event::window(
        window,
        host::WindowEvent::Scrolled {
            point,
            delta: interaction::ScrollDelta::vertical(32),
        },
    ))
    .expect("scroll should route by hit test");

    assert_eq!(
        host.shell()
            .runtime()
            .session()
            .interaction(window)
            .expect("window should have interaction")
            .scroll()
            .offset(&target),
        interaction::ScrollOffset::new(0, 32)
    );

    host.handle_event(host::Event::window(
        window,
        host::WindowEvent::PointerUp {
            point,
            button: pointer::Button::Primary,
        },
    ))
    .expect("pointer up should release capture");
    host.handle_event(host::Event::window(window, host::WindowEvent::PointerLeft))
        .expect("pointer left should clear hover");

    let interaction = host
        .shell()
        .runtime()
        .session()
        .interaction(window)
        .expect("window should have interaction");
    assert_eq!(interaction.pointer().hovered(), None);
    assert_eq!(interaction.pointer().pressed(), None);
    assert_eq!(interaction.pointer().capture(), None);

    let redraw = host
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("redraw request should present retained view");

    assert_eq!(redraw.presentations().len(), 1);
    assert!(host.presentation(window).is_some());
}

#[test]
fn text_editor_host_drains_scene_work() {
    let mut app = text_editor::app(text_editor::State::default());

    app.start();

    let window = app.session().windows()[0].id();
    let work = app.drain_scenes(|id| {
        assert_eq!(id, window);
        text_editor::window_size()
    });

    assert_eq!(work.presentations().len(), 1);
    assert!(work.requests().is_empty());
    assert_eq!(work.pending_tasks(), 0);
    assert_eq!(work.presentations()[0].window(), window);
    assert_eq!(work.presentations()[0].size(), text_editor::window_size());
    assert_eq!(
        work.presentations()[0].scene().clear(),
        text_editor::CANVAS_COLOR
    );
    assert!(
        work.presentations()[0]
            .scene()
            .texts()
            .iter()
            .any(|text| text.value() == "File")
    );
    assert!(
        work.presentations()[0]
            .scene()
            .quads()
            .iter()
            .any(|quad| quad.fill().channels() == (28, 28, 30, 255))
    );
    assert!(!app.session().windows()[0].redraw_requested());
    assert!(app.drain_scenes(|_| text_editor::window_size()).is_empty());

    app.handle_input(window, Input::shortcut("Ctrl+O"))
        .expect("open shortcut should be handled");

    let work = app.drain_scenes(|id| {
        assert_eq!(id, window);
        geometry::Size::new(640, 480)
    });

    assert_eq!(work.presentations().len(), 1);
    assert_eq!(
        work.presentations()[0].size(),
        geometry::Size::new(640, 480)
    );
    assert_eq!(work.requests().len(), 1);
    assert_eq!(work.pending_tasks(), 0);
    assert_eq!(work.requests()[0].window(), window);
    assert_eq!(
        work.requests()[0].kind(),
        session::RequestKind::FileDialog(session::FileDialog::Open)
    );
}

#[test]
fn host_drops_stale_window_and_dialog_events_after_departure() {
    let mut host = Host::new(text_editor::shell(text_editor::State::default()));

    host.start().expect("host should start shell");
    let window = host.windows()[0].id();
    host.handle_event(host::Event::window(
        window,
        host::WindowEvent::CloseRequested,
    ))
    .expect("close should depart the host window");

    assert!(!host.shell().runtime().session().contains(window));
    assert!(host.windows().is_empty());
    assert!(host.presentations().is_empty());
    let revision = host.shell().runtime().revision();
    let status = host.shell().runtime().state().last_status.clone();

    for event in [
        host::Event::window(
            window,
            host::WindowEvent::PointerDown {
                point: geometry::Point::new(10, 10),
                button: pointer::Button::Primary,
                modifiers: input::Modifiers::default(),
            },
        ),
        host::Event::window(
            window,
            host::WindowEvent::KeyDown {
                key: input::Key::Character('x'),
                modifiers: input::Modifiers::default(),
                text: Some("x".to_owned()),
            },
        ),
        host::Event::window(window, host::WindowEvent::RedrawRequested),
        host::Event::FilePathSelected {
            window,
            path: Some(temp_text_path("stale_dialog_result.txt")),
        },
    ] {
        let work = host
            .handle_event(event)
            .expect("stale host event should be ignored safely");
        assert!(work.is_empty(), "stale event must not create host work");
    }

    assert_eq!(host.shell().runtime().revision(), revision);
    assert_eq!(host.shell().runtime().state().last_status, status);
    assert!(host.windows().is_empty());
    assert!(host.presentations().is_empty());
}
