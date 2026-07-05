use super::super::{
    command::{self, Command},
    context as command_context,
    error::Error,
    input,
    response::{self, Response},
    session, state, window,
};
use super::Runtime;

mod dialog;
mod effect;
mod key;
mod text;

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub fn focus(&mut self, window: window::Id, focus: session::Focus) -> bool {
        let changed = self.session.focus(window, focus);
        if changed {
            self.session.request_redraw(window);
        }

        changed
    }

    pub fn clear_focus(&mut self, window: window::Id) -> bool {
        let changed = self.session.clear_focus(window);
        if changed {
            self.session.request_redraw(window);
        }

        changed
    }

    pub fn invoke_focused<C: Command>(
        &mut self,
        window: window::Id,
        trigger: command::Trigger<C>,
    ) -> Response<C::Output> {
        self.invoke_focused_with_source(window, trigger, command_context::Source::Programmatic)
    }

    pub(in crate::scratch::runtime::input) fn invoke_focused_with_source<C: Command>(
        &mut self,
        window: window::Id,
        trigger: command::Trigger<C>,
        source: command_context::Source,
    ) -> Response<C::Output> {
        if !self.session.contains(window) {
            return Response::failed(Error::MissingTarget { command: C::NAME });
        }

        let response =
            self.invoke_with_focus(self.session.focused(window), Some(window), trigger, source);
        if response.is_ok() {
            self.apply_window_update(window, response.changed_state(), &response.effect);
        }

        response
    }

    pub fn handle_input(
        &mut self,
        window: window::Id,
        input: input::Input,
    ) -> std::result::Result<input::Outcome, Error> {
        if !self.session.contains(window) {
            return Ok(input::Outcome::ignored());
        }

        match input {
            input::Input::Cancel => {
                if self.session.close_menu(window) {
                    return Ok(self.window_outcome(window, false, response::Effect::Repaint));
                }

                if self.session.clear_text_input(window) {
                    return Ok(self.window_outcome(window, false, response::Effect::Repaint));
                }

                if self.session.cancel_pointer(window) {
                    self.finish_pointer_gesture();
                    return Ok(self.window_outcome(window, false, response::Effect::Repaint));
                }

                if self.clear_focus(window) {
                    return Ok(self.window_outcome(window, false, response::Effect::Repaint));
                }

                Ok(input::Outcome::ignored())
            }
            input::Input::Focus(focus) => {
                let effect = if self.focus(window, focus) {
                    response::Effect::Repaint
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::PointerMove(target) => {
                let effect = if self.session.pointer_move(window, target) {
                    response::Effect::Repaint
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::PointerDown(target) => {
                self.begin_pointer_gesture(&target);
                let effect = if self.session.pointer_down(window, target) {
                    response::Effect::Repaint
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::PointerDrag(hovered) => {
                let effect = if self.session.pointer_move(window, hovered) {
                    response::Effect::Repaint
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::PointerUp(target) => self.handle_pointer_up_input(window, target, true),
            input::Input::PointerLeft => {
                let effect = if self.session.pointer_left(window) {
                    response::Effect::Repaint
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::Scroll { target, delta } => {
                let scrolled = self.session.scroll_by(window, target, delta);
                let effect = if scrolled {
                    response::Effect::Repaint
                } else {
                    response::Effect::None
                };
                self.record_scroll_input(window, scrolled, &effect);

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::ToggleMenu(menu) => {
                let effect = if self.session.toggle_menu(window, menu) {
                    response::Effect::Repaint
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::FilePathSelected(path) => self.handle_file_path_selected(window, path),
            input::Input::Shortcut(shortcut) => self.handle_shortcut(window, shortcut),
            input::Input::KeyDown {
                key,
                modifiers,
                text,
            } => self.handle_key_down(window, key, modifiers, text),
            input::Input::TextEdit(edit) => {
                self.handle_text_edit(window, edit, command_context::Source::Keyboard)
            }
            input::Input::TextCommit(text) => self.handle_text_commit(window, text),
            input::Input::TextPreedit(preedit) => {
                let Some(focus) = self.session.focused(window) else {
                    return Ok(input::Outcome::ignored());
                };
                let target = self.text_input_target(window, focus);
                let changed = self.session.set_text_preedit_for(window, target, preedit);
                let effect = if changed {
                    response::Effect::Repaint
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::TextDrop(drop) => self.handle_text_drop(window, drop),
        }
    }
}
