use std::collections::VecDeque;

use crate::{action, event};

#[derive(Debug, Clone, PartialEq)]
pub enum Message<T> {
    Event(event::Event<T>),
    RunAction(action::Invocation),
    ActionTaskCompleted {
        invocation: action::Invocation,
        event: T,
    },
    AppTaskCompleted(T),
}

#[derive(Debug)]
pub struct Mailbox<T> {
    events: VecDeque<Message<T>>,
}

impl<T> Mailbox<T> {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }

    pub fn push(&mut self, event: event::Event<T>) {
        self.push_message(Message::Event(event));
    }

    pub fn push_message(&mut self, message: Message<T>) {
        self.events.push_back(message);
    }

    pub fn run_action(&mut self, invocation: action::Invocation) {
        self.push_message(Message::RunAction(invocation));
    }

    pub fn push_app(&mut self, event: T) {
        self.push(event::Event::App(event));
    }

    pub fn pop(&mut self) -> Option<Message<T>> {
        self.events.pop_front()
    }
}

impl<T> Default for Mailbox<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::window;

    const CLICK: action::Id = action::Id::new("click");

    #[test]
    fn mailbox_drains_events_fifo() {
        let mut mailbox = Mailbox::new();
        let window = window::Id::new(1);

        mailbox.push_app(1);
        mailbox.run_action(action::Invocation::new(
            CLICK,
            action::Source::Programmatic,
            action::Context::window(window),
        ));

        assert_eq!(mailbox.pop(), Some(Message::Event(event::Event::App(1))));
        assert_eq!(
            mailbox.pop(),
            Some(Message::RunAction(action::Invocation::new(
                CLICK,
                action::Source::Programmatic,
                action::Context::window(window),
            )))
        );
        assert_eq!(mailbox.pop(), None);
    }

    #[test]
    fn events_emitted_while_handling_are_deferred() {
        let mut mailbox = Mailbox::new();

        mailbox.push_app(1);
        mailbox.push_app(2);

        assert_eq!(mailbox.pop(), Some(Message::Event(event::Event::App(1))));
        mailbox.push_app(3);

        assert_eq!(mailbox.pop(), Some(Message::Event(event::Event::App(2))));
        assert_eq!(mailbox.pop(), Some(Message::Event(event::Event::App(3))));
        assert_eq!(mailbox.pop(), None);
    }

    #[test]
    fn action_requests_are_queued_in_fifo_order() {
        let mut mailbox = Mailbox::<()>::new();
        let window = window::Id::new(1);

        mailbox.run_action(action::Invocation::new(
            CLICK,
            action::Source::Pointer,
            action::Context::window(window),
        ));
        mailbox.push_app(());

        assert_eq!(
            mailbox.pop(),
            Some(Message::RunAction(action::Invocation::new(
                CLICK,
                action::Source::Pointer,
                action::Context::window(window),
            )))
        );
        assert_eq!(mailbox.pop(), Some(Message::Event(event::Event::App(()))));
    }

    #[test]
    fn user_event_messages_share_fifo_order() {
        let mut mailbox = Mailbox::new();

        mailbox.push_message(Message::Event(event::Event::App(1)));
        mailbox.push_app(2);

        assert_eq!(mailbox.pop(), Some(Message::Event(event::Event::App(1))));
        assert_eq!(mailbox.pop(), Some(Message::Event(event::Event::App(2))));
        assert_eq!(mailbox.pop(), None);
    }

    #[test]
    fn action_task_completion_messages_share_fifo_order() {
        let mut mailbox = Mailbox::new();
        let window = window::Id::new(1);
        let invocation = action::Invocation::new(
            CLICK,
            action::Source::Programmatic,
            action::Context::window(window),
        );

        mailbox.push_app(1);
        mailbox.push_message(Message::ActionTaskCompleted {
            invocation: invocation.clone(),
            event: 2,
        });

        assert_eq!(mailbox.pop(), Some(Message::Event(event::Event::App(1))));
        assert_eq!(
            mailbox.pop(),
            Some(Message::ActionTaskCompleted {
                invocation,
                event: 2
            })
        );
    }

    #[test]
    fn app_task_completion_messages_share_fifo_order() {
        let mut mailbox = Mailbox::new();

        mailbox.push_app(1);
        mailbox.push_message(Message::AppTaskCompleted(2));

        assert_eq!(mailbox.pop(), Some(Message::Event(event::Event::App(1))));
        assert_eq!(mailbox.pop(), Some(Message::AppTaskCompleted(2)));
        assert_eq!(mailbox.pop(), None);
    }
}
