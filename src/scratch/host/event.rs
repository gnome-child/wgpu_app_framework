use std::path::PathBuf;

use crate::text;

use super::super::{geometry, input, interaction, shell, window};

pub enum Event {
    Started,
    Window {
        window: window::Id,
        event: WindowEvent,
    },
    FilePathSelected {
        window: window::Id,
        path: Option<PathBuf>,
    },
    Poll,
}

pub enum WindowEvent {
    Resized {
        size: geometry::Size,
    },
    RedrawRequested,
    CloseRequested,
    PointerMoved {
        point: geometry::Point,
    },
    PointerDown {
        point: geometry::Point,
    },
    PointerUp {
        point: geometry::Point,
    },
    PointerLeft,
    Scrolled {
        point: geometry::Point,
        delta: interaction::ScrollDelta,
    },
    KeyDown {
        key: input::Key,
        modifiers: input::Modifiers,
        text: Option<String>,
    },
    TextCommitted {
        text: String,
    },
    TextPreedit {
        preedit: text::edit::Preedit,
    },
}

impl Event {
    pub fn window(window: window::Id, event: WindowEvent) -> Self {
        Self::Window { window, event }
    }
}

impl WindowEvent {
    pub(super) fn into_shell_event(self, window: window::Id) -> shell::Event {
        match self {
            Self::Resized { size } => shell::Event::WindowResized { window, size },
            Self::RedrawRequested => shell::Event::RedrawRequested { window },
            Self::CloseRequested => shell::Event::CloseRequested { window },
            Self::PointerMoved { point } => shell::Event::PointerMoved { window, point },
            Self::PointerDown { point } => shell::Event::PointerDown { window, point },
            Self::PointerUp { point } => shell::Event::PointerUp { window, point },
            Self::PointerLeft => shell::Event::PointerLeft { window },
            Self::Scrolled { point, delta } => shell::Event::Scrolled {
                window,
                point,
                delta,
            },
            Self::KeyDown {
                key,
                modifiers,
                text,
            } => shell::Event::KeyDown {
                window,
                key,
                modifiers,
                text,
            },
            Self::TextCommitted { text } => shell::Event::TextCommitted { window, text },
            Self::TextPreedit { preedit } => shell::Event::TextPreedit { window, preedit },
        }
    }
}
