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
        platform::key(&WinitKey::Named(NamedKey::F10)),
        input::Key::F10
    );
    assert_eq!(
        platform::key(&WinitKey::Named(NamedKey::ContextMenu)),
        input::Key::ContextMenu
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
            DeviceId, ElementState, Ime, Modifiers as WinitModifiers, MouseButton,
            MouseScrollDelta, TouchPhase, WindowEvent as WinitWindowEvent,
        },
    };

    let window = window::Id::new(42);
    let mut events = platform::Events::new().with_scale_factor(2.0);

    assert!(
        events
            .window_event(window, &WinitWindowEvent::Focused(false))
            .is_none(),
        "system window deactivation must not become a deliberate framework focus departure"
    );

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

    let modifiers_changed = events
        .window_event(
            window,
            &WinitWindowEvent::ModifiersChanged(WinitModifiers::default()),
        )
        .expect("modifier state should reach the host");
    assert!(matches!(
        modifiers_changed,
        host::Event::Window {
            window: event_window,
            event: host::WindowEvent::ModifiersChanged { modifiers },
        } if event_window == window && modifiers == input::Modifiers::default()
    ));

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
            event: host::WindowEvent::PointerDown { point, button, .. },
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
            event: host::WindowEvent::PointerDown { point, button, .. },
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
            event: host::WindowEvent::PointerDown { point, button, .. },
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
fn pixel_scroll_trace_preserves_sum_before_visual_quantization() {
    use winit::{dpi::PhysicalPosition, event::MouseScrollDelta};

    let actual = (0..5)
        .map(|_| {
            f64::from(
                platform::scroll_delta(
                    MouseScrollDelta::PixelDelta(PhysicalPosition::new(0.0, -0.4)),
                    1.0,
                )
                .y(),
            )
        })
        .sum::<f64>();

    assert!(
        (actual - 2.0).abs() < f64::EPSILON,
        "five 0.4-logical-pixel inputs must preserve their 2.0-pixel sum before the interaction owner quantizes visual motion; actual={actual}"
    );
}

#[test]
fn legacy_per_event_rounding_loses_fractional_pixel_sum() {
    let legacy_visual_sum = (0..5).map(|_| 0.4_f64.round() as i32).sum::<i32>();
    let aggregate_visual_sum = (5.0_f64 * 0.4).trunc() as i32;

    assert_eq!(legacy_visual_sum, 0);
    assert_eq!(aggregate_visual_sum, 2);
    assert_ne!(legacy_visual_sum, aggregate_visual_sum);
}

#[test]
fn input_precision_case_fractional_line_wheel_preserves_line_scale() {
    use winit::event::MouseScrollDelta;

    let delta = platform::scroll_delta(MouseScrollDelta::LineDelta(0.0, -0.25), 1.75);
    assert_eq!(
        delta,
        interaction::ScrollDelta::from_logical_pixels(0.0, 7.0)
    );
}

fn test_popup_realization(
    parent: window::Id,
    bounds: geometry::Rect,
    panel_offset: geometry::Point,
    scale: f64,
) -> crate::popup::Realization {
    crate::popup::Realization::native(
        interaction::Id::new("test.popup"),
        parent,
        crate::popup::Generation::initial(),
        crate::popup::Geometry::new(
            bounds,
            bounds,
            bounds,
            geometry::Rect::new(
                bounds.x().saturating_sub(panel_offset.x()),
                bounds.y().saturating_sub(panel_offset.y()),
                bounds.width().saturating_add(panel_offset.x()),
                bounds.height().saturating_add(panel_offset.y()),
            ),
            panel_offset,
            scale,
        ),
    )
}

