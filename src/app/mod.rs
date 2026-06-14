mod context;
mod mailbox;
mod runtime;
mod state;

use thiserror::Error;
use winit::event_loop::EventLoop;

use crate::{event, native, render, ui, window};

pub use context::{ActionState, Context};

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

pub fn run<A: Application>(app: A) -> Result<()> {
    let event_loop = EventLoop::new()?;
    let mut runtime = Runtime::new(app);

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
