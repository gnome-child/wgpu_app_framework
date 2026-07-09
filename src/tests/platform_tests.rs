use super::*;

#[test]
fn platform_key_and_modifier_conversion_matches_winit_inputs() {
    use winit::keyboard::{Key as WinitKey, ModifiersState, NamedKey};

    assert_eq!(
        platform::key(&WinitKey::Named(NamedKey::Tab)),
        input::Key::Tab
    );
    assert_eq!(
        platform::key(&WinitKey::Named(NamedKey::Enter)),
        input::Key::Enter
    );
    assert_eq!(
        platform::key(&WinitKey::Named(NamedKey::F4)),
        input::Key::F4
    );
    assert_eq!(
        platform::key(&WinitKey::Character("A".into())),
        input::Key::Character('A')
    );
    assert_eq!(
        platform::key(&WinitKey::Character("ab".into())),
        input::Key::Other
    );
    assert_eq!(platform::key_text(Some("é")).as_deref(), Some("é"));
    assert_eq!(platform::key_text(Some("a\u{8}")), None);
    assert_eq!(platform::key_text(None), None);

    let modifiers = platform::modifiers(
        ModifiersState::SHIFT | ModifiersState::CONTROL | ModifiersState::SUPER,
    );

    assert!(modifiers.shift());
    assert!(modifiers.control());
    assert!(!modifiers.alt());
    assert!(modifiers.super_key());
}

#[test]
fn platform_events_translate_winit_window_events_to_host_events() {
    use winit::{
        dpi::{PhysicalPosition, PhysicalSize},
        event::{
            DeviceId, ElementState, Ime, MouseButton, MouseScrollDelta, TouchPhase,
            WindowEvent as WinitWindowEvent,
        },
    };

    let window = window::Id::new(42);
    let mut events = platform::Events::new().with_scale_factor(2.0);

    let resized = events
        .window_event(
            window,
            &WinitWindowEvent::Resized(PhysicalSize::new(1840, 1360)),
        )
        .expect("resize should map");
    match resized {
        host::Event::Window {
            window: event_window,
            event: host::WindowEvent::Resized { size },
        } => {
            assert_eq!(event_window, window);
            assert_eq!(size, geometry::Size::new(920, 680));
        }
        _ => panic!("expected resize event"),
    }

    let moved = events
        .window_event(
            window,
            &WinitWindowEvent::CursorMoved {
                device_id: DeviceId::dummy(),
                position: PhysicalPosition::new(21.0, 31.0),
            },
        )
        .expect("cursor move should map");
    match moved {
        host::Event::Window {
            event: host::WindowEvent::PointerMoved { point },
            ..
        } => assert_eq!(point, geometry::Point::new(11, 16)),
        _ => panic!("expected pointer move event"),
    }
    assert_eq!(events.pointer(window), geometry::Point::new(11, 16));

    let pressed = events
        .window_event(
            window,
            &WinitWindowEvent::MouseInput {
                device_id: DeviceId::dummy(),
                state: ElementState::Pressed,
                button: MouseButton::Left,
            },
        )
        .expect("left press should map");
    match pressed {
        host::Event::Window {
            event: host::WindowEvent::PointerDown { point, button },
            ..
        } => {
            assert_eq!(point, geometry::Point::new(11, 16));
            assert_eq!(button, pointer::Button::Primary);
        }
        _ => panic!("expected pointer down event"),
    }

    let secondary = events
        .window_event(
            window,
            &WinitWindowEvent::MouseInput {
                device_id: DeviceId::dummy(),
                state: ElementState::Pressed,
                button: MouseButton::Right,
            },
        )
        .expect("secondary press should map");
    match secondary {
        host::Event::Window {
            event: host::WindowEvent::PointerDown { point, button },
            ..
        } => {
            assert_eq!(point, geometry::Point::new(11, 16));
            assert_eq!(button, pointer::Button::Secondary);
        }
        _ => panic!("expected secondary pointer down event"),
    }

    let scrolled = events
        .window_event(
            window,
            &WinitWindowEvent::MouseWheel {
                device_id: DeviceId::dummy(),
                delta: MouseScrollDelta::PixelDelta(PhysicalPosition::new(0.0, -48.0)),
                phase: TouchPhase::Moved,
            },
        )
        .expect("wheel should map");
    match scrolled {
        host::Event::Window {
            event: host::WindowEvent::Scrolled { point, delta },
            ..
        } => {
            assert_eq!(point, geometry::Point::new(11, 16));
            assert_eq!(delta, interaction::ScrollDelta::new(0, 24));
        }
        _ => panic!("expected scroll event"),
    }

    let line_scrolled = events
        .window_event(
            window,
            &WinitWindowEvent::MouseWheel {
                device_id: DeviceId::dummy(),
                delta: MouseScrollDelta::LineDelta(0.0, -1.0),
                phase: TouchPhase::Moved,
            },
        )
        .expect("line wheel should map");
    match line_scrolled {
        host::Event::Window {
            event: host::WindowEvent::Scrolled { point, delta },
            ..
        } => {
            assert_eq!(point, geometry::Point::new(11, 16));
            assert_eq!(delta, interaction::ScrollDelta::new(0, 28));
        }
        _ => panic!("expected line scroll event"),
    }

    let preedit = events
        .window_event(
            window,
            &WinitWindowEvent::Ime(Ime::Preedit("compose".to_owned(), Some((1, 4)))),
        )
        .expect("preedit should map");
    match preedit {
        host::Event::Window {
            event: host::WindowEvent::TextPreedit { preedit },
            ..
        } => {
            assert_eq!(preedit.text(), "compose");
            assert_eq!(preedit.selection(), Some((1, 4)));
        }
        _ => panic!("expected preedit event"),
    }

    let committed = events
        .window_event(
            window,
            &WinitWindowEvent::Ime(Ime::Commit("text".to_owned())),
        )
        .expect("commit should map");
    match committed {
        host::Event::Window {
            event: host::WindowEvent::TextCommitted { text },
            ..
        } => assert_eq!(text, "text"),
        _ => panic!("expected committed text event"),
    }

    let redraw = events
        .window_event(window, &WinitWindowEvent::RedrawRequested)
        .expect("redraw should map");
    match redraw {
        host::Event::Window {
            event: host::WindowEvent::RedrawRequested,
            ..
        } => {}
        _ => panic!("expected redraw event"),
    }
}

