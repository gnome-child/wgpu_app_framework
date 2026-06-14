use thiserror::Error;
use winit::event_loop::EventLoopProxy;

use crate::app::mailbox::Message;
use crate::event;

#[derive(Debug)]
pub struct Sender<T: Send + 'static> {
    proxy: EventLoopProxy<Message<T>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("app event loop is closed")]
pub struct SendError;

impl<T: Send + 'static> Sender<T> {
    pub fn emit(&self, event: T) -> Result<(), SendError> {
        self.proxy.send_event(message(event)).map_err(|_| SendError)
    }
}

impl<T: Send + 'static> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            proxy: self.proxy.clone(),
        }
    }
}

pub fn new<T: Send + 'static>(proxy: EventLoopProxy<Message<T>>) -> Sender<T> {
    Sender { proxy }
}

fn message<T>(event: T) -> Message<T> {
    Message::Event(event::Event::App(event))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_event_is_wrapped_as_mailbox_message() {
        assert_eq!(message(7), Message::Event(event::Event::App(7)));
    }
}
