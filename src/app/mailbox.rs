use std::collections::VecDeque;

use crate::{Command, command, event};

#[derive(Debug)]
pub enum Message<T> {
    Event(event::Event<T>),
    RunCommand(command::call::Raw),
    RunCall(command::call::Any),
    CommandTaskCompleted {
        command: command::Key,
        context: command::call::Context,
        response: Result<command::Response<()>, command::registry::Rejection>,
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

    #[cfg(test)]
    pub fn run_command(&mut self, request: command::call::Raw) {
        self.push_message(Message::RunCommand(request));
    }

    pub fn run_any_call(&mut self, call: command::call::Any) {
        self.push_message(Message::RunCall(call));
    }

    pub fn run_call<C: Command>(&mut self, call: command::Call<C>) {
        self.run_any_call(command::call::Any::new(call));
    }

    pub fn push_app(&mut self, event: T) {
        self.push(event::Event::App(event));
    }

    pub fn pop(&mut self) -> Option<Message<T>> {
        self.events.pop_front()
    }
}

impl<T: PartialEq> PartialEq for Message<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Event(left), Self::Event(right)) => left == right,
            (Self::RunCommand(left), Self::RunCommand(right)) => left == right,
            (Self::RunCall(left), Self::RunCall(right)) => left == right,
            (
                Self::CommandTaskCompleted {
                    command: left_command,
                    context: left_context,
                    response: left_response,
                },
                Self::CommandTaskCompleted {
                    command: right_command,
                    context: right_context,
                    response: right_response,
                },
            ) => {
                left_command == right_command
                    && left_context == right_context
                    && left_response == right_response
            }
            (Self::AppTaskCompleted(left), Self::AppTaskCompleted(right)) => left == right,
            _ => false,
        }
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

    struct Click;

    impl Command for Click {
        type Args = ();
        type Output = ();

        const NAME: &'static str = "click";
        const DISPLAY: &'static str = "Click";
    }

    const CLICK: command::Key = command::Key::of::<Click>();

    #[test]
    fn mailbox_drains_events_fifo() {
        let mut mailbox = Mailbox::new();
        let window = window::Id::new(1);

        mailbox.push_app(1);
        mailbox.run_command(command::call::Raw::from_key(
            CLICK,
            command::call::Source::Programmatic,
            command::call::Context::window(window),
        ));

        assert_eq!(mailbox.pop(), Some(Message::Event(event::Event::App(1))));
        assert_eq!(
            mailbox.pop(),
            Some(Message::RunCommand(command::call::Raw::from_key(
                CLICK,
                command::call::Source::Programmatic,
                command::call::Context::window(window),
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
    fn command_requests_are_queued_in_fifo_order() {
        let mut mailbox = Mailbox::<()>::new();
        let window = window::Id::new(1);

        mailbox.run_command(command::call::Raw::from_key(
            CLICK,
            command::call::Source::Pointer,
            command::call::Context::window(window),
        ));
        mailbox.push_app(());

        assert_eq!(
            mailbox.pop(),
            Some(Message::RunCommand(command::call::Raw::from_key(
                CLICK,
                command::call::Source::Pointer,
                command::call::Context::window(window),
            )))
        );
        assert_eq!(mailbox.pop(), Some(Message::Event(event::Event::App(()))));
    }

    #[test]
    fn typed_command_calls_are_queued_as_any_calls() {
        let mut mailbox = Mailbox::<()>::new();
        let window = window::Id::new(1);
        let context = command::call::Context::window(window);
        let expected = command::Call::<Click>::for_context::<command::TestTarget>((), context)
            .expect("unit args should validate");

        mailbox.run_call(
            command::Call::<Click>::for_context::<command::TestTarget>(
                (),
                command::call::Context::window(window),
            )
            .expect("unit args should validate"),
        );

        assert_eq!(
            mailbox.pop(),
            Some(Message::RunCall(command::call::Any::new(expected)))
        );
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
    fn command_task_completion_messages_share_fifo_order() {
        let mut mailbox = Mailbox::new();
        let window = window::Id::new(1);
        let context = command::call::Context::window(window);

        mailbox.push_app(1);
        mailbox.push_message(Message::CommandTaskCompleted {
            command: CLICK,
            context: context.clone(),
            response: Ok(command::Response::none()),
        });

        assert_eq!(mailbox.pop(), Some(Message::Event(event::Event::App(1))));
        assert_eq!(
            mailbox.pop(),
            Some(Message::CommandTaskCompleted {
                command: CLICK,
                context,
                response: Ok(command::Response::none())
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
