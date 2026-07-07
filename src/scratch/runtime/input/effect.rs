use super::super::Runtime;
use crate::scratch::{
    context as command_context, error::Error, input, interaction, response, session, state, view,
    window,
};

impl<M: state::State, E: Send + 'static, V> Runtime<M, E, V> {
    pub(in crate::scratch::runtime) fn handle_pointer_up_input(
        &mut self,
        window: window::Id,
        target: Option<interaction::Target>,
        finish_gesture: bool,
    ) -> std::result::Result<input::Outcome, Error> {
        let effect = if self.session.pointer_up(window, target) {
            response::Effect::Paint
        } else {
            response::Effect::None
        };

        if finish_gesture {
            self.finish_pointer_gesture();
        }

        Ok(self.window_outcome(window, false, effect))
    }

    pub(in crate::scratch::runtime) fn window_outcome(
        &mut self,
        window: window::Id,
        changed_state: bool,
        effect: response::Effect,
    ) -> input::Outcome {
        self.apply_window_update(window, changed_state, &effect);
        input::Outcome::handled(changed_state, effect)
    }

    pub(in crate::scratch::runtime::input) fn record_scroll_input(
        &mut self,
        window: window::Id,
        offset_changed: bool,
        effect: &response::Effect,
    ) {
        let diagnostics = self.diagnostics.get_mut(window);
        diagnostics.scroll.wheel_events += 1;
        if offset_changed {
            diagnostics.scroll.scroll_offset_changes += 1;
        }
        if effect.contains_invalidation() {
            diagnostics.scroll.scroll_redraw_requests += 1;
        }
    }

    pub(in crate::scratch::runtime) fn apply_window_update(
        &mut self,
        window: window::Id,
        changed_state: bool,
        effect: &response::Effect,
    ) {
        if changed_state {
            self.session
                .request_invalidation(window, response::Invalidation::Rebuild);
        }

        self.apply_window_effect(window, effect);
    }

    fn apply_window_effect(&mut self, window: window::Id, effect: &response::Effect) {
        match effect {
            response::Effect::OpenFileDialog => {
                self.session
                    .request_file_dialog(window, session::FileDialog::Open);
            }
            response::Effect::SaveFileDialog => {
                self.session
                    .request_file_dialog(window, session::FileDialog::SaveAs);
            }
            response::Effect::Paint | response::Effect::Layout | response::Effect::Rebuild => {
                if let Some(invalidation) = effect.invalidation() {
                    self.session.request_invalidation(window, invalidation);
                }
            }
            response::Effect::CloseFloatingPanel => {
                if self.session.close_menu(window) {
                    self.session
                        .request_invalidation(window, response::Invalidation::Rebuild);
                }
            }
            response::Effect::Batch(effects) => {
                for effect in effects {
                    self.apply_window_effect(window, effect);
                }
            }
            response::Effect::None => {}
        }
    }

    pub(in crate::scratch::runtime) fn close_menu_after_binding(
        &mut self,
        window: window::Id,
        binding: &view::Binding,
    ) {
        if binding.source() == command_context::Source::Menu && self.session.close_menu(window) {
            self.session
                .request_invalidation(window, response::Invalidation::Rebuild);
        }
    }
}
