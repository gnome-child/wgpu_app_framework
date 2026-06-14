use crate::app::mailbox::Mailbox;
use crate::{action, window};

pub fn execute<T>(
    actions: &mut action::Registry<T>,
    invocation: action::Invocation,
    request_redraw: &mut impl FnMut(window::Id),
) -> Option<action::Effect<T>> {
    let action = invocation.action();
    let context = invocation.context().clone();
    let window = context.window_id();

    if !actions.can_invoke(action, context) {
        return None;
    }

    request_redraw(window);
    let effect = actions.execute(invocation)?;
    request_redraw(window);

    Some(effect)
}

pub fn enqueue_effect<T>(mailbox: &mut Mailbox<T>, effect: action::Effect<T>) {
    match effect {
        action::Effect::None => {}
        action::Effect::Emit(event) => {
            mailbox.push_app(event);
        }
        action::Effect::Batch(events) => {
            for event in events {
                mailbox.push_app(event);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::mailbox::Message;

    use super::*;

    const WORK: action::Id = action::Id::new("work");

    #[test]
    fn skips_disabled_actions_without_redraw() {
        let window = window::Id::new(1);
        let context = action::Context::window(window);
        let mut registry = action::Registry::<i32>::new();
        let mut redraws = Vec::new();

        registry.register(crate::Action::new(WORK, "Work").emit(|_| 1));
        registry.set_state(WORK, context.clone(), action::State::disabled());

        let effect = execute(
            &mut registry,
            action::Invocation::new(WORK, action::Source::Programmatic, context),
            &mut |window| redraws.push(window),
        );

        assert_eq!(effect, None);
        assert!(redraws.is_empty());
    }

    #[test]
    fn enabled_actions_redraw_around_execution_and_enqueue_effects() {
        let window = window::Id::new(1);
        let context = action::Context::window(window);
        let mut registry = action::Registry::<i32>::new();
        let mut mailbox = Mailbox::new();
        let mut redraws = Vec::new();

        registry.register(crate::Action::new(WORK, "Work").emit(|_| 7));
        let effect = execute(
            &mut registry,
            action::Invocation::new(WORK, action::Source::Programmatic, context),
            &mut |window| redraws.push(window),
        );

        enqueue_effect(&mut mailbox, effect.expect("action should run"));

        assert_eq!(redraws, vec![window, window]);
        assert_eq!(mailbox.pop(), Some(Message::Event(crate::Event::App(7))));
    }
}
