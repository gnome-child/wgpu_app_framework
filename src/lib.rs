extern crate self as wgpu_l3;

pub mod clipboard;
pub mod command;
pub mod context;
pub mod diagnostics;
pub mod document;
pub mod error;
pub mod geometry;
pub mod host;
pub mod icon;
pub mod input;
pub mod interaction;
pub mod keymap;
pub mod notification;
pub mod platform;
pub mod pointer;
pub mod responder;
pub mod response;
pub mod runtime;
pub mod scene;
pub mod session;
pub mod shell;
pub mod state;
pub mod subject;
pub mod target;
pub mod task;
pub mod text;
pub mod theme;
pub mod timeline;
pub mod view;
pub mod widget;
pub mod window;

mod animation;
mod composition;
mod draft;
mod layout;
mod overlay;
mod paint;
mod render;

#[cfg(test)]
#[path = "../examples/control_gallery/app/mod.rs"]
mod control_gallery;
#[cfg(test)]
#[path = "../examples/glass_tuner/app/mod.rs"]
mod glass_tuner;
#[cfg(test)]
#[path = "../examples/text_editor/app/mod.rs"]
mod text_editor;

pub use clipboard::Clipboard;
pub use command::Command;
pub use context::Context;
pub use diagnostics::Diagnostics;
pub use document::Document;
pub use error::Error;
pub use host::Host;
pub use input::Input;
pub use notification::{Listener, Notification};
pub use platform::Platform;
pub use responder::Responder;
pub use response::Response;
pub use runtime::Runtime;
pub use scene::Scene;
pub use session::Session;
pub use shell::Shell;
pub use state::State;
pub use subject::Segment as Subject;
pub use target::Target;
pub use task::Task;
pub use theme::Theme;
pub use timeline::Timeline;
pub use view::View;
pub use widget::Widget;

#[cfg(test)]
mod tests;
