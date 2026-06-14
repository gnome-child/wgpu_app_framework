use winit::{
    event::MouseButton,
    keyboard::{Key as WinitKey, ModifiersState, NamedKey},
};

use crate::app::state::{PressSource, WindowState, action_invocation};
use crate::geometry::point;
use crate::{action, ui, window};

#[derive(Debug, Default)]
pub struct Outcome {
    pub events: Vec<ui::Event>,
    pub invocation: Option<action::Invocation>,
    pub redraw: bool,
}

pub fn pointer_moved(state: &mut WindowState, position: point::Logical) -> Outcome {
    let target = state.hit_test(position);
    let hover_events = state.set_hovered(target.clone());
    state.cursor_position = Some(position);

    let redraw = !hover_events.is_empty();
    let mut events = hover_events;
    events.push(ui::Event::PointerMoved { position, target });

    Outcome {
        events,
        invocation: None,
        redraw,
    }
}

pub fn pointer_pressed(
    state: &mut WindowState,
    position: point::Logical,
    button: ui::Button,
) -> Outcome {
    let target = state.hit_test(position);
    let event = state.pointer_down(position, target, button);

    Outcome {
        events: vec![event],
        invocation: None,
        redraw: true,
    }
}

pub fn pointer_released<T>(
    registry: &action::Registry<T>,
    state: &mut WindowState,
    window: window::Id,
    position: point::Logical,
    button: ui::Button,
) -> Outcome {
    let target = state.hit_test(position);
    let (event, invoke_target) = state.pointer_up(position, target, button);
    let invocation = invoke_target.and_then(|target| {
        action_invocation(
            registry,
            &state.actions,
            window,
            target,
            action::Source::Pointer,
        )
    });

    Outcome {
        events: vec![event],
        invocation,
        redraw: true,
    }
}

pub fn pointer_button(button: MouseButton) -> Option<ui::Button> {
    match button {
        MouseButton::Left => Some(ui::Button::Left),
        MouseButton::Right => Some(ui::Button::Right),
        MouseButton::Middle => Some(ui::Button::Middle),
        MouseButton::Back | MouseButton::Forward => None,
        MouseButton::Other(value) => Some(ui::Button::Other(value)),
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

    Outcome {
        events: vec![event],
        invocation: None,
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
    let mut invocation = None;

    if matches!(key, ui::Key::Enter | ui::Key::Space)
        && state.pressed_source == Some(PressSource::Keyboard)
    {
        let pressed = state.pressed.take();
        state.pressed_source = None;
        redraw = pressed.is_some();

        if pressed == target {
            invocation = target.and_then(|target| {
                action_invocation(
                    registry,
                    &state.actions,
                    window,
                    target,
                    action::Source::Keyboard,
                )
            });
        }
    }

    Outcome {
        events: vec![event],
        invocation,
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

    registry.can_invoke(*action, action::Context::path(window, path.clone()))
}

fn invokable_focused_path<T>(
    registry: &action::Registry<T>,
    state: &WindowState,
    window: window::Id,
) -> Option<ui::Path> {
    let target = state.focused_path()?;
    action_invocation(
        registry,
        &state.actions,
        window,
        target.clone(),
        action::Source::Keyboard,
    )
    .map(|_| target)
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

        let outcome = pointer_pressed(&mut state, point::logical(1.0, 1.0), ui::Button::Left);

        assert!(outcome.redraw);
        assert_eq!(state.focused_path(), Some(path(CHILD)));
        assert_eq!(
            state.focus.as_ref().map(|focus| focus.reason),
            Some(ui::focus::Reason::Pointer)
        );
        assert_eq!(state.focus_visibility(), ui::focus::Visibility::Hidden);
        assert_eq!(state.pressed, Some(path(CHILD)));
    }

    #[test]
    fn released_control_returns_contextual_action_invocation() {
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
            ui::Button::Left,
        );

        assert_eq!(
            outcome.invocation,
            Some(action::Invocation::new(
                CLICK,
                action::Source::Pointer,
                action::Context::path(window, path(CHILD))
            ))
        );
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
            released.invocation,
            Some(action::Invocation::new(
                CLICK,
                action::Source::Keyboard,
                action::Context::path(window, path(CHILD))
            ))
        );
    }
}
