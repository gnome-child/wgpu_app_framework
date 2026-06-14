use std::collections::HashMap;

use crate::geometry::point;
use crate::{action, layout, ui, window};

#[derive(Debug, Default)]
pub struct WindowState {
    pub hovered: Option<ui::Path>,
    pub focused: Option<ui::Path>,
    pub pressed: Option<ui::Path>,
    pub cursor_position: Option<point::Logical>,
    pub layout: Option<layout::Box>,
    pub actions: HashMap<ui::Path, action::Id>,
    pub interactivity: HashMap<ui::Path, ui::Interactivity>,
}

impl WindowState {
    pub fn hit_test(&self, position: point::Logical) -> Option<ui::Path> {
        self.layout.as_ref().and_then(|layout| {
            layout.hit_test_where(position, |path| {
                self.interactivity
                    .get(path)
                    .is_some_and(|interactivity| interactivity.hit_test())
            })
        })
    }

    pub fn is_focusable(&self, target: &ui::Path) -> bool {
        self.interactivity
            .get(target)
            .is_some_and(|interactivity| interactivity.focusable())
    }

    pub fn is_actionable(&self, target: &ui::Path) -> bool {
        self.interactivity
            .get(target)
            .is_some_and(|interactivity| interactivity.actionable())
    }

    pub fn set_hovered(&mut self, target: Option<ui::Path>) -> Vec<ui::Event> {
        if self.hovered == target {
            return Vec::new();
        }

        let old = self.hovered.clone();
        self.hovered = target.clone();
        let mut events = Vec::new();

        if let Some(target) = old {
            events.push(ui::Event::PointerLeft { target });
        }

        if let Some(target) = target {
            events.push(ui::Event::PointerEntered { target });
        }

        events
    }

    pub fn pointer_down(
        &mut self,
        position: point::Logical,
        target: Option<ui::Path>,
        button: ui::Button,
    ) -> ui::Event {
        self.focused = target.clone().filter(|target| self.is_focusable(target));
        self.pressed = target.clone();

        ui::Event::PointerDown {
            position,
            target,
            button,
        }
    }

    pub fn pointer_up(
        &mut self,
        position: point::Logical,
        target: Option<ui::Path>,
        button: ui::Button,
    ) -> (ui::Event, Option<ui::Path>) {
        let pressed = self.pressed.take();
        let routed_target = pressed.clone().or(target);
        let invoke = if button == ui::Button::Left {
            pressed
        } else {
            None
        }
        .filter(|target| self.is_actionable(target));

        (
            ui::Event::PointerUp {
                position,
                target: routed_target,
                button,
            },
            invoke,
        )
    }
}

pub fn action_invocation<T>(
    registry: &action::Registry<T>,
    bindings: &HashMap<ui::Path, action::Id>,
    window: window::Id,
    target: ui::Path,
    source: action::Source,
) -> Option<action::Invocation> {
    let action = *bindings.get(&target)?;
    let context = action::Context::path(window, target);

    if !registry.can_invoke(action, context.clone()) {
        return None;
    }

    Some(action::Invocation::new(action, source, context))
}