#[test]
fn platform_events_keep_pointer_and_scale_per_window() {
    use winit::{
        dpi::{PhysicalPosition, PhysicalSize},
        event::{DeviceId, ElementState, MouseButton, WindowEvent as WinitWindowEvent},
    };

    let first = window::Id::new(1);
    let second = window::Id::new(2);
    let mut events = platform::Events::new().with_scale_factor(2.0);

    events
        .window_event(
            first,
            &WinitWindowEvent::CursorMoved {
                device_id: DeviceId::dummy(),
                position: PhysicalPosition::new(40.0, 60.0),
            },
        )
        .expect("first pointer move should map");
    events.set_window_scale_factor(second, 1.0);

    let second_resize = events
        .window_event(
            second,
            &WinitWindowEvent::Resized(PhysicalSize::new(920, 680)),
        )
        .expect("second resize should map with second scale factor");
    match second_resize {
        host::Event::Window {
            window: event_window,
            event: host::WindowEvent::Resized { size },
        } => {
            assert_eq!(event_window, second);
            assert_eq!(size, geometry::Size::new(920, 680));
        }
        _ => panic!("expected second resize event"),
    }

    let second_press = events
        .window_event(
            second,
            &WinitWindowEvent::MouseInput {
                device_id: DeviceId::dummy(),
                state: ElementState::Pressed,
                button: MouseButton::Left,
            },
        )
        .expect("second press should map");
    match second_press {
        host::Event::Window {
            window: event_window,
            event: host::WindowEvent::PointerDown { point, button },
        } => {
            assert_eq!(event_window, second);
            assert_eq!(point, geometry::Point::new(0, 0));
            assert_eq!(button, pointer::Button::Primary);
        }
        _ => panic!("expected second pointer down event"),
    }

    assert_eq!(events.pointer(first), geometry::Point::new(20, 30));
    assert_eq!(events.pointer(second), geometry::Point::new(0, 0));
    assert_eq!(events.scale_factor(first), 2.0);
    assert_eq!(events.scale_factor(second), 1.0);
}

