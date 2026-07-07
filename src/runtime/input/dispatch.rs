use super::super::Runtime;
use crate::{
    context as command_context, error::Error, input, interaction, response, state, window,
};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
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
                if self.session.close_command_palette(window) {
                    return Ok(self.window_outcome(window, false, response::Effect::Rebuild));
                }

                if self.session.close_menu(window) {
                    return Ok(self.window_outcome(window, false, response::Effect::Rebuild));
                }

                if self.session.clear_text_input(window) {
                    return Ok(self.window_outcome(window, false, response::Effect::Layout));
                }

                if self.session.cancel_pointer(window) {
                    self.finish_pointer_gesture();
                    return Ok(self.window_outcome(window, false, response::Effect::Paint));
                }

                self.clear_focus_committing_text_box(window)
            }
            input::Input::Focus(focus) => self.focus_committing_text_box(window, focus),
            input::Input::PointerMove(target) => {
                let menu_switch = target
                    .as_ref()
                    .and_then(interaction::Target::as_menu)
                    .is_some_and(|menu| {
                        self.session
                            .interaction(window)
                            .and_then(|interaction| interaction.open_menu())
                            .is_some_and(|open| *open != menu)
                    });
                let effect = if self.session.pointer_move(window, target) {
                    if menu_switch {
                        response::Effect::Rebuild
                    } else {
                        response::Effect::Paint
                    }
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::PointerDown(target) => {
                self.begin_pointer_gesture(&target);
                let effect =
                    if self
                        .session
                        .pointer_down(window, target, interaction::PressIntent::Activate)
                    {
                        response::Effect::Paint
                    } else {
                        response::Effect::None
                    };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::PointerManipulate(target) => {
                self.begin_pointer_gesture(&target);
                let effect = if self.session.pointer_down(
                    window,
                    target,
                    interaction::PressIntent::Manipulate,
                ) {
                    response::Effect::Paint
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::PointerDrag(hovered) => {
                let effect = if self.session.pointer_move(window, hovered) {
                    response::Effect::Paint
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::PointerUp(target) => self.handle_pointer_up_input(window, target, true),
            input::Input::PointerLeft => {
                let effect = if self.session.pointer_left(window) {
                    response::Effect::Paint
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::Scroll { target, delta } => {
                let scrolled = self.session.scroll_by(window, target, delta);
                let effect = if scrolled {
                    response::Effect::Layout
                } else {
                    response::Effect::None
                };
                self.record_scroll_input(window, scrolled, &effect);

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::ScrollTo { target, offset } => {
                let scrolled = self.session.resolve_scroll(window, target, offset);
                let effect = if scrolled {
                    response::Effect::Layout
                } else {
                    response::Effect::None
                };
                self.record_scroll_input(window, scrolled, &effect);

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::ToggleMenu(menu) => {
                let effect = if self.session.toggle_menu(window, menu) {
                    response::Effect::Rebuild
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
                    response::Effect::Layout
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::TextDrop(drop) => self.handle_text_drop(window, drop),
        }
    }
}
