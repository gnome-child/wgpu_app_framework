use std::fmt;

use crate::command::Error as FrameworkError;

#[derive(Debug)]
pub enum Error<E> {
    Framework(FrameworkError),
    Backend(E),
}

#[derive(Debug)]
pub enum RunError<E> {
    EventLoop(winit::error::EventLoopError),
    Platform(Error<E>),
}

impl<E: fmt::Display> fmt::Display for Error<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Framework(error) => write!(formatter, "framework error: {error}"),
            Self::Backend(error) => write!(formatter, "backend error: {error}"),
        }
    }
}

impl<E> std::error::Error for Error<E>
where
    E: std::error::Error + fmt::Debug + fmt::Display + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Framework(error) => Some(error),
            Self::Backend(error) => Some(error),
        }
    }
}

impl<E: fmt::Display> fmt::Display for RunError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EventLoop(error) => write!(formatter, "event loop error: {error}"),
            Self::Platform(error) => write!(formatter, "platform error: {error}"),
        }
    }
}

impl<E> std::error::Error for RunError<E>
where
    E: std::error::Error + fmt::Debug + fmt::Display + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::EventLoop(error) => Some(error),
            Self::Platform(error) => Some(error),
        }
    }
}

impl<E> From<winit::error::EventLoopError> for RunError<E> {
    fn from(error: winit::error::EventLoopError) -> Self {
        Self::EventLoop(error)
    }
}

impl<E> From<Error<E>> for RunError<E> {
    fn from(error: Error<E>) -> Self {
        Self::Platform(error)
    }
}
