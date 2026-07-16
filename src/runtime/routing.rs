use super::super::{
    command::Error, context as command_context, input, interaction, response, state, view, window,
};
use super::Runtime;

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
        if binding.source() == command_context::Source::Palette {
            let effect = self.activate_command_palette_binding(window, binding)?;
            self.apply_window_update(window, before != self.revision(), &effect);
            return Ok(effect);
        }

        let focus = if binding.source() == command_context::Source::Menu {
            self.session.command_focus(window)
        } else {
            self.session.focused(window)
        };
        let focused_text = if binding.source() == command_context::Source::Menu {
            Some(self.prepare_focused_text_for_command(window, binding.command_type())?)
        } else {
            None
        };
        if focused_text
            .as_ref()
            .is_some_and(|focused_text| !focused_text.is_accepted())
        {
            let effect = focused_text
                .as_ref()
                .and_then(|focused_text| focused_text.committed())
                .map(|outcome| outcome.effect().clone())
                .unwrap_or(response::Effect::None);
            self.apply_window_update(window, self.revision() != before, &effect);
            return Ok(effect);
        }

        let result = self.activate_with_focus(focus, Some(window), binding);
        if let Ok(effect) = &result {
            let effect = focused_text
                .as_ref()
                .and_then(|focused_text| focused_text.committed())
                .map(|outcome| outcome.effect().clone())
                .unwrap_or(response::Effect::None)
                .then(effect.clone());
            self.apply_window_update(window, self.revision() != before, &effect);
            self.close_menu_after_binding(window, binding);
            return Ok(effect);
        }

        result
    }

    pub(crate) fn handle_view(
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
            view::Action::PointerDown {
                target,
                intent,
                cursor,
            } => self.handle_pointer_down_input(window, target, intent, cursor),
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
                    == Some(&target);
                let pointer = self.handle_pointer_up_input(window, Some(target.clone()), false)?;

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
            view::Action::PointerUpOutside => {
                let pointer = self.handle_pointer_up_input(window, None, false)?;
                self.finish_pointer_gesture();
                Ok(pointer)
            }
            view::Action::PointerLeft => self.handle_input(window, input::Input::pointer_left()),
            view::Action::ResizeTableColumn { column, width } => {
                let changed = self.session.resize_table_column(window, column, width);
                Ok(self.window_outcome(
                    window,
                    false,
                    if changed {
                        response::Effect::Layout
                    } else {
                        response::Effect::None
                    },
                ))
            }
            view::Action::Scroll { target, delta } => {
                self.handle_input(window, input::Input::scroll(target, delta))
            }
            view::Action::ScrollTo {
                target,
                offset,
                axis,
            } => {
                let current = self
                    .session
                    .interaction(window)
                    .map(interaction::Interaction::scroll)
                    .map(|scroll| scroll.desired_offset(&target))
                    .unwrap_or_default();
                let offset = scrollbar_offset(axis, current, offset);
                self.handle_input(window, input::Input::scroll_to(target, offset))
            }
            view::Action::ToggleMenu(menu) => {
                self.handle_input(window, input::Input::toggle_menu(menu))
            }
            view::Action::TextSelection(operation) => {
                self.handle_input(window, input::Input::text_selection(operation))
            }
        }
    }
}

fn scrollbar_offset(
    axis: interaction::ScrollbarAxis,
    current: interaction::ScrollOffset,
    candidate: interaction::ScrollOffset,
) -> interaction::ScrollOffset {
    match axis {
        interaction::ScrollbarAxis::Horizontal => {
            interaction::ScrollOffset::new(candidate.x(), current.y())
        }
        interaction::ScrollbarAxis::Vertical => {
            interaction::ScrollOffset::new(current.x(), candidate.y())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrollbar_drag_preserves_the_other_desired_axis() {
        let desired = interaction::ScrollOffset::new(70, 90);
        let stale_layout = interaction::ScrollOffset::new(10, 20);

        assert_eq!(
            scrollbar_offset(
                interaction::ScrollbarAxis::Horizontal,
                desired,
                stale_layout,
            ),
            interaction::ScrollOffset::new(10, 90)
        );
        assert_eq!(
            scrollbar_offset(interaction::ScrollbarAxis::Vertical, desired, stale_layout,),
            interaction::ScrollOffset::new(70, 20)
        );
    }
}
