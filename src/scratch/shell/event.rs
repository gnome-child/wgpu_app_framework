use std::path::PathBuf;

use crate::text;

use crate::scratch::{Error, geometry, input, interaction, state::State, window};

use super::{Shell, Work};

pub enum Event {
    Started,
    WindowResized {
        window: window::Id,
        size: geometry::Size,
    },
    RedrawRequested {
        window: window::Id,
    },
    CloseRequested {
        window: window::Id,
    },
    PointerMoved {
        window: window::Id,
        point: geometry::Point,
    },
    PointerDown {
        window: window::Id,
        point: geometry::Point,
    },
    PointerUp {
        window: window::Id,
        point: geometry::Point,
    },
    PointerLeft {
        window: window::Id,
    },
    Scrolled {
        window: window::Id,
        point: geometry::Point,
        delta: interaction::ScrollDelta,
    },
    KeyDown {
        window: window::Id,
        key: input::Key,
        modifiers: input::Modifiers,
        text: Option<String>,
    },
    TextCommitted {
        window: window::Id,
        text: String,
    },
    TextPreedit {
        window: window::Id,
        preedit: text::edit::Preedit,
    },
    FilePathSelected {
        window: window::Id,
        path: Option<PathBuf>,
    },
    Poll,
}

impl<M: State, E: Send + 'static> Shell<M, E> {
    pub fn handle_event(&mut self, event: Event) -> Result<Work, Error> {
        match event {
            Event::Started => {
                self.start();
                Ok(self.drain())
            }
            Event::WindowResized { window, size } => {
                self.set_window_size(window, size);
                Ok(self.drain())
            }
            Event::RedrawRequested { window } => {
                self.runtime.request_redraw(window);
                Ok(self.drain())
            }
            Event::CloseRequested { window } => {
                self.request_close_window(window);
                Ok(self.drain())
            }
            Event::PointerMoved { window, point } => {
                self.pointer_move(window, point)?;
                Ok(self.drain())
            }
            Event::PointerDown { window, point } => {
                self.pointer_down(window, point)?;
                Ok(self.drain())
            }
            Event::PointerUp { window, point } => {
                self.pointer_up(window, point)?;
                Ok(self.drain())
            }
            Event::PointerLeft { window } => {
                self.pointer_left(window)?;
                Ok(self.drain())
            }
            Event::Scrolled {
                window,
                point,
                delta,
            } => {
                self.scroll(window, point, delta)?;
                Ok(self.drain())
            }
            Event::KeyDown {
                window,
                key,
                modifiers,
                text,
            } => {
                self.handle_input(
                    window,
                    input::Input::key_down_with_text(key, modifiers, text),
                )?;
                Ok(self.drain())
            }
            Event::TextCommitted { window, text } => {
                self.handle_input(window, input::Input::text_commit(text))?;
                Ok(self.drain())
            }
            Event::TextPreedit { window, preedit } => {
                self.handle_input(window, input::Input::text_preedit(preedit))?;
                Ok(self.drain())
            }
            Event::FilePathSelected { window, path } => {
                self.file_path_selected(window, path)?;
                Ok(self.drain())
            }
            Event::Poll => Ok(self.step()),
        }
    }
}