#[test]
fn popup_window_events_map_to_parent_overlay_coordinates() {
    use winit::{
        dpi::PhysicalPosition,
        event::{
            DeviceId, ElementState, MouseButton, MouseScrollDelta, TouchPhase,
            WindowEvent as WinitWindowEvent,
        },
    };

    let parent = window::Id::new(7);
    let bounds = geometry::Rect::new(100, 50, 300, 200);
    let mut events = platform::Events::new().with_scale_factor(1.0);

    let moved = events
        .popup_window_event(
            parent,
            bounds,
            1.5,
            &WinitWindowEvent::CursorMoved {
                device_id: DeviceId::dummy(),
                position: PhysicalPosition::new(15.0, 30.0),
            },
        )
        .expect("popup cursor move should map into parent coordinates");
    match moved {
        host::Event::Window {
            window: event_window,
            event: host::WindowEvent::PointerMoved { point },
        } => {
            assert_eq!(event_window, parent);
            assert_eq!(point, geometry::Point::new(110, 70));
        }
        _ => panic!("expected parent pointer move event"),
    }
    assert_eq!(events.pointer(parent), geometry::Point::new(110, 70));

    let pressed = events
        .popup_window_event(
            parent,
            bounds,
            1.5,
            &WinitWindowEvent::MouseInput {
                device_id: DeviceId::dummy(),
                state: ElementState::Pressed,
                button: MouseButton::Right,
            },
        )
        .expect("popup secondary press should be forwarded");
    match pressed {
        host::Event::Window {
            window: event_window,
            event: host::WindowEvent::PointerDown { point, button },
        } => {
            assert_eq!(event_window, parent);
            assert_eq!(point, geometry::Point::new(110, 70));
            assert_eq!(button, pointer::Button::Secondary);
        }
        _ => panic!("expected parent pointer down event"),
    }

    let scrolled = events
        .popup_window_event(
            parent,
            bounds,
            1.5,
            &WinitWindowEvent::MouseWheel {
                device_id: DeviceId::dummy(),
                delta: MouseScrollDelta::PixelDelta(PhysicalPosition::new(0.0, -30.0)),
                phase: TouchPhase::Moved,
            },
        )
        .expect("popup wheel should map through popup scale");
    match scrolled {
        host::Event::Window {
            window: event_window,
            event: host::WindowEvent::Scrolled { point, delta },
        } => {
            assert_eq!(event_window, parent);
            assert_eq!(point, geometry::Point::new(110, 70));
            assert_eq!(delta, interaction::ScrollDelta::new(0, 20));
        }
        _ => panic!("expected parent scroll event"),
    }
}

#[test]
fn popup_window_event_adapter_forwards_non_left_buttons() {
    use winit::{
        dpi::PhysicalPosition,
        event::{DeviceId, ElementState, MouseButton, WindowEvent as WinitWindowEvent},
    };

    let parent = window::Id::new(8);
    let bounds = geometry::Rect::new(40, 12, 200, 120);
    let mut events = platform::Events::new();
    events
        .popup_window_event(
            parent,
            bounds,
            2.0,
            &WinitWindowEvent::CursorMoved {
                device_id: DeviceId::dummy(),
                position: PhysicalPosition::new(20.0, 10.0),
            },
        )
        .expect("popup cursor move should establish pointer position");

    for (mouse, expected) in [
        (MouseButton::Middle, pointer::Button::Middle),
        (MouseButton::Back, pointer::Button::Back),
        (MouseButton::Forward, pointer::Button::Forward),
        (MouseButton::Other(9), pointer::Button::Other(9)),
    ] {
        let event = events
            .popup_window_event(
                parent,
                bounds,
                2.0,
                &WinitWindowEvent::MouseInput {
                    device_id: DeviceId::dummy(),
                    state: ElementState::Pressed,
                    button: mouse,
                },
            )
            .expect("popup mouse button should be forwarded");
        match event {
            host::Event::Window {
                event: host::WindowEvent::PointerDown { point, button },
                ..
            } => {
                assert_eq!(point, geometry::Point::new(50, 17));
                assert_eq!(button, expected);
            }
            _ => panic!("expected pointer down event"),
        }
    }
}

