use std::path::PathBuf;

use crate::text;

use super::super::{geometry, input, interaction, pointer, shell, window};

pub enum Event {
    Started,
    Window {
        window: window::Id,
        event: WindowEvent,
    },
    Popup {
        parent: window::Id,
        popup: interaction::Id,
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
        button: pointer::Button,
        modifiers: input::Modifiers,
    },
    PointerUp {
        point: geometry::Point,
        button: pointer::Button,
    },
    PointerLeft,
    ModifiersChanged {
        modifiers: input::Modifiers,
    },
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
        preedit: text::Preedit,
    },
}

impl Event {
    pub fn window(window: window::Id, event: WindowEvent) -> Self {
        Self::Window { window, event }
    }

    pub(crate) fn popup(parent: window::Id, popup: interaction::Id, event: WindowEvent) -> Self {
        Self::Popup {
            parent,
            popup,
            event,
        }
    }

    pub(crate) fn window_id(&self) -> Option<window::Id> {
        match self {
            Self::Window { window, .. } | Self::FilePathSelected { window, .. } => Some(*window),
            Self::Popup { parent, .. } => Some(*parent),
            Self::Started | Self::Poll => None,
        }
    }
}

impl WindowEvent {
    pub(super) fn into_shell_event(self, window: window::Id) -> shell::Event {
        match self {
            Self::Resized { size } => shell::Event::WindowResized { window, size },
            Self::RedrawRequested => shell::Event::RedrawRequested { window },
            Self::CloseRequested => shell::Event::CloseRequested { window },
            Self::PointerMoved { point } => shell::Event::PointerMoved { window, point },
            Self::PointerDown {
                point,
                button,
                modifiers,
            } => shell::Event::PointerDown {
                window,
                point,
                button,
                modifiers,
            },
            Self::PointerUp { point, button } => shell::Event::PointerUp {
                window,
                point,
                button,
            },
            Self::PointerLeft => shell::Event::PointerLeft { window },
            Self::ModifiersChanged { modifiers } => {
                shell::Event::ModifiersChanged { window, modifiers }
            }
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

    pub(super) fn into_popup_shell_event(
        self,
        window: window::Id,
        popup: interaction::Id,
    ) -> shell::Event {
        match self {
            Self::PointerMoved { point } => shell::Event::PopupPointerMoved {
                window,
                popup,
                point,
            },
            Self::PointerDown {
                point,
                button,
                modifiers,
            } => shell::Event::PopupPointerDown {
                window,
                popup,
                point,
                button,
                modifiers,
            },
            Self::PointerUp { point, button } => shell::Event::PopupPointerUp {
                window,
                popup,
                point,
                button,
            },
            Self::PointerLeft => shell::Event::PopupPointerLeft { window, popup },
            Self::ModifiersChanged { modifiers } => shell::Event::PopupModifiersChanged {
                window,
                popup,
                modifiers,
            },
            Self::Scrolled { point, delta } => shell::Event::PopupScrolled {
                window,
                popup,
                point,
                delta,
            },
            event => event.into_shell_event(window),
        }
    }
}
