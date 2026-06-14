use winit::{
    event::MouseButton,
    keyboard::{Key as WinitKey, ModifiersState, NamedKey},
};

use crate::app::state::{PressSource, WindowState, action_request};
use crate::geometry::point;
use crate::{action, pointer, ui, window};

#[derive(Debug, Default)]
pub struct Outcome {
    pub events: Vec<ui::Event>,
    pub request: Option<action::Request>,
    pub redraw: bool,
}

pub fn pointer_moved(state: &mut WindowState, position: point::Logical) -> Outcome {
    state
        .pointer
        .handle_event(pointer::Event::Moved { position });
    let delta = state.pointer.delta();
    let target = state.hit_test(position);
    let hover_events = state.set_hovered(target.clone());

    let redraw = !hover_events.is_empty();
    let mut events = hover_events;
    events.push(ui::Event::PointerMoved {
        position,
        delta,
        target,
    });

    Outcome {
        events,
        request: None,
        redraw,
    }
}

pub fn pointer_pressed(
    state: &mut WindowState,
    position: point::Logical,
    button: pointer::Button,
) -> Outcome {
    state.pointer.handle_event(pointer::Event::Button {
        button,
        pressed: true,
    });
    let target = state.hit_test(position);
    let event = state.pointer_down(position, state.pointer.delta(), target, button);

    Outcome {
        events: vec![event],
        request: None,
        redraw: true,
    }
}

pub fn pointer_released<T>(
    registry: &action::Registry<T>,
    state: &mut WindowState,
    window: window::Id,
    position: point::Logical,
    button: pointer::Button,
) -> Outcome {
    state.pointer.handle_event(pointer::Event::Button {
        button,
        pressed: false,
    });
    let target = state.hit_test(position);
    let (event, invoke_target) = state.pointer_up(position, state.pointer.delta(), target, button);
    let request = invoke_target
        .and_then(|target| action_request(state, window, target, action::Source::Pointer))
        .filter(|request| registry.can_invoke(request.action(), request.target().clone()));

    Outcome {
        events: vec![event],
        request,
        redraw: true,
    }
}

pub fn pointer_button(button: MouseButton) -> pointer::Button {
    match button {
        MouseButton::Left => pointer::Button::Primary,
        MouseButton::Right => pointer::Button::Secondary,
        MouseButton::Middle => pointer::Button::Middle,
        MouseButton::Back => pointer::Button::Back,
        MouseButton::Forward => pointer::Button::Forward,
        MouseButton::Other(value) => pointer::Button::Other(value),
    }
}

pub fn pointer_left(state: &mut WindowState) -> Outcome {
    state.pointer.handle_event(pointer::Event::Left);
    let events = state.set_hovered(None);
    let cleared_pressed = if state.pressed_source == Some(PressSource::Pointer) {
        state.pressed = None;
        state.pressed_source = None;
        true
    } else {
        false
    };

    Outcome {
        redraw: !events.is_empty() || cleared_pressed,
        events,
        request: None,
    }
}

pub fn modifiers(modifiers: ModifiersState) -> ui::Modifiers {
    ui::Modifiers::new(
        modifiers.shift_key(),
        modifiers.control_key(),
        modifiers.alt_key(),
        modifiers.super_key(),
    )
}

pub fn key(key: &WinitKey) -> ui::Key {
    match key {
        WinitKey::Named(NamedKey::Tab) => ui::Key::Tab,
        WinitKey::Named(NamedKey::Enter) => ui::Key::Enter,
        WinitKey::Named(NamedKey::Space) => ui::Key::Space,
        WinitKey::Character(value) => {
            let mut chars = value.chars();
            let Some(character) = chars.next() else {
                return ui::Key::Other;
            };

            if chars.next().is_none() {
                ui::Key::Character(character.to_ascii_lowercase())
            } else {
                ui::Key::Other
            }
        }
        _ => ui::Key::Other,
    }
}

