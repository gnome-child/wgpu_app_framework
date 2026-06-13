use std::collections::HashMap;

use crate::geometry::point;
use crate::{action, layout, ui, window};

#[derive(Debug, Default)]
pub(super) struct WindowState {
    pub(super) hovered: Option<ui::Id>,
    pub(super) focused: Option<ui::Id>,
    pub(super) pressed: Option<ui::Id>,
    pub(super) cursor_position: Option<point::Logical>,
    pub(super) layout: Option<layout::Box>,
    pub(super) actions: HashMap<ui::Id, action::Id>,
    pub(super) interactivity: HashMap<ui::Id, ui::Interactivity>,
}

impl WindowState {
    pub(super) fn hit_test(&self, position: point::Logical) -> Option<ui::Id> {
        self.layout.as_ref().and_then(|layout| {
            layout.hit_test_where(position, |id| {
                self.interactivity
                    .get(&id)
                    .is_some_and(|interactivity| interactivity.hit_test)
            })
        })
    }

    pub(super) fn is_focusable(&self, target: ui::Id) -> bool {
        self.interactivity
            .get(&target)
            .is_some_and(|interactivity| interactivity.focusable)
    }

    pub(super) fn is_actionable(&self, target: ui::Id) -> bool {
        self.interactivity
            .get(&target)
            .is_some_and(|interactivity| interactivity.actionable)
    }

    pub(super) fn set_hovered(&mut self, target: Option<ui::Id>) -> Vec<ui::Event> {
        if self.hovered == target {
            return Vec::new();
        }

        let old = self.hovered;
        self.hovered = target;
        let mut events = Vec::new();

        if let Some(target) = old {
            events.push(ui::Event::PointerLeft { target });
        }

        if let Some(target) = target {
            events.push(ui::Event::PointerEntered { target });
        }

        events
    }

    pub(super) fn pointer_down(
        &mut self,
        position: point::Logical,
        target: Option<ui::Id>,
        button: ui::Button,
    ) -> ui::Event {
        self.focused = target.filter(|target| self.is_focusable(*target));
        self.pressed = target;

        ui::Event::PointerDown {
            position,
            target,
            button,
        }
    }

