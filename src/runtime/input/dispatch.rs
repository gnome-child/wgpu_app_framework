use super::super::Runtime;
use crate::{
    command::Error, context as command_context, input, interaction, response, session, state,
    window,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScrollTransition {
    Unchanged,
    PropertyTick(interaction::ScrollOffset),
    NeedsResidency(interaction::ScrollOffset),
}

impl ScrollTransition {
    fn offset(self) -> Option<interaction::ScrollOffset> {
        match self {
            Self::Unchanged => None,
            Self::PropertyTick(offset) | Self::NeedsResidency(offset) => Some(offset),
        }
    }

    fn effect(self) -> response::Effect {
        match self {
            Self::Unchanged | Self::PropertyTick(_) => response::Effect::None,
            Self::NeedsResidency(_) => response::Effect::Layout,
        }
    }
}

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

                let active_text_focus = self
                    .session
                    .interaction(window)
                    .and_then(|interaction| interaction.text_input().target())
                    .and_then(session::Focus::from_text_target);
                if let Some(focus) = active_text_focus {
                    self.session.clear_text_draft(window, focus);
                    self.session
                        .request_invalidation(window, response::effect::Invalidation::Rebuild);
                    return Ok(self.window_outcome(window, false, response::Effect::Rebuild));
                }

                if self.session.clear_text_input(window) {
                    return Ok(self.window_outcome(window, false, response::Effect::Rebuild));
                }

                if self.session.cancel_pointer(window) {
                    self.finish_pointer_gesture();
                    return Ok(self.window_outcome(window, false, response::Effect::Paint));
                }

                self.clear_focus_committing_text_box(window)
            }
            input::Input::Focus(focus) => self.focus_committing_text_box(window, focus),
            input::Input::PointerMove(target) => {
                let hover_tip_was_visible = self.session.hover_tip_visible(window);
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
                    if menu_switch || hover_tip_was_visible {
                        response::Effect::Rebuild
                    } else {
                        response::Effect::Paint
                    }
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::PointerDown(target) => self.handle_pointer_down_input(
                window,
                target,
                interaction::pointer::PressIntent::Activate,
                crate::pointer::Cursor::Default,
            ),
            input::Input::PointerManipulate(target) => self.handle_pointer_down_input(
                window,
                target,
                interaction::pointer::PressIntent::Manipulate,
                crate::pointer::Cursor::Default,
            ),
            input::Input::PointerDrag(hovered) => {
                let hover_tip_was_visible = self.session.hover_tip_visible(window);
                let effect = if self.session.pointer_move(window, hovered) {
                    if hover_tip_was_visible {
                        response::Effect::Rebuild
                    } else {
                        response::Effect::Paint
                    }
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::PointerUp(target) => self.handle_pointer_up_input(window, target, true),
            input::Input::PointerLeft => {
                let hover_tip_was_visible = self.session.hover_tip_visible(window);
                let effect = if self.session.pointer_left(window) {
                    if hover_tip_was_visible {
                        response::Effect::Rebuild
                    } else {
                        response::Effect::Paint
                    }
                } else {
                    response::Effect::None
                };

                Ok(self.window_outcome(window, false, effect))
            }
            input::Input::Scroll { target, delta } => {
                let transition = self.apply_scroll_transition(
                    window,
                    target,
                    interaction::ScrollUpdate::Relative(delta),
                );
                let scrolled = transition.offset().is_some();
                self.record_scroll_input(window, scrolled, scrolled);

                Ok(self.window_outcome(window, false, transition.effect()))
            }
            input::Input::ScrollTo { target, offset } => {
                let transition = self.apply_scroll_transition(
                    window,
                    target,
                    interaction::ScrollUpdate::Absolute(offset),
                );
                let scrolled = transition.offset().is_some();
                self.record_scroll_input(window, scrolled, scrolled);

                Ok(self.window_outcome(window, false, transition.effect()))
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
            input::Input::TextSelection(operation) => {
                self.handle_text_selection(window, operation, command_context::Source::Keyboard)
            }
            input::Input::TextEdit(edit) => {
                self.handle_text_edit(window, edit, command_context::Source::Keyboard)
            }
            input::Input::TextCommit(text) => self.handle_text_commit(window, text),
            input::Input::TextPreedit(preedit) => {
                let Some(focus) = self.session.focused(window) else {
                    return Ok(input::Outcome::ignored());
                };
                let Some(target) = self.text_input_target(window, focus) else {
                    return Ok(input::Outcome::ignored());
                };
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

    fn apply_scroll_transition(
        &mut self,
        window: window::Id,
        target: interaction::Target,
        update: interaction::ScrollUpdate,
    ) -> ScrollTransition {
        let Some(offset) = self.session.apply_scroll(window, target.clone(), update) else {
            return ScrollTransition::Unchanged;
        };
        if self
            .presented_layout(window)
            .is_some_and(|layout| layout.scroll_property_accepts(&target, offset))
        {
            self.session.request_property_tick(window);
            ScrollTransition::PropertyTick(offset)
        } else {
            ScrollTransition::NeedsResidency(offset)
        }
    }
}