#[test]
fn popup_window_focused_events_do_not_change_framework_focus_truth() {
    use winit::{
        dpi::PhysicalPosition,
        event::{DeviceId, WindowEvent as WinitWindowEvent},
    };

    let parent = window::Id::new(9);
    let bounds = geometry::Rect::new(40, 12, 200, 120);
    let mut events = platform::Events::new();
    events
        .popup_window_event(
            parent,
            bounds,
            1.0,
            &WinitWindowEvent::CursorMoved {
                device_id: DeviceId::dummy(),
                position: PhysicalPosition::new(5.0, 6.0),
            },
        )
        .expect("popup cursor move should establish parent pointer");

    for focused in [true, false] {
        assert!(
            events
                .popup_window_event(parent, bounds, 1.0, &WinitWindowEvent::Focused(focused))
                .is_none(),
            "popup focused({focused}) must not become a framework window event"
        );
        assert_eq!(
            events.pointer(parent),
            geometry::Point::new(45, 18),
            "popup focus notifications must not disturb parent pointer/session truth"
        );
    }
}

#[test]
fn native_platform_backend_exposes_runner_state_without_starting_wgpu() {
    fn assert_backend<B: platform::Backend>() {}

    assert_backend::<platform::Native>();

    let mut native = platform::Native::new();
    let window = window::Id::new(99);

    assert!(!native.ready());
    assert!(!native.contains(window));
    assert!(native.requests().is_empty());
    assert!(!native.poll_requested());
    assert!(!native.take_poll_requested());
    assert!(matches!(
        native.request_redraw(window),
        Err(platform::NativeError::MissingWindow { window: missing }) if missing == window
    ));
}

#[test]
fn native_platform_backend_drains_pending_file_dialog_requests() {
    let mut native = platform::Native::new();
    let window = window::Id::new(44);
    let open = session::Request::file_dialog(window, session::FileDialog::Open);
    let save = session::Request::file_dialog(window, session::FileDialog::SaveAs);

    native.track_request_for_test(open);
    native.track_request_for_test(save);

    assert_eq!(native.requests(), &[open, save]);
    assert_eq!(native.take_requests(), vec![open, save]);
    assert!(native.requests().is_empty());
    assert!(native.take_requests().is_empty());
}

#[test]
fn platform_file_dialog_request_maps_to_file_path_selected_event() {
    let window = window::Id::new(55);
    let path = temp_text_path("dialog_selection.txt");
    let request = session::Request::file_dialog(window, session::FileDialog::SaveAs);

    let selected = platform::file_dialog_selected(request, Some(path.clone()));
    match selected {
        host::Event::FilePathSelected {
            window: event_window,
            path: Some(event_path),
        } => {
            assert_eq!(event_window, window);
            assert_eq!(event_path, path);
        }
        _ => panic!("expected selected file path event"),
    }

    let canceled = platform::file_dialog_selected(request, None);
    match canceled {
        host::Event::FilePathSelected {
            window: event_window,
            path: None,
        } => assert_eq!(event_window, window),
        _ => panic!("expected canceled file path event"),
    }
}

