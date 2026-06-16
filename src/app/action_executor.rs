use std::thread;

use crate::app::mailbox::Mailbox;
use crate::app::mailbox::Message;
use crate::app::{MailboxSender, Sender};
use crate::{action, window};

pub fn execute<T: Send + 'static>(
    actions: &mut action::Registry<T>,
    request: action::Request,
    spawn: impl FnOnce(action::Invocation, crate::Task<T>),
    request_redraw: &mut impl FnMut(window::Id),
) -> Option<action::Effect<T>> {
    let action = request.action();
    let context = request.target().clone();
    let window = context.window_id();

    if !actions.can_execute(&request) {
        return None;
    }

    let invocation = action::Invocation::from(request);
    let effect = actions.execute(invocation.clone())?;

    if let action::Effect::Task(task) = effect {
        actions.set_busy(action, context, true);
        request_redraw(window);
        spawn(invocation, task);
        return None;
    }

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
        action::Effect::Task(_) => {}
    }
}

pub fn complete_task<T>(
    actions: &mut action::Registry<T>,
    invocation: action::Invocation,
    request_redraw: &mut impl FnMut(window::Id),
) {
    let action = invocation.action();
    let context = invocation.context().clone();
    let window = context.window_id();

    if actions.set_busy(action, context, false) {
        request_redraw(window);
    }
}

pub fn spawn_task<T: Send + 'static>(
    invocation: action::Invocation,
    task: crate::Task<T>,
    sender: Sender<T>,
) {
    thread::spawn(move || {
        let event = task.run();
        let _ = sender.send_message(Message::ActionTaskCompleted { invocation, event });
    });
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
            action::Request::new(WORK, action::Source::Programmatic, context),
            |_, _| panic!("disabled action should not spawn a task"),
            &mut |window| redraws.push(window),
        );

        assert_eq!(effect, None);
        assert!(redraws.is_empty());
    }

    #[test]
    fn enabled_sync_actions_enqueue_effects_without_busy_redraw() {
        let window = window::Id::new(1);
        let context = action::Context::window(window);
        let mut registry = action::Registry::<i32>::new();
        let mut mailbox = Mailbox::new();
        let mut redraws = Vec::new();

        registry.register(crate::Action::new(WORK, "Work").emit(|_| 7));
        let effect = execute(
            &mut registry,
            action::Request::new(WORK, action::Source::Programmatic, context),
            |_, _| panic!("sync action should not spawn a task"),
            &mut |window| redraws.push(window),
        );

        enqueue_effect(&mut mailbox, effect.expect("action should run"));

        assert!(redraws.is_empty());
        assert_eq!(mailbox.pop(), Some(Message::Event(crate::Event::App(7))));
    }

    #[test]
    fn task_actions_set_busy_and_spawn_completion_work() {
        let window = window::Id::new(1);
        let context = action::Context::window(window);
        let invocation =
            action::Invocation::new(WORK, action::Source::Programmatic, context.clone());
        let request = action::Request::new(WORK, action::Source::Programmatic, context.clone());
        let mut registry = action::Registry::<i32>::new();
        let mut redraws = Vec::new();
        let mut completed = None;

        registry
            .register(crate::Action::new(WORK, "Work").task(|_| crate::Task::future(async { 7 })));

        let effect = execute(
            &mut registry,
            request,
            |invocation, task| completed = Some((invocation, task.run())),
            &mut |window| redraws.push(window),
        );

        assert_eq!(effect, None);
        assert!(registry.state(WORK, context).is_busy());
        assert_eq!(redraws, vec![window]);
        assert_eq!(completed, Some((invocation, 7)));
    }

    #[test]
    fn completing_task_clears_busy_and_requests_redraw() {
        let window = window::Id::new(1);
        let context = action::Context::window(window);
        let mut registry = action::Registry::<i32>::new();
        let mut redraws = Vec::new();

        registry.register(crate::Action::new(WORK, "Work"));
        registry.set_busy(WORK, context.clone(), true);

        complete_task(
            &mut registry,
            action::Invocation::new(WORK, action::Source::Programmatic, context.clone()),
            &mut |window| redraws.push(window),
        );

        assert!(!registry.state(WORK, context).is_busy());
        assert_eq!(redraws, vec![window]);
    }
}