#[test]
fn popup_window_events_retain_popup_surface_and_map_to_retained_coordinates() {
    use winit::{
        dpi::PhysicalPosition,
        event::{
            DeviceId, ElementState, MouseButton, MouseScrollDelta, TouchPhase,
            WindowEvent as WinitWindowEvent,
        },
    };

    let parent = window::Id::new(7);
    let bounds = geometry::Rect::new(100, 50, 300, 200);
    let realization = test_popup_realization(parent, bounds, geometry::Point::new(8, 6), 1.5);
    let mut events = platform::Events::new().with_scale_factor(1.0);

    let moved = events
        .popup_window_event(
            realization,
            1.5,
            &WinitWindowEvent::CursorMoved {
                device_id: DeviceId::dummy(),
                position: PhysicalPosition::new(27.0, 39.0),
            },
        )
        .expect("popup cursor move should map into parent coordinates");
    match moved {
        host::Event::Popup {
            parent: event_window,
            popup,
            event: host::WindowEvent::PointerMoved { point },
        } => {
            assert_eq!(event_window, parent);
            assert_eq!(popup, realization.popup());
            assert_eq!(point, geometry::Point::new(110, 70));
        }
        _ => panic!("expected parent pointer move event"),
    }
    assert_eq!(events.pointer(parent), geometry::Point::new(110, 70));

    let pressed = events
        .popup_window_event(
            realization,
            1.5,
            &WinitWindowEvent::MouseInput {
                device_id: DeviceId::dummy(),
                state: ElementState::Pressed,
                button: MouseButton::Right,
            },
        )
        .expect("popup secondary press should be forwarded");
    match pressed {
        host::Event::Popup {
            parent: event_window,
            popup,
            event: host::WindowEvent::PointerDown { point, button, .. },
        } => {
            assert_eq!(event_window, parent);
            assert_eq!(popup, realization.popup());
            assert_eq!(point, geometry::Point::new(110, 70));
            assert_eq!(button, pointer::Button::Secondary);
        }
        _ => panic!("expected parent pointer down event"),
    }

    let scrolled = events
        .popup_window_event(
            realization,
            1.5,
            &WinitWindowEvent::MouseWheel {
                device_id: DeviceId::dummy(),
                delta: MouseScrollDelta::PixelDelta(PhysicalPosition::new(0.0, -30.0)),
                phase: TouchPhase::Moved,
            },
        )
        .expect("popup wheel should map through popup scale");
    match scrolled {
        host::Event::Popup {
            parent: event_window,
            popup,
            event: host::WindowEvent::Scrolled { point, delta },
        } => {
            assert_eq!(event_window, parent);
            assert_eq!(popup, realization.popup());
            assert_eq!(point, geometry::Point::new(110, 70));
            assert_eq!(delta, interaction::ScrollDelta::new(0, 20));
        }
        _ => panic!("expected parent scroll event"),
    }
}

#[test]
fn popup_ime_events_route_to_parent_text_input() {
    use winit::event::{Ime, WindowEvent as WinitWindowEvent};

    let parent = window::Id::new(71);
    let bounds = geometry::Rect::new(100, 50, 300, 200);
    let realization = test_popup_realization(parent, bounds, geometry::Point::new(0, 0), 1.25);
    let mut events = platform::Events::new();

    let preedit = events
        .popup_window_event(
            realization,
            1.25,
            &WinitWindowEvent::Ime(Ime::Preedit("compose".to_owned(), Some((1, 4)))),
        )
        .expect("popup preedit should route to its logical parent");
    match preedit {
        host::Event::Popup {
            parent: window,
            popup: _,
            event: host::WindowEvent::TextPreedit { preedit },
        } => {
            assert_eq!(window, parent);
            assert_eq!(preedit.text(), "compose");
            assert_eq!(preedit.selection(), Some((1, 4)));
        }
        _ => panic!("expected parent preedit event"),
    }

    let commit = events
        .popup_window_event(
            realization,
            1.25,
            &WinitWindowEvent::Ime(Ime::Commit("text".to_owned())),
        )
        .expect("popup commit should route to its logical parent");
    assert!(matches!(
        commit,
        host::Event::Popup {
            parent: window,
            popup: _,
            event: host::WindowEvent::TextCommitted { ref text },
        } if window == parent && text == "text"
    ));
}