    pub(super) fn pointer_up(
        &mut self,
        position: point::Logical,
        target: Option<ui::Id>,
        button: ui::Button,
    ) -> (ui::Event, Option<ui::Id>) {
        let pressed = self.pressed.take();
        let routed_target = pressed.or(target);
        let invoke = if button == ui::Button::Left {
            pressed
        } else {
            None
        }
        .filter(|target| self.is_actionable(*target));

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

pub(super) fn action_invocation_event(
    registry: &action::Registry,
    bindings: &HashMap<ui::Id, action::Id>,
    window: window::Id,
    target: ui::Id,
    source: action::Source,
) -> Option<ui::Event> {
    let action = *bindings.get(&target)?;
    let context = action::Context {
        window,
        target: Some(target),
    };

    if !registry.can_invoke(action, context) {
        return None;
    }

    Some(ui::Event::ActionInvoked {
        action,
        source,
        context,
    })
}

pub(super) fn resolve_action_target(
    state: Option<&WindowState>,
    requested_target: Option<ui::Id>,
) -> Option<ui::Id> {
    requested_target.or_else(|| state.and_then(|state| state.focused.or(state.hovered)))
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

    #[test]
    fn hover_changes_emit_leave_then_enter() {
        let mut state = WindowState {
            hovered: Some(ROOT),
            ..WindowState::default()
        };

        let events = state.set_hovered(Some(CHILD));

        assert_eq!(
            events,
            vec![
                ui::Event::PointerLeft { target: ROOT },
                ui::Event::PointerEntered { target: CHILD }
            ]
        );
    }

    #[test]
    fn pointer_down_updates_focused_element() {
        let mut state = WindowState {
            interactivity: HashMap::from([(CHILD, ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };

        let event = state.pointer_down(point::logical(1.0, 2.0), Some(CHILD), ui::Button::Left);

        assert_eq!(state.focused, Some(CHILD));
        assert_eq!(state.pressed, Some(CHILD));
        assert_eq!(
            event,
            ui::Event::PointerDown {
                position: point::logical(1.0, 2.0),
                target: Some(CHILD),
                button: ui::Button::Left
            }
        );
    }

    #[test]
    fn passive_pointer_down_does_not_focus_element() {
        let mut state = WindowState::default();

        state.pointer_down(point::logical(1.0, 2.0), Some(CHILD), ui::Button::Left);

        assert_eq!(state.focused, None);
        assert_eq!(state.pressed, Some(CHILD));
    }

    #[test]
    fn pointer_capture_routes_release_to_pressed_element() {
        let mut state = WindowState {
            pressed: Some(CHILD),
            interactivity: HashMap::from([(CHILD, ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };

        let (event, invoke) =
            state.pointer_up(point::logical(50.0, 50.0), Some(OUTSIDE), ui::Button::Left);

        assert_eq!(
            event,
            ui::Event::PointerUp {
                position: point::logical(50.0, 50.0),
                target: Some(CHILD),
                button: ui::Button::Left
            }
        );
        assert_eq!(invoke, Some(CHILD));
    }

    #[test]
    fn non_primary_release_does_not_invoke_action() {
        let mut state = WindowState {
            pressed: Some(CHILD),
            interactivity: HashMap::from([(CHILD, ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };

        let (_, invoke) =
            state.pointer_up(point::logical(1.0, 1.0), Some(CHILD), ui::Button::Right);

        assert_eq!(invoke, None);
    }

    #[test]
    fn passive_pressed_element_does_not_invoke_action() {
        let mut state = WindowState {
            pressed: Some(CHILD),
            ..WindowState::default()
        };

        let (_, invoke) = state.pointer_up(point::logical(1.0, 1.0), Some(CHILD), ui::Button::Left);

        assert_eq!(invoke, None);
    }

    #[test]
    fn focused_context_wins_over_hovered_context() {
        let state = WindowState {
            hovered: Some(ROOT),
            focused: Some(CHILD),
            ..WindowState::default()
        };

        assert_eq!(resolve_action_target(Some(&state), None), Some(CHILD));
    }

    #[test]
    fn requested_context_wins_over_ambient_focus() {
        let state = WindowState {
            focused: Some(CHILD),
            ..WindowState::default()
        };

        assert_eq!(resolve_action_target(Some(&state), Some(ROOT)), Some(ROOT));
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
            actions: HashMap::from([(CHILD, CLICK)]),
            interactivity: HashMap::from([(CHILD, ui::Interactivity::CONTROL)]),
            ..WindowState::default()
        };
        let mut registry = action::Registry::new();

        registry.register(Action::new(CLICK, "Click"));
        state.pointer_down(point::logical(1.0, 1.0), Some(CHILD), ui::Button::Left);
        let (_, target) = state.pointer_up(point::logical(1.0, 1.0), Some(CHILD), ui::Button::Left);
        let event = action_invocation_event(
            &registry,
            &state.actions,
            window,
            target.expect("release should target pressed element"),
            action::Source::Pointer,
        );

        assert_eq!(
            event,
            Some(ui::Event::ActionInvoked {
                action: CLICK,
                source: action::Source::Pointer,
                context: action::Context {
                    window,
                    target: Some(CHILD)
                }
            })
        );
    }

    #[test]
    fn disabled_action_bound_node_does_not_invoke() {
        let window = window::Id::new(1);
        let context = action::Context {
            window,
            target: Some(CHILD),
        };
        let mut registry = action::Registry::new();
        let bindings = HashMap::from([(CHILD, CLICK)]);

        registry.register(Action::new(CLICK, "Click"));
        registry.set_state(CLICK, context, action::State::disabled());

        assert_eq!(
            action_invocation_event(&registry, &bindings, window, CHILD, action::Source::Pointer),
            None
        );
    }
}