pub fn key_pressed<T>(
    registry: &action::Registry<T>,
    state: &mut WindowState,
    window: window::Id,
    key: ui::Key,
    repeat: bool,
) -> Outcome {
    let target = state.focused_path();
    let mut redraw = false;

    let event = ui::Event::KeyDown {
        key,
        modifiers: state.modifiers,
        target: target.clone(),
        repeat,
    };

    match key {
        ui::Key::Tab => {
            let reverse = state.modifiers.shift();
            if let Some(path) = next_focus(registry, state, window, reverse) {
                redraw |= state.set_focus(
                    path,
                    ui::focus::Reason::Keyboard,
                    ui::focus::Visibility::Visible,
                );
            }
        }
        ui::Key::Enter | ui::Key::Space if !repeat => {
            if let Some(path) = invokable_focused_path(registry, state, window) {
                redraw |= state.pressed.as_ref() != Some(&path)
                    || state.pressed_source != Some(PressSource::Keyboard);
                state.pressed = Some(path);
                state.pressed_source = Some(PressSource::Keyboard);
            }
        }
        _ => {}
    }

    let request = if !repeat {
        shortcut_request(registry, state, window, key)
    } else {
        None
    };

    Outcome {
        events: vec![event],
        request,
        redraw,
    }
}

pub fn key_released<T>(
    registry: &action::Registry<T>,
    state: &mut WindowState,
    window: window::Id,
    key: ui::Key,
) -> Outcome {
    let target = state.focused_path();
    let event = ui::Event::KeyUp {
        key,
        modifiers: state.modifiers,
        target: target.clone(),
    };
    let mut redraw = false;
    let mut request = None;

    if matches!(key, ui::Key::Enter | ui::Key::Space)
        && state.pressed_source == Some(PressSource::Keyboard)
    {
        let pressed = state.pressed.take();
        state.pressed_source = None;
        redraw = pressed.is_some();

        if pressed == target {
            request = target
                .and_then(|target| action_request(state, window, target, action::Source::Keyboard))
                .filter(|request| registry.can_invoke(request.action(), request.target().clone()));
        }
    }

    Outcome {
        events: vec![event],
        request,
        redraw,
    }
}

fn next_focus<T>(
    registry: &action::Registry<T>,
    state: &WindowState,
    window: window::Id,
    reverse: bool,
) -> Option<ui::Path> {
    let order = focusable_paths(registry, state, window);
    if order.is_empty() {
        return None;
    }

    let current = state
        .focused_path()
        .and_then(|focused| order.iter().position(|path| path == &focused));
    let next = match (current, reverse) {
        (Some(0), true) | (None, true) => order.len() - 1,
        (Some(index), true) => index - 1,
        (Some(index), false) => (index + 1) % order.len(),
        (None, false) => 0,
    };

    Some(order[next].clone())
}

fn focusable_paths<T>(
    registry: &action::Registry<T>,
    state: &WindowState,
    window: window::Id,
) -> Vec<ui::Path> {
    state
        .focus_order
        .iter()
        .filter(|path| can_focus(registry, state, window, path))
        .cloned()
        .collect()
}

fn can_focus<T>(
    registry: &action::Registry<T>,
    state: &WindowState,
    window: window::Id,
    path: &ui::Path,
) -> bool {
    if !state.is_focusable(path) {
        return false;
    }

    let Some(action) = state.actions.get(path) else {
        return true;
    };

    registry.can_invoke(*action, state.action_context_for_path(window, path))
}

fn invokable_focused_path<T>(
    registry: &action::Registry<T>,
    state: &WindowState,
    window: window::Id,
) -> Option<ui::Path> {
    let target = state.focused_path()?;
    action_request(state, window, target.clone(), action::Source::Keyboard)
        .filter(|request| registry.can_invoke(request.action(), request.target().clone()))
        .map(|_| target)
}