#[test]
fn popup_window_event_adapter_forwards_non_left_buttons() {
    use winit::{
        dpi::PhysicalPosition,
        event::{
            DeviceId, ElementState, Modifiers as WinitModifiers, MouseButton,
            WindowEvent as WinitWindowEvent,
        },
    };

    let parent = window::Id::new(8);
    let bounds = geometry::Rect::new(40, 12, 200, 120);
    let realization = test_popup_realization(parent, bounds, geometry::Point::new(0, 0), 2.0);
    let mut events = platform::Events::new();
    events
        .popup_window_event(
            realization,
            2.0,
            &WinitWindowEvent::CursorMoved {
                device_id: DeviceId::dummy(),
                position: PhysicalPosition::new(20.0, 10.0),
            },
        )
        .expect("popup cursor move should establish pointer position");

    let modifiers_changed = events
        .popup_window_event(
            realization,
            2.0,
            &WinitWindowEvent::ModifiersChanged(WinitModifiers::default()),
        )
        .expect("popup modifier state should reach the parent session");
    assert!(matches!(
        modifiers_changed,
        host::Event::Popup {
            parent: event_parent,
            popup,
            event: host::WindowEvent::ModifiersChanged { modifiers },
        } if event_parent == parent
            && popup == realization.popup()
            && modifiers == input::Modifiers::default()
    ));

    for (mouse, expected) in [
        (MouseButton::Middle, pointer::Button::Middle),
        (MouseButton::Back, pointer::Button::Back),
        (MouseButton::Forward, pointer::Button::Forward),
        (MouseButton::Other(9), pointer::Button::Other(9)),
    ] {
        let event = events
            .popup_window_event(
                realization,
                2.0,
                &WinitWindowEvent::MouseInput {
                    device_id: DeviceId::dummy(),
                    state: ElementState::Pressed,
                    button: mouse,
                },
            )
            .expect("popup mouse button should be forwarded");
        match event {
            host::Event::Popup {
                event: host::WindowEvent::PointerDown { point, button, .. },
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
    let realization = test_popup_realization(parent, bounds, geometry::Point::new(0, 0), 1.0);
    let mut events = platform::Events::new();
    events
        .popup_window_event(
            realization,
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
                .popup_window_event(realization, 1.0, &WinitWindowEvent::Focused(focused),)
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
fn platform_propagates_operational_backend_failure_at_the_failing_boundary() {
    let runtime = Runtime::new(EditorState::default())
        .started(|cx| {
            cx.open_window(window::Options::new("Failing Backend"));
        })
        .view(|_, _| View::new(view::Node::root()));
    let mut platform = Platform::new(Shell::new(runtime), FakeBackend::default().failing_open());

    let error = platform
        .start()
        .expect_err("backend open failure should cross the Platform boundary");

    assert!(matches!(error, platform::Error::Backend("open failed")));
    assert!(
        platform.backend().events().is_empty(),
        "a failed open must not be recorded as completed backend work"
    );
}

#[test]
fn native_platform_runner_is_winit_application_handler_without_starting_wgpu() {
    fn assert_handler<
        A: winit::application::ApplicationHandler<platform::RunnerEvent<text_editor::Event>>,
    >() {
    }

    assert_handler::<platform::Runner<text_editor::State, text_editor::Event>>();

    let runner = platform::Runner::new(text_editor::shell(text_editor::State::default()));

    assert!(!runner.started());
    assert!(runner.error().is_none());
    assert!(runner.platform().host().windows().is_empty());
    assert!(!runner.platform().backend().ready());
    assert!(runner.platform().backend().is_empty());
}

#[test]
fn native_launch_defaults_to_system_clipboard_while_headless_stays_in_memory() {
    let shell = text_editor::shell(text_editor::State::default());
    let native_shell = platform::native_shell(text_editor::app(text_editor::State::default()));

    assert!(!shell.runtime().clipboard().is_system_enabled());
    assert!(native_shell.runtime().clipboard().is_system_enabled());
}

#[test]
fn native_launch_preserves_an_explicit_clipboard_choice() {
    let explicit = Clipboard::default();
    explicit
        .put(&clipboard::Text::new("chosen"))
        .expect("explicit clipboard should accept text");
    let native_shell = platform::native_shell(
        text_editor::app(text_editor::State::default()).with_clipboard(explicit.clone()),
    );

    assert!(!native_shell.runtime().clipboard().is_system_enabled());
    native_shell
        .runtime()
        .clipboard()
        .put(&clipboard::Text::new("shared"))
        .expect("preserved clipboard should remain writable");
    assert_eq!(
        explicit.text().expect("clipboard read should succeed"),
        Some("shared".to_owned())
    );
}

#[test]
fn application_launch_and_lower_level_run_signatures_stay_distinct() {
    let platform_launch: fn(
        Runtime<text_editor::State, text_editor::Event, View>,
    ) -> Result<(), platform::RunError<platform::NativeError>> =
        platform::launch::<text_editor::State, text_editor::Event>;
    let platform_run: fn(
        Shell<text_editor::State, text_editor::Event>,
    ) -> Result<(), platform::RunError<platform::NativeError>> =
        platform::run::<text_editor::State, text_editor::Event>;

    let _ = platform_launch;
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
        0
    );

    assert!(
        runner
            .platform_mut()
            .host_mut()
            .shell_mut()
            .run_next_task()
            .is_some()
    );
    runner
        .platform_mut()
        .drain()
        .expect("deterministic completion should drain");

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
fn pending_tasks_do_not_schedule_ui_poll_wakes() {
    let runtime = Runtime::new(EditorState::default())
        .started(|cx| {
            assert!(cx.spawn(Task::ready(())).is_some());
            assert!(cx.spawn(Task::ready(())).is_some());
        })
        .view(|_, _| View::new(view::Node::root()));
    let mut platform = Platform::new(Shell::new(runtime), FakeBackend::default());

    platform
        .start()
        .expect("start should retain pending tasks for worker dispatch");
    assert_eq!(
        platform
            .backend()
            .events()
            .iter()
            .filter(|event| matches!(event, BackendEvent::Poll))
            .count(),
        0
    );
    assert!(!platform.host().needs_poll());

    assert!(platform.host_mut().shell_mut().run_next_task().is_some());
    platform.drain().expect("first completion should drain");
    assert_eq!(
        platform
            .backend()
            .events()
            .iter()
            .filter(|event| matches!(event, BackendEvent::Poll))
            .count(),
        0,
        "remaining worker work must not schedule a UI poll wake"
    );
    assert!(!platform.host().needs_poll());

    platform.backend_mut().events.clear();
    platform.drain().expect("duplicate drain should stay idle");
    assert_eq!(
        platform
            .backend()
            .events()
            .iter()
            .filter(|event| matches!(event, BackendEvent::Poll))
            .count(),
        0
    );

    assert!(platform.host_mut().shell_mut().run_next_task().is_some());
    platform.drain().expect("second completion should drain");
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
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("first redraw should present");
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
    assert!(
        platform.backend().events().iter().any(|event| matches!(
            event,
            BackendEvent::Present {
                window: presented,
                size,
                clear_color,
            } if *presented == window
                && *size == text_editor::window_size()
                && *clear_color == text_editor::CANVAS_COLOR
        )),
        "events: {:?}",
        platform.backend().events()
    );
    let render = &platform
        .host()
        .shell()
        .runtime()
        .diagnostics(window)
        .expect("diagnostics should exist")
        .render;
    assert_eq!(render.frames_present_submitted, 1);
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
fn high_rate_events_mutate_immediately_but_present_once_at_redraw() {
    let mut platform = Platform::new(
        Shell::new(control_gallery::app(control_gallery::State::default())),
        FakeBackend::default(),
    );
    platform.start().expect("platform should start");
    let window = platform.host().windows()[0].id();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("first redraw should present");
    let click = platform
        .host()
        .presentation(window)
        .expect("initial presentation should exist")
        .layout()
        .find_role(view::Role::Button)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Click"))
        .expect("gallery click button should be laid out");
    let point = frame_point(click);
    let before_frames = platform
        .host()
        .shell()
        .runtime()
        .diagnostics(window)
        .unwrap()
        .render
        .frames_present_submitted;
    platform.backend_mut().events.clear();

    for _ in 0..1_000 {
        platform
            .handle_event(host::Event::window(
                window,
                host::WindowEvent::PointerMoved { point },
            ))
            .expect("pointer movement should update session truth");
    }
    for _ in 0..10 {
        platform
            .handle_event(host::Event::window(
                window,
                host::WindowEvent::PointerDown {
                    point,
                    button: pointer::Button::Primary,
                    modifiers: input::Modifiers::default(),
                },
            ))
            .expect("pointer down should execute immediately");
        platform
            .handle_event(host::Event::window(
                window,
                host::WindowEvent::PointerUp {
                    point,
                    button: pointer::Button::Primary,
                },
            ))
            .expect("pointer up should execute immediately");
    }

    assert_eq!(
        platform.host().shell().runtime().state().clicks,
        10,
        "all discrete commands must execute before the frame"
    );
    assert!(
        platform
            .backend()
            .events()
            .iter()
            .all(|event| !matches!(event, BackendEvent::Present { .. })),
        "ordinary input must never synchronously present"
    );
    assert_eq!(
        platform
            .backend()
            .events()
            .iter()
            .filter(|event| matches!(event, BackendEvent::RequestRedraw { .. }))
            .count(),
        1,
        "all in-flight high-rate demand must share one backend redraw request"
    );
    assert_eq!(
        platform
            .host()
            .shell()
            .runtime()
            .diagnostics(window)
            .unwrap()
            .render
            .frames_present_submitted,
        before_frames
    );

    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("redraw should sample the latest truth");
    assert_eq!(
        platform
            .backend()
            .events()
            .iter()
            .filter(|event| matches!(event, BackendEvent::Present { .. }))
            .count(),
        1
    );
}

#[test]
fn unsolicited_redraw_delivery_is_counted_as_no_progress() {
    let mut platform = Platform::new(
        Shell::new(control_gallery::app(control_gallery::State::default())),
        FakeBackend::default(),
    );
    platform.start().expect("platform should start");
    let window = platform.host().windows()[0].id();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("initial redraw should present");
    let before = platform
        .host()
        .shell()
        .runtime()
        .diagnostics(window)
        .expect("window diagnostics")
        .render
        .redraw_no_progress;

    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("unsolicited redraw should remain a valid no-op");

    let render = &platform
        .host()
        .shell()
        .runtime()
        .diagnostics(window)
        .expect("window diagnostics")
        .render;
    assert_eq!(render.redraw_no_progress, before + 1);
    assert!(render.redraw_no_progress <= render.redraw_deliveries);
}

#[test]
fn wheel_deltas_accumulate_losslessly_before_one_frame() {
    let document = (0..2_000)
        .map(|line| format!("coalesced line {line:04}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut platform = Platform::new(
        text_editor::shell(text_editor::State {
            document: TextDocument::from_multiline_text(document),
            ..text_editor::State::default()
        }),
        FakeBackend::default(),
    );
    platform.start().expect("platform should start");
    let window = platform.host().windows()[0].id();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("first redraw should present");
    let text_area = platform
        .host()
        .presentation(window)
        .unwrap()
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let target = text_area.target().unwrap().clone();
    let point = frame_point(text_area);
    platform.backend_mut().events.clear();

    for _ in 0..1_000 {
        platform
            .handle_event(host::Event::window(
                window,
                host::WindowEvent::Scrolled {
                    point,
                    delta: interaction::ScrollDelta::vertical(1),
                },
            ))
            .expect("wheel delta should update session truth");
    }

    let admitted_before_frame = platform
        .host()
        .shell()
        .runtime()
        .session()
        .interaction(window)
        .unwrap()
        .scroll()
        .offset(&target);
    assert!(admitted_before_frame.y() > 0);
    assert!(admitted_before_frame.y() < 1_000);
    assert_eq!(
        platform
            .host()
            .shell()
            .runtime()
            .session()
            .interaction(window)
            .unwrap()
            .scroll()
            .desired_offset(&target),
        interaction::ScrollOffset::new(0, 1_000)
    );
    assert!(
        platform
            .backend()
            .events()
            .iter()
            .all(|event| !matches!(event, BackendEvent::Present { .. }))
    );
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("one redraw should present the cumulative offset");
    assert_eq!(
        platform
            .backend()
            .events()
            .iter()
            .filter(|event| matches!(event, BackendEvent::Present { .. }))
            .count(),
        1
    );
    assert_eq!(
        platform
            .host()
            .shell()
            .runtime()
            .session()
            .interaction(window)
            .unwrap()
            .scroll()
            .offset(&target),
        interaction::ScrollOffset::new(0, 1_000)
    );
}

#[test]
fn skipped_frame_requests_redraw_until_one_receipt_succeeds() {
    let mut platform = Platform::new(
        text_editor::shell(text_editor::State::default()),
        FakeBackend::default().skipping_present(),
    );
    platform.start().expect("platform should start");
    let window = platform.host().windows()[0].id();
    platform.backend_mut().events.clear();

    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("skipped redraw should remain recoverable");
    let render = &platform
        .host()
        .shell()
        .runtime()
        .diagnostics(window)
        .unwrap()
        .render;
    assert_eq!(render.frames_attempted, 1);
    assert_eq!(render.frames_present_submitted, 0);
    assert!(
        platform
            .backend()
            .events()
            .iter()
            .all(|event| !matches!(event, BackendEvent::SetIme { .. })),
        "a failed present must not expose candidate IME geometry"
    );
    assert!(
        platform
            .host()
            .shell()
            .runtime()
            .presented_layout(window)
            .is_none()
    );
    assert!(platform.backend().events().iter().any(|event| matches!(
        event,
        BackendEvent::RequestRedraw { window: requested } if *requested == window
    )));

    platform.backend_mut().skip_present = false;
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("retry should present");
    let render = &platform
        .host()
        .shell()
        .runtime()
        .diagnostics(window)
        .unwrap()
        .render;
    assert_eq!(render.frames_attempted, 2);
    assert_eq!(render.frames_present_submitted, 1);
    assert_eq!(
        platform
            .backend()
            .events()
            .iter()
            .filter(|event| matches!(event, BackendEvent::SetIme { .. }))
            .count(),
        1,
        "the successful retry must apply the selected presentation's IME geometry exactly once"
    );
    assert!(
        platform
            .host()
            .shell()
            .runtime()
            .presented_layout(window)
            .is_some()
    );
}

#[test]
fn present_submitted_ime_geometry_follows_text_scroll_property_snapshot() {
    let document = (0..160)
        .map(|line| format!("scroll IME line {line:03}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut platform = Platform::new(
        text_editor::shell(text_editor::State {
            document: TextDocument::from_multiline_text(document),
            ..text_editor::State::default()
        }),
        FakeBackend::default(),
    );
    platform.start().expect("platform should start");
    let window = platform.host().windows()[0].id();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("initial redraw should present");
    let text_area = platform
        .host()
        .presentation(window)
        .expect("initial presentation should exist")
        .layout()
        .find_role(view::Role::TextArea)
        .into_iter()
        .next()
        .expect("text area should be laid out");
    let point = geometry::Point::new(text_area.rect().x() + 12, text_area.rect().y() + 12);
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::PointerDown {
                point,
                button: pointer::Button::Primary,
                modifiers: input::Modifiers::default(),
            },
        ))
        .expect("text press should focus and place the caret");
    platform.backend_mut().events.clear();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("focused frame should present");
    let before = platform
        .backend()
        .events()
        .iter()
        .rev()
        .find_map(|event| match event {
            BackendEvent::SetIme { update } => match update.target() {
                Some(ime::Target::Parent { area }) => Some(area),
                _ => None,
            },
            _ => None,
        })
        .expect("focused text frame should expose parent IME geometry");

    platform.backend_mut().events.clear();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::Scrolled {
                point,
                delta: interaction::ScrollDelta::vertical(20),
            },
        ))
        .expect("resident scroll should update property state");
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("property frame should present");
    let after = platform
        .backend()
        .events()
        .iter()
        .rev()
        .find_map(|event| match event {
            BackendEvent::SetIme { update } => match update.target() {
                Some(ime::Target::Parent { area }) => Some(area),
                _ => None,
            },
            _ => None,
        })
        .expect("scrolled frame should expose parent IME geometry");

    assert_eq!(after.x(), before.x());
    assert_eq!(after.y(), before.y() - 20);
    assert_eq!(after.width(), before.width());
    assert_eq!(after.height(), before.height());
}

#[test]
fn menu_dropdown_uses_native_popup_work_when_backend_supports_it() {
    let mut platform = Platform::new(
        text_editor::shell(text_editor::State::default()),
        FakeBackend::default().with_native_popups(),
    );

    platform.start().expect("platform should start host");

    let window = platform.host().windows()[0].id();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("first redraw should present");
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
                modifiers: input::Modifiers::default(),
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
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("redraw should present the open menu");

    assert!(
        platform.backend().events().iter().any(|event| matches!(
            event,
            BackendEvent::PresentPopup {
                parent,
                id: _,
                size,
                clear_color,
                framework_glass_panes,
                material_regions,
            } if *parent == window
                && size.width() > 0
                && size.height() > 0
                && *clear_color == scene::Color::rgba(0, 0, 0, 0)
                && *framework_glass_panes == 1
                && *material_regions == 1
        )),
        "events: {:?}",
        platform.backend().events()
    );
    assert_eq!(
        platform.host().shell().runtime().session().windows()[0].kind(),
        window::Kind::Application,
        "native popups do not become framework windows"
    );
}

#[test]
fn native_popup_frames_are_interactive_only_on_their_realized_surface() {
    let mut platform = Platform::new(
        text_editor::shell(text_editor::State::default()),
        FakeBackend::default().with_native_popups(),
    );

    platform.start().expect("platform should start host");
    let window = platform.host().windows()[0].id();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("first redraw should present");
    let initial = platform
        .host()
        .presentation(window)
        .expect("initial presentation should exist");
    let file = initial
        .layout()
        .find_role(view::Role::Menu)
        .into_iter()
        .find(|frame| frame.label_text() == Some("File"))
        .expect("file menu should be laid out");
    let file_point = frame_point(file);
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::PointerDown {
                point: file_point,
                button: pointer::Button::Primary,
                modifiers: input::Modifiers::default(),
            },
        ))
        .expect("pointer down should be handled");
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::PointerUp {
                point: file_point,
                button: pointer::Button::Primary,
            },
        ))
        .expect("pointer up should open menu");
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("open menu should present");

    let presented = platform
        .host()
        .presentation(window)
        .expect("open menu presentation should exist");
    let layout = presented.layout();
    let panel = layout
        .find_role(view::Role::FloatingPanel)
        .into_iter()
        .next()
        .expect("open menu should have a floating panel");
    let popup = panel
        .target()
        .and_then(interaction::Target::element_id)
        .expect("floating panel should name its popup surface");
    let row = layout
        .find_role(view::Role::Binding)
        .into_iter()
        .find(|frame| frame.label_text() == Some("Open"))
        .expect("file menu should contain Open");
    let point = frame_point(row);
    let expected = row
        .target()
        .cloned()
        .expect("menu row should be interactive");

    assert_ne!(
        layout
            .hit_test_on_surface(point, crate::popup::Surface::Parent)
            .and_then(|hit| hit.target().cloned()),
        Some(expected.clone()),
        "the abandoned in-frame copy must not remain interactive"
    );
    assert_eq!(
        layout
            .hit_test_on_surface(point, crate::popup::Surface::Native(popup))
            .and_then(|hit| hit.target().cloned()),
        Some(expected.clone())
    );
    assert!(
        layout
            .hit_test_on_surface(
                point,
                crate::popup::Surface::Native(interaction::Id::new("other.popup")),
            )
            .is_none(),
        "one popup surface may not address another popup's frames"
    );

    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::PointerMoved { point },
        ))
        .expect("parent pointer motion should route");
    assert_ne!(
        platform
            .host()
            .shell()
            .runtime()
            .session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().hovered())
            .cloned(),
        Some(expected.clone())
    );

    platform
        .handle_event(host::Event::popup(
            window,
            popup,
            host::WindowEvent::PointerMoved { point },
        ))
        .expect("popup pointer motion should route with surface identity");
    assert_eq!(
        platform
            .host()
            .shell()
            .runtime()
            .session()
            .interaction(window)
            .and_then(|interaction| interaction.pointer().hovered())
            .cloned(),
        Some(expected.clone())
    );

    for frame in 0..4 {
        platform
            .handle_event(host::Event::window(
                window,
                host::WindowEvent::RedrawRequested,
            ))
            .expect("popup hover repaint should preserve surface ownership");
        assert_eq!(
            platform
                .host()
                .shell()
                .runtime()
                .session()
                .interaction(window)
                .and_then(|interaction| interaction.pointer().hovered())
                .cloned(),
            Some(expected.clone()),
            "frame {frame} must classify native ownership before hover projection"
        );
        let layout = platform
            .host()
            .presentation(window)
            .expect("each redraw should retain a current presentation")
            .layout();
        assert_ne!(
            layout
                .hit_test_on_surface(point, crate::popup::Surface::Parent)
                .and_then(|hit| hit.target().cloned()),
            Some(expected.clone()),
            "frame {frame} must not resurrect the abandoned parent hit region"
        );
        assert_eq!(
            layout
                .hit_test_on_surface(point, crate::popup::Surface::Native(popup))
                .and_then(|hit| hit.target().cloned()),
            Some(expected.clone()),
            "frame {frame} must retain the popup surface hit region"
        );
    }
}

