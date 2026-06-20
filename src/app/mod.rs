mod action_executor;
mod clipboard;
mod command;
mod context;
mod drag_drop;
mod floating;
mod focus;
mod input;
mod mailbox;
mod rendering;
mod runtime;
pub(crate) mod scroll;
mod sender;
mod state;
mod task_runner;
mod text_input;
mod view;
mod windows;

use crate::{event, native, render, ui, window};
use thiserror::Error;

pub use context::{ActionState, Context, Diagnostics, ScrollDiagnostics};
pub use sender::{SendError, Sender};

use mailbox::Message;
use runtime::Runtime;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("event loop error")]
    EventLoop(#[from] winit::error::EventLoopError),

    #[error("native platform error")]
    Native(#[from] native::Error),

    #[error("render error")]
    Render(#[from] render::Error),
}

pub trait Application {
    type Event: Send + 'static;

    fn started(&mut self, _cx: &mut Context<'_, Self::Event>) {}

    fn event(&mut self, _cx: &mut Context<'_, Self::Event>, _event: event::Event<Self::Event>) {}

    fn view(
        &mut self,
        _cx: &mut Context<'_, Self::Event>,
        _window: window::Id,
        _tree: &mut ui::Tree,
    ) {
    }
}

trait MailboxSender<T: Send + 'static> {
    fn send_message(&self, message: Message<T>) -> std::result::Result<(), SendError>;
}

pub fn run<A: Application>(app: A) -> Result<()> {
    let event_loop =
        winit::event_loop::EventLoop::<Message<A::Event>>::with_user_event().build()?;
    let sender = sender::new(event_loop.create_proxy());
    let mut runtime = Runtime::new(app, sender);

    event_loop.run_app(&mut runtime)?;

    if let Some(error) = runtime.take_error() {
        return Err(error);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error;

    use super::*;

    #[test]
    fn error_can_be_matched_directly_by_variant() {
        let error = Error::Render(render::Error::from(
            render::surface::Error::NoSurfaceConfiguration,
        ));

        assert!(matches!(
            error,
            Error::Render(render::Error::Surface(
                render::surface::Error::NoSurfaceConfiguration
            ))
        ));
        assert_eq!(error.to_string(), "render error");
        assert_eq!(
            error::Error::source(&error)
                .expect("render error should have source")
                .to_string(),
            "surface could not be configured"
        );
        assert!(error::Error::source(&error).is_some());
    }
}