#[test]
fn platform_error_exposes_wrapped_source_errors() {
    use std::error::Error as StdError;

    let window = window::Id::new(99);
    let backend = platform::Error::Backend(platform::NativeError::MissingWindow { window });
    let framework = platform::Error::<platform::NativeError>::Framework(Error::Disabled {
        command: "app.save",
    });

    assert_eq!(
        StdError::source(&backend)
            .expect("backend error should expose source")
            .to_string(),
        "native window is not open: Id(99)"
    );
    assert_eq!(
        StdError::source(&framework)
            .expect("framework error should expose source")
            .to_string(),
        "command is disabled: app.save"
    );

    let run_error = platform::RunError::Platform(backend);
    assert_eq!(
        StdError::source(&run_error)
            .expect("run error should expose platform source")
            .to_string(),
        "backend error: native window is not open: Id(99)"
    );
}

#[test]
fn native_platform_runner_is_winit_application_handler_without_starting_wgpu() {
    fn assert_handler<A: winit::application::ApplicationHandler<text_editor::Event>>() {}

    assert_handler::<platform::Runner<text_editor::State, text_editor::Event>>();

    let runner = platform::Runner::new(text_editor::shell(text_editor::State::default()));

    assert!(!runner.started());
    assert!(runner.error().is_none());
    assert!(runner.platform().host().windows().is_empty());
    assert!(!runner.platform().backend().ready());
    assert!(runner.platform().backend().is_empty());
}

#[test]
fn text_editor_runner_entrypoint_builds_native_runner_without_starting_wgpu() {
    let runner = text_editor::runner(text_editor::State::default());

    assert!(!runner.started());
    assert!(runner.error().is_none());
    assert!(runner.platform().host().windows().is_empty());
    assert!(!runner.platform().backend().ready());
    assert!(
        runner
            .platform()
            .host()
            .shell()
            .runtime()
            .clipboard()
            .is_system_enabled()
    );
}

#[test]
fn text_editor_headless_shell_keeps_in_memory_clipboard() {
    let shell = text_editor::shell(text_editor::State::default());
    let native_shell = text_editor::native_shell(text_editor::State::default());

    assert!(!shell.runtime().clipboard().is_system_enabled());
    assert!(native_shell.runtime().clipboard().is_system_enabled());
}

#[test]
fn text_editor_run_entrypoint_uses_platform_native_runner_signature() {
    let text_editor_run: fn(
        text_editor::State,
    ) -> Result<(), platform::RunError<platform::NativeError>> = text_editor::run;
    let platform_run: fn(
        Shell<text_editor::State, text_editor::Event>,
    ) -> Result<(), platform::RunError<platform::NativeError>> =
        platform::run::<text_editor::State, text_editor::Event>;

    let _ = text_editor_run;
    let _ = platform_run;
}

#[test]
fn native_platform_runner_translates_raw_window_events_through_event_state() {
    use winit::{
        dpi::PhysicalPosition,
        event::{DeviceId, WindowEvent as WinitWindowEvent},
    };

    let mut runner = platform::Runner::new(text_editor::shell(text_editor::State::default()));
    let raw = winit::window::WindowId::dummy();
    let window = window::Id::new(88);

    runner
        .platform_mut()
        .backend_mut()
        .track_window_for_test(raw, window);
    runner.events_mut().set_window_scale_factor(window, 2.0);

    let translated = runner
        .translate_window_event(
            raw,
            &WinitWindowEvent::CursorMoved {
                device_id: DeviceId::dummy(),
                position: PhysicalPosition::new(40.0, 64.0),
            },
        )
        .expect("raw window should resolve to a host event");

    match translated {
        host::Event::Window {
            window: event_window,
            event: host::WindowEvent::PointerMoved { point },
        } => {
            assert_eq!(event_window, window);
            assert_eq!(point, geometry::Point::new(20, 32));
        }
        _ => panic!("expected pointer move event"),
    }
    assert_eq!(
        runner.events().pointer(window),
        geometry::Point::new(20, 32)
    );
}