#[test]
fn command_palette_uses_native_popup_work_when_backend_supports_it() {
    let mut platform = Platform::new(
        text_editor::shell(text_editor::State::default()),
        FakeBackend::default().with_native_popups(),
    );

    platform.start().expect("platform should start host");

    let window = platform.host().windows()[0].id();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("first redraw should present");
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
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("redraw should present the command palette");

    assert!(
        platform.backend().events().iter().any(|event| matches!(
            event,
            BackendEvent::PresentPopup {
                parent,
                id,
                size,
                clear_color,
                framework_glass_panes,
                material_regions,
            } if *parent == window
                && *id == interaction::CommandPalette::panel_id()
                && size.width() > 0
                && size.height() > 0
                && *clear_color == scene::Color::rgba(0, 0, 0, 0)
            && *framework_glass_panes == 1
            && *material_regions == 1
        )),
        "events: {:?}",
        platform.backend().events()
    );
    assert!(
        platform.backend().events().iter().any(|event| matches!(
            event,
            BackendEvent::SetIme { update }
                if update.parent() == window
                    && matches!(
                        update.target(),
                        Some(ime::Target::Popup { id, area })
                            if id == interaction::CommandPalette::panel_id()
                                && area.x() >= 0
                                && area.y() >= 0
                                && area.width() == 1
                                && area.height() > 0
                    )
        )),
        "the palette caret must be projected in popup-local coordinates"
    );
}

