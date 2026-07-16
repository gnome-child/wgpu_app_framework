use super::super::Runtime;
use crate::{
    command::Error, context as command_context, input, interaction, response, session, state,
    window,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ScrollTransition {
    Unchanged,
    PropertyTick(interaction::ScrollOffset),
    NeedsResidency {
        desired: interaction::ScrollOffset,
        admitted: interaction::ScrollOffset,
    },
}

impl ScrollTransition {
    fn offset(self) -> Option<interaction::ScrollOffset> {
        match self {
            Self::Unchanged => None,
            Self::PropertyTick(offset)
            | Self::NeedsResidency {
                desired: offset, ..
            } => Some(offset),
        }
    }

    fn effect(self) -> response::Effect {
        match self {
            Self::Unchanged | Self::PropertyTick(_) => response::Effect::None,
            Self::NeedsResidency { .. } => response::Effect::Rebuild,
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
                let started = std::time::Instant::now();
                let transition = self.apply_scroll_transition(
                    window,
                    target,
                    interaction::ScrollUpdate::Relative(delta),
                );
                let scrolled = transition.offset().is_some();
                self.record_scroll_input(window, transition, scrolled, started.elapsed());

                Ok(self.window_outcome(window, false, transition.effect()))
            }
            input::Input::ScrollTo { target, offset } => {
                let started = std::time::Instant::now();
                let transition = self.apply_scroll_transition(
                    window,
                    target,
                    interaction::ScrollUpdate::Absolute(offset),
                );
                let scrolled = transition.offset().is_some();
                self.record_scroll_input(window, transition, scrolled, started.elapsed());

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
        let target_key = target.focus_key();
        let before = self
            .session
            .interaction(window)
            .map(|interaction| interaction.scroll().desired_offset(&target))
            .unwrap_or_default();
        let Some(requested) = self.session.request_scroll(window, target.clone(), update) else {
            let resident_offset = self
                .session
                .interaction(window)
                .map(|interaction| interaction.scroll().offset(&target))
                .unwrap_or_default();
            self.record_scroll_trace(
                window,
                target_key,
                before,
                before,
                resident_offset,
                false,
                "unchanged",
            );
            return ScrollTransition::Unchanged;
        };
        let presented = self.presented_layout(window);
        let offset = presented.as_ref().map_or(requested, |layout| {
            layout.resolve_scroll_offset(&target, requested)
        });
        if offset != requested {
            self.session.request_scroll(
                window,
                target.clone(),
                interaction::ScrollUpdate::Geometry(offset),
            );
        }
        let resident_accepted = presented
            .as_ref()
            .is_some_and(|layout| layout.scroll_property_accepts(&target, offset));
        if offset == before {
            let resident_offset = self
                .session
                .interaction(window)
                .map(|interaction| interaction.scroll().offset(&target))
                .unwrap_or_default();
            self.record_scroll_trace(
                window,
                target_key,
                requested,
                offset,
                resident_offset,
                resident_accepted,
                "unchanged",
            );
            return ScrollTransition::Unchanged;
        }
        if resident_accepted {
            self.session.admit_scroll(window, target, offset);
            self.session.request_property_tick(window);
            self.record_scroll_trace(
                window,
                target_key,
                requested,
                offset,
                offset,
                true,
                "property-tick",
            );
            ScrollTransition::PropertyTick(offset)
        } else {
            let admitted = self
                .session
                .interaction(window)
                .map(|interaction| interaction.scroll().offset(&target))
                .unwrap_or_default();
            let request = self
                .presented_layout(window)
                .and_then(|layout| layout.virtual_request_for_scroll_offset(&target, offset));
            if let Some(request) = request {
                self.install_virtual_request(window, request);
            }
            self.record_scroll_trace(
                window,
                target_key,
                requested,
                offset,
                admitted,
                false,
                "needs-residency",
            );
            ScrollTransition::NeedsResidency {
                desired: offset,
                admitted,
            }
        }
    }

    fn record_scroll_trace(
        &mut self,
        window: window::Id,
        target_key: u64,
        requested: interaction::ScrollOffset,
        clamped: interaction::ScrollOffset,
        resident_offset: interaction::ScrollOffset,
        resident_accepted: bool,
        outcome: &'static str,
    ) {
        let Some(epoch) = self
            .session
            .window(window)
            .map(session::Window::desired_presentation_epoch)
        else {
            return;
        };
        self.diagnostics.get_mut(window).scroll.record_transition(
            epoch,
            target_key,
            requested,
            clamped,
            resident_offset,
            resident_accepted,
            outcome,
        );
    }
}