#[test]
fn platform_runner_delegates_lifecycle_and_poll_to_platform() {
    let runtime = Runtime::new(EditorState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Runner"));
            assert!(cx.spawn(Task::ready(EditorEvent::Edited)).is_some());
        })
        .event(|cx, event: EditorEvent| {
            if let EditorEvent::Edited = event {
                cx.change(state::Reason::event("edited"), |state| {
                    state.event_count += 1;
                });
            }
        })
        .view(|_, _| View::new(view::Node::root()));
    let mut runner =
        platform::Runner::with_platform(Platform::new(Shell::new(runtime), FakeBackend::default()));

    runner.start().expect("runner should start platform");

    assert!(runner.started());
    assert_eq!(runner.platform().host().windows()[0].title(), "Runner");
    assert_eq!(
        runner
            .platform()
            .backend()
            .events()
            .iter()
            .filter(|event| matches!(event, BackendEvent::Poll))
            .count(),
        1
    );

    runner.poll().expect("runner poll should run the task");

    assert_eq!(
        runner
            .platform()
            .host()
            .shell()
            .runtime()
            .state()
            .event_count,
        1
    );
    assert!(!runner.platform().host().needs_poll());
}

#[test]
fn platform_poll_scheduling_rearms_after_each_poll_event() {
    let runtime = Runtime::new(EditorState::default())
        .started(|cx| {
            assert!(cx.spawn(Task::ready(())).is_some());
            assert!(cx.spawn(Task::ready(())).is_some());
        })
        .view(|_, _| View::new(view::Node::root()));
    let mut platform = Platform::new(Shell::new(runtime), FakeBackend::default());

    platform
        .start()
        .expect("start should schedule pending tasks");
    assert_eq!(
        platform
            .backend()
            .events()
            .iter()
            .filter(|event| matches!(event, BackendEvent::Poll))
            .count(),
        1
    );

    platform.backend_mut().events.clear();
    platform.poll().expect("first poll should run one task");
    assert_eq!(
        platform
            .backend()
            .events()
            .iter()
            .filter(|event| matches!(event, BackendEvent::Poll))
            .count(),
        1,
        "remaining task should schedule another poll wake"
    );
    assert!(platform.host().needs_poll());

    platform.backend_mut().events.clear();
    platform
        .drain()
        .expect("duplicate drain should not duplicate scheduled poll");
    assert_eq!(
        platform
            .backend()
            .events()
            .iter()
            .filter(|event| matches!(event, BackendEvent::Poll))
            .count(),
        0
    );

    platform
        .poll()
        .expect("second poll should drain task queue");
    assert!(!platform.host().needs_poll());
    assert_eq!(
        platform.animation_schedule(),
        crate::animation::Schedule::Idle,
        "poll handling should not leave a sticky animation schedule"
    );
}

#[test]
fn text_editor_platform_applies_host_work_to_backend() {
    let mut platform = Platform::new(
        text_editor::shell(text_editor::State::default()),
        FakeBackend::default(),
    );

    platform.start().expect("platform should start host");

    let window = platform.host().windows()[0].id();
    assert!(matches!(
        platform.backend().events().first(),
        Some(BackendEvent::OpenWindow {
            id,
            title,
            size,
            canvas_color,
            kind,
        }) if *id == window
            && title == text_editor::WINDOW_TITLE
            && *size == text_editor::window_size()
            && *canvas_color == text_editor::CANVAS_COLOR
            && *kind == window::Kind::Application
    ));
    assert!(platform.backend().events().iter().any(|event| matches!(
        event,
        BackendEvent::Present {
            window: presented,
            size,
            clear_color,
        } if *presented == window
            && *size == text_editor::window_size()
            && *clear_color == text_editor::CANVAS_COLOR
    )));
    let render = &platform
        .host()
        .shell()
        .runtime()
        .diagnostics(window)
        .expect("diagnostics should exist")
        .render;
    assert_eq!(render.frames_presented, 1);
    assert_eq!(render.acquire_wait_p95_us(), 10);
    assert_eq!(render.draw_p95_us(), 20);

    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::CloseRequested,
        ))
        .expect("platform should close host window");

    assert!(
        platform
            .backend()
            .events()
            .iter()
            .any(|event| { matches!(event, BackendEvent::CloseWindow { id } if *id == window) })
    );
}