#[test]
fn deferred_parent_preparation_syncs_popup_but_holds_ime_until_submission() {
    let mut platform = Platform::new(
        text_editor::shell(text_editor::State::default()),
        FakeBackend::default().with_native_popups(),
    );

    platform.start().expect("platform should start host");
    let window = platform.host().windows()[0].id();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("first redraw should present");
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::KeyDown {
                key: input::Key::Character('p'),
                modifiers: input::Modifiers::new(true, true, false, false),
                text: None,
            },
        ))
        .expect("palette shortcut should prepare command palette state");

    platform.backend_mut().events.clear();
    platform.backend_mut().clear_popup_sync_counts();
    platform.backend_mut().defer_next_present();
    platform.backend_mut().defer_next_present();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("pending parent preparation should remain drawable");

    assert!(
        platform.backend().events().iter().any(|event| matches!(
            event,
            BackendEvent::RequestRedraw { window: requested } if *requested == window
        )),
        "a deferred candidate must continue on its window-local redraw clock"
    );
    assert!(
        !platform
            .backend()
            .events()
            .iter()
            .any(|event| matches!(event, BackendEvent::Poll)),
        "renderer preparation must not wait for an idle poll behind a continuous input stream"
    );
    assert!(
        platform.backend().events().iter().any(|event| matches!(
            event,
            BackendEvent::PresentPopup { parent, id, .. }
                if *parent == window && *id == interaction::CommandPalette::panel_id()
        )),
        "an independently presentable popup must not wait behind parent GPU preparation"
    );
    assert!(
        platform
            .backend()
            .events()
            .iter()
            .all(|event| !matches!(event, BackendEvent::SetIme { .. })),
        "candidate IME geometry must wait while the parent presentation is deferred"
    );
    assert_eq!(
        platform.backend().popup_sync_counts(),
        &[1],
        "popup synchronization must consume the semantic work exactly once"
    );

    platform.backend_mut().events.clear();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("unfinished backend preparation should continue on the next window frame");
    assert!(
        platform.backend().events().iter().any(|event| matches!(
            event,
            BackendEvent::RequestRedraw { window: requested } if *requested == window
        )),
        "an unfinished continuation must schedule its next bounded window-local slice"
    );
    assert!(
        !platform.backend().events().iter().any(|event| matches!(
            event,
            BackendEvent::Poll | BackendEvent::PresentPopup { .. } | BackendEvent::SetIme { .. }
        )),
        "an unfinished continuation must not re-enter or duplicate semantic child work"
    );

    platform.backend_mut().events.clear();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("ready parent should activate from its exact pending state");

    assert!(
        !platform
            .backend()
            .events()
            .iter()
            .any(|event| matches!(event, BackendEvent::PresentPopup { .. })),
        "activating the parent must not replay already-synchronized popup work"
    );
    assert_eq!(
        platform
            .backend()
            .events()
            .iter()
            .filter(|event| matches!(
                event,
                BackendEvent::SetIme { update }
                    if update.parent() == window
                        && matches!(
                            update.target(),
                            Some(ime::Target::Popup { id, .. })
                                if id == interaction::CommandPalette::panel_id()
                        )
            ))
            .count(),
        1,
        "the matching present-submitted parent epoch must release popup IME geometry exactly once"
    );
}

