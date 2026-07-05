use super::super::{
    context as command_context, error::Error, input, response, state, view, window,
};
use super::{Runtime, services};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub fn activate(
        &mut self,
        binding: &view::Binding,
    ) -> std::result::Result<response::Effect, Error> {
        self.activate_with_focus(None, None, binding)
    }

    pub fn activate_in(
        &mut self,
        window: window::Id,
        binding: &view::Binding,
    ) -> std::result::Result<response::Effect, Error> {
        if !self.session.contains(window) {
            return Err(Error::MissingTarget {
                command: binding.command_name(),
            });
        }

        let before = self.revision();
        let focus = if binding.source() == command_context::Source::Menu {
            self.session.command_focus(window)
        } else {
            self.session.focused(window)
        };
        let text_box_command = binding.source() == command_context::Source::Menu
            && services::text::handles(
                &self.session,
                &self.composition,
                Some(window),
                binding.command_type(),
            );
        let text_box_commit =
            if binding.source() == command_context::Source::Menu && !text_box_command {
                self.commit_and_deactivate_focused_text_box(window)?
            } else {
                None
            };

        let result = self.activate_with_focus(focus, Some(window), binding);
        if let Ok(effect) = &result {
            let effect = text_box_commit
                .as_ref()
                .map(|outcome| outcome.effect().clone())
                .unwrap_or(response::Effect::None)
                .then(effect.clone());
            self.apply_window_update(window, self.revision() != before, &effect);
            self.close_menu_after_binding(window, binding);
            return Ok(effect);
        }

        result
    }

    pub fn handle_view(
        &mut self,
        window: window::Id,
        action: view::Action,
    ) -> std::result::Result<input::Outcome, Error> {
        match action {
            view::Action::Sequence(actions) => {
                let mut handled = false;
                let mut changed_state = false;
                let mut effect = response::Effect::None;

                for action in actions {
                    let outcome = self.handle_view(window, action)?;
                    handled |= outcome.is_handled();
                    changed_state |= outcome.changed_state();
                    effect = effect.then(outcome.effect().clone());
                }

                if handled {
                    Ok(input::Outcome::handled(changed_state, effect))
                } else {
                    Ok(input::Outcome::ignored())
                }
            }
            view::Action::Activate(binding) => {
                let before = self.revision();
                let effect = self.activate_in(window, &binding)?;

                Ok(input::Outcome::handled(before != self.revision(), effect))
            }
            view::Action::Focus(focus) => self.handle_input(window, input::Input::focus(focus)),
            view::Action::PointerMove(target) => {
                self.handle_input(window, input::Input::pointer_move(target))
            }
            view::Action::PointerDown(target) => {
                self.handle_input(window, input::Input::pointer_down(target))
            }
            view::Action::PointerDrag {
                hovered,
                target,
                action,
            } => {
                let captured = self
                    .session
                    .interaction(window)
                    .and_then(|interaction| interaction.pointer().capture())
                    .map(|capture| capture.target())
                    == Some(&target);
                let pointer = self.handle_input(window, input::Input::pointer_drag(hovered))?;

                if !captured {
                    return Ok(pointer);
                }

                let Some(action) = action else {
                    return Ok(pointer);
                };

                let dragged = self.handle_view(window, *action)?;
                let effect = pointer.effect().clone().then(dragged.effect().clone());

                Ok(input::Outcome::handled(
                    pointer.changed_state() || dragged.changed_state(),
                    effect,
                ))
            }
            view::Action::PointerUp { target, action } => {
                let activate = self
                    .session
                    .interaction(window)
                    .and_then(|interaction| interaction.pointer().pressed())
                    == target.as_ref();
                let pointer = self.handle_pointer_up_input(window, target.clone(), false)?;

                if !activate {
                    self.finish_pointer_gesture();
                    return Ok(pointer);
                }

                let Some(action) = action else {
                    self.finish_pointer_gesture();
                    return Ok(pointer);
                };

                let activated = self.handle_view(window, *action);
                self.finish_pointer_gesture();
                let activated = activated?;
                let effect = pointer.effect().clone().then(activated.effect().clone());

                Ok(input::Outcome::handled(
                    pointer.changed_state() || activated.changed_state(),
                    effect,
                ))
            }
            view::Action::PointerLeft => self.handle_input(window, input::Input::pointer_left()),
            view::Action::Scroll { target, delta } => {
                self.handle_input(window, input::Input::scroll(target, delta))
            }
            view::Action::ToggleMenu(menu) => {
                self.handle_input(window, input::Input::toggle_menu(menu))
            }
            view::Action::TextEdit(edit) => {
                self.handle_input(window, input::Input::text_edit(edit))
            }
            view::Action::TextDrop(drop) => self.handle_input(window, input::Input::TextDrop(drop)),
        }
    }
}