#[test]
fn menu_dropdown_uses_native_popup_work_when_backend_supports_it() {
    let mut platform = Platform::new(
        text_editor::shell(text_editor::State::default()),
        FakeBackend::default().with_native_popups(),
    );

    platform.start().expect("platform should start host");

    let window = platform.host().windows()[0].id();
    let presentation = platform
        .host()
        .presentation(window)
        .expect("initial presentation should exist");
    let file = presentation
        .layout()
        .find_role(view::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("File"))
        .expect("file menu should be laid out");
    let point = frame_point(file);

    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::PointerDown {
                point,
                button: pointer::Button::Primary,
            },
        ))
        .expect("pointer down should be handled");
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::PointerUp {
                point,
                button: pointer::Button::Primary,
            },
        ))
        .expect("pointer up should open menu");

    assert!(platform.backend().events().iter().any(|event| matches!(
        event,
        BackendEvent::PresentPopup {
            parent,
            id: _,
            size,
            clear_color,
            framework_glass_panes,
            fallback_framework_glass_panes,
        } if *parent == window
            && size.width() > 0
            && size.height() > 0
            && *clear_color == scene::Color::rgba(0, 0, 0, 0)
            && *framework_glass_panes == 0
            && *fallback_framework_glass_panes == 0
    )));
    assert_eq!(
        platform.host().shell().runtime().session().windows()[0].kind(),
        window::Kind::Application,
        "native popups do not become framework windows"
    );
}

#[test]
fn command_palette_uses_native_popup_work_when_backend_supports_it() {
    let mut platform = Platform::new(
        text_editor::shell(text_editor::State::default()),
        FakeBackend::default().with_native_popups(),
    );

    platform.start().expect("platform should start host");

    let window = platform.host().windows()[0].id();
    platform.backend_mut().events.clear();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::KeyDown {
                key: input::Key::Character('p'),
                modifiers: input::Modifiers::new(true, true, false, false),
                text: None,
            },
        ))
        .expect("palette shortcut should open command palette");

    assert!(platform.backend().events().iter().any(|event| matches!(
        event,
        BackendEvent::PresentPopup {
            parent,
            id,
            size,
            clear_color,
            framework_glass_panes,
            fallback_framework_glass_panes,
        } if *parent == window
            && *id == interaction::CommandPalette::panel_id()
            && size.width() > 0
            && size.height() > 0
            && *clear_color == scene::Color::rgba(0, 0, 0, 0)
            && *framework_glass_panes == 0
            && *fallback_framework_glass_panes == 0
    )));
}

#[test]
fn popup_pointer_motion_without_presentation_does_not_close_native_popups() {
    let mut platform = Platform::new(
        text_editor::shell(text_editor::State::default()),
        FakeBackend::default().with_native_popups(),
    );

    platform.start().expect("platform should start host");

    let window = platform.host().windows()[0].id();
    let presentation = platform
        .host()
        .presentation(window)
        .expect("initial presentation should exist");
    let file = presentation
        .layout()
        .find_role(view::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("File"))
        .expect("file menu should be laid out");
    let point = frame_point(file);

    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::PointerDown {
                point,
                button: pointer::Button::Primary,
            },
        ))
        .expect("pointer down should be handled");
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::PointerUp {
                point,
                button: pointer::Button::Primary,
            },
        ))
        .expect("pointer up should open menu");
    assert!(
        platform
            .backend()
            .popup_sync_counts()
            .iter()
            .any(|count| *count > 0),
        "opening the menu should present a native popup"
    );

    let row_point = {
        let presentation = platform
            .host()
            .presentation(window)
            .expect("open menu presentation should exist");
        let row = presentation
            .layout()
            .frames()
            .iter()
            .find(|frame| frame.is_menu_row())
            .expect("open menu should lay out a row");
        frame_point(row)
    };
    platform.backend_mut().clear_popup_sync_counts();

    for _ in 0..3 {
        platform
            .handle_event(host::Event::window(
                window,
                host::WindowEvent::PointerMoved { point: row_point },
            ))
            .expect("popup pointer move should route through parent session");
    }

    assert!(
        !platform.backend().popup_sync_counts().contains(&0),
        "non-presentational pointer work must not authoritatively close native popups"
    );
}

