use winit::event::MouseButton;

use crate::app::state::{WindowState, action_invocation};
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::Action;

    use super::*;

    const CHILD: ui::Id = ui::Id::new("child");
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
        assert_eq!(state.focused, Some(path(CHILD)));
        assert_eq!(state.pressed, Some(path(CHILD)));
    }

    #[test]
    fn released_control_returns_contextual_action_invocation() {
        let window = window::Id::new(1);
        let mut registry = action::Registry::<()>::new();
        let mut state = WindowState {
            pressed: Some(path(CHILD)),
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
}
