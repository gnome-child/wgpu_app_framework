use std::path::PathBuf;
use std::time::Instant;

use crate::text;

use crate::{command::Error, geometry, input, interaction, pointer, state::State, window};

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
    PopupPointerMoved {
        window: window::Id,
        popup: interaction::Id,
        point: geometry::Point,
    },
    PointerDown {
        window: window::Id,
        point: geometry::Point,
        button: pointer::Button,
        modifiers: input::Modifiers,
    },
    PopupPointerDown {
        window: window::Id,
        popup: interaction::Id,
        point: geometry::Point,
        button: pointer::Button,
        modifiers: input::Modifiers,
    },
    PointerUp {
        window: window::Id,
        point: geometry::Point,
        button: pointer::Button,
    },
    PopupPointerUp {
        window: window::Id,
        popup: interaction::Id,
        point: geometry::Point,
        button: pointer::Button,
    },
    PointerLeft {
        window: window::Id,
    },
    PopupPointerLeft {
        window: window::Id,
        popup: interaction::Id,
    },
    ModifiersChanged {
        window: window::Id,
        modifiers: input::Modifiers,
    },
    PopupModifiersChanged {
        window: window::Id,
        popup: interaction::Id,
        modifiers: input::Modifiers,
    },
    Scrolled {
        window: window::Id,
        point: geometry::Point,
        delta: interaction::ScrollDelta,
    },
    PopupScrolled {
        window: window::Id,
        popup: interaction::Id,
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
        preedit: text::Preedit,
    },
    FilePathSelected {
        window: window::Id,
        path: Option<PathBuf>,
    },
    Poll,
}

impl Event {
    fn window_id(&self) -> Option<window::Id> {
        match self {
            Self::WindowResized { window, .. }
            | Self::RedrawRequested { window }
            | Self::CloseRequested { window }
            | Self::PointerMoved { window, .. }
            | Self::PopupPointerMoved { window, .. }
            | Self::PointerDown { window, .. }
            | Self::PopupPointerDown { window, .. }
            | Self::PointerUp { window, .. }
            | Self::PopupPointerUp { window, .. }
            | Self::PointerLeft { window }
            | Self::PopupPointerLeft { window, .. }
            | Self::ModifiersChanged { window, .. }
            | Self::PopupModifiersChanged { window, .. }
            | Self::Scrolled { window, .. }
            | Self::PopupScrolled { window, .. }
            | Self::KeyDown { window, .. }
            | Self::TextCommitted { window, .. }
            | Self::TextPreedit { window, .. }
            | Self::FilePathSelected { window, .. } => Some(*window),
            Self::Started | Self::Poll => None,
        }
    }
}

impl<M: State, E: Send + 'static> Shell<M, E> {
    pub fn handle_event(&mut self, event: Event) -> Result<Work, Error> {
        let window = event.window_id();
        let started_at = Instant::now();
        let mut redraw = None;
        let result = match event {
            Event::Started => {
                self.start();
                Ok(())
            }
            Event::WindowResized { window, size } => {
                self.set_window_size(window, size);
                Ok(())
            }
            Event::RedrawRequested { window } => {
                redraw = Some(window);
                Ok(())
            }
            Event::CloseRequested { window } => {
                self.request_close_window(window);
                Ok(())
            }
            Event::PointerMoved { window, point } => {
                self.pointer_move(window, point)?;
                Ok(())
            }
            Event::PopupPointerMoved {
                window,
                popup,
                point,
            } => {
                self.pointer_move_on_popup(window, popup, point)?;
                Ok(())
            }
            Event::PointerDown {
                window,
                point,
                button,
                modifiers,
            } => {
                self.pointer_down_with_modifiers(window, point, button, modifiers)?;
                Ok(())
            }
            Event::PopupPointerDown {
                window,
                popup,
                point,
                button,
                modifiers,
            } => {
                self.pointer_down_on_popup(window, popup, point, button, modifiers)?;
                Ok(())
            }
            Event::PointerUp {
                window,
                point,
                button,
            } => {
                self.pointer_up(window, point, button)?;
                Ok(())
            }
            Event::PopupPointerUp {
                window,
                popup,
                point,
                button,
            } => {
                self.pointer_up_on_popup(window, popup, point, button)?;
                Ok(())
            }
            Event::PointerLeft { window } => {
                self.pointer_left(window)?;
                Ok(())
            }
            Event::PopupPointerLeft { window, popup } => {
                self.pointer_left_popup(window, popup)?;
                Ok(())
            }
            Event::ModifiersChanged { window, modifiers }
            | Event::PopupModifiersChanged {
                window, modifiers, ..
            } => {
                self.pointer_modifiers_changed(window, modifiers)?;
                Ok(())
            }
            Event::Scrolled {
                window,
                point,
                delta,
            } => {
                let epoch_before = self.runtime.requested_presentation_epoch(window);
                let outcome = self.scroll(window, point, delta)?;
                if outcome.is_handled()
                    && self.runtime.requested_presentation_epoch(window) > epoch_before
                {
                    self.runtime.record_input_latency_sample(window, started_at);
                }
                Ok(())
            }
            Event::PopupScrolled {
                window,
                popup,
                point,
                delta,
            } => {
                let epoch_before = self.runtime.requested_presentation_epoch(window);
                let outcome = self.scroll_popup(window, popup, point, delta)?;
                if outcome.is_handled()
                    && self.runtime.requested_presentation_epoch(window) > epoch_before
                {
                    self.runtime.record_input_latency_sample(window, started_at);
                }
                Ok(())
            }
            Event::KeyDown {
                window,
                key,
                modifiers,
                text,
            } => {
                let started_at = Instant::now();
                let outcome = self.handle_input(
                    window,
                    input::Input::key_down_with_text(key, modifiers, text),
                )?;
                if outcome.is_handled() {
                    self.runtime.record_input_latency_sample(window, started_at);
                }
                Ok(())
            }
            Event::TextCommitted { window, text } => {
                let started_at = Instant::now();
                let outcome = self.handle_input(window, input::Input::text_commit(text))?;
                if outcome.is_handled() {
                    self.runtime.record_input_latency_sample(window, started_at);
                }
                Ok(())
            }
            Event::TextPreedit { window, preedit } => {
                let started_at = Instant::now();
                let outcome = self.handle_input(window, input::Input::text_preedit(preedit))?;
                if outcome.is_handled() {
                    self.runtime.record_input_latency_sample(window, started_at);
                }
                Ok(())
            }
            Event::FilePathSelected { window, path } => {
                self.file_path_selected(window, path)?;
                Ok(())
            }
            Event::Poll => return Ok(self.step()),
        };

        if let Some(window) = window {
            self.runtime
                .record_event_handling(window, started_at.elapsed());
        }

        result?;
        Ok(match redraw {
            Some(window) => self.redraw(window),
            None => self.drain_immediate(),
        })
    }
}
