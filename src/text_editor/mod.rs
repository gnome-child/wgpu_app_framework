#![allow(dead_code, unused_imports)]

mod command;
mod event;
mod runtime;
#[cfg(test)]
mod smoke;
mod state;
mod target;
mod view;

pub use command::{LoadStressText, ToggleDebugPanel, ToggleWrapText};
pub use event::Event;
pub use runtime::{app, native_shell, run, runner, runtime, shell};
#[cfg(test)]
pub use smoke::smoke;
pub use state::{STRESS_TEXT, State};
pub use view::{CANVAS_COLOR, WINDOW_TITLE, compact_path, view, window_size};
