pub mod action;
pub mod app;
pub mod event;
pub mod geometry;
pub mod icon;
pub mod layout;
pub mod menu;
pub mod paint;
pub mod pointer;
pub mod task;
pub mod text;
pub mod theme;
pub mod ui;
pub mod widget;
pub mod window;

mod native;
mod render;
mod text_backend;

pub use action::Action;
pub use event::Event;
pub use icon::Icon;
pub use menu::{Bar, Menu};
pub use task::Task;
pub use theme::Theme;

#[cfg(test)]
mod tests {
    #[test]
    fn architecture_audit_notes_file_exists() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ARCHITECTURE_AUDIT.txt");
        let notes = std::fs::read_to_string(path).expect("architecture audit notes should exist");

        assert!(notes.contains("Widget extension boundary"));
        assert!(notes.contains("Pointer capture ownership"));
        assert!(notes.contains("Text measurement quantization"));
    }
}