#[test]
fn closing_a_window_retires_its_pending_presentation_continuation() {
    let mut platform = Platform::new(
        text_editor::shell(text_editor::State::default()),
        FakeBackend::default().with_native_popups(),
    );

    platform.start().expect("platform should start host");
    let window = platform.host().windows()[0].id();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("first redraw should present");
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::KeyDown {
                key: input::Key::Character('p'),
                modifiers: input::Modifiers::new(true, true, false, false),
                text: None,
            },
        ))
        .expect("palette shortcut should prepare candidate state");
    platform.backend_mut().defer_next_present();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("candidate should defer");

    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::CloseRequested,
        ))
        .expect("window close should retire backend state");
    platform.backend_mut().events.clear();
    platform
        .continue_presentations()
        .expect("a stale poll wake should be harmless after teardown");

    assert!(platform.host().windows().is_empty());
    assert!(
        platform.backend().events().is_empty(),
        "retired pending work must not draw or expose popup/IME state"
    );
}

#[test]
fn popup_pointer_motion_without_presentation_does_not_close_native_popups() {
    let mut platform = Platform::new(
        text_editor::shell(text_editor::State::default()),
        FakeBackend::default().with_native_popups(),
    );

    platform.start().expect("platform should start host");

    let window = platform.host().windows()[0].id();
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("first redraw should present");
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
                modifiers: input::Modifiers::default(),
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
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("redraw should present the open menu");
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
    platform
        .handle_event(host::Event::window(
            window,
            host::WindowEvent::RedrawRequested,
        ))
        .expect("first redraw should present");
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
        0
    );

    assert!(platform.host_mut().shell_mut().run_next_task().is_some());
    platform.drain().expect("save completion should drain");

    assert!(path.exists());
    assert_eq!(
        platform.host().shell().runtime().state().last_status,
        format!("saved {}", text_editor::display_path(&path))
    );
    assert!(!platform.host().needs_poll());

    let _ = std::fs::remove_file(path);
}