#[test]
fn platform_applies_and_deduplicates_pointer_cursor_updates() {
    let focus = session::Focus::text("platform.cursor");
    let runtime = Runtime::new(SourceState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Cursor Platform"));
        })
        .view(move |_, _| {
            widget::view(|ui| {
                ui.column(|ui| {
                    ui.text_box(widget::TextBox::new("field").focus(focus));
                    ui.label("plain");
                });
            })
        });
    let mut platform = Platform::new(Shell::new(runtime), FakeBackend::default());

    platform.start().expect("platform should start");
    let window = platform.host().windows()[0].id();
    let presentation = platform
        .host()
        .presentation(window)
        .expect("platform should retain a presentation");
    let text_box = presentation
        .layout()
        .find_role(view::Role::TextBox)
        .into_iter()
        .next()
        .expect("text box should be laid out");
    let label = presentation
        .layout()
        .find_role(view::Role::Label)
        .into_iter()
        .next()
        .expect("label should be laid out");
    let text_point = frame_point(text_box);
    let label_point = frame_point(label);

    platform.backend_mut().events.clear();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::PointerMoved { point: text_point },
        ))
        .expect("text cursor move should be handled");
    assert_eq!(
        platform
            .backend()
            .events()
            .iter()
            .filter(|event| matches!(event, BackendEvent::SetCursor { .. }))
            .cloned()
            .collect::<Vec<_>>(),
        vec![BackendEvent::SetCursor {
            window,
            cursor: pointer::Cursor::Text,
        }]
    );

    platform.backend_mut().events.clear();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::PointerMoved { point: text_point },
        ))
        .expect("duplicate text cursor move should be handled");
    assert!(
        platform
            .backend()
            .events()
            .iter()
            .all(|event| !matches!(event, BackendEvent::SetCursor { .. })),
        "same cursor should be deduped"
    );

    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::PointerMoved { point: label_point },
        ))
        .expect("default cursor move should be handled");
    assert!(platform.backend().events().iter().any(|event| matches!(
        event,
        BackendEvent::SetCursor {
            window: event_window,
            cursor: pointer::Cursor::Default,
        } if *event_window == window
    )));
}

#[test]
fn text_editor_platform_deduplicates_dialogs_and_poll_scheduling() {
    let path = temp_text_path("text_editor_platform_save.txt");
    let _ = std::fs::remove_file(&path);
    let mut platform = Platform::new(
        text_editor::shell(text_editor::State::default()),
        FakeBackend::default(),
    );

    platform.start().expect("platform should start host");
    let window = platform.host().windows()[0].id();
    platform.backend_mut().events.clear();

    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::KeyDown {
                key: input::Key::Character('s'),
                modifiers: input::Modifiers::new(true, true, false, false),
                text: None,
            },
        ))
        .expect("save-as shortcut should request a file dialog");
    platform
        .drain()
        .expect("duplicate drain should not reopen dialog");

    assert_eq!(
        platform
            .backend()
            .events()
            .iter()
            .filter(|event| matches!(
                event,
                BackendEvent::FileDialog {
                    window: request_window,
                    kind: session::RequestKind::FileDialog(session::FileDialog::SaveAs),
                } if *request_window == window
            ))
            .count(),
        1
    );

    platform
        .handle_event(host::Event::FilePathSelected {
            window,
            path: Some(path.clone()),
        })
        .expect("selected path should schedule save task");
    platform
        .drain()
        .expect("duplicate drain should not reschedule poll");

    assert_eq!(
        platform
            .backend()
            .events()
            .iter()
            .filter(|event| matches!(event, BackendEvent::Poll))
            .count(),
        1
    );

    platform.poll().expect("poll should complete save task");

    assert!(path.exists());
    assert_eq!(
        platform.host().shell().runtime().state().last_status,
        format!("saved {}", text_editor::compact_path(&path))
    );
    assert!(!platform.host().needs_poll());

    let _ = std::fs::remove_file(path);
}
