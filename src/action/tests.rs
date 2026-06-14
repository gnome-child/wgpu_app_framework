use super::*;
use crate::{ui, window};

const SELECT_ALL: Id = Id::new("select_all");
const TEXT_BOX: ui::Id = ui::Id::new("text_box");

#[test]
fn unregistered_action_is_disabled() {
    let registry = Registry::<()>::new();
    let context = Context::path(window::Id::new(1), ui::Path::from(TEXT_BOX));

    assert_eq!(registry.state(SELECT_ALL, context), State::disabled());
}

#[test]
fn context_specific_state_wins_over_window_fallback() {
    let mut registry = Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(Action::new(SELECT_ALL, "Select All"));
    registry.set_state(SELECT_ALL, Context::window(window), State::disabled());
    registry.set_state(
        SELECT_ALL,
        Context::path(window, ui::Path::from(TEXT_BOX)),
        State::active(),
    );

    assert_eq!(
        registry.state(SELECT_ALL, Context::path(window, ui::Path::from(TEXT_BOX))),
        State::active()
    );
}

#[test]
fn disabled_action_cannot_invoke() {
    let mut registry = Registry::<()>::new();
    let context = Context::path(window::Id::new(1), ui::Path::from(TEXT_BOX));

    registry.register(Action::new(SELECT_ALL, "Select All"));
    registry.set_state(SELECT_ALL, context.clone(), State::disabled());

    assert!(!registry.can_invoke(SELECT_ALL, context));
}

#[test]
fn busy_action_cannot_invoke() {
    let mut registry = Registry::<()>::new();
    let context = Context::path(window::Id::new(1), ui::Path::from(TEXT_BOX));

    registry.register(Action::new(SELECT_ALL, "Select All"));
    registry.set_busy(SELECT_ALL, context.clone(), true);

    assert!(registry.state(SELECT_ALL, context.clone()).is_enabled());
    assert!(registry.state(SELECT_ALL, context.clone()).is_busy());
    assert!(!registry.can_invoke(SELECT_ALL, context));
}

#[test]
fn busy_state_is_distinct_from_active_state() {
    let state = State::active().with_busy(true);

    assert!(state.is_active());
    assert!(state.is_busy());

    let state = state.with_active(false);

    assert!(!state.is_active());
    assert!(state.is_busy());
}

#[test]
fn clearing_context_states_keeps_window_fallback() {
    let mut registry = Registry::<()>::new();
    let window = window::Id::new(1);

    registry.register(Action::new(SELECT_ALL, "Select All"));
    registry.set_state(SELECT_ALL, Context::window(window), State::disabled());
    registry.set_state(
        SELECT_ALL,
        Context::path(window, ui::Path::from(TEXT_BOX)),
        State::active(),
    );

    registry.clear_context_states(window);

    assert_eq!(
        registry.state(SELECT_ALL, Context::path(window, ui::Path::from(TEXT_BOX))),
        State::disabled()
    );
}

#[test]
fn enabled_action_invokes_behavior_and_emits_event() {
    let mut registry = Registry::<i32>::new();
    let window = window::Id::new(1);

    registry.register(Action::new(SELECT_ALL, "Select All").emit(|_| 7));

    assert_eq!(
        registry.execute(Invocation::new(
            SELECT_ALL,
            Source::Programmatic,
            Context::window(window)
        )),
        Some(Effect::Emit(7))
    );
}

#[test]
fn batched_action_effect_preserves_event_order() {
    let mut registry = Registry::<&'static str>::new();
    let window = window::Id::new(1);

    registry.register(
        Action::new(SELECT_ALL, "Select All").on_invoke(|_| Effect::Batch(vec!["first", "second"])),
    );

    assert_eq!(
        registry.execute(Invocation::new(
            SELECT_ALL,
            Source::Programmatic,
            Context::window(window)
        )),
        Some(Effect::Batch(vec!["first", "second"]))
    );
}

#[test]
fn sync_execution_does_not_make_action_active_or_busy() {
    let mut registry = Registry::<()>::new();
    let window = window::Id::new(1);
    let context = Context::path(window, ui::Path::from(TEXT_BOX));

    registry.register(Action::new(SELECT_ALL, "Select All"));
    assert_eq!(
        registry.execute(Invocation::new(
            SELECT_ALL,
            Source::Programmatic,
            context.clone()
        )),
        Some(Effect::None)
    );

    assert!(!registry.state(SELECT_ALL, context.clone()).is_active());
    assert!(!registry.state(SELECT_ALL, context).is_busy());
}

#[test]
fn window_busy_state_falls_back_to_path_context() {
    let mut registry = Registry::<()>::new();
    let window = window::Id::new(1);
    let path = ui::Path::from(TEXT_BOX);

    registry.register(Action::new(SELECT_ALL, "Select All"));
    registry.set_busy(SELECT_ALL, Context::window(window), true);

    assert!(
        registry
            .state(SELECT_ALL, Context::path(window, path))
            .is_busy()
    );
}

#[test]
fn state_accessors_expose_enabled_active_and_busy_flags() {
    let state = State::new(true, false).with_active(true).with_busy(true);

    assert!(state.is_enabled());
    assert!(state.is_active());
    assert!(state.is_busy());
}