fn shortcut_request<T>(
    registry: &action::Registry<T>,
    state: &WindowState,
    window: window::Id,
    key: ui::Key,
) -> Option<action::Request> {
    let shortcut = action::Shortcut::new(key.normalized(), state.modifiers);
    let action = registry.shortcut_action(shortcut)?;
    let context = state.command_context(window);

    Some(action::Request::new(
        action,
        action::Source::Shortcut,
        context,
    ))
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::Action;

    use super::*;

    const CHILD: ui::Id = ui::Id::new("child");
    const SECOND: ui::Id = ui::Id::new("second");
    const CLICK: action::Id = action::Id::new("click");

    fn path(id: ui::Id) -> ui::Path {
        ui::Path::from(id)
    }

    #[test]
    fn pressed_control_focuses_and_requests_redraw() {
        let mut state = WindowState {
            interactivity: HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };

        state.layout = Some(crate::layout::Box::new(
            CHILD,
            crate::geometry::Rect::new(
                point::logical(0.0, 0.0),
                crate::geometry::area::logical(10.0, 10.0),
            ),
            Vec::new(),
        ));

        let outcome = pointer_pressed(
            &mut state,
            point::logical(1.0, 1.0),
            pointer::Button::Primary,
        );

        assert!(outcome.redraw);
        assert!(state.pointer.primary_down());
        assert_eq!(state.focused_path(), Some(path(CHILD)));
        assert_eq!(
            state.focus.as_ref().map(|focus| focus.reason),
            Some(ui::focus::Reason::Pointer)
        );
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Hidden);
        assert_eq!(state.pressed, Some(path(CHILD)));
    }

    #[test]
    fn released_control_returns_contextual_action_request() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = WindowState {
            pressed: Some(path(CHILD)),
            pressed_source: Some(PressSource::Pointer),
            actions: HashMap::from([(path(CHILD), CLICK)]),
            interactivity: HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };

        state.layout = Some(crate::layout::Box::new(
            CHILD,
            crate::geometry::Rect::new(
                point::logical(0.0, 0.0),
                crate::geometry::area::logical(10.0, 10.0),
            ),
            Vec::new(),
        ));
        registry.register(Action::new(CLICK, "Click"));

        let outcome = pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(1.0, 1.0),
            pointer::Button::Primary,
        );

        assert_eq!(
            outcome.request,
            Some(
                action::Request::new(
                    CLICK,
                    action::Source::Pointer,
                    action::Context::path(window, path(CHILD))
                )
                .with_origin(path(CHILD))
            )
        );
        assert!(!state.pointer.primary_down());
    }

    #[test]
    fn winit_mouse_buttons_map_to_pointer_buttons() {
        assert_eq!(pointer_button(MouseButton::Left), pointer::Button::Primary);
        assert_eq!(
            pointer_button(MouseButton::Right),
            pointer::Button::Secondary
        );
        assert_eq!(pointer_button(MouseButton::Middle), pointer::Button::Middle);
        assert_eq!(pointer_button(MouseButton::Back), pointer::Button::Back);
        assert_eq!(
            pointer_button(MouseButton::Forward),
            pointer::Button::Forward
        );
        assert_eq!(
            pointer_button(MouseButton::Other(9)),
            pointer::Button::Other(9)
        );
    }

    #[test]
    fn character_keys_are_normalized() {
        assert_eq!(
            key(&WinitKey::Character("A".into())),
            ui::Key::Character('a')
        );
        assert_eq!(key(&WinitKey::Character("ab".into())), ui::Key::Other);
    }

    #[test]
    fn pointer_movement_uses_pointer_delta_and_hover_order() {
        let mut state = WindowState {
            hovered: Some(path(CHILD)),
            interactivity: HashMap::from([(path(SECOND), ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };
        state.layout = Some(crate::layout::Box::new(
            SECOND,
            crate::geometry::Rect::new(
                point::logical(0.0, 0.0),
                crate::geometry::area::logical(20.0, 20.0),
            ),
            Vec::new(),
        ));
        let entered = pointer_moved(&mut state, point::logical(2.0, 3.0));

        assert_eq!(
            entered.events,
            vec![
                ui::Event::PointerLeft {
                    target: path(CHILD)
                },
                ui::Event::PointerEntered {
                    target: path(SECOND)
                },
                ui::Event::PointerMoved {
                    position: point::logical(2.0, 3.0),
                    delta: point::logical(0.0, 0.0),
                    target: Some(path(SECOND)),
                }
            ]
        );

        let outcome = pointer_moved(&mut state, point::logical(5.0, 8.0));

        assert_eq!(state.pointer.position(), Some(point::logical(5.0, 8.0)));
        assert_eq!(
            state.pointer.previous_position(),
            Some(point::logical(2.0, 3.0))
        );
        assert_eq!(state.pointer.delta(), point::logical(3.0, 5.0));
        assert_eq!(
            outcome.events,
            vec![ui::Event::PointerMoved {
                position: point::logical(5.0, 8.0),
                delta: point::logical(3.0, 5.0),
                target: Some(path(SECOND)),
            }]
        );
    }

    #[test]
    fn pointer_left_clears_pointer_and_hover() {
        let mut state = WindowState {
            hovered: Some(path(CHILD)),
            ..WindowState::default()
        };
        state.pointer.handle_event(pointer::Event::Moved {
            position: point::logical(2.0, 3.0),
        });
        state.pointer.handle_event(pointer::Event::Button {
            button: pointer::Button::Primary,
            pressed: true,
        });

        let outcome = pointer_left(&mut state);

        assert_eq!(state.pointer.position(), None);
        assert_eq!(state.pointer.delta(), point::logical(0.0, 0.0));
        assert!(!state.pointer.primary_down());
        assert_eq!(state.hovered, None);
        assert_eq!(
            outcome.events,
            vec![ui::Event::PointerLeft {
                target: path(CHILD)
            }]
        );
        assert!(outcome.redraw);
    }

    #[test]
    fn pointer_left_clears_pointer_press_state() {
        let mut state = WindowState {
            pressed: Some(path(CHILD)),
            pressed_source: Some(PressSource::Pointer),
            ..WindowState::default()
        };

        let outcome = pointer_left(&mut state);

        assert_eq!(state.pressed, None);
        assert_eq!(state.pressed_source, None);
        assert!(outcome.redraw);
    }

    #[test]
    fn tab_moves_focus_in_layout_order_and_shows_outline() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut state = WindowState {
            focus_order: vec![path(CHILD), path(SECOND)],
            interactivity: HashMap::from([
                (path(CHILD), ui::Interactivity::CONTROL),
                (path(SECOND), ui::Interactivity::CONTROL),
            ]),
            ..WindowState::default()
        };

        let outcome = key_pressed(&registry, &mut state, window, ui::Key::Tab, false);

        assert!(outcome.redraw);
        assert_eq!(state.focused_path(), Some(path(CHILD)));
        assert_eq!(
            state.focus.as_ref().map(|focus| focus.reason),
            Some(ui::focus::Reason::Keyboard)
        );
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Visible);
        assert_eq!(
            outcome.events,
            vec![ui::Event::KeyDown {
                key: ui::Key::Tab,
                modifiers: ui::Modifiers::default(),
                target: None,
                repeat: false,
            }]
        );

        key_pressed(&registry, &mut state, window, ui::Key::Tab, false);

        assert_eq!(state.focused_path(), Some(path(SECOND)));
    }

    #[test]
    fn shift_tab_moves_focus_backward() {
        let window = window::Id::new(1);
        let registry = action::Registry::<()>::new();
        let mut state = WindowState {
            focus_order: vec![path(CHILD), path(SECOND)],
            modifiers: ui::Modifiers::new(true, false, false, false),
            interactivity: HashMap::from([
                (path(CHILD), ui::Interactivity::CONTROL),
                (path(SECOND), ui::Interactivity::CONTROL),
            ]),
            ..WindowState::default()
        };

        key_pressed(&registry, &mut state, window, ui::Key::Tab, false);

        assert_eq!(state.focused_path(), Some(path(SECOND)));
    }

    #[test]
    fn disabled_action_bound_controls_are_skipped_by_tab_focus() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = WindowState {
            focus_order: vec![path(CHILD), path(SECOND)],
            actions: HashMap::from([(path(CHILD), CLICK)]),
            interactivity: HashMap::from([
                (path(CHILD), ui::Interactivity::CONTROL),
                (path(SECOND), ui::Interactivity::CONTROL),
            ]),
            ..WindowState::default()
        };

        registry.register(Action::new(CLICK, "Click"));
        registry.set_state(
            CLICK,
            action::Context::path(window, path(CHILD)),
            action::State::disabled(),
        );
        key_pressed(&registry, &mut state, window, ui::Key::Tab, false);

        assert_eq!(state.focused_path(), Some(path(SECOND)));
    }

    #[test]
    fn enter_releases_focused_action_with_keyboard_source() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = WindowState {
            focus: Some(crate::app::state::Focus::new(
                path(CHILD),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            actions: HashMap::from([(path(CHILD), CLICK)]),
            interactivity: HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };

        registry.register(Action::new(CLICK, "Click"));
        let pressed = key_pressed(&registry, &mut state, window, ui::Key::Enter, false);

        assert!(pressed.redraw);
        assert_eq!(state.pressed, Some(path(CHILD)));
        assert_eq!(state.pressed_source, Some(PressSource::Keyboard));

        let released = key_released(&registry, &mut state, window, ui::Key::Enter);

        assert!(released.redraw);
        assert_eq!(state.pressed, None);
        assert_eq!(state.pressed_source, None);
        assert_eq!(
            released.request,
            Some(
                action::Request::new(
                    CLICK,
                    action::Source::Keyboard,
                    action::Context::path(window, path(CHILD))
                )
                .with_origin(path(CHILD))
            )
        );
    }

    #[test]
    fn shortcut_press_emits_shortcut_request_for_command_target() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = WindowState {
            modifiers: ui::Modifiers::new(false, true, false, false),
            command_subject: Some(action::Scope::Path(path(SECOND))),
            responders: HashMap::from([(path(SECOND), vec![action::SELECT_ALL])]),
            ..WindowState::default()
        };

        registry.register(
            Action::new(action::SELECT_ALL, "Select All")
                .with_shortcut(action::Shortcut::control('a')),
        );

        let outcome = key_pressed(
            &registry,
            &mut state,
            window,
            ui::Key::Character('a'),
            false,
        );

        assert_eq!(
            outcome.request,
            Some(action::Request::new(
                action::SELECT_ALL,
                action::Source::Shortcut,
                action::Context::path(window, path(SECOND))
            ))
        );
    }

    #[test]
    fn shortcut_and_command_button_use_same_automatic_subject() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = WindowState {
            focus: Some(crate::app::state::Focus::new(
                path(SECOND),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            modifiers: ui::Modifiers::new(false, true, false, false),
            responders: HashMap::from([(path(SECOND), vec![action::SELECT_ALL])]),
            actions: HashMap::from([(path(CHILD), action::SELECT_ALL)]),
            action_targets: HashMap::from([(path(CHILD), ui::ActionTarget::Command)]),
            ..WindowState::default()
        };

        registry.register(
            Action::new(action::SELECT_ALL, "Select All")
                .with_shortcut(action::Shortcut::control('a')),
        );

        let shortcut = key_pressed(
            &registry,
            &mut state,
            window,
            ui::Key::Character('a'),
            false,
        )
        .request
        .expect("shortcut should request the action");
        let button = action_request(&state, window, path(CHILD), action::Source::Pointer)
            .expect("command button should request the action");

        assert_eq!(shortcut.target(), button.target());
        assert_eq!(
            shortcut.target(),
            &action::Context::path(window, path(SECOND))
        );
    }

    #[test]
    fn repeated_shortcut_press_does_not_emit_request() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = WindowState {
            modifiers: ui::Modifiers::new(false, true, false, false),
            command_subject: Some(action::Scope::Path(path(SECOND))),
            responders: HashMap::from([(path(SECOND), vec![action::SELECT_ALL])]),
            ..WindowState::default()
        };

        registry.register(
            Action::new(action::SELECT_ALL, "Select All")
                .with_shortcut(action::Shortcut::control('a')),
        );

        let outcome = key_pressed(&registry, &mut state, window, ui::Key::Character('a'), true);

        assert_eq!(outcome.request, None);
    }

    #[test]
    fn command_target_control_uses_stored_command_target() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = WindowState {
            pressed: Some(path(CHILD)),
            pressed_source: Some(PressSource::Pointer),
            command_subject: Some(action::Scope::Path(path(SECOND))),
            actions: HashMap::from([(path(CHILD), CLICK)]),
            action_targets: HashMap::from([(path(CHILD), ui::ActionTarget::Command)]),
            interactivity: HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };

        state.layout = Some(crate::layout::Box::new(
            CHILD,
            crate::geometry::Rect::new(
                point::logical(0.0, 0.0),
                crate::geometry::area::logical(10.0, 10.0),
            ),
            Vec::new(),
        ));
        registry.register(Action::new(CLICK, "Click"));

        let outcome = pointer_released(
            &registry,
            &mut state,
            window,
            point::logical(1.0, 1.0),
            pointer::Button::Primary,
        );

        assert_eq!(
            outcome.request,
            Some(
                action::Request::new(
                    CLICK,
                    action::Source::Pointer,
                    action::Context::path(window, path(SECOND))
                )
                .with_origin(path(CHILD))
            )
        );
    }

    #[test]
    fn window_target_control_uses_window_context() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = WindowState {
            focus: Some(crate::app::state::Focus::new(
                path(CHILD),
                ui::focus::Reason::Keyboard,
                ui::focus::Visibility::Visible,
            )),
            command_subject: Some(action::Scope::Path(path(SECOND))),
            actions: HashMap::from([(path(CHILD), CLICK)]),
            action_targets: HashMap::from([(path(CHILD), ui::ActionTarget::Window)]),
            interactivity: HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };

        registry.register(Action::new(CLICK, "Click"));
        key_pressed(&registry, &mut state, window, ui::Key::Space, false);
        let outcome = key_released(&registry, &mut state, window, ui::Key::Space);

        assert_eq!(
            outcome.request,
            Some(
                action::Request::new(
                    CLICK,
                    action::Source::Keyboard,
                    action::Context::window(window)
                )
                .with_origin(path(CHILD))
            )
        );
    }
}