pub fn resolve_action_path(
    state: Option<&WindowState>,
    requested_path: Option<ui::Path>,
) -> Option<ui::Path> {
    requested_path
        .or_else(|| state.and_then(|state| state.focused.clone().or_else(|| state.hovered.clone())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Action;
    use crate::geometry::{Rect, area};

    const ROOT: ui::Id = ui::Id::new("root");
    const CHILD: ui::Id = ui::Id::new("child");
    const OUTSIDE: ui::Id = ui::Id::new("outside");
    const CLICK: action::Id = action::Id::new("click");

    fn path(id: ui::Id) -> ui::Path {
        ui::Path::from(id)
    }

    #[test]
    fn hover_changes_emit_leave_then_enter() {
        let mut state = WindowState {
            hovered: Some(path(ROOT)),
            ..WindowState::default()
        };

        let events = state.set_hovered(Some(path(CHILD)));

        assert_eq!(
            events,
            vec![
                ui::Event::PointerLeft { target: path(ROOT) },
                ui::Event::PointerEntered {
                    target: path(CHILD)
                }
            ]
        );
    }

    #[test]
    fn pointer_down_updates_focused_element() {
        let mut state = WindowState {
            interactivity: HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };

        let event = state.pointer_down(
            point::logical(1.0, 2.0),
            Some(path(CHILD)),
            ui::Button::Left,
        );

        assert_eq!(state.focused, Some(path(CHILD)));
        assert_eq!(state.pressed, Some(path(CHILD)));
        assert_eq!(
            event,
            ui::Event::PointerDown {
                position: point::logical(1.0, 2.0),
                target: Some(path(CHILD)),
                button: ui::Button::Left
            }
        );
    }

    #[test]
    fn passive_pointer_down_does_not_focus_element() {
        let mut state = WindowState::default();

        state.pointer_down(
            point::logical(1.0, 2.0),
            Some(path(CHILD)),
            ui::Button::Left,
        );

        assert_eq!(state.focused, None);
        assert_eq!(state.pressed, Some(path(CHILD)));
    }

    #[test]
    fn pointer_capture_routes_release_to_pressed_element() {
        let mut state = WindowState {
            pressed: Some(path(CHILD)),
            interactivity: HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };

        let (event, invoke) = state.pointer_up(
            point::logical(50.0, 50.0),
            Some(path(OUTSIDE)),
            ui::Button::Left,
        );

        assert_eq!(
            event,
            ui::Event::PointerUp {
                position: point::logical(50.0, 50.0),
                target: Some(path(CHILD)),
                button: ui::Button::Left
            }
        );
        assert_eq!(invoke, Some(path(CHILD)));
    }

    #[test]
    fn non_primary_release_does_not_invoke_action() {
        let mut state = WindowState {
            pressed: Some(path(CHILD)),
            interactivity: HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };

        let (_, invoke) = state.pointer_up(
            point::logical(1.0, 1.0),
            Some(path(CHILD)),
            ui::Button::Right,
        );

        assert_eq!(invoke, None);
    }

    #[test]
    fn passive_pressed_element_does_not_invoke_action() {
        let mut state = WindowState {
            pressed: Some(path(CHILD)),
            ..WindowState::default()
        };

        let (_, invoke) = state.pointer_up(
            point::logical(1.0, 1.0),
            Some(path(CHILD)),
            ui::Button::Left,
        );

        assert_eq!(invoke, None);
    }

    #[test]
    fn focused_context_wins_over_hovered_context() {
        let state = WindowState {
            hovered: Some(path(ROOT)),
            focused: Some(path(CHILD)),
            ..WindowState::default()
        };

        assert_eq!(resolve_action_path(Some(&state), None), Some(path(CHILD)));
    }

    #[test]
    fn requested_context_wins_over_ambient_focus() {
        let state = WindowState {
            focused: Some(path(CHILD)),
            ..WindowState::default()
        };

        assert_eq!(
            resolve_action_path(Some(&state), Some(path(ROOT))),
            Some(path(ROOT))
        );
    }

    #[test]
    fn pointer_release_over_pressed_action_emits_contextual_action() {
        let window = window::Id::new(1);
        let mut state = WindowState {
            layout: Some(layout::Box::new(
                CHILD,
                Rect::new(point::logical(0.0, 0.0), area::logical(10.0, 10.0)),
                Vec::new(),
            )),
            actions: HashMap::from([(path(CHILD), CLICK)]),
            interactivity: HashMap::from([(path(CHILD), ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };
        let mut registry = action::Registry::<()>::new();

        registry.register(Action::new(CLICK, "Click"));
        state.pointer_down(
            point::logical(1.0, 1.0),
            Some(path(CHILD)),
            ui::Button::Left,
        );
        let (_, target) = state.pointer_up(
            point::logical(1.0, 1.0),
            Some(path(CHILD)),
            ui::Button::Left,
        );
        let invocation = action_invocation(
            &registry,
            &state.actions,
            window,
            target.expect("release should target pressed element"),
            action::Source::Pointer,
        );

        assert_eq!(
            invocation,
            Some(action::Invocation::new(
                CLICK,
                action::Source::Pointer,
                action::Context::path(window, path(CHILD))
            ))
        );
    }

    #[test]
    fn disabled_action_bound_node_does_not_invoke() {
        let window = window::Id::new(1);
        let context = action::Context::path(window, path(CHILD));
        let mut registry = action::Registry::<()>::new();
        let bindings = HashMap::from([(path(CHILD), CLICK)]);

        registry.register(Action::new(CLICK, "Click"));
        registry.set_state(CLICK, context, action::State::disabled());

        assert_eq!(
            action_invocation(
                &registry,
                &bindings,
                window,
                path(CHILD),
                action::Source::Pointer
            ),
            None
        );
    }
}
