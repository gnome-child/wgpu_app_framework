use std::collections::VecDeque;

use crate::event;

#[derive(Debug)]
pub(super) struct Mailbox<T> {
    events: VecDeque<event::Event<T>>,
}

impl<T> Mailbox<T> {
    pub(super) fn new() -> Self {
        Self {
            events: VecDeque::new(),
        }
    }

    pub(super) fn push(&mut self, event: event::Event<T>) {
        self.events.push_back(event);
    }

    pub(super) fn push_app(&mut self, event: T) {
        self.push(event::Event::App(event));
    }

    pub(super) fn pop(&mut self) -> Option<event::Event<T>> {
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
    use crate::{action, window};

    const CLICK: action::Id = action::Id::new("click");

    #[test]
    fn mailbox_drains_events_fifo() {
        let mut mailbox = Mailbox::new();
        let window = window::Id::new(1);

        mailbox.push_app(1);
        mailbox.push(event::Event::ActionInvoked {
            action: CLICK,
            source: action::Source::Programmatic,
            context: action::Context::window(window),
        });

        assert_eq!(mailbox.pop(), Some(event::Event::App(1)));
        assert_eq!(
            mailbox.pop(),
            Some(event::Event::ActionInvoked {
                action: CLICK,
                source: action::Source::Programmatic,
                context: action::Context::window(window),
            })
        );
        assert_eq!(mailbox.pop(), None);
    }

    #[test]
    fn events_emitted_while_handling_are_deferred() {
        let mut mailbox = Mailbox::new();

        mailbox.push_app(1);
        mailbox.push_app(2);

        assert_eq!(mailbox.pop(), Some(event::Event::App(1)));
        mailbox.push_app(3);

        assert_eq!(mailbox.pop(), Some(event::Event::App(2)));
        assert_eq!(mailbox.pop(), Some(event::Event::App(3)));
        assert_eq!(mailbox.pop(), None);
    }
}
