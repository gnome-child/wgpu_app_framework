mod access;
mod event;
mod input;
mod lifecycle;
mod presentation;
mod task;
mod window;
mod work;

pub use event::Event;
pub use presentation::Presentation;
pub use window::Window;
pub use work::Work;

use super::{runtime, state::State, view};

pub struct Shell<M: State, E: Send + 'static = ()> {
    runtime: runtime::Runtime<M, E, view::View>,
    windows: Vec<Window>,
}
